//! Type Definitions Module
//!
//! Shared data structures used across the application.
//!
//! Core types:
//! - `HttpMethod`: HTTP request method (GET, POST, etc.)
//! - `Request`: Represents an HTTP request (method, url, headers, body)
//! - `Response`: Represents an HTTP response (status, body, timing)
//! - `JsonRequest`: JSON file format for collection request storage
//! - `RecentRequest`: A saved recent request with timestamp
//! - `TimelineEntry`: A history entry combining request + response

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// HTTP request method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "PATCH" => Some(HttpMethod::PATCH),
            "DELETE" => Some(HttpMethod::DELETE),
            "HEAD" => Some(HttpMethod::HEAD),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            "CONNECT" => Some(HttpMethod::CONNECT),
            "TRACE" => Some(HttpMethod::TRACE),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::CONNECT => "CONNECT",
            HttpMethod::TRACE => "TRACE",
        }
    }
}

/// JSON request file format for collection storage
///
/// This struct represents the JSON format used to store API requests
/// in collection files (.json files in workspace).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRequest {
    pub method: HttpMethod,
    pub url: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub body: String,
}

impl Default for JsonRequest {
    fn default() -> Self {
        Self {
            method: HttpMethod::GET,
            url: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }
}

/// Core HTTP request data
///
/// This is the unified type for representing request data across the application.
/// Used by `RecentRequest` (recent items) and `TimelineEntry` (history).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Request {
    pub method: HttpMethod,
    pub url: String,
    pub headers: String,
    pub body: String,
}

/// Core HTTP response data
///
/// Represents the essential response information stored in history.
/// Note: Full response details (raw bytes, cookies, etc.) are in `HttpResponse` in request.rs
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub content_type: String,
    pub response_type: String,
    pub size_bytes: usize,
    pub duration_ms: u128,
}

/// Recent request entry (unsaved requests)
///
/// Replaces the old `TempRequest` type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentRequest {
    pub request: Request,
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

/// Timeline entry for request history (full data - stored on disk)
///
/// Combines request and response data with a timestamp.
/// Full response body is stored on disk; loaded on-demand when user clicks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: f64,
    pub request: Request,
    pub response: Response,
}

/// Lightweight timeline summary for in-memory display
///
/// Contains only the metadata needed to render the history list.
/// Full data is loaded from disk via `load_history_entry()` when clicked.
#[derive(Clone, Debug)]
pub struct TimelineSummary {
    pub timestamp: f64,
    pub method: HttpMethod,
    pub url: String,
    pub status: u16,
    pub duration_ms: u128,
}

impl From<&TimelineEntry> for TimelineSummary {
    fn from(entry: &TimelineEntry) -> Self {
        Self {
            timestamp: entry.timestamp,
            method: entry.request.method.clone(),
            url: entry.request.url.clone(),
            status: entry.response.status,
            duration_ms: entry.response.duration_ms,
        }
    }
}
