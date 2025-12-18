//! Mercury Application Core
//!
//! This module contains the main application state and logic for the Mercury API client.
//! It handles:
//! - Application state management (MercuryApp)
//! - Workspace and file operations
//! - Request execution coordination
//! - UI state and rendering dispatch
//! - Session persistence (state, history, recent requests)

use crate::core::persistence;
use crate::core::types::{AppState, CollectionItem, TempRequest, TimelineEntry};
use crate::core::{execute_request, HttpResponse};
use crate::parser::{
    parse_env_file, parse_http_file, substitute_variables, HttpMethod, HttpRequest,
};

use eframe::egui;
use notify_debouncer_mini::new_debouncer;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;
use walkdir::WalkDir;

pub struct MercuryApp {
    pub workspace_path: Option<PathBuf>,
    pub workspace_name: String,
    pub collection_tree: Vec<CollectionItem>,

    pub current_file: Option<PathBuf>,
    pub method: HttpMethod,
    pub url: String,
    pub query_params: Vec<crate::utils::QueryParam>,
    pub headers_text: String,
    pub body_text: String,
    pub auth_text: String,
    pub auth_mode: AuthMode,
    // Auth helpers (ephemeral)
    pub auth_username: String,
    pub auth_password: String,
    pub auth_token: String,

    pub response: Option<HttpResponse>,
    pub previous_response: Option<HttpResponse>,
    pub response_view_raw: bool,
    pub show_response_headers: bool,
    pub show_response_diff: bool,
    // Cached formatted response to avoid cloning every frame
    pub formatted_response_cache: Option<String>,

    pub env_files: Vec<String>,
    pub selected_env: usize,
    pub env_variables: HashMap<String, String>,

    pub search_query: String,
    pub show_shortcuts: bool,
    pub selected_tab: usize,
    pub focus_mode: bool,
    pub headers_bulk_edit: bool, // Toggle between key-value and bulk edit

    pub timeline: Vec<TimelineEntry>,
    pub timeline_search: String,
    pub show_timeline: bool,

    pub temp_requests: Vec<TempRequest>,
    pub recent_expanded: bool,

    pub context_menu_item: Option<PathBuf>,
    pub selected_folder: Option<PathBuf>,
    pub show_rename_dialog: bool,
    pub rename_text: String,
    pub show_new_request_dialog: bool,
    pub new_request_name: String,
    pub show_new_folder_dialog: bool,
    pub new_folder_name: String,
    pub show_new_env_dialog: bool,
    pub new_env_name: String,
    pub show_delete_confirm: bool,
    pub delete_target: Option<PathBuf>,

    pub should_create_new_request: bool,
    pub should_execute_request: bool,
    pub should_open_folder_dialog: bool,
    pub should_open_insomnia_import: bool,
    pub should_open_postman_import: bool,
    pub should_focus_search: bool,
    pub should_focus_url_bar: bool,
    pub should_copy_curl: bool,

    pub last_action_message: Option<(String, f64, bool)>,
    pub copied_feedback_until: f64,
    pub request_error: Option<String>,

    pub show_about: bool,

    pub ongoing_request: Option<(u64, f64)>, // (id, start_time)
    request_id_counter: u64,
    response_rx: Receiver<(u64, Result<HttpResponse, String>)>,
    response_tx: Sender<(u64, Result<HttpResponse, String>)>,

    folder_rx: Receiver<PathBuf>,
    folder_tx: Sender<PathBuf>,

    // Auto-save tracking
    pub has_unsaved_changes: bool,
    last_save_time: f64,
    last_saved_content: Option<String>, // Content at last save for comparison

    // File system watcher
    watcher_rx: Receiver<Result<(), String>>,
    #[allow(dead_code)]
    watcher_tx: Sender<Result<(), String>>,
    watcher_shutdown: Option<Sender<()>>,
    watched_path: Option<PathBuf>,
    expanded_folders: HashSet<PathBuf>,
    file_watcher_error: Option<String>,
}

pub use crate::utils::AuthMode;

impl MercuryApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (response_tx, response_rx) = channel();
        let (folder_tx, folder_rx) = channel();
        let (watcher_tx, watcher_rx) = channel();

        // Load saved state
        let saved_state = persistence::load_state();

        let mut app = Self {
            workspace_path: None,
            workspace_name: String::new(),
            collection_tree: Vec::new(),
            current_file: None,
            method: HttpMethod::GET,
            url: String::new(),
            query_params: Vec::new(),
            headers_text: String::new(),
            body_text: String::new(),
            auth_text: String::new(),
            auth_mode: AuthMode::None,
            auth_username: String::new(),
            auth_password: String::new(),
            auth_token: String::new(),
            response: None,
            previous_response: None,
            response_view_raw: false,
            show_response_headers: false,
            show_response_diff: false,
            formatted_response_cache: None,

            env_files: vec!["None".to_string()],
            selected_env: 0,
            env_variables: HashMap::new(),
            search_query: String::new(),
            show_shortcuts: false,
            selected_tab: 0,
            focus_mode: false,
            headers_bulk_edit: false,
            timeline: persistence::load_history(),
            timeline_search: String::new(),
            show_timeline: false,
            temp_requests: persistence::load_temp_requests(),
            recent_expanded: true,
            context_menu_item: None,
            selected_folder: None,
            show_rename_dialog: false,
            rename_text: String::new(),
            show_new_request_dialog: false,
            new_request_name: String::new(),
            show_new_folder_dialog: false,
            new_folder_name: String::new(),
            show_new_env_dialog: false,
            new_env_name: String::new(),
            show_delete_confirm: false,
            delete_target: None,
            should_create_new_request: false,
            should_execute_request: false,
            should_open_folder_dialog: false,
            should_open_insomnia_import: false,
            should_open_postman_import: false,
            should_focus_search: false,
            should_focus_url_bar: false,
            should_copy_curl: false,
            last_action_message: None,
            copied_feedback_until: 0.0,
            request_error: None,
            show_about: false,
            ongoing_request: None,
            request_id_counter: 0,
            response_rx,
            response_tx,
            folder_rx,
            folder_tx,
            has_unsaved_changes: false,
            last_save_time: f64::MAX, // Start high so first auto-save waits for actual save/load
            last_saved_content: None,
            watcher_rx,
            watcher_tx,
            watcher_shutdown: None,
            watched_path: None,
            expanded_folders: HashSet::new(),
            file_watcher_error: None,
        };

        // Restore saved state
        if let Some(state) = saved_state {
            // Restore method
            app.method = match state.method.as_str() {
                "POST" => HttpMethod::POST,
                "PUT" => HttpMethod::PUT,
                "DELETE" => HttpMethod::DELETE,
                "PATCH" => HttpMethod::PATCH,
                "HEAD" => HttpMethod::HEAD,
                "OPTIONS" => HttpMethod::OPTIONS,
                _ => HttpMethod::GET,
            };
            app.url = state.url;
            app.headers_text = state.headers_text;
            app.body_text = state.body_text;
            app.auth_text = state.auth_text;

            // Infer auth mode from text
            let (mode, username, password, token) = crate::utils::infer_auth_config(&app.auth_text);
            app.auth_mode = mode;
            if mode == AuthMode::Basic {
                app.auth_username = username;
                app.auth_password = password;
            } else if mode == AuthMode::Bearer {
                app.auth_token = token;
            }

            app.selected_tab = state.selected_tab;

            // Restore workspace if it exists
            if let Some(workspace_str) = state.workspace_path {
                let workspace_path = PathBuf::from(&workspace_str);
                if workspace_path.exists() {
                    app.load_workspace(workspace_path);
                    // Restore selected env after loading workspace
                    if state.selected_env < app.env_files.len() {
                        app.selected_env = state.selected_env;
                        app.load_env();
                    }
                }
            }
        }

        app
    }

    fn load_workspace(&mut self, path: PathBuf) {
        self.workspace_path = Some(path.clone());

        // Scan for .env files
        self.env_files = vec!["None".to_string()];
        for entry in WalkDir::new(&path).max_depth(2).into_iter().flatten() {
            let file_name = entry.file_name().to_string_lossy();
            if file_name.starts_with(".env") {
                self.env_files.push(file_name.to_string());
            }
        }

        // Auto-select first non-production environment if available
        if self.env_files.len() > 1 {
            // Try to find .env.dev or .env.development first
            if let Some(pos) = self.env_files.iter().position(|e| e.contains(".dev")) {
                self.selected_env = pos;
            } else {
                // Otherwise pick first non-None environment
                self.selected_env = 1;
            }
            self.load_env();
        } else {
            self.selected_env = 0;
        }

        // Build collection tree
        self.build_collection_tree();

        // Start file system watcher for external changes
        self.start_file_watcher();

        // Scan for .http files (backward compatibility)
        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("http") {
                // Try to parse the file to get the method
                let _method = if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(request) = parse_http_file(&content) {
                        Some(request.method)
                    } else {
                        None
                    }
                } else {
                    None
                };
            }
        }
    }

    fn load_file(&mut self, path: &Path) {
        // Save current file before loading new one
        if self.has_unsaved_changes {
            self.save_current_file();
        }

        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(request) = parse_http_file(&content) {
                self.current_file = Some(path.to_path_buf());
                self.method = request.method;
                self.url = request.url;

                // Convert headers map to text
                self.headers_text = request
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                self.body_text = request.body.unwrap_or_default();
                self.response = None;

                // Sync query params from URL
                self.query_params = crate::utils::parse_query_params(&self.url);

                // Track the loaded content for change detection
                self.last_saved_content = Some(self.get_current_content());
                self.has_unsaved_changes = false;
            }
        }
    }

    /// Get the current request content as an .http file string
    fn get_current_content(&self) -> String {
        let mut content = format!("{} {}", self.method.as_str(), self.url);

        if !self.headers_text.is_empty() {
            content.push('\n');
            content.push_str(&self.headers_text);
        }

        if !self.body_text.is_empty() {
            content.push_str("\n\n");
            content.push_str(&self.body_text);
        }

        content
    }

    /// Save current file to disk
    pub fn save_current_file(&mut self) -> bool {
        if let Some(ref path) = self.current_file {
            let content = self.get_current_content();
            if fs::write(path, &content).is_ok() {
                self.last_saved_content = Some(content);
                self.has_unsaved_changes = false;
                return true;
            }
        }
        false
    }

    /// Check if current content differs from last saved content
    pub fn check_for_changes(&mut self) {
        if self.current_file.is_some() {
            let current = self.get_current_content();
            self.has_unsaved_changes = self.last_saved_content.as_ref() != Some(&current);
        }
    }

    /// Clear the request form to empty state (used by new request, delete, etc.)
    pub fn clear_request_form(&mut self) {
        self.current_file = None;
        self.method = HttpMethod::GET;
        self.url = String::new();
        self.query_params.clear();
        self.headers_text = String::new();
        self.body_text = String::new();
        self.auth_text = String::new();
        self.response = None;
        self.previous_response = None;
        self.has_unsaved_changes = false;
        self.last_saved_content = None;
    }

    /// Load request data into the form (used by history, recent, cURL, file load)
    pub fn load_request_data(
        &mut self,
        method: HttpMethod,
        url: String,
        headers: String,
        body: String,
    ) {
        self.current_file = None;
        self.method = method;
        self.url = url;
        self.headers_text = headers;
        self.body_text = body;
        self.query_params = crate::utils::parse_query_params(&self.url);
        self.response = None;
    }

    fn load_env(&mut self) {
        self.env_variables.clear();

        if self.selected_env > 0 && self.selected_env < self.env_files.len() {
            if let Some(workspace) = &self.workspace_path {
                let env_file = workspace.join(&self.env_files[self.selected_env]);
                if let Ok(vars) = parse_env_file(&env_file) {
                    self.env_variables = vars;
                }
            }
        }
    }

    pub fn extract_variables(text: &str) -> Vec<String> {
        let mut vars = Vec::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second {
                let mut var_name = String::new();

                while let Some(c) = chars.next() {
                    if c == '}' {
                        if chars.peek() == Some(&'}') {
                            chars.next(); // consume second }
                            if !var_name.is_empty() {
                                vars.push(var_name.trim().to_string());
                            }
                            break;
                        }
                    } else {
                        var_name.push(c);
                    }
                }
            }
        }

        vars
    }

    fn build_collection_tree(&mut self) {
        if let Some(workspace) = self.workspace_path.clone() {
            // Save current expanded state before rebuilding
            let old_tree = std::mem::take(&mut self.collection_tree);
            self.save_expanded_state(&old_tree);

            // Rebuild tree
            self.collection_tree = self.scan_directory(&workspace, &workspace);

            self.workspace_name = workspace
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
        }
    }

    /// Save the expanded state of all folders to the HashSet
    fn save_expanded_state(&mut self, items: &[CollectionItem]) {
        for item in items {
            if let CollectionItem::Folder {
                path,
                expanded,
                children,
                ..
            } = item
            {
                if *expanded {
                    self.expanded_folders.insert(path.clone());
                } else {
                    self.expanded_folders.remove(path);
                }
                self.save_expanded_state(children);
            }
        }
    }

    /// Start file system watcher for the workspace directory
    /// Start file system watcher for the workspace directory
    fn start_file_watcher(&mut self) {
        if let Some(workspace) = &self.workspace_path {
            // Avoid restarting if path implementation hasn't changed
            if self.watched_path.as_ref() == Some(workspace) {
                return;
            }

            // Shutdown existing watcher if running
            if let Some(shutdown_tx) = self.watcher_shutdown.take() {
                let _ = shutdown_tx.send(());
            }

            // Update watched path
            self.watched_path = Some(workspace.clone());
            self.file_watcher_error = None;

            let workspace_path = workspace.clone();
            let tx = self.watcher_tx.clone();
            let (shutdown_tx, shutdown_rx) = channel();
            self.watcher_shutdown = Some(shutdown_tx);

            std::thread::spawn(move || {
                // Initial delay to avoid race condition with workspace loading
                std::thread::sleep(Duration::from_secs(1));

                let (debouncer_tx, debouncer_rx) = std::sync::mpsc::channel();

                // Create debounced watcher (500ms debounce to avoid rapid rebuilds)
                let mut debouncer = match new_debouncer(Duration::from_millis(500), debouncer_tx) {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to create file watcher: {}", e)));
                        return;
                    }
                };

                // Watch the workspace directory recursively
                if let Err(e) = debouncer
                    .watcher()
                    .watch(&workspace_path, notify::RecursiveMode::Recursive)
                {
                    let _ = tx.send(Err(format!("Failed to watch directory: {}", e)));
                    return;
                }

                // Listen for events and signal main thread
                // Keep debouncer alive throughout the thread
                #[allow(unused_variables)]
                let _debouncer = debouncer;
                loop {
                    // Check for shutdown signal
                    if let Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) =
                        shutdown_rx.try_recv()
                    {
                        break;
                    }

                    match debouncer_rx.recv_timeout(Duration::from_millis(200)) {
                        Ok(Ok(events)) => {
                            if !events.is_empty() {
                                let _ = tx.send(Ok(()));
                            }
                        }
                        Ok(Err(error)) => {
                            let _ = tx.send(Err(format!("File watcher error: {:?}", error)));
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            // Timeout hit, loop back to check shutdown
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            // Debouncer channel closed
                            break;
                        }
                    }
                }
            });
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn scan_directory(&self, dir: &Path, workspace_root: &Path) -> Vec<CollectionItem> {
        let mut folders = Vec::new();
        let mut requests = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.path());

            for entry in entries {
                let path = entry.path();
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // Skip hidden files and env files
                if name.starts_with('.') {
                    continue;
                }

                if path.is_dir() {
                    let children = self.scan_directory(&path, workspace_root);
                    // Check saved state; expand all folders on first load (when expanded_folders is empty)
                    let is_expanded = self.expanded_folders.contains(&path);
                    folders.push(CollectionItem::Folder {
                        name,
                        path: path.clone(),
                        expanded: is_expanded || self.expanded_folders.is_empty(),
                        children,
                    });
                } else if path.extension().and_then(|s| s.to_str()) == Some("http") {
                    let method = if let Ok(content) = fs::read_to_string(&path) {
                        parse_http_file(&content).ok().map(|r| r.method)
                    } else {
                        None
                    };

                    requests.push(CollectionItem::Request {
                        name,
                        path: path.clone(),
                        method,
                    });
                }
            }
        }

        // Combine folders first, then requests
        folders.extend(requests);
        folders
    }

    fn create_new_request(&mut self, parent_path: &Path, name: &str) -> Result<(), String> {
        let file_name = if name.ends_with(".http") {
            name.to_string()
        } else {
            format!("{}.http", name)
        };

        let file_path = parent_path.join(&file_name);

        if file_path.exists() {
            return Err("File already exists".to_string());
        }

        // Use current form content instead of template
        let content = format!(
            "{:?} {}\n{}\n{}",
            self.method,
            self.url,
            if !self.headers_text.is_empty() {
                format!("\n{}", self.headers_text)
            } else {
                String::new()
            },
            if !self.body_text.is_empty() {
                format!("\n{}", self.body_text)
            } else {
                String::new()
            }
        );

        fs::write(&file_path, content).map_err(|e| format!("Failed to create file: {}", e))?;

        // Remove from temp requests if it exists
        if let Some(pos) = self.temp_requests.iter().position(|t| t.url == self.url) {
            self.temp_requests.remove(pos);
            self.save_temp_requests();
        }

        self.build_collection_tree();
        self.load_file(&file_path);
        Ok(())
    }

    fn delete_item(&mut self, path: &Path) -> Result<(), String> {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("Failed to delete folder: {}", e))?;
        } else {
            fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;
        }

        self.build_collection_tree();
        if self.current_file.as_ref() == Some(&path.to_path_buf()) {
            self.clear_request_form();
        }
        Ok(())
    }

    fn rename_item(&mut self, old_path: &Path, new_name: &str) -> Result<(), String> {
        let parent = old_path.parent().ok_or("No parent directory")?;
        let new_path = parent.join(new_name);

        if new_path.exists() {
            return Err("Name already exists".to_string());
        }

        fs::rename(old_path, &new_path).map_err(|e| format!("Failed to rename: {}", e))?;

        // Update current file if it was renamed
        if self.current_file.as_ref() == Some(&old_path.to_path_buf()) {
            self.current_file = Some(new_path.clone());
        }

        self.build_collection_tree();
        Ok(())
    }

    fn create_new_folder(&mut self, parent_path: &Path, name: &str) -> Result<(), String> {
        let folder_path = parent_path.join(name);

        if folder_path.exists() {
            return Err("Folder already exists".to_string());
        }

        fs::create_dir(&folder_path).map_err(|e| format!("Failed to create folder: {}", e))?;

        self.build_collection_tree();
        Ok(())
    }

    fn duplicate_request(&mut self, path: &Path) -> Result<(), String> {
        if !path.is_file() {
            return Err("Not a file".to_string());
        }

        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let parent = path.parent().ok_or("No parent directory")?;
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().unwrap_or_default().to_string_lossy();

        let mut counter = 1;
        let mut new_path;
        loop {
            new_path = parent.join(format!("{}_copy{}.{}", stem, counter, ext));
            if !new_path.exists() {
                break;
            }
            counter += 1;
        }

        fs::write(&new_path, content).map_err(|e| format!("Failed to duplicate file: {}", e))?;

        self.build_collection_tree();
        Ok(())
    }

    fn create_new_env(&mut self, name: &str) -> Result<(), String> {
        if let Some(workspace) = &self.workspace_path {
            let env_name = if name.starts_with(".env") {
                name.to_string()
            } else {
                format!(".env.{}", name)
            };

            let env_path = workspace.join(&env_name);

            if env_path.exists() {
                return Err("Environment already exists".to_string());
            }

            fs::write(&env_path, "# Environment variables\n")
                .map_err(|e| format!("Failed to create environment: {}", e))?;

            self.load_workspace(workspace.clone());
            Ok(())
        } else {
            Err("No workspace loaded".to_string())
        }
    }

    pub fn execute_request(&mut self, ctx: &egui::Context) {
        // Auto-prefix http://
        self.url = crate::utils::sanitize_url(&self.url);

        // Auto-add Content-Type for JSON
        if crate::utils::should_add_json_header(&self.body_text, &self.headers_text) {
            if !self.headers_text.is_empty() && !self.headers_text.ends_with('\n') {
                self.headers_text.push('\n');
            }
            self.headers_text.push_str("Content-Type: application/json");
        }

        let url = substitute_variables(&self.url, &self.env_variables);
        let headers_text = substitute_variables(&self.headers_text, &self.env_variables);
        let body = substitute_variables(&self.body_text, &self.env_variables);

        // Parse headers
        let mut headers = HashMap::new();
        for line in headers_text.lines() {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        let request = HttpRequest {
            method: self.method.clone(),
            url,
            headers,
            body: if body.is_empty() { None } else { Some(body) },
        };

        // Execute async request in background thread
        // Execute async request in background thread
        let ctx = ctx.clone();
        let tx = self.response_tx.clone();

        // Assign new ID
        self.request_id_counter += 1;
        let request_id = self.request_id_counter;
        let start_time = ctx.input(|i| i.time);

        self.ongoing_request = Some((request_id, start_time));

        std::thread::spawn(move || {
            let response = execute_request(&request, 30, true);
            let _ = tx.send((request_id, response));
            ctx.request_repaint();
        });
    }

    /// Cancel the currently running request (soft cancel)
    /// We can't easily kill the thread, so we just ignore its result
    pub fn cancel_request(&mut self) {
        self.ongoing_request = None;
    }

    fn generate_curl(&self) -> String {
        let url = substitute_variables(&self.url, &self.env_variables);
        let headers_text = substitute_variables(&self.headers_text, &self.env_variables);
        let body = substitute_variables(&self.body_text, &self.env_variables);

        let mut curl = format!("curl -X {} '{}'", self.method.as_str(), url);

        // Add headers
        for line in headers_text.lines() {
            if let Some((key, value)) = line.split_once(':') {
                curl.push_str(&format!(" \\\n  -H '{}: {}'", key.trim(), value.trim()));
            }
        }

        // Add body
        if !body.is_empty() {
            curl.push_str(&format!(" \\\n  -d '{}'", body.replace('\'', "'\\''")));
        }

        curl
    }

    fn copy_as_curl(&self, ctx: &egui::Context) {
        let curl = self.generate_curl();
        ctx.copy_text(curl);
    }
}

impl MercuryApp {
    /// Save app state to disk
    pub fn save_state(&self) {
        let state = AppState {
            workspace_path: self
                .workspace_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            method: self.method.as_str().to_string(),
            url: self.url.clone(),
            headers_text: self.headers_text.clone(),
            body_text: self.body_text.clone(),
            auth_text: self.auth_text.clone(),
            selected_tab: self.selected_tab,
            selected_env: self.selected_env,
        };
        persistence::save_state(&state);
    }

    /// Save recent/temp requests to disk
    pub fn save_temp_requests(&self) {
        persistence::save_temp_requests(&self.temp_requests);
    }

    /// Save timeline history to disk
    pub fn save_history(&self) {
        persistence::save_history(&self.timeline);
    }

    pub fn render_collection_tree(
        &mut self,
        ui: &mut egui::Ui,
        items: &mut [CollectionItem],
        depth: usize,
    ) {
        let search = self.search_query.to_lowercase();

        for item in items {
            match item {
                CollectionItem::Folder {
                    name,
                    path,
                    expanded,
                    children,
                } => {
                    // If searching, check if any child matches
                    let folder_matches = if search.is_empty() {
                        true
                    } else {
                        // Folder matches if its name or any descendant matches
                        name.to_lowercase().contains(&search)
                            || Self::folder_has_matching_children(children, &search)
                    };

                    if !folder_matches {
                        continue;
                    }

                    let folder_row = ui.horizontal(|ui| {
                        ui.add_space(
                            (depth * crate::theme::Indent::TREE_LEVEL as usize) as f32 + 12.0,
                        );

                        // Folder icon (open/closed state)
                        let folder_icon = if *expanded { "📂" } else { "📁" };
                        ui.label(egui::RichText::new(folder_icon).size(crate::theme::FontSize::MD));

                        let is_selected = self.selected_folder.as_ref() == Some(path);

                        ui.add_space(crate::theme::Spacing::XS);
                        let mut name_text =
                            egui::RichText::new(name.as_str()).size(crate::theme::FontSize::SM);
                        if is_selected {
                            name_text = name_text
                                .color(crate::theme::Colors::SELECTED_ITEM)
                                .strong();
                        }

                        ui.label(name_text);
                    });

                    // Create interactive area covering the full row
                    let row_rect = folder_row.response.rect;
                    let full_rect = egui::Rect::from_min_max(
                        egui::pos2(row_rect.min.x, row_rect.min.y),
                        egui::pos2(ui.available_width() + row_rect.min.x, row_rect.max.y),
                    );
                    let folder_response = ui.interact(
                        full_rect,
                        egui::Id::new(("folder", path.as_path())),
                        egui::Sense::click(),
                    );

                    if folder_response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    if folder_response.clicked() {
                        *expanded = !*expanded;
                        self.selected_folder = Some(path.clone());
                    }

                    folder_response.context_menu(|ui| {
                        if ui.button("➕ New Request").clicked() {
                            self.context_menu_item = Some(path.clone());
                            self.show_new_request_dialog = true;
                            self.new_request_name = String::new();
                            ui.close();
                        }
                        if ui.button("📁 New Folder").clicked() {
                            self.context_menu_item = Some(path.clone());
                            self.show_new_folder_dialog = true;
                            self.new_folder_name = String::new();
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("✏️ Rename").clicked() {
                            self.context_menu_item = Some(path.clone());
                            self.show_rename_dialog = true;
                            self.rename_text = name.clone();
                            ui.close();
                        }
                        if ui.button("🗑 Delete").clicked() {
                            self.delete_target = Some(path.clone());
                            self.show_delete_confirm = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("📋 Copy Path").clicked() {
                            if let Some(path_str) = path.to_str() {
                                ui.ctx().copy_text(path_str.to_string());
                            }
                            ui.close();
                        }
                    });

                    if *expanded || !search.is_empty() {
                        // If searching, always show children (auto-expand)
                        self.render_collection_tree(ui, children, depth + 1);
                    }
                }
                CollectionItem::Request { name, path, method } => {
                    // If searching, skip non-matching requests
                    if !search.is_empty() && !name.to_lowercase().contains(&search) {
                        continue;
                    }

                    let request_row = ui.horizontal(|ui| {
                        ui.add_space(
                            (depth * crate::theme::Indent::TREE_LEVEL as usize) as f32 + 14.0,
                        );

                        // Request/document icon
                        ui.label(egui::RichText::new("📄").size(crate::theme::FontSize::SM));

                        if let Some(method) = method {
                            let color = crate::theme::Colors::method_color(method.as_str());
                            ui.label(
                                egui::RichText::new(method.as_str())
                                    .color(color)
                                    .size(crate::theme::FontSize::XS)
                                    .strong(),
                            );
                        }

                        ui.add_space(crate::theme::Spacing::XS);

                        let is_current = self.current_file.as_ref() == Some(path);
                        let mut name_text =
                            egui::RichText::new(name.as_str()).size(crate::theme::FontSize::SM);
                        if is_current {
                            name_text = name_text
                                .strong()
                                .color(crate::theme::Colors::SELECTED_ITEM);
                        }

                        ui.label(name_text);
                    });

                    // Create interactive area covering the full row
                    let row_rect = request_row.response.rect;
                    let full_rect = egui::Rect::from_min_max(
                        egui::pos2(row_rect.min.x, row_rect.min.y),
                        egui::pos2(ui.available_width() + row_rect.min.x, row_rect.max.y),
                    );
                    let request_response = ui.interact(
                        full_rect,
                        egui::Id::new(("request", path.as_path())),
                        egui::Sense::click(),
                    );

                    if request_response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    if request_response.clicked() {
                        self.load_file(path);
                    }

                    request_response.context_menu(|ui| {
                        if ui.button("📋 Duplicate").clicked() {
                            let _ = self.duplicate_request(path);
                            ui.close();
                        }
                        if ui.button("✏️ Rename").clicked() {
                            self.context_menu_item = Some(path.clone());
                            self.show_rename_dialog = true;
                            self.rename_text = name.clone();
                            ui.close();
                        }
                        if ui.button("🗑 Delete").clicked() {
                            self.delete_target = Some(path.clone());
                            self.show_delete_confirm = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("📋 Copy Path").clicked() {
                            if let Some(path_str) = path.to_str() {
                                ui.ctx().copy_text(path_str.to_string());
                            }
                            ui.close();
                        }
                    });
                }
            }
        }
    }

    /// Helper to check if a folder has any matching children
    fn folder_has_matching_children(children: &[CollectionItem], search: &str) -> bool {
        for child in children {
            match child {
                CollectionItem::Request { name, .. } => {
                    if name.to_lowercase().contains(search) {
                        return true;
                    }
                }
                CollectionItem::Folder { name, children, .. } => {
                    if name.to_lowercase().contains(search) {
                        return true;
                    }
                    if Self::folder_has_matching_children(children, search) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(crate::theme::Layout::STATUS_BAR_HEIGHT)
            .frame(
                egui::Frame::NONE
                    .fill(crate::theme::Colors::BG_SURFACE)
                    .stroke(egui::Stroke::new(
                        crate::theme::StrokeWidth::THIN,
                        crate::theme::Colors::BORDER_SUBTLE,
                    ))
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let current_time = ui.input(|i| i.time);

                    if let Some((msg, timestamp, is_error)) = &self.last_action_message {
                        if current_time - timestamp < crate::core::constants::FADE_DURATION_SECONDS
                        {
                            let alpha = ((crate::core::constants::FADE_DURATION_SECONDS
                                - (current_time - timestamp))
                                / crate::core::constants::FADE_DURATION_SECONDS
                                * 255.0) as u8;
                            let color = if *is_error {
                                egui::Color32::from_rgba_unmultiplied(
                                    crate::theme::Colors::ERROR_FLASH.r(),
                                    crate::theme::Colors::ERROR_FLASH.g(),
                                    crate::theme::Colors::ERROR_FLASH.b(),
                                    alpha,
                                )
                            } else {
                                egui::Color32::from_rgba_unmultiplied(
                                    crate::theme::Colors::SUCCESS_FLASH.r(),
                                    crate::theme::Colors::SUCCESS_FLASH.g(),
                                    crate::theme::Colors::SUCCESS_FLASH.b(),
                                    alpha,
                                )
                            };
                            ui.label(
                                egui::RichText::new(msg)
                                    .color(color)
                                    .size(crate::theme::FontSize::SM),
                            );
                            ctx.request_repaint();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Label::new(
                                    egui::RichText::new("? Shortcuts")
                                        .size(crate::theme::FontSize::SM)
                                        .color(crate::theme::Colors::TEXT_MUTED),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.show_shortcuts = true;
                        }

                        ui.add_space(crate::theme::Spacing::SM * 2.0);

                        if !self.workspace_name.is_empty() {
                            ui.label(
                                egui::RichText::new(&self.workspace_name)
                                    .size(crate::theme::FontSize::SM)
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                        }
                    });
                });
            });
    }
}

impl eframe::App for MercuryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global Shortcuts
        // Escape cancels running request
        if self.ongoing_request.is_some() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.cancel_request();
            ctx.request_repaint();
        }

        // Check for changes and auto-save (every 5 seconds)
        let current_time = ctx.input(|i| i.time);
        self.check_for_changes();
        if self.has_unsaved_changes
            && current_time - self.last_save_time > 5.0
            && self.save_current_file()
        {
            self.last_save_time = current_time;
        }

        if let Ok((id, result)) = self.response_rx.try_recv() {
            // Only process if it matches ongoing request
            let is_match = self
                .ongoing_request
                .is_some_and(|(ongoing_id, _)| ongoing_id == id);

            if is_match {
                self.ongoing_request = None;
                match result {
                    Ok(response) => {
                        let time = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64();

                        // Format response type as string for history
                        let response_type_str = format!("{:?}", response.response_type);

                        // Store only a small preview of response body to save memory
                        // Full response is always viewable by re-running the request
                        let stored_body = if response.body.len() > 500 {
                            let preview: String = response.body.chars().take(500).collect();
                            format!("{}...", preview)
                        } else {
                            response.body.clone()
                        };

                        let entry = TimelineEntry {
                            timestamp: time,
                            method: self.method.clone(),
                            url: self.url.clone(),
                            status: response.status,
                            status_text: response.status_text.clone(),
                            duration_ms: response.duration_ms,
                            request_body: self.body_text.clone(),
                            request_headers: self.headers_text.clone(),
                            response_body: stored_body,
                            response_type: response_type_str,
                            response_size: response.size_bytes,
                            content_type: response.content_type.clone(),
                        };
                        self.timeline.push(entry);

                        if self.timeline.len() > crate::core::constants::MAX_TIMELINE_ENTRIES {
                            self.timeline.remove(0);
                        }

                        // Save history to disk after each request
                        self.save_history();

                        // Save to Recent (only if not a saved file AND it's a new unique request)
                        if self.current_file.is_none() && !self.url.is_empty() {
                            let method_str = format!("{:?}", self.method);
                            // Check if this exact request already exists
                            let exists = self.temp_requests.iter().any(|t| {
                                t.url == self.url
                                    && t.method == method_str
                                    && t.headers == self.headers_text
                                    && t.body == self.body_text
                            });

                            if !exists {
                                let temp_req = TempRequest {
                                    method: method_str,
                                    url: self.url.clone(),
                                    headers: self.headers_text.clone(),
                                    body: self.body_text.clone(),
                                    timestamp: time,
                                };
                                self.temp_requests.push(temp_req);
                                self.save_temp_requests();
                            }
                        }

                        // Track previous response for diff
                        self.previous_response = self.response.take();
                        self.response = Some(response);
                        self.formatted_response_cache = None; // Invalidate cache
                        self.request_error = None;
                        self.last_action_message =
                            Some(("Request completed".to_string(), time, false));
                    }
                    Err(e) => {
                        self.request_error = Some(e.clone());
                        let time = ctx.input(|i| i.time);
                        self.last_action_message =
                            Some((format!("Request failed: {}", e), time, true));
                        ctx.request_repaint();
                    }
                }
            } // matched
        } // received

        // Check for folder selection from async dialog
        if let Ok(path) = self.folder_rx.try_recv() {
            self.load_workspace(path);
            ctx.request_repaint();
        }

        // Check for file system changes from watcher
        // Check for file system changes from watcher
        let mut needs_rebuild = false;
        while let Ok(msg) = self.watcher_rx.try_recv() {
            match msg {
                Ok(_) => needs_rebuild = true,
                Err(e) => {
                    self.last_action_message = Some((e, ctx.input(|i| i.time), true));
                    ctx.request_repaint();
                }
            }
        }

        if needs_rebuild {
            // Rebuild tree while preserving expanded state
            self.build_collection_tree();

            // Handle edge case: current file was deleted externally
            if let Some(ref current_path) = self.current_file {
                if !current_path.exists() {
                    self.current_file = None;
                    self.url.clear();
                    self.headers_text.clear();
                    self.body_text.clear();
                    self.response = None;
                    self.last_action_message = Some((
                        "File was deleted externally".to_string(),
                        ctx.input(|i| i.time),
                        true,
                    ));
                }
            }
            ctx.request_repaint();
        }

        // Execute deferred actions (after keyboard input processing)
        if self.should_create_new_request {
            self.should_create_new_request = false;
            self.clear_request_form();
            self.should_focus_url_bar = true;
            self.last_action_message =
                Some(("New request".to_string(), ctx.input(|i| i.time), false));
            ctx.request_repaint();
        }

        if self.should_execute_request {
            self.should_execute_request = false;
            self.execute_request(ctx);
        }

        if self.should_open_folder_dialog {
            self.should_open_folder_dialog = false;
            let tx = self.folder_tx.clone();
            std::thread::spawn(move || {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let _ = tx.send(path);
                }
            });
        }

        if self.should_open_insomnia_import {
            self.should_open_insomnia_import = false;
            let current_workspace = self.workspace_path.clone();
            let folder_tx = self.folder_tx.clone();

            std::thread::spawn(move || {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("Insomnia Export", &["json", "yaml", "yml"])
                    .set_title("Select Insomnia Export File")
                    .pick_file()
                {
                    // Determine where to save:
                    // 1. If we have a workspace, use it.
                    // 2. If not, ask user to pick a folder.
                    let target_folder = if let Some(ws_path) = current_workspace {
                        Some(ws_path)
                    } else {
                        rfd::FileDialog::new()
                            .set_title("Choose where to save imported collection")
                            .set_directory(
                                dirs::document_dir()
                                    .unwrap_or_else(|| std::path::PathBuf::from("~")),
                            )
                            .set_file_name("Mercury")
                            .pick_folder()
                    };

                    if let Some(folder_path) = target_folder {
                        match crate::importer::import_insomnia_collection(&file_path, &folder_path)
                        {
                            Ok((req_count, env_count)) => {
                                println!(
                                    "✅ Imported {} requests and {} environments to {}",
                                    req_count,
                                    env_count,
                                    folder_path.display()
                                );
                                // Always reload workspace (if we picked a new one, or just refreshed current)
                                let _ = folder_tx.send(folder_path);
                            }
                            Err(e) => {
                                eprintln!("❌ Import failed: {}", e);
                            }
                        }
                    }
                }
            });
        }

        if self.should_open_postman_import {
            self.should_open_postman_import = false;
            let current_workspace = self.workspace_path.clone();
            let folder_tx = self.folder_tx.clone();

            std::thread::spawn(move || {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("Postman Collection", &["json"])
                    .set_title("Select Postman Collection File")
                    .pick_file()
                {
                    // Determine where to save:
                    // 1. If we have a workspace, use it.
                    // 2. If not, ask user to pick a folder.
                    let target_folder = if let Some(ws_path) = current_workspace {
                        Some(ws_path)
                    } else {
                        rfd::FileDialog::new()
                            .set_title("Choose where to save imported collection")
                            .set_directory(
                                dirs::document_dir()
                                    .unwrap_or_else(|| std::path::PathBuf::from("~")),
                            )
                            .set_file_name("Mercury")
                            .pick_folder()
                    };

                    if let Some(folder_path) = target_folder {
                        match crate::importer::import_postman_collection(&file_path, &folder_path) {
                            Ok((req_count, env_count)) => {
                                println!(
                                    "✅ Imported {} requests and {} environments to {}",
                                    req_count,
                                    env_count,
                                    folder_path.display()
                                );
                                // Always reload workspace (if we picked a new one, or just refreshed current)
                                let _ = folder_tx.send(folder_path);
                            }
                            Err(e) => {
                                eprintln!("❌ Import failed: {}", e);
                            }
                        }
                    }
                }
            });
        }

        if self.should_focus_search {
            self.should_focus_search = false;
            ctx.memory_mut(|mem| mem.request_focus(egui::Id::new("search_box")));
        }

        if self.should_copy_curl {
            self.should_copy_curl = false;
            self.copy_as_curl(ctx);
            let time = ctx.input(|i| i.time);
            self.copied_feedback_until = time + 2.0;
            self.last_action_message = Some(("Copied as cURL".to_string(), time, false));
            ctx.request_repaint();
        }

        // Top panel
        if let Ok(path) = self.folder_rx.try_recv() {
            self.load_workspace(path);
            ctx.request_repaint();
        }

        // Top panel with breadcrumb navigation
        egui::TopBottomPanel::top("top_panel")
            .exact_height(crate::theme::Layout::TOPBAR_HEIGHT)
            .frame(
                egui::Frame::NONE
                    .fill(crate::theme::Colors::BG_SURFACE)
                    .stroke(egui::Stroke::new(
                        crate::theme::StrokeWidth::THIN,
                        crate::theme::Colors::BORDER_SUBTLE,
                    ))
                    .inner_margin(egui::Margin::symmetric(crate::theme::Spacing::MD as i8, 0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Breadcrumb navigation: workspace / folder / request
                    if !self.workspace_name.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.workspace_name)
                                .size(crate::theme::FontSize::MD)
                                .color(crate::theme::Colors::TEXT_SECONDARY),
                        );

                        // Show folder path if request is in a subfolder
                        if let Some(ref path) = self.current_file {
                            if let Some(workspace) = &self.workspace_path {
                                if let Ok(relative) = path.strip_prefix(workspace) {
                                    let parts: Vec<_> = relative
                                        .parent()
                                        .and_then(|p| p.to_str())
                                        .unwrap_or("")
                                        .split(std::path::MAIN_SEPARATOR)
                                        .filter(|s| !s.is_empty())
                                        .collect();

                                    for part in parts {
                                        ui.label(
                                            egui::RichText::new("/")
                                                .size(crate::theme::FontSize::MD)
                                                .color(crate::theme::Colors::TEXT_MUTED),
                                        );
                                        ui.label(
                                            egui::RichText::new(part)
                                                .size(crate::theme::FontSize::MD)
                                                .color(crate::theme::Colors::TEXT_SECONDARY),
                                        );
                                    }
                                }
                            }

                            ui.label(
                                egui::RichText::new("/")
                                    .size(crate::theme::FontSize::MD)
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );

                            let request_name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("Untitled");

                            // HTTP Method badge
                            let method_color =
                                crate::theme::Colors::method_color(self.method.as_str());
                            ui.label(
                                egui::RichText::new(format!("{:?}", self.method))
                                    .size(crate::theme::FontSize::SM)
                                    .strong()
                                    .color(method_color),
                            );

                            ui.label(
                                egui::RichText::new(request_name)
                                    .size(crate::theme::FontSize::MD)
                                    .strong()
                                    .color(crate::theme::Colors::TEXT_PRIMARY),
                            );

                            // Unsaved changes indicator
                            if self.has_unsaved_changes {
                                ui.label(
                                    egui::RichText::new("●")
                                        .size(crate::theme::FontSize::SM)
                                        .color(crate::theme::Colors::WARNING),
                                );
                            }
                        } else {
                            ui.label(
                                egui::RichText::new("/")
                                    .size(crate::theme::FontSize::MD)
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.label(
                                egui::RichText::new("Untitled")
                                    .size(crate::theme::FontSize::MD)
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("No workspace")
                                .size(crate::theme::FontSize::MD)
                                .color(crate::theme::Colors::TEXT_MUTED),
                        );
                    }

                    ui.add_space(crate::theme::Spacing::LG);

                    // Search - minimal, fills space
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text(
                                egui::RichText::new("Search (Cmd+K)")
                                    .color(crate::theme::Colors::PLACEHOLDER),
                            )
                            .desired_width(crate::theme::Layout::POPUP_WIDE_WIDTH)
                            .frame(false)
                            .id(egui::Id::new("search_box")),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Environment selector - borderless, just text
                        let env_name = &self.env_files[self.selected_env];
                        let env_color = if env_name.contains("prod") {
                            crate::theme::Colors::ERROR
                        } else if env_name.contains("stag") {
                            crate::theme::Colors::WARNING
                        } else {
                            crate::theme::Colors::TEXT_SECONDARY
                        };

                        // Show disabled state if no workspace
                        let env_display = if self.workspace_path.is_none() && env_name == "None" {
                            "No env (open folder)"
                        } else {
                            &env_name.to_string()
                        };

                        let env_response = ui
                            .add_enabled(
                                self.workspace_path.is_some(),
                                egui::Label::new(
                                    egui::RichText::new(env_display)
                                        .size(crate::theme::FontSize::MD)
                                        .color(env_color),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        // Clone env_files to avoid borrow issues
                        let env_files_clone: Vec<_> = self.env_files.clone();
                        let current_selection = self.selected_env;
                        let mut new_selection = None;

                        egui::Popup::menu(&env_response)
                            .width(crate::theme::Layout::POPUP_MIN_WIDTH)
                            .gap(4.0)
                            .frame(
                                egui::Frame::popup(&ui.ctx().style())
                                    .fill(crate::theme::Colors::BG_MODAL)
                                    .corner_radius(crate::theme::Radius::MD)
                                    .stroke(egui::Stroke::new(
                                        crate::theme::StrokeWidth::THIN,
                                        crate::theme::Colors::BORDER_SUBTLE,
                                    ))
                                    .inner_margin(crate::theme::Spacing::SM),
                            )
                            .style(|style: &mut egui::Style| {
                                style.visuals.selection.bg_fill =
                                    crate::theme::Colors::popup_selection_bg();
                                style.visuals.widgets.hovered.bg_fill =
                                    crate::theme::Colors::popup_hover_bg();
                            })
                            .show(|ui| {
                                ui.set_min_height(100.0);
                                for (i, env) in env_files_clone.iter().enumerate() {
                                    let color = if env.contains("prod") {
                                        crate::theme::Colors::ERROR
                                    } else if env.contains("stag") {
                                        crate::theme::Colors::WARNING
                                    } else {
                                        crate::theme::Colors::TEXT_SECONDARY
                                    };
                                    if ui
                                        .selectable_label(
                                            current_selection == i,
                                            egui::RichText::new(env).color(color),
                                        )
                                        .clicked()
                                    {
                                        new_selection = Some(i);
                                        ui.close();
                                    }
                                }
                            });

                        // Apply selection change after popup closes
                        if let Some(i) = new_selection {
                            self.selected_env = i;
                            self.load_env();
                        }

                        ui.add_space(crate::theme::Spacing::XL);

                        // Open - borderless, just text
                        let open_response = ui
                            .add(
                                egui::Label::new(
                                    egui::RichText::new("Open")
                                        .size(crate::theme::FontSize::MD)
                                        .color(crate::theme::Colors::TEXT_SECONDARY),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);

                        egui::Popup::menu(&open_response)
                            .width(crate::theme::Layout::POPUP_WIDE_WIDTH)
                            .gap(4.0)
                            .frame(
                                egui::Frame::popup(&ui.ctx().style())
                                    .fill(crate::theme::Colors::BG_MODAL)
                                    .corner_radius(crate::theme::Radius::MD)
                                    .stroke(egui::Stroke::new(
                                        crate::theme::StrokeWidth::THIN,
                                        crate::theme::Colors::BORDER_SUBTLE,
                                    ))
                                    .inner_margin(crate::theme::Spacing::SM),
                            )
                            .style(|style: &mut egui::Style| {
                                style.visuals.selection.bg_fill =
                                    crate::theme::Colors::popup_selection_bg();
                                style.visuals.widgets.hovered.bg_fill =
                                    crate::theme::Colors::popup_hover_bg();
                            })
                            .show(|ui| {
                                if ui.selectable_label(false, "Open Folder...").clicked() {
                                    let tx = self.folder_tx.clone();
                                    std::thread::spawn(move || {
                                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                            let _ = tx.send(path);
                                        }
                                    });
                                    ui.close();
                                }
                                if ui.selectable_label(false, "Import Insomnia...").clicked() {
                                    self.should_open_insomnia_import = true;
                                    ui.close();
                                }
                                if ui.selectable_label(false, "Import Postman...").clicked() {
                                    self.should_open_postman_import = true;
                                    ui.close();
                                }
                            });

                        ui.add_space(crate::theme::Spacing::XL);

                        // Help - borderless
                        let help_response = ui
                            .add(
                                egui::Label::new(
                                    egui::RichText::new("Help")
                                        .size(crate::theme::FontSize::MD)
                                        .color(crate::theme::Colors::TEXT_SECONDARY),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);

                        egui::Popup::menu(&help_response)
                            .width(crate::theme::Layout::POPUP_MIN_WIDTH)
                            .gap(4.0)
                            .frame(
                                egui::Frame::popup(&ui.ctx().style())
                                    .fill(crate::theme::Colors::BG_MODAL)
                                    .corner_radius(crate::theme::Radius::MD)
                                    .stroke(egui::Stroke::new(
                                        crate::theme::StrokeWidth::THIN,
                                        crate::theme::Colors::BORDER_SUBTLE,
                                    ))
                                    .inner_margin(crate::theme::Spacing::SM),
                            )
                            .style(|style: &mut egui::Style| {
                                style.visuals.selection.bg_fill =
                                    crate::theme::Colors::popup_selection_bg();
                                style.visuals.widgets.hovered.bg_fill =
                                    crate::theme::Colors::popup_hover_bg();
                            })
                            .show(|ui| {
                                if ui.selectable_label(false, "Keyboard Shortcuts").clicked() {
                                    self.show_shortcuts = true;
                                    ui.close();
                                }
                                if ui.selectable_label(false, "About Mercury").clicked() {
                                    self.show_about = true;
                                    ui.close();
                                }
                                ui.separator();
                                if ui.selectable_label(false, "Check for Updates").clicked() {
                                    let _ = open::that(crate::core::constants::get_releases_url());
                                    ui.close();
                                }
                                if ui.selectable_label(false, "Documentation").clicked() {
                                    let _ = open::that(crate::core::constants::get_docs_url());
                                    ui.close();
                                }
                                if ui.selectable_label(false, "Report Issue").clicked() {
                                    let _ = open::that(crate::core::constants::get_issues_url());
                                    ui.close();
                                }
                            });
                    });
                });
            });

        // Render panels using new modular methods
        if !self.focus_mode {
            self.render_sidebar_panel(ctx);
        }

        self.render_response_panel_new(ctx);

        // Center: Request editor
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(crate::theme::Colors::BG_BASE)
                    .inner_margin(egui::Margin::same(crate::theme::Spacing::MD as i8)),
            )
            .show(ctx, |ui| {
                self.render_request_panel(ui, ctx);
            });

        // Status bar at bottom
        self.render_status_bar(ctx);

        // New Request Dialog
        if self.show_new_request_dialog {
            // Escape to close
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_new_request_dialog = false;
            }
            egui::Window::new("New Request")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::NONE
                        .fill(crate::theme::Colors::BG_MODAL)
                        .stroke(egui::Stroke::new(
                            crate::theme::StrokeWidth::THIN,
                            crate::theme::Colors::BORDER_SUBTLE,
                        ))
                        .corner_radius(crate::theme::Radius::MD)
                        .inner_margin(crate::theme::Spacing::MD),
                )
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Request name:")
                            .color(crate::theme::Colors::TEXT_SECONDARY),
                    );
                    ui.add_space(crate::theme::Spacing::XS);
                    let response = ui.text_edit_singleline(&mut self.new_request_name);
                    if response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !self.new_request_name.is_empty()
                    {
                        if let Some(parent) = self.context_menu_item.clone() {
                            let name = self.new_request_name.clone();
                            if let Err(e) = self.create_new_request(&parent, &name) {
                                self.last_action_message = Some((e, ctx.input(|i| i.time), true));
                            }
                        }
                        self.show_new_request_dialog = false;
                    }
                    ui.add_space(crate::theme::Spacing::SM);
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() && !self.new_request_name.is_empty() {
                            if let Some(parent) = self.context_menu_item.clone() {
                                let name = self.new_request_name.clone();
                                if let Err(e) = self.create_new_request(&parent, &name) {
                                    self.last_action_message =
                                        Some((e, ctx.input(|i| i.time), true));
                                } else {
                                    self.last_action_message = Some((
                                        "Request created".to_string(),
                                        ctx.input(|i| i.time),
                                        false,
                                    ));
                                }
                            }
                            self.show_new_request_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_request_dialog = false;
                        }
                    });
                });
        }

        // New Folder Dialog
        if self.show_new_folder_dialog {
            // Escape to close
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_new_folder_dialog = false;
            }
            egui::Window::new("New Folder")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::NONE
                        .fill(crate::theme::Colors::BG_MODAL)
                        .stroke(egui::Stroke::new(
                            crate::theme::StrokeWidth::THIN,
                            crate::theme::Colors::BORDER_SUBTLE,
                        ))
                        .corner_radius(crate::theme::Radius::MD)
                        .inner_margin(crate::theme::Spacing::MD),
                )
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Folder name:")
                            .color(crate::theme::Colors::TEXT_SECONDARY),
                    );
                    ui.add_space(crate::theme::Spacing::XS);
                    let response = ui.text_edit_singleline(&mut self.new_folder_name);
                    if response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !self.new_folder_name.is_empty()
                    {
                        if let Some(parent) = self.context_menu_item.clone() {
                            let name = self.new_folder_name.clone();
                            if let Err(e) = self.create_new_folder(&parent, &name) {
                                self.last_action_message = Some((e, ctx.input(|i| i.time), true));
                            }
                        }
                        self.show_new_folder_dialog = false;
                    }
                    ui.add_space(crate::theme::Spacing::SM);
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() && !self.new_folder_name.is_empty() {
                            if let Some(parent) = self.context_menu_item.clone() {
                                let name = self.new_folder_name.clone();
                                if let Err(e) = self.create_new_folder(&parent, &name) {
                                    self.last_action_message =
                                        Some((e, ctx.input(|i| i.time), true));
                                } else {
                                    self.last_action_message = Some((
                                        "Folder created".to_string(),
                                        ctx.input(|i| i.time),
                                        false,
                                    ));
                                }
                            }
                            self.show_new_folder_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_folder_dialog = false;
                        }
                    });
                });
        }

        // Rename Dialog
        if self.show_rename_dialog {
            // Escape to close
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_rename_dialog = false;
            }
            egui::Window::new("Rename")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::NONE
                        .fill(crate::theme::Colors::BG_MODAL)
                        .stroke(egui::Stroke::new(
                            crate::theme::StrokeWidth::THIN,
                            crate::theme::Colors::BORDER_SUBTLE,
                        ))
                        .corner_radius(crate::theme::Radius::MD)
                        .inner_margin(crate::theme::Spacing::MD),
                )
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("New name:")
                            .color(crate::theme::Colors::TEXT_SECONDARY),
                    );
                    ui.add_space(crate::theme::Spacing::XS);
                    let response = ui.text_edit_singleline(&mut self.rename_text);
                    if response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !self.rename_text.is_empty()
                    {
                        if let Some(old_path) = self.context_menu_item.clone() {
                            let new_name = self.rename_text.clone();
                            if let Err(e) = self.rename_item(&old_path, &new_name) {
                                self.last_action_message = Some((e, ctx.input(|i| i.time), true));
                            }
                        }
                        self.show_rename_dialog = false;
                    }
                    ui.add_space(crate::theme::Spacing::SM);
                    ui.horizontal(|ui| {
                        if ui.button("Rename").clicked() && !self.rename_text.is_empty() {
                            if let Some(old_path) = self.context_menu_item.clone() {
                                let new_name = self.rename_text.clone();
                                if let Err(e) = self.rename_item(&old_path, &new_name) {
                                    self.last_action_message =
                                        Some((e, ctx.input(|i| i.time), true));
                                } else {
                                    self.last_action_message = Some((
                                        "Renamed successfully".to_string(),
                                        ctx.input(|i| i.time),
                                        false,
                                    ));
                                }
                            }
                            self.show_rename_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_rename_dialog = false;
                        }
                    });
                });
        }

        // Delete Confirmation Dialog
        if self.show_delete_confirm {
            // Escape to cancel
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_delete_confirm = false;
            }
            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::NONE
                        .fill(crate::theme::Colors::BG_MODAL)
                        .stroke(egui::Stroke::new(
                            crate::theme::StrokeWidth::THIN,
                            crate::theme::Colors::BORDER_SUBTLE,
                        ))
                        .corner_radius(crate::theme::Radius::MD)
                        .inner_margin(crate::theme::Spacing::MD),
                )
                .show(ctx, |ui| {
                    let target_info = self.delete_target.as_ref().map(|t| {
                        (
                            t.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            t.is_dir(),
                            t.clone(),
                        )
                    });

                    if let Some((name, is_dir, target_path)) = target_info {
                        ui.label(
                            egui::RichText::new(format!(
                                "Are you sure you want to delete '{}'?",
                                name
                            ))
                            .color(crate::theme::Colors::TEXT_PRIMARY),
                        );
                        if is_dir {
                            ui.add_space(crate::theme::Spacing::XS);
                            ui.label(
                                egui::RichText::new(
                                    "⚠ This will delete the folder and all its contents!",
                                )
                                .color(crate::theme::Colors::ERROR),
                            );
                        }
                        ui.add_space(crate::theme::Spacing::MD);

                        // Check for Enter key to confirm
                        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

                        ui.horizontal(|ui| {
                            if ui
                                .button(
                                    egui::RichText::new("Delete")
                                        .color(crate::theme::Colors::ERROR),
                                )
                                .clicked()
                                || enter_pressed
                            {
                                if let Err(e) = self.delete_item(&target_path) {
                                    self.last_action_message =
                                        Some((e, ctx.input(|i| i.time), true));
                                } else {
                                    self.last_action_message = Some((
                                        "Deleted successfully".to_string(),
                                        ctx.input(|i| i.time),
                                        false,
                                    ));
                                }
                                self.show_delete_confirm = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_delete_confirm = false;
                            }
                        });
                    }
                });
        }

        // New Environment Dialog
        if self.show_new_env_dialog {
            // Escape to close
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_new_env_dialog = false;
            }
            egui::Window::new("New Environment")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::NONE
                        .fill(crate::theme::Colors::BG_MODAL)
                        .stroke(egui::Stroke::new(
                            crate::theme::StrokeWidth::THIN,
                            crate::theme::Colors::BORDER_SUBTLE,
                        ))
                        .corner_radius(crate::theme::Radius::MD)
                        .inner_margin(crate::theme::Spacing::MD),
                )
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Environment name (e.g., 'staging', 'production'):")
                            .color(crate::theme::Colors::TEXT_SECONDARY),
                    );
                    ui.add_space(crate::theme::Spacing::XS);
                    let response = ui.text_edit_singleline(&mut self.new_env_name);
                    if response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !self.new_env_name.is_empty()
                    {
                        let name = self.new_env_name.clone();
                        if let Err(e) = self.create_new_env(&name) {
                            self.last_action_message = Some((e, ctx.input(|i| i.time), true));
                        } else {
                            self.last_action_message = Some((
                                "Environment created".to_string(),
                                ctx.input(|i| i.time),
                                false,
                            ));
                        }
                        self.show_new_env_dialog = false;
                    }
                    ui.add_space(crate::theme::Spacing::SM);
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() && !self.new_env_name.is_empty() {
                            let name = self.new_env_name.clone();
                            if let Err(e) = self.create_new_env(&name) {
                                self.last_action_message = Some((e, ctx.input(|i| i.time), true));
                            } else {
                                self.last_action_message = Some((
                                    "Environment created".to_string(),
                                    ctx.input(|i| i.time),
                                    false,
                                ));
                            }
                            self.show_new_env_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_env_dialog = false;
                        }
                    });
                });
        }

        // Keyboard shortcuts help window
        if self.show_shortcuts {
            // Escape to close
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_shortcuts = false;
            }
            egui::Window::new("Keyboard Shortcuts")
                .collapsible(false)
                .resizable(false)
                .default_width(crate::theme::Layout::MODAL_WIDTH)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::NONE
                        .fill(crate::theme::Colors::BG_MODAL)
                        .stroke(egui::Stroke::new(
                            crate::theme::StrokeWidth::THIN,
                            crate::theme::Colors::BORDER_SUBTLE,
                        ))
                        .corner_radius(crate::theme::Radius::MD)
                        .inner_margin(crate::theme::Spacing::MD),
                )
                .show(ctx, |ui| {
                    ui.add_space(crate::theme::Spacing::SM);

                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Action")
                                    .strong()
                                    .color(crate::theme::Colors::TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new("Shortcut")
                                    .strong()
                                    .color(crate::theme::Colors::TEXT_PRIMARY),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("New Request")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + N")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Send Request")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + Enter")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Focus Search")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + K")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Focus URL Bar")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + L")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Copy as cURL")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + Shift + C")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Open Folder")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + O")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Switch Environment")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + E")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Delete Current Request")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Right-click > Delete")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Toggle Raw View")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + R")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Clear Search")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Escape")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Show Shortcuts")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("?").color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Focus Mode")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + Shift + F")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Toggle History")
                                    .color(crate::theme::Colors::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("Cmd/Ctrl + H")
                                    .color(crate::theme::Colors::TEXT_MUTED),
                            );
                            ui.end_row();
                        });

                    ui.add_space(crate::theme::Spacing::MD);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                self.show_shortcuts = false;
                            }
                        });
                    });
                });
        }

        // Handle keyboard shortcuts
        ctx.input(|i| {
            // Cmd/Ctrl + N: New request
            if i.key_pressed(egui::Key::N) && i.modifiers.command {
                self.should_create_new_request = true;
            }

            // Cmd/Ctrl + S: Save temp request (if not already saved)
            if i.key_pressed(egui::Key::S)
                && i.modifiers.command
                && self.current_file.is_none()
                && !self.url.is_empty()
            {
                if let Some(workspace) = self.workspace_path.as_ref() {
                    self.show_new_request_dialog = true;
                    self.new_request_name = String::new();
                    self.context_menu_item = Some(workspace.clone());
                } else {
                    // No workspace - prompt to open folder first
                    self.should_open_folder_dialog = true;
                    self.last_action_message = Some((
                        "Open a folder first to save requests".to_string(),
                        i.time,
                        false,
                    ));
                }
            }

            // Cmd/Ctrl + S: Save current file (if already saved)
            if i.key_pressed(egui::Key::S)
                && i.modifiers.command
                && self.current_file.is_some()
                && self.has_unsaved_changes
                && self.save_current_file()
            {
                self.last_save_time = i.time;
                self.last_action_message = Some(("Saved".to_string(), i.time, false));
            }

            // Cmd/Ctrl + Enter: Send request
            if i.key_pressed(egui::Key::Enter)
                && i.modifiers.command
                && self.ongoing_request.is_none()
            {
                self.should_execute_request = true;
            }

            // Cmd/Ctrl + K: Focus search
            if i.key_pressed(egui::Key::K) && i.modifiers.command {
                self.should_focus_search = true;
            }

            // Cmd/Ctrl + L: Focus URL bar
            if i.key_pressed(egui::Key::L) && i.modifiers.command {
                self.should_focus_url_bar = true;
            }

            // Cmd/Ctrl + Shift + C: Copy as cURL
            if i.key_pressed(egui::Key::C) && i.modifiers.command && i.modifiers.shift {
                self.should_copy_curl = true;
            }

            // Cmd/Ctrl + O: Open folder
            if i.key_pressed(egui::Key::O) && i.modifiers.command {
                self.should_open_folder_dialog = true;
            }

            // Cmd/Ctrl + R: Toggle raw view (if response exists)
            if i.key_pressed(egui::Key::R) && i.modifiers.command && self.response.is_some() {
                self.response_view_raw = !self.response_view_raw;
            }

            // Cmd/Ctrl + E: Cycle through environments
            if i.key_pressed(egui::Key::E) && i.modifiers.command && !self.env_files.is_empty() {
                self.selected_env = (self.selected_env + 1) % self.env_files.len();
                self.load_env();
            }

            // Escape: Clear search
            if i.key_pressed(egui::Key::Escape) && !self.search_query.is_empty() {
                self.search_query.clear();
            }

            // ? : Show keyboard shortcuts
            if i.key_pressed(egui::Key::Questionmark)
                || (i.key_pressed(egui::Key::Slash) && i.modifiers.shift)
            {
                self.show_shortcuts = !self.show_shortcuts;
            }

            // Cmd+Shift+F: Focus Mode
            if i.key_pressed(egui::Key::F) && i.modifiers.command && i.modifiers.shift {
                self.focus_mode = !self.focus_mode;
            }

            // Cmd+H: Toggle Timeline/History
            if i.key_pressed(egui::Key::H) && i.modifiers.command {
                self.show_timeline = !self.show_timeline;
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save current file if there are unsaved changes
        if self.has_unsaved_changes {
            self.save_current_file();
        }
        // Save app state when app closes
        self.save_state();
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;

    #[test]
    fn test_timeline_entry_serialization() {
        let entry = TimelineEntry {
            timestamp: 1702400000.0,
            method: HttpMethod::GET,
            url: "https://api.example.com/users".to_string(),
            status: 200,
            status_text: "OK".to_string(),
            duration_ms: 150,
            request_body: "".to_string(),
            request_headers: "Content-Type: application/json".to_string(),
            response_body: r#"{"id": 1}"#.to_string(),
            response_type: "Json".to_string(),
            response_size: 10,
            content_type: "application/json".to_string(),
        };

        // Serialize
        let json = serde_json::to_string(&entry).expect("Failed to serialize");
        assert!(json.contains("1702400000"));
        assert!(json.contains("GET"));
        assert!(json.contains("api.example.com"));

        // Deserialize
        let deserialized: TimelineEntry =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.url, entry.url);
        assert_eq!(deserialized.status, 200);
        assert_eq!(deserialized.duration_ms, 150);
        assert_eq!(deserialized.response_type, "Json");
    }

    #[test]
    fn test_timeline_entry_roundtrip() {
        let entries = vec![
            TimelineEntry {
                timestamp: 1702400000.0,
                method: HttpMethod::POST,
                url: "https://api.example.com/login".to_string(),
                status: 201,
                status_text: "Created".to_string(),
                duration_ms: 250,
                request_body: r#"{"email": "test@test.com"}"#.to_string(),
                request_headers: "".to_string(),
                response_body: r#"{"token": "abc123"}"#.to_string(),
                response_type: "Json".to_string(),
                response_size: 20,
                content_type: "application/json".to_string(),
            },
            TimelineEntry {
                timestamp: 1702400100.0,
                method: HttpMethod::DELETE,
                url: "https://api.example.com/users/5".to_string(),
                status: 204,
                status_text: "No Content".to_string(),
                duration_ms: 50,
                request_body: "".to_string(),
                request_headers: "Authorization: Bearer token".to_string(),
                response_body: "".to_string(),
                response_type: "Empty".to_string(),
                response_size: 0,
                content_type: "".to_string(),
            },
        ];

        let json = serde_json::to_string_pretty(&entries).expect("Failed to serialize");
        let deserialized: Vec<TimelineEntry> =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].method.as_str(), "POST");
        assert_eq!(deserialized[1].method.as_str(), "DELETE");
    }

    #[test]
    fn test_history_expiry_constant() {
        // 7 days in seconds
        let expected = 7.0 * 24.0 * 60.0 * 60.0;
        assert_eq!(crate::core::constants::HISTORY_EXPIRY_SECONDS, expected);
        assert_eq!(crate::core::constants::HISTORY_EXPIRY_SECONDS, 604800.0);
    }

    #[test]
    fn test_max_timeline_entries() {
        assert_eq!(crate::core::constants::MAX_TIMELINE_ENTRIES, 50);
    }
}
