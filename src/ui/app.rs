//! Application state and the frame loop: top bar, status bar, dialogs,
//! keyboard shortcuts, background jobs. Panels live in sidebar.rs,
//! editor.rs and response.rs.

use super::theme::{Colors, FontSize, Icons, Layout, Spacing};
use super::widgets::{self, confirm_modal, input_modal, link, modal, popup_menu, ModalAction};
use crate::http::{self, HttpResponse};
use crate::import::{self, ImportCount};
use crate::kv::{self, KeyValue};
use crate::model::{
    AppState, CollectionItem, HistoryEntry, HistorySummary, HttpMethod, RecentRequest, Request,
    RequestFile,
};
use crate::{curl, storage, vars, workspace};
use eframe::egui::{self, Key, RichText};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

const ISSUES_URL: &str = "https://github.com/Harry-kp/mercury/issues";
const RELEASES_URL: &str = "https://github.com/Harry-kp/mercury/releases";
const DOCS_URL: &str = "https://harry-kp.github.io/mercury/docs/getting-started";
const AUTOSAVE_SECS: f64 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Send,
    New,
    Save,
    OpenFolder,
    Search,
    FocusUrl,
    NextEnv,
    History,
    ToggleRaw,
    CopyCurl,
    FocusMode,
    Help,
}

/// One keyboard shortcut. `SHORTCUTS` is the single list: the in-app help
/// modal renders it, and the README/website tables must match it.
pub struct Shortcut {
    pub keys: &'static str,
    pub label: &'static str,
    key: Key,
    command: bool,
    shift: bool,
    action: Action,
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
    cmd("⌘ Enter", "Send request", Key::Enter, false, Action::Send),
    cmd("⌘ N", "New request", Key::N, false, Action::New),
    cmd("⌘ S", "Save request", Key::S, false, Action::Save),
    cmd("⌘ O", "Open folder", Key::O, false, Action::OpenFolder),
    cmd("⌘ K", "Search collection", Key::K, false, Action::Search),
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
    cmd("⌘ Shift C", "Copy as cURL", Key::C, true, Action::CopyCurl),
    cmd("⌘ Shift F", "Focus mode", Key::F, true, Action::FocusMode),
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
pub const ESCAPE_HELP: (&str, &str) = ("Esc", "Cancel request / close dialog / clear search");

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
    const ALL: [Tab; 4] = [Tab::Body, Tab::Params, Tab::Headers, Tab::Auth];
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
    pub show_response_headers: bool,
    pub show_response_cookies: bool,
    pub in_flight: Option<u64>,
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
            env_files: Vec::new(),
            selected_env: None,
            env_vars: HashMap::new(),
            watcher: None,
            current_file: None,
            method: HttpMethod::GET,
            url: String::new(),
            query_params: Vec::new(),
            params_text: String::new(),
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
            show_response_headers: false,
            show_response_cookies: false,
            in_flight: None,
            request_counter: 0,
            history: Vec::new(),
            history_loaded: false,
            history_search: String::new(),
            show_history: false,
            recent: storage::load_recent(),
            recent_expanded: true,
            focus_mode: false,
            dialog: None,
            dialog_text: String::new(),
            toast: None,
            time: 0.0,
            ctx: ctx.clone(),
            tx,
            rx,
            client: http::client(),
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
            app.tab = Tab::ALL
                .get(state.selected_tab)
                .copied()
                .unwrap_or(Tab::Body);
            if let Some(path) = state.workspace_path {
                app.open_workspace(PathBuf::from(path), state.env_name.as_deref());
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

    pub fn rebuild_tree(&mut self) {
        if let Some(root) = &self.workspace {
            self.tree = workspace::scan(root, &self.expanded);
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
            Err(e) => self.notify(format!("Could not open {}: {e}", path.display()), true),
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
        } else if let Some(root) = self.workspace.clone() {
            self.open_dialog(Dialog::NewRequest(root), "");
        } else {
            self.pick_folder();
            self.notify("Open a folder first to save requests", false);
        }
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
        let client = self.client.clone();
        self.spawn(move || Some(Event::Response(id, http::execute(&client, &request))));
    }

    /// Soft cancel: the thread keeps running but its result is ignored.
    pub fn cancel_request(&mut self) {
        self.in_flight = None;
    }

    fn on_response(&mut self, result: Result<HttpResponse, String>) {
        self.in_flight = None;
        match result {
            Ok(response) => {
                self.ensure_history_loaded();
                let request = self.form_request();
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
                    storage::save_recent(&self.recent);
                }
                self.response = Some(response);
                self.request_error = None;
                self.notify("Request completed", false);
            }
            Err(e) => {
                self.response = None;
                self.notify(format!("Request failed: {e}"), true);
                self.request_error = Some(e);
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
            &sub(&self.url),
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
        self.dialog = Some(dialog);
        self.dialog_text = text.to_string();
    }

    fn show_dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.dialog.clone() else {
            return;
        };
        let text = &mut self.dialog_text;
        let action = match &dialog {
            Dialog::NewRequest(_) => {
                input_modal(ctx, "New Request", "Request name:", "Create", text)
            }
            Dialog::NewFolder(_) => input_modal(ctx, "New Folder", "Folder name:", "Create", text),
            Dialog::Rename(_) => input_modal(ctx, "Rename", "New name:", "Rename", text),
            Dialog::Delete(path) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                confirm_modal(
                    ctx,
                    "Confirm Delete",
                    &format!("Delete '{name}'?"),
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
    // Keyboard
    // -----------------------------------------------------------------------

    fn handle_keys(&mut self, ctx: &egui::Context) {
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
            match action {
                Action::Send => self.send_request(),
                Action::New => self.new_request(),
                Action::Save => self.save(),
                Action::OpenFolder => self.pick_folder(),
                Action::Search => self.focus("search_box"),
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
                Action::FocusMode => self.focus_mode = !self.focus_mode,
                Action::Help => {
                    self.dialog = match self.dialog {
                        Some(Dialog::Shortcuts) => None,
                        _ => Some(Dialog::Shortcuts),
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Top bar and status bar
    // -----------------------------------------------------------------------

    fn top_bar(&mut self, ctx: &egui::Context) {
        let text = |s: &str, color| RichText::new(s).size(FontSize::MD).color(color);
        egui::TopBottomPanel::top("top_panel")
            .exact_height(Layout::TOPBAR_HEIGHT)
            .frame(bar_frame())
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    self.breadcrumb(ui);
                    ui.add_space(Spacing::LG);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text(RichText::new("Search (⌘K)").color(Colors::PLACEHOLDER))
                            .desired_width(Layout::SEARCH_WIDTH)
                            .frame(false)
                            .id(egui::Id::new("search_box")),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Environment picker
                        let (label, color) = match (self.env_name(), &self.workspace) {
                            (Some(name), _) => (name.to_string(), Colors::env(name)),
                            (None, Some(_)) => {
                                ("No environment".to_string(), Colors::TEXT_SECONDARY)
                            }
                            (None, None) => {
                                ("No env (open folder)".to_string(), Colors::TEXT_MUTED)
                            }
                        };
                        let env_button = ui.add_enabled(
                            self.workspace.is_some(),
                            egui::Label::new(text(&label, color)).sense(egui::Sense::click()),
                        );
                        let mut picked = None;
                        popup_menu(ui, &env_button, Layout::POPUP_WIDTH, |ui| {
                            if ui
                                .selectable_label(self.selected_env.is_none(), "None")
                                .clicked()
                            {
                                picked = Some(None);
                            }
                            for (i, env) in self.env_files.iter().enumerate() {
                                let label = RichText::new(env).color(Colors::env(env));
                                if ui
                                    .selectable_label(self.selected_env == Some(i), label)
                                    .clicked()
                                {
                                    picked = Some(Some(i));
                                }
                            }
                            if self.env_files.is_empty() {
                                ui.label(
                                    RichText::new("Add .env files to the workspace root")
                                        .size(FontSize::SM)
                                        .color(Colors::TEXT_MUTED),
                                );
                            }
                        });
                        if let Some(index) = picked {
                            self.select_env(index);
                        }

                        ui.add_space(Spacing::XL);
                        let open = link(ui, text("Open", Colors::TEXT_SECONDARY));
                        popup_menu(ui, &open, Layout::SEARCH_WIDTH, |ui| {
                            if ui.selectable_label(false, "Open Folder...").clicked() {
                                self.pick_folder();
                            }
                            for importer in [&POSTMAN, &INSOMNIA] {
                                let label = format!("Import {}...", importer.name);
                                if ui.selectable_label(false, label).clicked() {
                                    self.start_import(importer);
                                }
                            }
                        });

                        ui.add_space(Spacing::XL);
                        let help = link(ui, text("Help", Colors::TEXT_SECONDARY));
                        popup_menu(ui, &help, Layout::POPUP_WIDTH, |ui| {
                            if ui.selectable_label(false, "Keyboard Shortcuts").clicked() {
                                self.dialog = Some(Dialog::Shortcuts);
                            }
                            ui.separator();
                            for (label, url) in [
                                ("Documentation", DOCS_URL),
                                ("Check for Updates", RELEASES_URL),
                                ("Report Issue", ISSUES_URL),
                            ] {
                                if ui.selectable_label(false, label).clicked() {
                                    let _ = open::that(url);
                                }
                            }
                        });
                    });
                });
            });
    }

    /// workspace / folder / METHOD request •
    fn breadcrumb(&self, ui: &mut egui::Ui) {
        let text = |s: &str, color| RichText::new(s).size(FontSize::MD).color(color);
        let slash = |ui: &mut egui::Ui| ui.label(text("/", Colors::TEXT_MUTED));
        let Some(root) = &self.workspace else {
            ui.label(text("No workspace", Colors::TEXT_MUTED));
            return;
        };
        ui.label(text(&self.workspace_name(), Colors::TEXT_SECONDARY));
        let Some(file) = &self.current_file else {
            slash(ui);
            ui.label(text("Untitled", Colors::TEXT_MUTED));
            return;
        };
        if let Some(folder) = file.strip_prefix(root).ok().and_then(Path::parent) {
            for part in folder.iter() {
                slash(ui);
                ui.label(text(&part.to_string_lossy(), Colors::TEXT_SECONDARY));
            }
        }
        slash(ui);
        ui.label(
            RichText::new(self.method.as_str())
                .size(FontSize::SM)
                .strong()
                .color(Colors::method(self.method)),
        );
        let name = file
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        ui.label(text(&name, Colors::TEXT_PRIMARY).strong());
        if self.has_unsaved_changes() {
            ui.label(RichText::new(Icons::DOT).size(14.0).color(Colors::WARNING))
                .on_hover_text("Unsaved changes (auto-saves in a few seconds)");
        }
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(Layout::STATUS_BAR_HEIGHT)
            .frame(bar_frame())
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if let Some(t) = &self.toast {
                        if !widgets::fading_toast(ui, &t.text, t.shown_at, t.is_error) {
                            self.toast = None;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let muted = |s: &str| {
                            RichText::new(s)
                                .size(FontSize::SM)
                                .color(Colors::TEXT_MUTED)
                        };
                        if link(ui, muted("? Shortcuts")).clicked() {
                            self.dialog = Some(Dialog::Shortcuts);
                        }
                        ui.add_space(Spacing::LG);
                        ui.label(muted(&self.workspace_name()));
                    });
                });
            });
    }
}

fn bar_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(Colors::BG_SURFACE)
        .stroke(egui::Stroke::new(1.0_f32, Colors::BORDER_SUBTLE))
        .inner_margin(egui::Margin::symmetric(Spacing::MD as i8, 0))
}

fn shortcuts_modal(ctx: &egui::Context) -> ModalAction {
    modal(ctx, "Keyboard Shortcuts", |ui| {
        ui.add_space(Spacing::SM);
        egui::Grid::new("shortcuts_grid")
            .num_columns(2)
            .spacing([40.0, 10.0])
            .show(ui, |ui| {
                let rows = SHORTCUTS.iter().map(|s| (s.keys, s.label));
                for (keys, label) in rows.chain([ESCAPE_HELP]) {
                    ui.label(RichText::new(label).color(Colors::TEXT_SECONDARY));
                    ui.horizontal(|ui| {
                        for (i, key) in keys.split(' ').enumerate() {
                            if i > 0 {
                                ui.label(
                                    RichText::new("+")
                                        .color(Colors::TEXT_MUTED)
                                        .size(FontSize::XS),
                                );
                            }
                            widgets::key_cap(ui, key);
                        }
                    });
                    ui.end_row();
                }
            });
        ui.add_space(Spacing::MD);
        ui.label(
            RichText::new("On Windows and Linux, ⌘ is Ctrl.")
                .size(FontSize::XS)
                .color(Colors::TEXT_MUTED),
        );
        ui.add_space(Spacing::SM);
        let close = ui.button(RichText::new("Close").color(Colors::PRIMARY).strong());
        if close.clicked() {
            ModalAction::Close
        } else {
            ModalAction::Open
        }
    })
}

impl MercuryApp {
    /// One frame. Separate from `eframe::App::update` so tests can drive it.
    pub fn frame(&mut self, ctx: &egui::Context) {
        self.ctx = ctx.clone();
        self.time = ctx.input(|i| i.time);
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
                    .fill(Colors::BG_BASE)
                    .inner_margin(egui::Margin::same(Spacing::MD as i8)),
            )
            .show(ctx, |ui| self.editor(ui));

        // keys after widgets so text fields have claimed focus this frame
        self.handle_keys(ctx);
        self.show_dialog(ctx);
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
}
