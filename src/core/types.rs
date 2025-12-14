//! Type Definitions Module
//!
//! Shared data structures used across the application.

use crate::parser::HttpMethod;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Temporary request for "Recent" history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempRequest {
    pub method: String,
    pub url: String,
    pub headers: String,
    pub body: String,
    pub timestamp: f64,
}

/// Persisted app state for restoring sessions
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub workspace_path: Option<String>,
    pub method: String,
    pub url: String,
    pub headers_text: String,
    pub body_text: String,
    pub auth_text: String,
    pub selected_tab: usize,
    pub selected_env: usize,
}

/// Collection tree item - folder or request file
#[derive(Clone, Debug)]
pub enum CollectionItem {
    Folder {
        name: String,
        path: PathBuf,
        expanded: bool,
        children: Vec<CollectionItem>,
    },
    Request {
        name: String,
        path: PathBuf,
        method: Option<HttpMethod>,
    },
}

/// Timeline entry for request history
///
/// Note: response_body is intentionally kept small (~500 chars) to save memory.
/// Users can re-execute requests from history to see full responses.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: f64,
    pub method: HttpMethod,
    pub url: String,
    pub status: u16,
    pub status_text: String,
    pub duration_ms: u128,
    pub request_body: String,
    pub request_headers: String,
    #[serde(default)]
    pub response_body: String,
    pub response_type: String,
    pub response_size: usize,
    pub content_type: String,
}
