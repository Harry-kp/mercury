//! Data types shared across the app, and the on-disk request file format.
//!
//! Everything here is serialized somewhere (workspace `.json` files or
//! `~/.mercury/*.json`), so field names are a compatibility contract.

use crate::http::HttpResponse;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
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
    pub const ALL: [HttpMethod; 9] = [
        HttpMethod::GET,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::PATCH,
        HttpMethod::DELETE,
        HttpMethod::HEAD,
        HttpMethod::OPTIONS,
        HttpMethod::CONNECT,
        HttpMethod::TRACE,
    ];

    /// Case-insensitive parse ("post" -> POST).
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|m| m.as_str().eq_ignore_ascii_case(s.trim()))
    }

    pub fn as_str(self) -> &'static str {
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

/// A request as stored in a workspace `.json` file.
///
/// Headers are a `BTreeMap` so files are written with a stable key order
/// (clean git diffs).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RequestFile {
    pub method: HttpMethod,
    pub url: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub body: String,
}

impl RequestFile {
    pub fn from_json(content: &str) -> Result<Self, String> {
        serde_json::from_str(content).map_err(|e| format!("Invalid request file: {e}"))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("RequestFile always serializes")
    }
}

/// The request form as the user sees it: headers are the editable text
/// (`Key: Value` per line, `#` disables a line). Used by history and recent.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Request {
    pub method: HttpMethod,
    pub url: String,
    pub headers: String,
    pub body: String,
}

/// An unsaved request the user sent (sidebar "Recent").
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentRequest {
    pub request: Request,
    pub timestamp: f64,
}

/// One sent request + its response, stored in `~/.mercury/history.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: f64,
    pub request: Request,
    pub response: HttpResponse,
}

/// What the history list needs in memory; full entries load on click.
#[derive(Clone, Debug)]
pub struct HistorySummary {
    pub timestamp: f64,
    pub method: HttpMethod,
    pub url: String,
    pub status: u16,
    pub duration_ms: u128,
}

impl From<&HistoryEntry> for HistorySummary {
    fn from(entry: &HistoryEntry) -> Self {
        Self {
            timestamp: entry.timestamp,
            method: entry.request.method,
            url: entry.request.url.clone(),
            status: entry.response.status,
            duration_ms: entry.response.duration_ms,
        }
    }
}

/// Session restored on launch (`~/.mercury/state.json`).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppState {
    pub workspace_path: Option<String>,
    pub method: HttpMethod,
    pub url: String,
    pub headers_text: String,
    pub body_text: String,
    pub selected_tab: usize,
    /// Environment file name, e.g. ".env.dev".
    pub env_name: Option<String>,
}

/// Sidebar tree node. `children` is `None` until the folder is first expanded.
#[derive(Clone, Debug)]
pub enum CollectionItem {
    Folder {
        name: String,
        path: PathBuf,
        children: Option<Vec<CollectionItem>>,
    },
    Request {
        name: String,
        path: PathBuf,
        method: Option<HttpMethod>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_parse_roundtrips_every_method() {
        for m in HttpMethod::ALL {
            assert_eq!(HttpMethod::parse(m.as_str()), Some(m));
            assert_eq!(HttpMethod::parse(&m.as_str().to_lowercase()), Some(m));
        }
        assert_eq!(HttpMethod::parse("FETCH"), None);
    }

    #[test]
    fn request_file_parses_minimal_and_full() {
        let r = RequestFile::from_json(r#"{"method": "GET", "url": "https://a.com"}"#).unwrap();
        assert_eq!(r.method, HttpMethod::GET);
        assert!(r.headers.is_empty() && r.body.is_empty());

        let r = RequestFile::from_json(
            r#"{"method":"POST","url":"u","headers":{"B":"2","A":"1"},"body":"x"}"#,
        )
        .unwrap();
        assert_eq!(r.headers.get("A").map(String::as_str), Some("1"));
        assert_eq!(r.body, "x");
        assert!(RequestFile::from_json("NOT JSON").is_err());
    }

    #[test]
    fn request_file_roundtrips_with_stable_header_order() {
        let mut r = RequestFile {
            method: HttpMethod::DELETE,
            url: "https://a.com/1".into(),
            ..Default::default()
        };
        r.headers.insert("Zeta".into(), "z".into());
        r.headers.insert("Alpha".into(), "{{token}}".into());
        let json = r.to_json();
        assert!(json.find("Alpha").unwrap() < json.find("Zeta").unwrap());
        assert_eq!(RequestFile::from_json(&json).unwrap(), r);
    }

    #[test]
    fn app_state_ignores_legacy_fields() {
        // v0.2 wrote method as a string plus auth_text/selected_env
        let old = r#"{"workspace_path":null,"method":"TRACE","url":"u","headers_text":"",
            "body_text":"","auth_text":"","selected_tab":1,"selected_env":2}"#;
        let s: AppState = serde_json::from_str(old).unwrap();
        assert_eq!(s.method, HttpMethod::TRACE);
        assert_eq!(s.selected_tab, 1);
    }
}
