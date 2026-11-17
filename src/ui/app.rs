//! Application state and the frame loop: top bar, status bar, dialogs,
//! keyboard shortcuts, background jobs. Panels live in sidebar.rs,
//! editor.rs, response.rs and palette.rs.

use super::icon::Icon;
use super::palette::Palette;
use super::theme::{self, theme, Layout, Space};
use super::widgets::{
    self, confirm_modal, faint, icon_button, input_modal, key_combo, label, menu_item, modal,
    muted, popup_menu, strong, ModalAction,
};
use crate::http::{self, HttpResponse};
use crate::import::{self, ImportCount};
use crate::kv::{self, KeyValue};
use crate::model::{
    AppState, CollectionItem, HistoryEntry, HistorySummary, HttpMethod, RecentRequest, Request,
    RequestFile, ThemeMode,
};
use crate::{curl, storage, vars, workspace};
use eframe::egui::{self, Align, Key, Margin, Sense, Stroke, Vec2};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

const ISSUES_URL: &str = "https://github.com/Harry-kp/mercury/issues";
const RELEASES_URL: &str = "https://github.com/Harry-kp/mercury/releases";
const DOCS_URL: &str = "https://harry-kp.github.io/mercury/docs/getting-started";
const AUTOSAVE_SECS: f64 = 5.0;
const CRUMB_CHARS: usize = 28;

/// Every command Mercury can run. The keyboard table and the command palette
/// are both views over this list, so a new action needs no extra plumbing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Send,
    New,
    NewFolder,
    Save,
    OpenFolder,
    Palette,
    FocusUrl,
    NextEnv,
    History,
    ToggleRaw,
    CopyCurl,
    FormatBody,
    FocusMode,
    FindInResponse,
    ToggleTheme,
    ImportPostman,
    ImportInsomnia,
    Docs,
    Help,
    RenameRequest,
    DuplicateRequest,
    DeleteRequest,
    RevertFile,
}

/// One keyboard shortcut. `SHORTCUTS` is the single list: the in-app help
/// modal and the command palette render it, and the README/website tables
/// must match it.
pub struct Shortcut {
    pub keys: &'static str,
    pub label: &'static str,
    key: Key,
    command: bool,
    shift: bool,
    pub action: Action,
}

const fn cmd(
    keys: &'static str,
    label: &'static str,
    key: Key,
    shift: bool,
    action: Action,
) -> Shortcut {
    Shortcut {
        keys,
        label,
        key,
        command: true,
        shift,
        action,
    }
}

pub const SHORTCUTS: &[Shortcut] = &[
    cmd("⌘ ⏎", "Send request", Key::Enter, false, Action::Send),
    cmd("⌘ K", "Command palette", Key::K, false, Action::Palette),
    cmd("⌘ N", "New request", Key::N, false, Action::New),
    cmd("⌘ S", "Save request", Key::S, false, Action::Save),
    cmd("⌘ O", "Open folder", Key::O, false, Action::OpenFolder),
    cmd("⌘ L", "Focus URL bar", Key::L, false, Action::FocusUrl),
    cmd("⌘ E", "Next environment", Key::E, false, Action::NextEnv),
    cmd("⌘ H", "Toggle history", Key::H, false, Action::History),
    cmd(
        "⌘ R",
        "Toggle raw response",
        Key::R,
        false,
        Action::ToggleRaw,
    ),
    cmd(
        "⌘ D",
        "Toggle light / dark",
        Key::D,
        false,
        Action::ToggleTheme,
    ),
    cmd(
        "⌘ F",
        "Find in response",
        Key::F,
        false,
        Action::FindInResponse,
    ),
    cmd("⌘ ⇧ C", "Copy as cURL", Key::C, true, Action::CopyCurl),
    cmd(
        "⌘ ⇧ F",
        "Hide / show sidebar",
        Key::F,
        true,
        Action::FocusMode,
    ),
    Shortcut {
        keys: "?",
        label: "Keyboard shortcuts",
        key: Key::Questionmark,
        command: false,
        shift: false,
        action: Action::Help,
    },
];

/// Not a table entry because it does different things depending on context.
pub const ESCAPE_HELP: (&str, &str) = (
    "Esc",
    "Close find / cancel request / close dialog / clear filter",
);

impl Shortcut {
    fn pressed(&self, i: &egui::InputState, typing: bool) -> bool {
        if self.command {
            // (⌘⇧C also arrives as Event::Copy in the real app; see
            // `take_copy_as_curl`)
            i.modifiers.command && i.modifiers.shift == self.shift && i.key_pressed(self.key)
        } else {
            // bare keys must not fire while typing ("?" in a URL)
            !typing
                && !i.modifiers.command
                && (i.key_pressed(self.key) || (i.modifiers.shift && i.key_pressed(Key::Slash)))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Body,
    Params,
    Headers,
    Auth,
}

/// Which pane of a finished response is on screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseTab {
    Body,
    Headers,
    Cookies,
}

#[derive(Clone, Debug)]
pub enum Dialog {
    /// The `bool` is true when this is ⌘S on an unsaved request rather than
    /// ⌘N: the same input, but "Save request" / "Save" instead of "Create",
    /// because the words have to match the action the user invoked.
    NewRequest(PathBuf, bool),
    NewFolder(PathBuf),
    Rename(PathBuf),
    /// The path, and the confirmation text (counted once, when opened).
    Delete(PathBuf, String),
    Shortcuts,
    NewEnv,
    /// The `.env` file being edited (its text is `env_edit_text`).
    EditEnv(String),
}

pub struct Toast {
    pub text: String,
    pub shown_at: f64,
    pub is_error: bool,
    /// What the toast's "Undo" puts back. Clearing a list is instant and
    /// undoable rather than confirmed: a confirm on every clear is noise, and
    /// no confirm at all lost whole histories to one mis-click.
    pub undo: Option<Undo>,
}

pub enum Undo {
    /// What was on disk, and what the list showed (a send's background
    /// append may not have reached the disk yet when the list was cleared).
    History(Vec<HistoryEntry>, Vec<HistorySummary>),
    Recent(Vec<RecentRequest>),
}

/// Results from background threads, drained once per frame.
enum Event {
    Response(u64, Result<HttpResponse, String>),
    OpenFolder(PathBuf),
    /// A file picked for a Multipart body.
    AttachFile(PathBuf),
    Imported(Result<(PathBuf, ImportCount), String>),
    FilesChanged,
    Toast(String, bool),
}

pub struct Importer {
    pub name: &'static str,
    extensions: &'static [&'static str],
    run: fn(&Path, &Path) -> Result<ImportCount, String>,
}

pub const POSTMAN: Importer = Importer {
    name: "Postman",
    extensions: &["json"],
    run: import::import_postman,
};

pub const INSOMNIA: Importer = Importer {
    name: "Insomnia",
    extensions: &["json", "yaml", "yml"],
    run: import::import_insomnia,
};

pub struct MercuryApp {
    // Workspace
    pub workspace: Option<PathBuf>,
    pub tree: Vec<CollectionItem>,
    pub expanded: HashSet<PathBuf>,
    pub selected_folder: Option<PathBuf>,
    /// Where a new or saved request lands: the folder of the request opened
    /// last, or the folder clicked last, whichever came later.
    last_folder: Option<PathBuf>,
    /// Request files opened lately, newest first (the palette's empty query).
    pub opened_recently: Vec<PathBuf>,
    pub search: String,
    /// Filtering needs the whole tree in memory; see `sidebar::load_subtree`.
    pub tree_fully_loaded: bool,
    pub env_files: Vec<String>,
    pub selected_env: Option<usize>,
    pub env_vars: HashMap<String, String>,
    /// The variables whose value comes from the shared `.env`, not the
    /// selected `.env.<name>` (to say where a value is from).
    pub env_shared: HashSet<String>,
    watcher: Option<workspace::Watcher>,

    // Request form. `headers_text` is the source of truth for headers AND auth.
    pub current_file: Option<PathBuf>,
    pub method: HttpMethod,
    pub url: String,
    pub query_params: Vec<KeyValue>,
    pub params_text: String,
    /// A form body shown as a table; `body_text` stays the source of truth.
    pub form_text: String,
    pub form_bulk_edit: bool,
    pub headers_text: String,
    pub body_text: String,
    pub tab: Tab,
    pub headers_bulk_edit: bool,
    pub params_bulk_edit: bool,
    /// File content at last load/save; unsaved = current content differs.
    saved_content: Option<String>,
    /// Fields of the open file Mercury doesn't edit, saved back as they were.
    file_extra: std::collections::BTreeMap<String, serde_json::Value>,
    last_save: f64,

    // Response
    /// Shared, so drawing a frame never copies a multi-megabyte body.
    pub response: Option<std::sync::Arc<HttpResponse>>,
    pub request_error: Option<String>,
    pub formatted_body: Option<String>,
    /// Row index of the body on screen (see `response::BodyRows`).
    pub body_rows: Option<super::response::BodyRows>,
    pub raw_view: bool,
    /// What to find in the response body, and which match is current. Empty
    /// while the find bar is closed.
    pub find: Option<String>,
    pub find_at: usize,
    pub response_tab: ResponseTab,
    pub in_flight: Option<u64>,
    /// The form as it was when the in-flight request left, so history records
    /// what was sent even if the user keeps editing while it runs.
    sent: Option<Request>,
    /// The file that was open when it left. If the user navigates to another
    /// request meanwhile, the response belongs to neither of them on screen.
    sent_file: Option<PathBuf>,
    /// The form was replaced by another unsaved request since it left: no
    /// file to tell them apart, but the response belongs to neither.
    sent_replaced: bool,
    /// The environment it left with, recorded in history.
    sent_env: Option<String>,
    /// The environment the response on screen came from.
    pub response_env: Option<String>,
    /// The open file as it was when opened: "Revert to saved" goes back to
    /// it, since autosave has already written every edit since.
    opened_content: Option<String>,
    request_counter: u64,

    // History and recent
    pub history: Vec<HistorySummary>,
    history_loaded: bool,
    pub history_search: String,
    pub show_history: bool,
    pub recent: Vec<RecentRequest>,
    pub recent_expanded: bool,

    // UI
    pub focus_mode: bool,
    pub theme_mode: ThemeMode,
    pub palette: Option<Palette>,
    pub dialog: Option<Dialog>,
    pub dialog_text: String,
    /// The environment editor's text and its table / bulk-edit mode.
    pub env_edit_text: String,
    pub env_edit_bulk: bool,
    pub toast: Option<Toast>,
    /// See [`Self::focus`].
    pending_focus: Option<&'static str>,
    /// The Authorization value last used for each auth type in this request
    /// (indexed like `AuthMode::ALL`), restored when switching back.
    pub auth_stash: [Option<String>; 4],
    /// egui time at the start of this frame (for toasts).
    time: f64,

    ctx: egui::Context,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    client: reqwest::blocking::Client,
    /// Hosts whose certificate the user chose not to check, this session only.
    pub trusted_hosts: HashSet<String>,
    /// The client for those hosts, built on first use (own cookies, same-host redirects).
    lenient: Option<reqwest::blocking::Client>,
}

impl MercuryApp {
    pub fn new(ctx: &egui::Context) -> Self {
        let (tx, rx) = channel();
        let mut app = Self {
            workspace: None,
            tree: Vec::new(),
            expanded: HashSet::new(),
            selected_folder: None,
            search: String::new(),
            tree_fully_loaded: false,
            env_files: Vec::new(),
            selected_env: None,
            env_vars: HashMap::new(),
            env_shared: HashSet::new(),
            watcher: None,
            current_file: None,
            method: HttpMethod::GET,
            url: String::new(),
            query_params: Vec::new(),
            params_text: String::new(),
            form_text: String::new(),
            form_bulk_edit: false,
            headers_text: String::new(),
            body_text: String::new(),
            // a fresh request is a GET: Params, by the rule in `load_request`
            tab: Tab::Params,
            headers_bulk_edit: false,
            params_bulk_edit: false,
            saved_content: None,
            last_save: 0.0,
            response: None,
            request_error: None,
            formatted_body: None,
            body_rows: None,
            raw_view: false,
            find: None,
            find_at: 0,
            response_tab: ResponseTab::Body,
            in_flight: None,
            sent: None,
            sent_file: None,
            sent_replaced: false,
            sent_env: None,
            response_env: None,
            opened_content: None,
            file_extra: Default::default(),
            last_folder: None,
            opened_recently: Vec::new(),
            request_counter: 0,
            history: Vec::new(),
            history_loaded: false,
            history_search: String::new(),
            show_history: false,
            recent: storage::load_recent(),
            recent_expanded: true,
            focus_mode: false,
            theme_mode: ThemeMode::default(),
            palette: None,
            dialog: None,
            dialog_text: String::new(),
            env_edit_text: String::new(),
            env_edit_bulk: false,
            toast: None,
            pending_focus: None,
            auth_stash: Default::default(),
            time: 0.0,
            ctx: ctx.clone(),
            tx,
            rx,
            client: http::client_with(true),
            trusted_hosts: HashSet::new(),
            lenient: None,
        };

        if let Some(state) = storage::load_state() {
            app.load_request(
                Request {
                    method: state.method,
                    url: state.url,
                    headers: state.headers_text,
                    body: state.body_text,
                },
                None,
            );
            // `selected_tab` is still written (the state file is a
            // compatibility contract) but no longer read back: the tab
            // follows the request, and restoring the last one put an unsaved
            // GET back on an empty Body.
            app.theme_mode = state.theme;
            if let Some(path) = state.workspace_path {
                app.open_workspace(PathBuf::from(path), state.env_name.as_deref());
            }
            // reopen the file that was open, unless it is gone: the form we
            // just restored stays as an unsaved request in that case
            if let Some(file) = state
                .current_file
                .map(PathBuf::from)
                .filter(|p| p.is_file())
            {
                app.open_file(&file);
            }
        }
        app
    }

    pub fn notify(&mut self, text: impl Into<String>, is_error: bool) {
        self.toast = Some(Toast {
            text: text.into(),
            shown_at: self.time,
            is_error,
            undo: None,
        });
    }

    fn notify_undo(&mut self, text: String, undo: Undo) {
        self.notify(text, false);
        if let Some(toast) = &mut self.toast {
            toast.undo = Some(undo);
        }
    }

    fn undo(&mut self, undo: Undo) {
        match undo {
            Undo::History(entries, mut shown) => {
                if let Err(e) = storage::restore_history(entries) {
                    self.notify(e, true);
                }
                // the list is the truth for the UI; the disk catches up
                shown.append(&mut self.history);
                let excess = shown.len().saturating_sub(storage::MAX_HISTORY);
                shown.drain(..excess);
                self.history = shown;
                self.history_loaded = true;
            }
            Undo::Recent(mut old) => {
                // anything sent since the clear stays, after the old ones
                old.append(&mut self.recent);
                let excess = old.len().saturating_sub(storage::MAX_RECENT);
                old.drain(..excess);
                self.recent = old;
                self.save_recent();
            }
        }
    }

    /// Run `job` on a thread; its event (if any) is handled next frame.
    fn spawn(&self, job: impl FnOnce() -> Option<Event> + Send + 'static) {
        let (tx, ctx) = (self.tx.clone(), self.ctx.clone());
        std::thread::spawn(move || {
            if let Some(event) = job() {
                let _ = tx.send(event);
                ctx.request_repaint();
            }
        });
    }

    // -----------------------------------------------------------------------
    // Workspace
    // -----------------------------------------------------------------------

    pub fn workspace_name(&self) -> String {
        self.workspace
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn pick_folder(&self) {
        self.pick_folder_titled("Open a folder of requests");
    }

    fn pick_folder_titled(&self, title: &'static str) {
        self.spawn(move || {
            rfd::FileDialog::new()
                .set_title(title)
                .pick_folder()
                .map(Event::OpenFolder)
        });
    }

    /// Pick a file for a Multipart body; it arrives as a new `name=@path` line.
    pub fn attach_file(&self) {
        self.spawn(|| {
            rfd::FileDialog::new()
                .set_title("Attach a file")
                .pick_file()
                .map(Event::AttachFile)
        });
    }

    /// Open an imported collection and land the user inside it: its first
    /// request, with the environment it came with selected.
    pub fn finish_import(&mut self, folder: PathBuf, count: import::ImportCount) {
        let same = self.workspace.as_deref() == Some(folder.as_path());
        if same && count.requests == 0 {
            // just an environment: keep the open request, switch to it
            self.refresh_env_files();
            let index = self
                .env_files
                .iter()
                .position(|e| Some(e) == count.env.as_ref());
            self.selected_env = index;
            self.load_env();
        } else {
            self.open_workspace(folder.clone(), count.env.as_deref());
            // otherwise the editor still holds whatever was open before
            // the import, and the collection just imported is nowhere to be
            // seen
            if let Some(first) = workspace::all_requests(&count.folder)
                .into_iter()
                .find_map(|i| match i {
                    CollectionItem::Request { path, .. } => Some(path),
                    CollectionItem::Folder { .. } => None,
                })
            {
                self.open_file(&first);
            }
        }
        let mut summary = match (count.requests, &count.env) {
            (0, Some(env)) => format!("Imported environment {env}"),
            _ => plural_import(count.requests, count.envs),
        };
        if let Ok(sub) = count.folder.strip_prefix(&folder) {
            if !sub.as_os_str().is_empty() {
                // the workspace had files, so the import got its own folder
                summary += &format!(" into {}/", sub.display());
            }
        }
        // an import that silently drops a body or an auth scheme produces
        // requests that fail for no visible reason
        if count.skipped.is_empty() {
            self.notify(summary, false);
        } else {
            self.notify(
                format!(
                    "{summary}. Mercury could not carry over: {}",
                    count.skipped.join("; ")
                ),
                true,
            );
        }
    }

    pub fn open_workspace(&mut self, path: PathBuf, env: Option<&str>) {
        if !path.is_dir() {
            self.notify(format!("Folder not found: {}", path.display()), true);
            return;
        }
        self.env_files = workspace::env_files(&path);
        self.selected_env = env
            .and_then(|name| self.env_files.iter().position(|f| f == name))
            .or_else(|| self.env_files.iter().position(|f| f.contains(".dev")))
            .or((!self.env_files.is_empty()).then_some(0));

        let (tx, ctx) = (self.tx.clone(), self.ctx.clone());
        self.watcher = match workspace::watch(&path, move |result| {
            let _ = tx.send(match result {
                Ok(()) => Event::FilesChanged,
                Err(e) => Event::Toast(e, true),
            });
            ctx.request_repaint();
        }) {
            Ok(w) => Some(w),
            Err(e) => {
                self.notify(e, true);
                None
            }
        };
        self.workspace = Some(path);
        self.load_env();
        self.rebuild_tree();
    }

    /// Expand every folder between the workspace root and `file`, so a
    /// reopened request is visible in the tree instead of hidden three
    /// collapsed folders deep. Rescans only if a folder had to be opened:
    /// a rescan reads every visible request, and this runs on every click.
    pub fn expand_to(&mut self, file: &Path) {
        let Some(root) = self.workspace.clone() else {
            return;
        };
        let mut changed = false;
        let mut dir = file.parent();
        while let Some(path) = dir {
            if !path.starts_with(&root) {
                break;
            }
            changed |= self.expanded.insert(path.to_path_buf());
            if path == root {
                break;
            }
            dir = path.parent();
        }
        if changed {
            self.rebuild_tree();
        }
    }

    /// Forget the workspace without touching the request being edited: it
    /// becomes an unsaved request, which is what it now is on disk.
    fn close_workspace(&mut self) {
        self.watcher = None;
        self.workspace = None;
        self.tree.clear();
        self.expanded.clear();
        self.selected_folder = None;
        self.search.clear();
        self.tree_fully_loaded = false;
        self.env_files.clear();
        self.selected_env = None;
        self.env_vars.clear();
        self.env_shared.clear();
        self.current_file = None;
        self.saved_content = None;
    }

    pub fn rebuild_tree(&mut self) {
        if let Some(root) = &self.workspace {
            self.tree = workspace::scan(root, &self.expanded);
            self.tree_fully_loaded = false;
        }
    }

    pub fn env_name(&self) -> Option<&str> {
        self.selected_env
            .and_then(|i| self.env_files.get(i))
            .map(String::as_str)
    }

    /// A user-chosen switch (menu or ⌘E). It is announced: the only other
    /// signal is a chip in the corner, and ⌘E cycles in file-name order, so
    /// one press from dev can land on prod.
    pub fn select_env(&mut self, index: Option<usize>) {
        self.selected_env = index;
        // an error from the last send ("{{x}} isn't defined…") was about the
        // environment that's no longer selected
        self.request_error = None;
        self.load_env();
        let text = match self.env_name() {
            Some(name) => format!("Environment: {name}"),
            None => "No environment".to_string(),
        };
        self.notify(text, false);
    }

    /// "prod" when a production-looking environment is selected, so the Send
    /// button can say where the request is going.
    pub fn prod_target(&self) -> Option<&str> {
        self.env_name()
            .filter(|n| theme::is_prod(n))
            .map(|n| n.trim_start_matches(".env."))
    }

    fn load_env(&mut self) {
        self.env_vars.clear();
        self.env_shared.clear();
        let (Some(root), Some(name)) = (self.workspace.clone(), self.env_name()) else {
            return;
        };
        // dotenv's convention: `.env` holds what every environment shares,
        // and `.env.<name>` overrides it
        let mut files = vec![name.to_string()];
        if name != ".env" && self.env_files.iter().any(|f| f == ".env") {
            files.insert(0, ".env".into());
        }
        let layered = files.len() == 2;
        for file in files {
            match workspace::env_path(&root, &file)
                .and_then(|p| fs::read_to_string(p).map_err(|e| e.to_string()))
            {
                Ok(content) => {
                    let vars = vars::parse_env(&content);
                    if layered && file == ".env" {
                        self.env_shared.extend(vars.keys().cloned());
                    } else {
                        self.env_shared.retain(|k| !vars.contains_key(k));
                    }
                    self.env_vars.extend(vars);
                }
                Err(e) => self.notify(format!("Could not read {file}: {e}"), true),
            }
        }
    }

    pub fn start_import(&self, importer: &'static Importer) {
        let workspace = self.workspace.clone();
        self.spawn(move || {
            let files = rfd::FileDialog::new()
                .add_filter(format!("{} export", importer.name), importer.extensions)
                .set_title(format!(
                    "Select {} exports (collection and environments)",
                    importer.name
                ))
                .pick_files()?;
            let folder = workspace.or_else(|| {
                rfd::FileDialog::new()
                    .set_title("Choose where to save the imported collection")
                    .pick_folder()
            })?;
            // a collection and its environments are separate exports: take them together
            let mut total: Option<import::ImportCount> = None;
            let mut errors = Vec::new();
            for file in &files {
                match (importer.run)(file, &folder) {
                    Ok(count) => total = Some(import::merge(total, count)),
                    Err(e) => errors.push(match files.len() {
                        1 => e,
                        _ => format!(
                            "{}: {e}",
                            file.file_name().unwrap_or_default().to_string_lossy()
                        ),
                    }),
                }
            }
            let result = match total {
                Some(mut count) => {
                    count.skipped.extend(errors);
                    Ok((folder, count))
                }
                None => Err(errors.join("\n")),
            };
            Some(Event::Imported(result))
        });
    }

    // -----------------------------------------------------------------------
    // Request form
    // -----------------------------------------------------------------------

    fn form_request(&self) -> Request {
        Request {
            method: self.method,
            url: self.url.clone(),
            headers: self.headers_text.clone(),
            body: self.body_text.clone(),
        }
    }

    fn file_content(&self) -> String {
        RequestFile {
            method: self.method,
            url: self.url.clone(),
            headers: kv::headers_to_map(&self.headers_text),
            body: self.body_text.clone(),
            disabled_headers: kv::disabled_headers_to_map(&self.headers_text),
            extra: self.file_extra.clone(),
        }
        .to_json()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.current_file.is_some() && self.saved_content.as_deref() != Some(&self.file_content())
    }

    /// Replace the whole form (file, history, recent, new request).
    pub fn load_request(&mut self, request: Request, file: Option<PathBuf>) {
        self.method = request.method;
        self.url = request.url;
        self.headers_text = request.headers;
        self.body_text = request.body;
        self.query_params = kv::parse_query_params(&self.url);
        self.auth_stash = Default::default(); // another request's auth
                                              // Open on the tab that holds this request's content. The tab used to
                                              // be one global value, so a GET opened on an empty Body editor, or on
                                              // whatever the previous request was showing.
        self.tab = if !self.body_text.trim().is_empty()
            || matches!(
                self.method,
                HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
            ) {
            Tab::Body
        } else {
            Tab::Params
        };
        // a file reloaded in place is still the request that was sent
        self.sent_replaced |= file.is_none();
        self.current_file = file;
        self.file_extra.clear();
        self.saved_content = self.current_file.as_ref().map(|_| self.file_content());
        self.response = None;
        self.response_env = None;
        self.request_error = None;
        self.formatted_body = None;
        self.body_rows = None;
        self.response_tab = ResponseTab::Body;
    }

    /// Load a parsed request file into the form, as saved.
    fn load_file(&mut self, file: RequestFile, path: PathBuf) {
        self.load_request(
            Request {
                method: file.method,
                url: file.url,
                headers: kv::file_headers_text(&file.headers, &file.disabled_headers),
                body: file.body,
            },
            Some(path),
        );
        self.file_extra = file.extra;
        self.saved_content = Some(self.file_content());
    }

    pub fn open_file(&mut self, path: &Path) {
        self.autosave();
        let loaded = fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|c| RequestFile::from_json(&c));
        match loaded {
            Ok(file) => {
                self.load_file(file, path.to_path_buf());
                self.opened_content = self.saved_content.clone();
                // opened from ⌘K, history or a relaunch: show where it lives
                self.expand_to(path);
                self.last_folder = path.parent().map(Path::to_path_buf);
                self.opened_recently.retain(|p| p != path);
                self.opened_recently.insert(0, path.to_path_buf());
                self.opened_recently.truncate(5);
            }
            // the file name and the reason; the absolute path just wraps the
            // notification onto three lines
            Err(e) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                self.notify(format!("Could not open '{name}': {e}"), true)
            }
        }
    }

    fn save_file(&mut self) -> bool {
        let Some(path) = self.current_file.clone() else {
            return false;
        };
        let content = self.file_content();
        match fs::write(&path, &content) {
            Ok(()) => {
                self.saved_content = Some(content);
                self.last_save = self.time;
                true
            }
            Err(e) => {
                self.last_save = self.time; // retry after AUTOSAVE_SECS, not every frame
                self.notify(format!("Could not save: {e}"), true);
                false
            }
        }
    }

    pub fn autosave(&mut self) {
        if self.has_unsaved_changes() {
            self.save_file();
        }
    }

    fn new_request(&mut self) {
        self.autosave();
        self.load_request(Request::default(), None);
        self.focus("url_bar");
    }

    fn save(&mut self) {
        // an existing file: write now; the status bar says "Saved", a toast
        // on top of that was noise on every ⌘S
        if self.current_file.is_some() {
            self.save_file();
        } else if let Some(root) = self.target_folder() {
            self.open_dialog(Dialog::NewRequest(root, true), "");
        } else {
            // the picker's own title says why it opened; a success-ticked
            // toast beside it read as if something had been saved
            self.pick_folder_titled("Choose a folder to save requests in");
        }
    }

    /// Where "new request" / "new folder" / "save" land: the folder of the
    /// request opened last or the folder clicked last, else the root. It was
    /// the folder *clicked* last, which could be minutes and requests ago.
    fn target_folder(&self) -> Option<PathBuf> {
        self.last_folder
            .clone()
            .filter(|p| p.is_dir() && self.workspace.as_ref().is_some_and(|w| p.starts_with(w)))
            .or_else(|| self.workspace.clone())
    }

    pub fn select_folder(&mut self, path: &Path) {
        self.selected_folder = Some(path.to_path_buf());
        self.last_folder = Some(path.to_path_buf());
    }

    /// Back to the file as it was when it was opened.
    fn revert_file(&mut self) {
        let (Some(path), Some(content)) = (self.current_file.clone(), self.opened_content.clone())
        else {
            return;
        };
        let Ok(file) = RequestFile::from_json(&content) else {
            return;
        };
        let response = self.response.take();
        self.load_file(file, path);
        self.response = response;
        self.saved_content = None; // differs from disk until written
        if self.save_file() {
            self.notify("Reverted to the version you opened", false);
        }
    }

    pub fn duplicate(&mut self, path: &Path) {
        self.autosave(); // the copy is what's on screen, not the last save
        match workspace::duplicate(path) {
            Ok(copy) => {
                self.rebuild_tree();
                let name = copy.file_stem().unwrap_or_default().to_string_lossy();
                self.notify(format!("Duplicated as '{name}'"), false);
            }
            Err(e) => self.notify(e, true),
        }
    }

    /// Focus a widget at the start of the next frame. Commands run from the
    /// keyboard or the palette after this frame's widgets are drawn, and the
    /// find bar or a dialog field may not exist yet: focusing it now put a
    /// node that isn't there into the accessibility tree, which the platform
    /// adapters reject outright.
    fn focus(&mut self, id: &'static str) {
        self.pending_focus = Some(id);
    }

    // -----------------------------------------------------------------------
    // Sending
    // -----------------------------------------------------------------------

    pub fn send_request(&mut self) {
        if self.in_flight.is_some() {
            return;
        }
        if self.url.trim().is_empty() {
            self.notify("Enter a URL first", true);
            self.focus("url_bar");
            return;
        }
        // otherwise last send's "404 Not Found" sits over this send's result
        self.toast = None;
        // A URL with an undefined variable left in it fails as something
        // else entirely ("invalid international domain name"), or reaches
        // the server as %7B%7BUSER_ID%7D%7D. Say what's missing instead.
        let missing: Vec<String> = vars::extract(&self.url)
            .into_iter()
            .filter(|v| !self.env_vars.contains_key(v))
            .collect();
        if !missing.is_empty() {
            self.response = None;
            self.show_history = false;
            self.request_error = Some(missing_vars_message(&missing, self.env_name()));
            return;
        }
        let request = RequestFile {
            method: self.method,
            url: vars::substitute(&self.url, &self.env_vars),
            headers: kv::headers_to_map(&vars::substitute(&self.headers_text, &self.env_vars)),
            body: self.substituted_body(),
            ..Default::default()
        };
        self.request_counter += 1;
        let id = self.request_counter;
        self.in_flight = Some(id);
        self.sent = Some(self.form_request());
        self.sent_file = self.current_file.clone();
        self.sent_replaced = false;
        self.sent_env = self.env_name().map(str::to_string);
        let trusted = http::host(&request.url).is_some_and(|h| self.trusted_hosts.contains(&h));
        let client = if trusted {
            self.lenient
                .get_or_insert_with(|| http::client_with(false))
                .clone()
        } else {
            self.client.clone()
        };
        self.spawn(move || Some(Event::Response(id, http::execute(&client, &request))));
    }

    /// Soft cancel: the thread keeps running but its result is ignored.
    pub fn cancel_request(&mut self) {
        self.in_flight = None;
        self.sent = None;
        self.sent_file = None;
    }

    fn on_response(&mut self, result: Result<HttpResponse, String>) {
        self.in_flight = None;
        self.response_tab = ResponseTab::Body;
        match result {
            Ok(response) => {
                let request = self.record_history(response.clone(), None);
                let here = self.sent_on_screen();
                let summary = widgets::status_label(response.status, &response.status_text);
                // a green tick next to "404 Not Found" reads as success
                let failed = response.status >= 400;
                if here {
                    self.response = Some(std::sync::Arc::new(response));
                    self.response_env = self.sent_env.take();
                    self.request_error = None;
                    // the pane puts the status badge at the top of the
                    // response; a toast saying "200 OK" next to it is noise on
                    // every single send. Speak up only when history is
                    // covering the pane and the badge cannot be seen.
                    if self.show_history {
                        self.notify(summary, failed);
                    } else {
                        self.toast = None;
                    }
                } else {
                    // the user opened a different request while this was in
                    // flight; showing its response here would attribute it to
                    // the wrong request
                    self.notify(format!("{summary} — in History (⌘ H)"), failed);
                }
                // sent without a file, whatever is open now; sent again, it
                // moves to the top. After the toasts above: a failed save
                // must not be hidden by them.
                if self.sent_file.take().is_none() {
                    self.recent.retain(|r| r.request != request);
                    self.recent.push(RecentRequest {
                        request,
                        timestamp: storage::now(),
                    });
                    // the file is capped; the sidebar has to match it
                    let excess = self.recent.len().saturating_sub(storage::MAX_RECENT);
                    self.recent.drain(..excess);
                    self.save_recent();
                }
            }
            Err(e) => {
                // a failed send is in history too: it's the attempt you most
                // want to look at again
                self.record_history(http::HttpResponse::failed(), Some(e.clone()));
                if self.sent_on_screen() {
                    self.response = None;
                    // the response panel shows the full error with a Retry
                    // button, so a toast repeating it is just noise
                    self.show_history = false;
                    self.request_error = Some(e);
                } else {
                    // failed after the user moved to another request
                    self.notify(format!("Request failed: {e}"), true);
                }
            }
        }
        self.sent_file = None;
        self.formatted_body = None;
        self.body_rows = None;
    }

    /// The request on screen is still the one in flight.
    fn sent_on_screen(&self) -> bool {
        self.sent_file == self.current_file && !self.sent_replaced
    }

    /// Add the send that just finished to history; returns what was sent.
    fn record_history(
        &mut self,
        mut response: http::HttpResponse,
        error: Option<String>,
    ) -> Request {
        self.ensure_history_loaded();
        if response.body.len() > storage::HISTORY_BODY_LIMIT {
            response.body = String::new();
            response.pretty = None;
            response.response_type = http::ResponseType::TooLarge;
        }
        let request = self.sent.take().unwrap_or_else(|| self.form_request());
        let entry = HistoryEntry {
            timestamp: storage::now(),
            request: request.clone(),
            response,
            env: self.sent_env.clone(),
            file: self
                .sent_file
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            error,
        };
        self.history.push(HistorySummary::from(&entry));
        let excess = self.history.len().saturating_sub(storage::MAX_HISTORY);
        self.history.drain(..excess);
        // queued now (instantly visible to history reads), written
        // off the UI thread: it rewrites the whole history file
        storage::queue_history(entry);
        self.spawn(|| {
            storage::flush_history()
                .err()
                .map(|e| Event::Toast(e, true))
        });
        request
    }

    pub fn ensure_history_loaded(&mut self) {
        if !self.history_loaded {
            self.history = storage::load_history_summaries();
            self.history_loaded = true;
        }
    }

    pub fn open_history_entry(&mut self, timestamp: f64) {
        match storage::load_history_entry(timestamp) {
            Some(entry) => {
                // back on the file it was sent from, if it still exists,
                // instead of a detached "Untitled" copy
                let file = entry.file.map(PathBuf::from).filter(|f| f.is_file());
                if let Some(file) = &file {
                    self.open_file(file);
                }
                // a file that no longer parses stays where it was, and the
                // entry's response landed under whatever request was open
                match file.filter(|f| self.current_file.as_ref() == Some(f)) {
                    Some(_) => {}
                    None => {
                        self.autosave();
                        self.load_request(entry.request, None);
                    }
                }
                match entry.error {
                    Some(e) => {
                        self.response = None;
                        self.request_error = Some(e);
                    }
                    None => self.response = Some(std::sync::Arc::new(entry.response)),
                }
                self.response_env = entry.env;
                self.show_history = false;
            }
            None => self.notify("History entry no longer exists", true),
        }
    }

    pub fn clear_history(&mut self) {
        match storage::take_history() {
            Ok(entries) => {
                let shown = std::mem::take(&mut self.history);
                let what = plural(shown.len(), "history entry", "history entries");
                self.notify_undo(format!("Cleared {what}"), Undo::History(entries, shown));
            }
            Err(e) => self.notify(e, true),
        }
    }

    pub fn clear_recent(&mut self) {
        let old = std::mem::take(&mut self.recent);
        let what = plural(old.len(), "unsaved request", "unsaved requests");
        self.notify_undo(format!("Cleared {what}"), Undo::Recent(old));
        // after the toast above, so a failure replaces it
        self.save_recent();
    }

    /// Write the Unsaved list; a failure is said, or it is gone on relaunch.
    pub fn save_recent(&mut self) {
        if let Err(e) = storage::save_recent(&self.recent) {
            self.notify(e, true);
        }
    }

    /// The body as sent. A Multipart body is filled in field by field, so a
    /// variable can't add a field or turn text into a file upload.
    pub fn substituted_body(&self) -> String {
        // decided from the headers as they're sent, like the sender does: a
        // `Content-Type: {{CT}}` must not make the two disagree
        let sent = kv::headers_to_map(&vars::substitute(&self.headers_text, &self.env_vars));
        if http::is_multipart(&sent) {
            http::substitute_multipart(&self.body_text, &self.env_vars)
        } else {
            vars::substitute(&self.body_text, &self.env_vars)
        }
    }

    fn copy_as_curl(&mut self) {
        let sub = |s: &str| vars::substitute(s, &self.env_vars);
        let curl = curl::generate(
            self.method,
            &http::with_scheme(&sub(&self.url)),
            &kv::headers_to_map(&sub(&self.headers_text)),
            &self.substituted_body(),
        );
        self.ctx.copy_text(curl);
        self.notify("Copied as cURL", false);
    }

    pub fn save_response(&self) {
        let Some(response) = &self.response else {
            return;
        };
        let data = response
            .raw_bytes
            .clone()
            .unwrap_or_else(|| response.body.clone().into_bytes());
        let file_name = format!("response{}", widgets::extension_for(&response.content_type));
        self.spawn(move || {
            let path = rfd::FileDialog::new()
                .set_title("Save Response")
                .set_file_name(file_name)
                .save_file()?;
            Some(match fs::write(&path, data) {
                Ok(()) => Event::Toast(format!("Saved to {}", path.display()), false),
                Err(e) => Event::Toast(format!("Could not save response: {e}"), true),
            })
        });
    }

    // -----------------------------------------------------------------------
    // Events
    // -----------------------------------------------------------------------

    fn handle_events(&mut self) {
        let mut files_changed = false;
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Response(id, result) if self.in_flight == Some(id) => {
                    self.on_response(result)
                }
                Event::Response(..) => {} // cancelled
                Event::OpenFolder(path) => self.open_workspace(path, None),
                Event::AttachFile(path) => {
                    let name = path.file_stem().unwrap_or_default().to_string_lossy();
                    let line = format!("{name}=@{}", path.display());
                    if !self.body_text.trim().is_empty() {
                        self.body_text.push('\n');
                    }
                    self.body_text.push_str(&line);
                }
                Event::Imported(Ok((folder, count))) => self.finish_import(folder, count),
                Event::Imported(Err(e)) => self.notify(e, true),
                Event::FilesChanged => files_changed = true,
                Event::Toast(text, is_error) => self.notify(text, is_error),
            }
        }
        if files_changed {
            self.on_files_changed();
        }
    }

    /// External edits: refresh the tree and reload the open file if it
    /// changed on disk (unless the user has unsaved edits).
    fn on_files_changed(&mut self) {
        // the folder itself can go away: keep claiming to have it open and
        // every button in the sidebar lies
        if self.workspace.as_deref().is_some_and(|root| !root.is_dir()) {
            self.close_workspace();
            self.notify("The workspace folder is gone", true);
            return;
        }
        self.rebuild_tree();
        self.refresh_env_files();
        let Some(path) = self.current_file.clone() else {
            return;
        };
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let disk = match fs::read_to_string(&path) {
            Ok(disk) => disk,
            // deleted, renamed, or not on the branch git switched to: what's
            // on screen (unsaved edits too) stays, as an unsaved request
            Err(_) if !workspace::taken(&path) => {
                let text = format!("'{name}' was deleted on disk; it's still open, unsaved");
                self.current_file = None;
                self.saved_content = None;
                self.notify(text, true);
                return;
            }
            Err(e) => {
                let text = format!("Could not read '{name}': {e}");
                self.notify(text, true);
                return;
            }
        };
        // compare parsed content, not text: hand-written files never match
        // our formatting byte-for-byte and would reload on every fs event
        let parse = |s: &str| RequestFile::from_json(s).ok();
        if parse(&disk) == self.saved_content.as_deref().and_then(parse) {
            return;
        }
        if self.has_unsaved_changes() {
            self.notify("File changed on disk; keeping your unsaved edits", true);
        } else {
            // the same request, newer: the tabs and the response stay put
            let keep = (
                self.response.take(),
                self.response_env.take(),
                self.tab,
                self.response_tab,
            );
            self.open_file(&path);
            (
                self.response,
                self.response_env,
                self.tab,
                self.response_tab,
            ) = keep;
        }
    }

    /// Re-list env files, keeping the selection by name, and reload values
    /// (the selected file may be what changed).
    fn refresh_env_files(&mut self) {
        let Some(root) = &self.workspace else {
            return;
        };
        let selected = self.env_name().map(str::to_string);
        self.env_files = workspace::env_files(root);
        self.selected_env = selected
            .as_ref()
            .and_then(|name| self.env_files.iter().position(|f| f == name));
        self.load_env();
        // deleted or renamed outside: say so, or requests quietly go out
        // without its variables
        if let (Some(name), None) = (selected, self.selected_env) {
            self.notify(format!("{name} is gone: no environment selected"), true);
        }
    }

    // -----------------------------------------------------------------------
    // Dialogs
    // -----------------------------------------------------------------------

    pub fn open_dialog(&mut self, dialog: Dialog, text: &str) {
        // otherwise focus stays wherever it was (the URL bar, after ⌘N) and
        // the name field silently swallows nothing
        if !matches!(dialog, Dialog::Delete(..) | Dialog::Shortcuts) {
            self.focus("modal_input");
        }
        self.dialog = Some(dialog);
        self.dialog_text = text.to_string();
    }

    pub fn ask_delete(&mut self, path: &Path) {
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        let is_dir = path.is_dir();
        let counts = if is_dir {
            workspace::count_contents(path)
        } else {
            (1, 0)
        };
        let message = delete_message(&name, is_dir, counts);
        self.open_dialog(Dialog::Delete(path.to_path_buf(), message), "");
    }

    /// "New request" vs "New request in users" — the destination matters.
    fn in_folder(&self, parent: &Path, what: &str) -> String {
        match self.workspace.as_deref() {
            Some(root) if parent != root => {
                let rest = parent.strip_prefix(root).unwrap_or(parent);
                format!("{what} in {}", rest.display())
            }
            _ => what.to_string(),
        }
    }

    fn show_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.dialog.clone() else {
            return;
        };
        let titles = match &dialog {
            Dialog::NewRequest(parent, saving) => self.in_folder(
                parent,
                if *saving {
                    "Save request"
                } else {
                    "New request"
                },
            ),
            Dialog::NewFolder(parent) => self.in_folder(parent, "New folder"),
            _ => String::new(),
        };
        let text = &mut self.dialog_text;
        let action = match &dialog {
            Dialog::NewRequest(_, saving) => input_modal(
                ctx,
                &titles,
                "Request name",
                if *saving { "Save" } else { "Create" },
                text,
            ),
            Dialog::NewFolder(_) => input_modal(ctx, &titles, "Folder name", "Create", text),
            Dialog::Rename(_) => input_modal(ctx, "Rename", "New name", "Rename", text),
            Dialog::Delete(path, message) => {
                let title = if path.is_dir() {
                    "Delete folder"
                } else {
                    "Delete request"
                };
                confirm_modal(ctx, title, message, title)
            }
            Dialog::Shortcuts => shortcuts_modal(ctx),
            Dialog::NewEnv => {
                input_modal(ctx, "New environment", "Name, e.g. staging", "Create", text)
            }
            Dialog::EditEnv(file) => {
                let (body, bulk) = (&mut self.env_edit_text, &mut self.env_edit_bulk);
                widgets::modal(ctx, &format!("Edit {file}"), |ui| {
                    widgets::key_value_editor(
                        ui,
                        body,
                        "=",
                        bulk,
                        "baseUrl=https://api.example.com\ntoken=…",
                    );
                    ui.add_space(Space::MD);
                    ui.add(
                        egui::Label::new(widgets::faint(
                            "Requests use these as {{name}}. Saved as a plain file in the workspace folder.",
                        ))
                        .wrap(),
                    );
                    ui.add_space(Space::XL);
                    widgets::modal_buttons(ui, "Save", false)
                })
            }
        };
        match action {
            ModalAction::Open => return,
            ModalAction::Close => {}
            ModalAction::Confirm if matches!(dialog, Dialog::NewEnv) => {
                let name = self.dialog_text.trim().to_string();
                match self.create_env(&name) {
                    // straight into the editor: the file is only a start
                    Ok(file) => return self.edit_env(&file),
                    // stays open on what was typed, to fix rather than retype
                    Err(e) => return self.notify(e, true),
                }
            }
            ModalAction::Confirm => {
                let name = self.dialog_text.trim().to_string();
                let keep_open = !matches!(dialog, Dialog::Delete(..));
                match self.confirm_dialog(dialog, &name) {
                    Ok(msg) => self.notify(msg, false),
                    Err(e) if keep_open => return self.notify(e, true),
                    Err(e) => self.notify(e, true),
                }
            }
        }
        self.dialog = None;
    }

    fn confirm_dialog(&mut self, dialog: Dialog, name: &str) -> Result<&'static str, String> {
        match dialog {
            Dialog::NewRequest(parent, saving) => {
                let path = workspace::create_request(&parent, name, &self.file_content())?;
                // it's saved now, so it no longer belongs in Unsaved; a
                // different method to the same URL is another request
                let (method, url) = (self.method, self.url.clone());
                self.recent
                    .retain(|r| r.request.method != method || r.request.url != url);
                self.expanded.insert(parent);
                self.rebuild_tree();
                self.open_file(&path);
                if let Err(e) = storage::save_recent(&self.recent) {
                    // the request is saved: said, but the dialog is done
                    self.dialog = None;
                    return Err(e);
                }
                Ok(if saving { "Saved" } else { "Request created" })
            }
            Dialog::NewFolder(parent) => {
                workspace::create_folder(&parent, name)?;
                self.expanded.insert(parent);
                self.rebuild_tree();
                Ok("Folder created")
            }
            Dialog::Rename(path) => {
                let new_path = workspace::rename(&path, name)?;
                // every path into the old name follows it: the open file, a
                // send in flight, the expanded folders, ⌘K's recent list
                let remap = |p: &mut PathBuf| {
                    if let Ok(rest) = p.strip_prefix(&path) {
                        *p = if rest.as_os_str().is_empty() {
                            new_path.clone()
                        } else {
                            new_path.join(rest)
                        };
                    }
                };
                self.current_file
                    .iter_mut()
                    .chain(&mut self.sent_file)
                    .chain(&mut self.last_folder)
                    .chain(&mut self.selected_folder)
                    .chain(&mut self.opened_recently)
                    .for_each(remap);
                self.expanded = self
                    .expanded
                    .drain()
                    .map(|mut p| {
                        remap(&mut p);
                        p
                    })
                    .collect();
                self.rebuild_tree();
                Ok("Renamed")
            }
            Dialog::Delete(path, _) => {
                workspace::delete(&path)?;
                if self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.starts_with(&path))
                {
                    self.load_request(Request::default(), None);
                }
                self.rebuild_tree();
                Ok("Deleted")
            }
            Dialog::Shortcuts | Dialog::NewEnv => Ok(""),
            Dialog::EditEnv(file) => {
                let root = self.workspace.clone().ok_or("No folder is open")?;
                let text = kv::format_lines(&kv::parse_lines(&self.env_edit_text, "="), "=");
                fs::write(workspace::env_path(&root, &file)?, format!("{text}\n"))
                    .map_err(|e| format!("Could not save {file}: {e}"))?;
                self.refresh_env_files();
                self.request_error = None;
                Ok("Environment saved")
            }
        }
    }

    /// A new environment, seeded with the variables the open request uses,
    /// then selected. Returns its file name.
    fn create_env(&mut self, name: &str) -> Result<String, String> {
        let root = self.workspace.clone().ok_or("Open a folder first")?;
        let used: std::collections::BTreeSet<String> =
            [&self.url, &self.headers_text, &self.body_text]
                .iter()
                .flat_map(|t| vars::extract(t))
                .collect();
        let content: String = used.iter().map(|k| format!("{k}=\n")).collect();
        let file = workspace::create_env(&root, name, &content)?;
        self.refresh_env_files();
        let index = self.env_files.iter().position(|f| *f == file);
        self.select_env(index);
        Ok(file)
    }

    /// Open the editor on an environment file of the open workspace.
    /// A response value kept for the next request: written into the selected
    /// environment file (other lines and comments untouched) and live at once.
    pub fn save_variable(&mut self, key: &str, value: &str) {
        let (Some(root), Some(file)) =
            (self.workspace.clone(), self.env_name().map(str::to_string))
        else {
            return;
        };
        let saved = workspace::env_path(&root, &file).and_then(|path| {
            let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            fs::write(&path, vars::set_env_var(&text, key, value)).map_err(|e| e.to_string())
        });
        match saved {
            Ok(()) => {
                self.load_env();
                self.notify(format!("Saved {{{{{key}}}}} in {file}"), false);
            }
            Err(e) => self.notify(format!("Could not save {file}: {e}"), true),
        }
    }

    pub fn edit_env(&mut self, file: &str) {
        let Some(root) = &self.workspace else {
            return;
        };
        match workspace::env_path(root, file)
            .and_then(|p| fs::read_to_string(p).map_err(|e| e.to_string()))
        {
            // a BOM glued to the first key made a `# comment` an enabled row
            Ok(text) => self.env_edit_text = text.trim_start_matches('\u{feff}').to_string(),
            Err(e) => return self.notify(e, true),
        }
        self.env_edit_bulk = false;
        self.dialog = Some(Dialog::EditEnv(file.to_string()));
    }

    // -----------------------------------------------------------------------
    // Commands
    // -----------------------------------------------------------------------

    /// Run one [`Action`], whatever triggered it (key, palette or click).
    pub fn run(&mut self, action: Action) {
        match action {
            Action::Send => self.send_request(),
            Action::New => self.new_request(),
            Action::NewFolder => match self.target_folder() {
                Some(folder) => self.open_dialog(Dialog::NewFolder(folder), ""),
                None => self.pick_folder(),
            },
            Action::Save => self.save(),
            Action::OpenFolder => self.pick_folder(),
            Action::Palette => self.open_palette(),
            Action::FocusUrl => self.focus("url_bar"),
            Action::NextEnv => {
                let next = match self.selected_env {
                    None if !self.env_files.is_empty() => Some(0),
                    Some(i) if i + 1 < self.env_files.len() => Some(i + 1),
                    _ => None,
                };
                self.select_env(next);
            }
            Action::History => self.show_history = !self.show_history,
            Action::ToggleRaw => self.raw_view = !self.raw_view,
            Action::FindInResponse => {
                // the find bar only exists over a text body
                let text = self.response.as_ref().is_some_and(|r| {
                    matches!(
                        r.response_type,
                        http::ResponseType::Json
                            | http::ResponseType::Xml
                            | http::ResponseType::Html
                            | http::ResponseType::PlainText
                    )
                });
                if text && !self.show_history {
                    self.response_tab = ResponseTab::Body;
                    self.find.get_or_insert_with(String::new);
                    self.focus("response_find");
                }
            }
            Action::CopyCurl => self.copy_as_curl(),
            Action::FormatBody => {
                // it used to do nothing at all for a body it couldn't parse
                match serde_json::from_str::<serde::de::IgnoredAny>(&self.body_text) {
                    Ok(_) => self.body_text = http::format_json(&self.body_text),
                    Err(e) => self.notify(format!("Not valid JSON: {e}"), true),
                }
                self.tab = Tab::Body;
            }
            Action::FocusMode => self.focus_mode = !self.focus_mode,
            Action::ToggleTheme => {
                self.theme_mode = if theme::is_dark() {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                };
            }
            Action::ImportPostman => self.start_import(&POSTMAN),
            Action::ImportInsomnia => self.start_import(&INSOMNIA),
            Action::RenameRequest
            | Action::DuplicateRequest
            | Action::DeleteRequest
            | Action::RevertFile => {
                let Some(file) = self.current_file.clone() else {
                    self.notify("Open a saved request first", false);
                    return;
                };
                match action {
                    Action::RenameRequest => {
                        let name = file.file_name().unwrap_or_default().to_string_lossy();
                        self.open_dialog(Dialog::Rename(file.clone()), &name);
                    }
                    Action::DuplicateRequest => self.duplicate(&file),
                    Action::DeleteRequest => self.ask_delete(&file),
                    _ => self.revert_file(),
                }
            }
            Action::Docs => {
                let _ = open::that(DOCS_URL);
            }
            Action::Help => {
                self.dialog = match self.dialog {
                    Some(Dialog::Shortcuts) => None,
                    _ => Some(Dialog::Shortcuts),
                }
            }
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        // the palette owns the keyboard while it is open
        if self.palette.is_some() {
            return;
        }
        let typing = ctx.wants_keyboard_input();
        let (actions, escape): (Vec<Action>, bool) = ctx.input(|i| {
            let actions = SHORTCUTS
                .iter()
                .filter(|s| s.pressed(i, typing))
                .map(|s| s.action)
                .collect();
            (actions, i.key_pressed(Key::Escape))
        });

        if escape && self.dialog.is_none() {
            // innermost first: the find bar is the thing the user is looking
            // at when they press it
            if self.find.is_some() {
                self.find = None;
            } else if self.in_flight.is_some() {
                self.cancel_request();
            } else {
                self.search.clear();
            }
        }
        for action in actions {
            self.run(action);
        }
    }

    // -----------------------------------------------------------------------
    // Chrome: top bar, status bar, toast
    // -----------------------------------------------------------------------

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel")
            .exact_height(Layout::TOPBAR_HEIGHT)
            .frame(widgets::bar_frame())
            .show(ctx, |ui| {
                ui.painter().hline(
                    ui.max_rect().expand(Space::LG).x_range(),
                    ui.max_rect().bottom() - 0.5,
                    Stroke::new(1.0_f32, theme().border),
                );
                // The top bar is the title bar on macOS: its empty space
                // drags the window and a double-click zooms it. Registered
                // before the controls, so they still get their own clicks.
                let title = ui.interact(
                    ui.max_rect(),
                    egui::Id::new("title_bar"),
                    // not FOCUSABLE: a window-drag area is not a Tab stop
                    Sense::CLICK | Sense::DRAG,
                );
                if title.drag_started_by(egui::PointerButton::Primary) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if title.double_clicked() {
                    let zoomed = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!zoomed));
                }
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = Space::SM;
                    ui.add_space(Layout::WINDOW_BUTTONS);
                    self.workspace_menu(ui);
                    self.breadcrumb(ui);

                    // The right-hand group is laid out left to right after a
                    // spacer, not in a right-to-left layout: that reversed
                    // the Tab order (… → theme → environment).
                    let (env, _) = self.env_label();
                    let right = widgets::chip_width(ui, &env)
                        + Space::SM
                        + Layout::ICON_BUTTON * 2.0
                        + ui.spacing().item_spacing.x * 3.0;
                    ui.add_space((ui.available_width() - right).max(0.0));
                    self.env_menu(ui);
                    ui.add_space(Space::SM);
                    let (what, tip) = if theme::is_dark() {
                        (Icon::Sun, "Switch to light (⌘ D)")
                    } else {
                        (Icon::Moon, "Switch to dark (⌘ D)")
                    };
                    if icon_button(ui, what, tip).clicked() {
                        self.run(Action::ToggleTheme);
                    }
                    self.help_menu(ui);
                });
            });
    }

    /// Folder chip on the far left: the workspace, and how to change it.
    fn workspace_menu(&mut self, ui: &mut egui::Ui) {
        let t = theme();
        let open = self.workspace.is_some();
        let name = if open {
            self.workspace_name()
        } else {
            "Open folder".to_string()
        };
        let mut chip = widgets::chip(ui, Icon::Folder, &name, t.text, open);
        if let Some(path) = &self.workspace {
            // the full path used to sit in the status bar, truncated
            chip = chip.on_hover_text(path.display().to_string());
        }
        popup_menu(ui, &chip, Layout::MENU_WIDTH, |ui| {
            if menu_item(ui, Some(Icon::Folder), "Open folder…", "⌘ O") {
                self.pick_folder();
            }
            if open && menu_item(ui, Some(Icon::Plus), "New folder", "") {
                self.run(Action::NewFolder);
            }
            widgets::divider(ui);
            if menu_item(ui, Some(Icon::Package), "Import from Postman…", "") {
                self.start_import(&POSTMAN);
            }
            if menu_item(ui, Some(Icon::Package), "Import from Insomnia…", "") {
                self.start_import(&INSOMNIA);
            }
            if open {
                widgets::divider(ui);
                if menu_item(ui, Some(Icon::Copy), "Copy folder path", "") {
                    let path = self.workspace.clone().unwrap_or_default();
                    ui.ctx().copy_text(path.display().to_string());
                }
            }
        });
    }

    /// folder / folder / METHOD request •
    fn breadcrumb(&mut self, ui: &mut egui::Ui) {
        let t = theme();
        let sep = |ui: &mut egui::Ui| {
            ui.label(muted("/").color(t.text_faint));
        };
        if self.current_file.is_none() {
            if self.workspace.is_some() {
                sep(ui);
                ui.label(muted("Untitled"));
            }
            // the only way to keep a request used to be ⌘S, which nothing on
            // screen mentioned once the first-run tips were gone
            if !self.url.trim().is_empty()
                && widgets::link(ui, muted("Save…").color(t.accent))
                    .on_hover_text("Save this request as a file (⌘ S)")
                    .clicked()
            {
                self.run(Action::Save);
            }
            return;
        }
        let (Some(root), Some(file)) = (&self.workspace, &self.current_file) else {
            return;
        };
        if let Some(folder) = file.strip_prefix(root).ok().and_then(Path::parent) {
            for part in folder.iter() {
                sep(ui);
                ui.label(muted(widgets::truncate(
                    &part.to_string_lossy(),
                    CRUMB_CHARS,
                )));
            }
        }
        sep(ui);
        ui.label(widgets::method_text(self.method));
        let name = file
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        ui.label(strong(widgets::truncate(&name, CRUMB_CHARS)));
        if self.has_unsaved_changes() {
            widgets::glyph(ui, Icon::Dot, 9.0, t.warning)
                .on_hover_text("Unsaved changes (auto-saves in a few seconds)");
        }
    }

    fn env_label(&self) -> (String, egui::Color32) {
        let t = theme();
        match (self.env_name(), &self.workspace) {
            (Some(name), _) => (name.to_string(), t.env(name)),
            (None, Some(_)) => ("No environment".to_string(), t.text_muted),
            (None, None) => ("No environment".to_string(), t.text_faint),
        }
    }

    fn env_menu(&mut self, ui: &mut egui::Ui) {
        let (name, color) = self.env_label();
        let chip = widgets::chip(ui, Icon::Layers, &name, color, self.workspace.is_some());
        let (mut picked, mut edit, mut create) = (None, None, false);
        popup_menu(ui, &chip, Layout::MENU_WIDTH, |ui| {
            let check = self.selected_env.is_none().then_some(Icon::Check);
            if menu_item(ui, check, "No environment", "") {
                picked = Some(None);
            }
            // scrolls: two dozen files ran off the window, taking Edit and
            // New environment… with them
            let max_height = ui.ctx().content_rect().height() * 0.6;
            egui::ScrollArea::vertical()
                .max_height(max_height)
                // or it opens at the height it last had and grows a frame at a time
                .min_scrolled_height(max_height)
                .show(ui, |ui| {
                    for (i, env) in self.env_files.iter().enumerate() {
                        let check = (self.selected_env == Some(i)).then_some(Icon::Check);
                        if menu_item(ui, check, env, "") {
                            picked = Some(Some(i));
                        }
                    }
                });
            if self.workspace.is_some() {
                widgets::divider(ui);
                if let Some(name) = self.env_name().map(str::to_string) {
                    if menu_item(ui, Some(Icon::Pencil), &format!("Edit {name}…"), "") {
                        edit = Some(name);
                    }
                }
                if menu_item(ui, Some(Icon::Plus), "New environment…", "") {
                    create = true;
                }
            }
        });
        if let Some(index) = picked {
            self.select_env(index);
        }
        if let Some(file) = edit {
            self.edit_env(&file);
        }
        if create {
            self.open_dialog(Dialog::NewEnv, "");
        }
    }

    fn help_menu(&mut self, ui: &mut egui::Ui) {
        let trigger = icon_button(ui, Icon::Ellipsis, "More");
        popup_menu(ui, &trigger, Layout::MENU_WIDTH, |ui| {
            if menu_item(ui, Some(Icon::Search), "Command palette", "⌘ K") {
                self.run(Action::Palette);
            }
            if menu_item(ui, Some(Icon::Keyboard), "Keyboard shortcuts", "?") {
                self.dialog = Some(Dialog::Shortcuts);
            }
            if menu_item(ui, Some(Icon::PanelLeft), "Hide / show sidebar", "⌘ ⇧ F") {
                self.run(Action::FocusMode);
            }
            widgets::divider(ui);
            for (what, text, url) in [
                (Icon::ExternalLink, "Documentation", DOCS_URL),
                (Icon::Download, "Check for updates", RELEASES_URL),
                (Icon::Alert, "Report an issue", ISSUES_URL),
            ] {
                if menu_item(ui, Some(what), text, "") {
                    let _ = open::that(url);
                }
            }
        });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        let t = theme();
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(Layout::STATUS_BAR_HEIGHT)
            .frame(widgets::bar_frame())
            .show(ctx, |ui| {
                ui.painter().hline(
                    ui.max_rect().expand(Space::LG).x_range(),
                    ui.max_rect().top() + 0.5,
                    Stroke::new(1.0_f32, t.border),
                );
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = Space::SM;
                    // status, not a second copy of the folder chip: is the
                    // open file on disk yet?
                    if self.workspace.is_none() {
                        ui.label(faint("No folder open — requests are files in a folder"));
                    } else if self.has_unsaved_changes() {
                        ui.label(faint("Unsaved changes · saves automatically"));
                    } else if self.current_file.is_some() {
                        ui.label(faint("Saved"));
                    }

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = Space::SM;
                        if widgets::link(ui, faint("Commands"))
                            .on_hover_text("Press ⌘K")
                            .clicked()
                        {
                            self.run(Action::Palette);
                        }
                        key_combo(ui, "⌘ K");
                        if self.focus_mode {
                            ui.add_space(Space::MD);
                            ui.label(faint("Sidebar hidden · ⌘ ⇧ F to show").color(t.accent));
                        }
                    });
                });
            });
    }

    /// Toast floats above the status bar so it never shifts the layout.
    fn show_toast(&mut self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else {
            return;
        };
        if self.time - toast.shown_at > widgets::TOAST_SECS {
            self.toast = None;
            return;
        }
        let (text, shown_at, is_error) = (toast.text.clone(), toast.shown_at, toast.is_error);
        let action = toast.undo.as_ref().map(|_| "Undo");
        let state = egui::Area::new(egui::Id::new("toast"))
            .anchor(
                egui::Align2::RIGHT_BOTTOM,
                Vec2::new(-Space::XL, -(Layout::STATUS_BAR_HEIGHT + Space::LG)),
            )
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                widgets::toast(ui, &text, shown_at, is_error, action)
            })
            .inner;
        if state.acted {
            if let Some(undo) = self.toast.take().and_then(|t| t.undo) {
                self.undo(undo);
            }
        } else if !state.alive {
            self.toast = None;
        }
    }
}

/// ⌘⇧C is Copy as cURL, not "copy the selected text". egui-winit turns any
/// ⌘C, with or without ⇧, into `Event::Copy` and never reports the C key,
/// and selected labels and fields answer that event at the end of the frame,
/// after Mercury, so their text won the clipboard. Take the event out before
/// any widget sees it. Plain ⌘C is untouched.
fn take_copy_as_curl(ctx: &egui::Context) -> bool {
    ctx.input_mut(|i| {
        let hit = i.modifiers.command
            && i.modifiers.shift
            && i.events.iter().any(|e| matches!(e, egui::Event::Copy));
        if hit {
            i.events.retain(|e| !matches!(e, egui::Event::Copy));
        }
        hit
    })
}

/// "1 request" / "3 requests" — "1 requests" reads like a bug.
fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Why a URL with undefined variables wasn't sent, and where to fix it.
fn missing_vars_message(missing: &[String], env: Option<&str>) -> String {
    let braced: Vec<String> = missing.iter().map(|v| format!("{{{{{v}}}}}")).collect();
    let names = match braced.as_slice() {
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
        [] => String::new(),
    };
    let (is, it) = if missing.len() == 1 {
        ("isn't", "it")
    } else {
        ("aren't", "them")
    };
    match env {
        Some(env) => format!(
            "The URL wasn't sent: {names} {is} defined in {env}. Add {it} with Edit {env}… in the environment menu, or pick another environment."
        ),
        None => format!(
            "The URL wasn't sent: {names} {is} defined, because no environment is selected. Pick one, or make one with New environment… in the environment menu."
        ),
    }
}

fn plural_import(requests: usize, envs: usize) -> String {
    format!(
        "Imported {}, {}",
        plural(requests, "request", "requests"),
        plural(envs, "environment", "environments")
    )
}

/// A folder delete is `rm -rf`: say what goes with it, including files
/// that aren't requests (fixtures, READMEs) and that the user may not expect.
fn delete_message(name: &str, is_dir: bool, (requests, others): (usize, usize)) -> String {
    if !is_dir {
        return format!("Delete the request '{name}'?");
    }
    let mut message = if requests == 0 {
        format!("Delete the folder '{name}'?")
    } else {
        format!(
            "Delete '{name}' and the {} in it?",
            plural(requests, "request", "requests")
        )
    };
    if others > 0 {
        message += &format!(
            " It also holds {}.",
            plural(others, "other file", "other files")
        );
    }
    message
}

fn shortcuts_modal(ctx: &egui::Context) -> ModalAction {
    modal(ctx, "Keyboard shortcuts", |ui| {
        let rows = SHORTCUTS
            .iter()
            .map(|s| (s.keys, s.label))
            .chain([ESCAPE_HELP]);
        // scrolls: at the minimum window the list pushed the title and Close
        // off the screen
        let max_height = ui.ctx().content_rect().height() - 240.0;
        egui::ScrollArea::vertical()
            .max_height(max_height)
            .show(ui, |ui| {
                for (keys, text) in rows {
                    ui.horizontal(|ui| {
                        ui.set_min_height(Layout::ROW_HEIGHT);
                        ui.label(label(text));
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            key_combo(ui, keys);
                        });
                    });
                }
            });
        ui.add_space(Space::LG);
        ui.label(faint("On Windows and Linux, ⌘ is Ctrl."));
        ui.add_space(Space::XL);
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if widgets::ghost_button(ui, "Close").clicked() {
                ModalAction::Close
            } else {
                ModalAction::Open
            }
        })
        .inner
    })
}

impl MercuryApp {
    /// One frame. Separate from `eframe::App::update` so tests can drive it.
    pub fn frame(&mut self, ctx: &egui::Context) {
        self.ctx = ctx.clone();
        if !theme::ensure_installed(ctx) {
            ctx.request_repaint();
            return;
        }
        self.time = ctx.input(|i| i.time);
        if let Some(id) = self.pending_focus.take() {
            ctx.memory_mut(|m| m.request_focus(egui::Id::new(id)));
        }
        if take_copy_as_curl(ctx) {
            self.run(Action::CopyCurl);
        }
        theme::set_dark(
            ctx,
            match self.theme_mode {
                ThemeMode::System => ctx.system_theme().is_none_or(|t| t == egui::Theme::Dark),
                ThemeMode::Dark => true,
                ThemeMode::Light => false,
            },
        );
        self.handle_events();

        if self.has_unsaved_changes() && self.time - self.last_save > AUTOSAVE_SECS {
            self.save_file();
        }

        self.top_bar(ctx);
        self.status_bar(ctx);
        if !self.focus_mode {
            self.sidebar(ctx);
        }
        self.response_panel(ctx);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme().bg)
                    .inner_margin(Margin::same(Space::LG as i8)),
            )
            .show(ctx, |ui| self.editor(ui));

        self.show_toast(ctx);
        // keys after widgets so text fields have claimed focus this frame
        self.handle_keys(ctx);
        self.show_dialog(ctx);
        self.command_palette(ctx);
    }
}

impl eframe::App for MercuryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.autosave();
        // nowhere left to show it: the window is closing
        if let Err(e) = storage::save_state(&AppState {
            workspace_path: self
                .workspace
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            method: self.method,
            url: self.url.clone(),
            headers_text: self.headers_text.clone(),
            body_text: self.body_text.clone(),
            selected_tab: self.tab as usize,
            env_name: self.env_name().map(str::to_string),
            theme: self.theme_mode,
            current_file: self
                .current_file
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
        }) {
            eprintln!("{e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_variables_say_what_and_where() {
        let one = missing_vars_message(&["baseUrl".into()], Some(".env.dev"));
        assert!(
            one.contains("{{baseUrl}} isn't defined in .env.dev. Add it with Edit .env.dev…"),
            "{one}"
        );
        let two = missing_vars_message(&["a".into(), "b".into()], None);
        assert!(
            two.contains("{{a}} and {{b}} aren't defined, because no environment is selected"),
            "{two}"
        );
        assert!(two.contains("New environment…"));
    }

    #[test]
    fn shortcuts_are_unique() {
        for (i, a) in SHORTCUTS.iter().enumerate() {
            for b in &SHORTCUTS[i + 1..] {
                let same = a.key == b.key && a.command == b.command && a.shift == b.shift;
                assert!(!same, "{} and {} share a key binding", a.label, b.label);
                assert_ne!(a.action, b.action);
            }
        }
    }

    /// A folder delete is permanent and takes everything inside with it, so
    /// the confirmation has to say what that is.
    #[test]
    fn delete_message_counts_what_goes() {
        assert_eq!(
            delete_message("users", true, (3, 2)),
            "Delete 'users' and the 3 requests in it? It also holds 2 other files."
        );
        assert_eq!(
            delete_message("users", true, (1, 0)),
            "Delete 'users' and the 1 request in it?"
        );
        assert_eq!(
            delete_message("empty", true, (0, 1)),
            "Delete the folder 'empty'? It also holds 1 other file."
        );
        assert_eq!(
            delete_message("list-users", false, (1, 0)),
            "Delete the request 'list-users'?"
        );
    }

    /// CLAUDE.md: a change to SHORTCUTS updates the README and the docs
    /// site's table in the same PR. This holds both to it, row for row.
    #[test]
    fn shortcut_tables_in_the_docs_match_the_app() {
        let expected: Vec<String> = SHORTCUTS
            .iter()
            .map(|s| (s.keys, s.label))
            .chain([ESCAPE_HELP])
            .map(|(keys, label)| format!("| `{keys}` | {label} |"))
            .collect();
        for (doc, text) in [
            ("README.md", include_str!("../../README.md")),
            (
                "website/docs/reference/keyboard-shortcuts.md",
                include_str!("../../website/docs/reference/keyboard-shortcuts.md"),
            ),
        ] {
            let rows: Vec<&str> = text
                .lines()
                .skip_while(|l| !l.starts_with(&expected[0]))
                .take_while(|l| l.starts_with("| `"))
                .collect();
            assert_eq!(
                rows, expected,
                "the shortcut table in {doc} is out of sync with SHORTCUTS"
            );
        }
    }

    /// The help modal and the palette render `keys` as one cap per space-run.
    #[test]
    fn shortcut_keys_are_renderable_caps() {
        for shortcut in SHORTCUTS {
            for cap in shortcut.keys.split(' ') {
                assert!(!cap.is_empty(), "{} has an empty key cap", shortcut.label);
                assert!(
                    cap.chars().count() <= 5,
                    "{} has a cap too wide to draw: {cap}",
                    shortcut.label
                );
            }
        }
    }
}
