//! Application state and the frame loop: top bar, status bar, dialogs,
//! keyboard shortcuts, background jobs. Panels live in sidebar.rs,
//! editor.rs, response.rs and palette.rs.

use super::icon::Icon;
use super::palette::Palette;
use super::theme::{self, theme, Layout, Radius, Space, Text};
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
    ToggleTheme,
    ImportPostman,
    ImportInsomnia,
    Docs,
    Help,
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
    cmd("⌘ ⇧ C", "Copy as cURL", Key::C, true, Action::CopyCurl),
    cmd("⌘ ⇧ F", "Focus mode", Key::F, true, Action::FocusMode),
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
pub const ESCAPE_HELP: (&str, &str) = ("Esc", "Cancel request / close dialog / clear filter");

impl Shortcut {
    fn pressed(&self, i: &egui::InputState, typing: bool) -> bool {
        if self.command {
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

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Body, Tab::Params, Tab::Headers, Tab::Auth];
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
    NewRequest(PathBuf),
    NewFolder(PathBuf),
    Rename(PathBuf),
    Delete(PathBuf),
    Shortcuts,
}

pub struct Toast {
    pub text: String,
    pub shown_at: f64,
    pub is_error: bool,
}

/// Results from background threads, drained once per frame.
enum Event {
    Response(u64, Result<HttpResponse, String>),
    OpenFolder(PathBuf),
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
    pub search: String,
    /// Filtering needs the whole tree in memory; see `sidebar::load_subtree`.
    pub tree_fully_loaded: bool,
    pub env_files: Vec<String>,
    pub selected_env: Option<usize>,
    pub env_vars: HashMap<String, String>,
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
    last_save: f64,

    // Response
    pub response: Option<HttpResponse>,
    pub request_error: Option<String>,
    pub formatted_body: Option<String>,
    pub raw_view: bool,
    pub response_tab: ResponseTab,
    pub in_flight: Option<u64>,
    /// The form as it was when the in-flight request left, so history records
    /// what was sent even if the user keeps editing while it runs.
    sent: Option<Request>,
    /// The file that was open when it left. If the user navigates to another
    /// request meanwhile, the response belongs to neither of them on screen.
    sent_file: Option<PathBuf>,
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
    pub toast: Option<Toast>,
    /// egui time at the start of this frame (for toasts).
    time: f64,

    ctx: egui::Context,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    client: reqwest::blocking::Client,
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
            tab: Tab::Body,
            headers_bulk_edit: false,
            params_bulk_edit: false,
            saved_content: None,
            last_save: 0.0,
            response: None,
            request_error: None,
            formatted_body: None,
            raw_view: false,
            response_tab: ResponseTab::Body,
            in_flight: None,
            sent: None,
            sent_file: None,
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
            toast: None,
            time: 0.0,
            ctx: ctx.clone(),
            tx,
            rx,
            client: http::client(),
        };

        let restored = storage::load_state();
        if restored.is_none() {
            // nothing to come back to, so the first thing anyone wants is to
            // type a URL
            app.focus("url_bar");
        }
        if let Some(state) = restored {
            app.load_request(
                Request {
                    method: state.method,
                    url: state.url,
                    headers: state.headers_text,
                    body: state.body_text,
                },
                None,
            );
            app.tab = Tab::ALL
                .get(state.selected_tab)
                .copied()
                .unwrap_or(Tab::Body);
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
                app.expand_to(&file);
            }
        }
        app
    }

    pub fn notify(&mut self, text: impl Into<String>, is_error: bool) {
        self.toast = Some(Toast {
            text: text.into(),
            shown_at: self.time,
            is_error,
        });
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
        self.spawn(|| rfd::FileDialog::new().pick_folder().map(Event::OpenFolder));
    }

    pub fn open_workspace(&mut self, path: PathBuf, env: Option<&str>) {
        if !path.is_dir() {
            self.notify(format!("Workspace not found: {}", path.display()), true);
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
    /// collapsed folders deep.
    pub fn expand_to(&mut self, file: &Path) {
        let Some(root) = self.workspace.clone() else {
            return;
        };
        let mut dir = file.parent();
        while let Some(path) = dir {
            if !path.starts_with(&root) {
                break;
            }
            self.expanded.insert(path.to_path_buf());
            if path == root {
                break;
            }
            dir = path.parent();
        }
        self.rebuild_tree();
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

    pub fn select_env(&mut self, index: Option<usize>) {
        self.selected_env = index;
        self.load_env();
    }

    fn load_env(&mut self) {
        self.env_vars.clear();
        let (Some(root), Some(name)) = (&self.workspace, self.env_name()) else {
            return;
        };
        match fs::read_to_string(root.join(name)) {
            Ok(content) => self.env_vars = vars::parse_env(&content),
            Err(e) => self.notify(format!("Could not read {name}: {e}"), true),
        }
    }

    pub fn start_import(&self, importer: &'static Importer) {
        let workspace = self.workspace.clone();
        self.spawn(move || {
            let file = rfd::FileDialog::new()
                .add_filter(format!("{} export", importer.name), importer.extensions)
                .set_title(format!("Select {} export", importer.name))
                .pick_file()?;
            let folder = workspace.or_else(|| {
                rfd::FileDialog::new()
                    .set_title("Choose where to save the imported collection")
                    .pick_folder()
            })?;
            let result = (importer.run)(&file, &folder).map(|count| (folder, count));
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
        self.current_file = file;
        self.saved_content = self.current_file.as_ref().map(|_| self.file_content());
        self.response = None;
        self.request_error = None;
        self.formatted_body = None;
        self.response_tab = ResponseTab::Body;
    }

    pub fn open_file(&mut self, path: &Path) {
        self.autosave();
        let loaded = fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|c| RequestFile::from_json(&c));
        match loaded {
            Ok(file) => self.load_request(
                Request {
                    method: file.method,
                    url: file.url,
                    headers: kv::map_to_headers(&file.headers),
                    body: file.body,
                },
                Some(path.to_path_buf()),
            ),
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
        self.notify("New request", false);
    }

    fn save(&mut self) {
        if self.current_file.is_some() {
            if self.save_file() {
                self.notify("Saved", false);
            }
        } else if let Some(root) = self.target_folder() {
            self.open_dialog(Dialog::NewRequest(root), "");
        } else {
            self.pick_folder();
            self.notify("Open a folder first to save requests", false);
        }
    }

    /// Where "new request" / "new folder" land: the selected collection, else
    /// the workspace root.
    fn target_folder(&self) -> Option<PathBuf> {
        self.selected_folder
            .clone()
            .filter(|p| p.is_dir())
            .or_else(|| self.workspace.clone())
    }

    fn focus(&self, id: &str) {
        self.ctx.memory_mut(|m| m.request_focus(egui::Id::new(id)));
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
        let request = RequestFile {
            method: self.method,
            url: vars::substitute(&self.url, &self.env_vars),
            headers: kv::headers_to_map(&vars::substitute(&self.headers_text, &self.env_vars)),
            body: vars::substitute(&self.body_text, &self.env_vars),
        };
        self.request_counter += 1;
        let id = self.request_counter;
        self.in_flight = Some(id);
        self.sent = Some(self.form_request());
        self.sent_file = self.current_file.clone();
        let client = self.client.clone();
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
                self.ensure_history_loaded();
                let request = self.sent.take().unwrap_or_else(|| self.form_request());
                let entry = HistoryEntry {
                    timestamp: storage::now(),
                    request: request.clone(),
                    response: response.clone(),
                };
                self.history.push(HistorySummary::from(&entry));
                let excess = self.history.len().saturating_sub(storage::MAX_HISTORY);
                self.history.drain(..excess);
                storage::append_history(entry);

                if self.current_file.is_none() && !self.recent.iter().any(|r| r.request == request)
                {
                    self.recent.push(RecentRequest {
                        request,
                        timestamp: storage::now(),
                    });
                    // the file is capped; the sidebar has to match it
                    let excess = self.recent.len().saturating_sub(storage::MAX_RECENT);
                    self.recent.drain(..excess);
                    storage::save_recent(&self.recent);
                }
                let summary = widgets::status_label(response.status, &response.status_text);
                // a green tick next to "404 Not Found" reads as success
                let failed = response.status >= 400;
                if self.sent_file.take() == self.current_file {
                    self.response = Some(response);
                    self.request_error = None;
                    self.notify(summary, failed);
                } else {
                    // the user opened a different request while this was in
                    // flight; showing its response here would attribute it to
                    // the wrong request
                    self.notify(format!("{summary} — in History (⌘ H)"), failed);
                }
            }
            Err(e) if self.sent_file.take() == self.current_file => {
                self.sent = None;
                self.response = None;
                // the response panel shows the full error with a Retry
                // button, so a toast repeating it is just noise
                self.show_history = false;
                self.request_error = Some(e);
            }
            Err(e) => {
                // failed after the user moved to another request
                self.sent = None;
                self.notify(format!("Request failed: {e}"), true);
            }
        }
        self.formatted_body = None;
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
                self.autosave();
                self.load_request(entry.request, None);
                self.response = Some(entry.response);
                self.show_history = false;
            }
            None => self.notify("History entry no longer exists", true),
        }
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        storage::clear_history();
    }

    fn copy_as_curl(&mut self) {
        let sub = |s: &str| vars::substitute(s, &self.env_vars);
        let curl = curl::generate(
            self.method,
            &http::with_scheme(&sub(&self.url)),
            &kv::headers_to_map(&sub(&self.headers_text)),
            &sub(&self.body_text),
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
                Event::Imported(Ok((folder, (requests, envs)))) => {
                    self.open_workspace(folder, None);
                    self.notify(
                        format!("Imported {requests} requests, {envs} environments"),
                        false,
                    );
                }
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
        let disk = match fs::read_to_string(&path) {
            Ok(disk) => disk,
            Err(_) => {
                self.load_request(Request::default(), None);
                self.notify("File was deleted externally", true);
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
            let response = self.response.take();
            self.open_file(&path);
            self.response = response;
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
        self.selected_env =
            selected.and_then(|name| self.env_files.iter().position(|f| *f == name));
        self.load_env();
    }

    // -----------------------------------------------------------------------
    // Dialogs
    // -----------------------------------------------------------------------

    pub fn open_dialog(&mut self, dialog: Dialog, text: &str) {
        // otherwise focus stays wherever it was (the URL bar, after ⌘N) and
        // the name field silently swallows nothing
        if !matches!(dialog, Dialog::Delete(_) | Dialog::Shortcuts) {
            self.focus("modal_input");
        }
        self.dialog = Some(dialog);
        self.dialog_text = text.to_string();
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
            Dialog::NewRequest(parent) => self.in_folder(parent, "New request"),
            Dialog::NewFolder(parent) => self.in_folder(parent, "New folder"),
            _ => String::new(),
        };
        let text = &mut self.dialog_text;
        let action = match &dialog {
            Dialog::NewRequest(_) => input_modal(ctx, &titles, "Request name", "Create", text),
            Dialog::NewFolder(_) => input_modal(ctx, &titles, "Folder name", "Create", text),
            Dialog::Rename(_) => input_modal(ctx, "Rename", "New name", "Rename", text),
            Dialog::Delete(path) => {
                let name = path.file_stem().unwrap_or_default().to_string_lossy();
                let (title, what) = if path.is_dir() {
                    ("Delete folder", "folder")
                } else {
                    ("Delete request", "request")
                };
                confirm_modal(
                    ctx,
                    title,
                    &format!("Delete the {what} '{name}'?"),
                    "Delete",
                )
            }
            Dialog::Shortcuts => shortcuts_modal(ctx),
        };
        match action {
            ModalAction::Open => return,
            ModalAction::Close => {}
            ModalAction::Confirm => {
                let name = self.dialog_text.trim().to_string();
                match self.confirm_dialog(dialog, &name) {
                    Ok(msg) => self.notify(msg, false),
                    Err(e) => self.notify(e, true),
                }
            }
        }
        self.dialog = None;
    }

    fn confirm_dialog(&mut self, dialog: Dialog, name: &str) -> Result<&'static str, String> {
        match dialog {
            Dialog::NewRequest(parent) => {
                let path = workspace::create_request(&parent, name, &self.file_content())?;
                // it's saved now, so it no longer belongs in Recent
                let url = self.url.clone();
                self.recent.retain(|r| r.request.url != url);
                storage::save_recent(&self.recent);
                self.expanded.insert(parent);
                self.rebuild_tree();
                self.open_file(&path);
                Ok("Request created")
            }
            Dialog::NewFolder(parent) => {
                workspace::create_folder(&parent, name)?;
                self.expanded.insert(parent);
                self.rebuild_tree();
                Ok("Folder created")
            }
            Dialog::Rename(path) => {
                let new_path = workspace::rename(&path, name)?;
                if let Some(rest) = self
                    .current_file
                    .as_ref()
                    .and_then(|f| f.strip_prefix(&path).ok())
                {
                    self.current_file = Some(if rest.as_os_str().is_empty() {
                        new_path
                    } else {
                        new_path.join(rest)
                    });
                }
                self.rebuild_tree();
                Ok("Renamed")
            }
            Dialog::Delete(path) => {
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
            Dialog::Shortcuts => Ok(""),
        }
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
            Action::CopyCurl => self.copy_as_curl(),
            Action::FormatBody => {
                self.body_text = http::format_json(&self.body_text);
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
            if self.in_flight.is_some() {
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
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = Space::SM;
                    self.workspace_menu(ui);
                    self.breadcrumb(ui);

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        self.help_menu(ui);
                        let (what, tip) = if theme::is_dark() {
                            (Icon::Sun, "Switch to light (⌘ D)")
                        } else {
                            (Icon::Moon, "Switch to dark (⌘ D)")
                        };
                        if icon_button(ui, what, tip).clicked() {
                            self.run(Action::ToggleTheme);
                        }
                        ui.add_space(Space::SM);
                        self.env_menu(ui);
                    });
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
        let chip = chip(ui, Icon::Folder, &name, t.text, open);
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
                if menu_item(ui, Some(Icon::Copy), "Copy workspace path", "") {
                    let path = self.workspace.clone().unwrap_or_default();
                    ui.ctx().copy_text(path.display().to_string());
                }
            }
        });
    }

    /// folder / folder / METHOD request •
    fn breadcrumb(&self, ui: &mut egui::Ui) {
        let t = theme();
        let Some(root) = &self.workspace else {
            return;
        };
        let sep = |ui: &mut egui::Ui| {
            ui.label(muted("/").color(t.text_faint));
        };
        let Some(file) = &self.current_file else {
            sep(ui);
            ui.label(muted("Untitled"));
            if !self.url.is_empty() {
                ui.label(faint("unsaved"));
            }
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

    fn env_menu(&mut self, ui: &mut egui::Ui) {
        let t = theme();
        let (name, color) = match (self.env_name(), &self.workspace) {
            (Some(name), _) => (name.to_string(), t.env(name)),
            (None, Some(_)) => ("No environment".to_string(), t.text_muted),
            (None, None) => ("No environment".to_string(), t.text_faint),
        };
        let chip = chip(ui, Icon::Layers, &name, color, self.workspace.is_some());
        let mut picked = None;
        popup_menu(ui, &chip, Layout::MENU_WIDTH, |ui| {
            if menu_item(ui, None, "None", "") {
                picked = Some(None);
            }
            for (i, env) in self.env_files.iter().enumerate() {
                let check = (self.selected_env == Some(i)).then_some(Icon::Check);
                if menu_item(ui, check, env, "") {
                    picked = Some(Some(i));
                }
            }
            if self.env_files.is_empty() {
                ui.add_space(Space::XS);
                ui.label(muted("Add .env files to the workspace root"));
                ui.add_space(Space::XS);
            }
        });
        if let Some(index) = picked {
            self.select_env(index);
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
            if menu_item(ui, Some(Icon::PanelLeft), "Focus mode", "⌘ ⇧ F") {
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
                    match &self.workspace {
                        Some(path) => {
                            widgets::glyph(ui, Icon::Folder, 12.0, t.text_faint);
                            ui.label(faint(widgets::workspace_label(path, 56)))
                                .on_hover_text(path.display().to_string());
                        }
                        None => {
                            ui.label(faint("No workspace — requests are files in a folder"));
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = Space::SM;
                        if widgets::link(ui, faint("Shortcuts"))
                            .on_hover_text("Press ?")
                            .clicked()
                        {
                            self.dialog = Some(Dialog::Shortcuts);
                        }
                        ui.add_space(Space::MD);
                        if widgets::link(ui, faint("Commands"))
                            .on_hover_text("Press ⌘K")
                            .clicked()
                        {
                            self.run(Action::Palette);
                        }
                        key_combo(ui, "⌘ K");
                        if self.focus_mode {
                            ui.add_space(Space::MD);
                            ui.label(faint("Focus mode · ⌘ ⇧ F to exit").color(t.accent));
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
        let alive = egui::Area::new(egui::Id::new("toast"))
            .anchor(
                egui::Align2::RIGHT_BOTTOM,
                Vec2::new(-Space::XL, -(Layout::STATUS_BAR_HEIGHT + Space::LG)),
            )
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| widgets::toast(ui, &text, shown_at, is_error))
            .inner;
        if !alive {
            self.toast = None;
        }
    }
}

/// A small pill of icon + text used for the workspace and environment menus.
fn chip(
    ui: &mut egui::Ui,
    what: Icon,
    text: &str,
    color: egui::Color32,
    enabled: bool,
) -> egui::Response {
    let t = theme();
    let text = widgets::truncate(text, CRUMB_CHARS);
    let galley =
        ui.fonts_mut(|f| f.layout_no_wrap(text.clone(), theme::semibold(Text::SMALL), color));
    let size = Vec2::new(galley.size().x + 14.0 + Space::SM + Space::MD * 2.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, Radius::SM, t.hover);
    }
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + Space::MD + 7.0, rect.center().y),
        Vec2::splat(14.0),
    );
    super::icon::paint(
        ui.painter(),
        what,
        icon_rect,
        if enabled { color } else { t.text_faint },
    );
    ui.painter().galley(
        egui::pos2(
            icon_rect.right() + Space::SM,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        color,
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &text));
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn shortcuts_modal(ctx: &egui::Context) -> ModalAction {
    modal(ctx, "Keyboard shortcuts", |ui| {
        let rows = SHORTCUTS
            .iter()
            .map(|s| (s.keys, s.label))
            .chain([ESCAPE_HELP]);
        for (keys, text) in rows {
            ui.horizontal(|ui| {
                ui.set_min_height(28.0);
                ui.label(label(text));
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    key_combo(ui, keys);
                });
            });
        }
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
        storage::save_state(&AppState {
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
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
