//! Persistence Module
//!
//! Handles saving and loading application state to disk.
//! All data is stored in ~/.mercury/ directory.

use crate::constants::{HISTORY_EXPIRY_SECONDS, MAX_TIMELINE_ENTRIES};
use crate::types::{AppState, TempRequest, TimelineEntry};
use std::fs;
use std::path::PathBuf;

/// Get the Mercury config directory (~/.mercury)
fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir()
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir);
    home.join(".mercury")
}

/// Ensure config directory exists
fn ensure_config_dir() {
    let dir = get_config_dir();
    let _ = fs::create_dir_all(&dir);
}

// ============ Recent Requests ============

pub fn get_recent_file_path() -> PathBuf {
    get_config_dir().join("recent.json")
}

pub fn load_temp_requests() -> Vec<TempRequest> {
    let path = get_recent_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(requests) = serde_json::from_str::<Vec<TempRequest>>(&content) {
                return requests;
            }
        }
    }
    Vec::new()
}

pub fn save_temp_requests(requests: &[TempRequest]) {
    ensure_config_dir();
    let path = get_recent_file_path();

    let skip = requests.len().saturating_sub(50);
    let to_save: Vec<_> = requests.iter().skip(skip).cloned().collect();

    if let Ok(json) = serde_json::to_string_pretty(&to_save) {
        if let Err(e) = fs::write(&path, json) {
            eprintln!("Failed to save temp history: {}", e);
        }
    }
}

// ============ App State ============

pub fn get_state_file_path() -> PathBuf {
    get_config_dir().join("state.json")
}

pub fn load_state() -> Option<AppState> {
    let path = get_state_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<AppState>(&content) {
                return Some(state);
            }
        }
    }
    None
}

pub fn save_state(state: &AppState) {
    ensure_config_dir();
    let path = get_state_file_path();

    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(&path, json);
    }
}

// ============ History ============

pub fn get_history_file_path() -> PathBuf {
    get_config_dir().join("history.json")
}

pub fn load_history() -> Vec<TimelineEntry> {
    let path = get_history_file_path();
    if !path.exists() {
        return Vec::new();
    }
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(entries) = serde_json::from_str::<Vec<TimelineEntry>>(&content) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            let cutoff = now - HISTORY_EXPIRY_SECONDS;
            return entries
                .into_iter()
                .filter(|e| e.timestamp > cutoff)
                .collect();
        }
    }
    Vec::new()
}

pub fn save_history(timeline: &[TimelineEntry]) {
    ensure_config_dir();
    let path = get_history_file_path();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let cutoff = now - HISTORY_EXPIRY_SECONDS;

    // Filter by expiry, take most recent entries, preserve chronological order
    let mut to_save: Vec<_> = timeline
        .iter()
        .filter(|e| e.timestamp > cutoff)
        .rev()
        .take(MAX_TIMELINE_ENTRIES)
        .cloned()
        .collect();
    to_save.reverse();

    if let Ok(json) = serde_json::to_string_pretty(&to_save) {
        if let Err(e) = fs::write(&path, json) {
            eprintln!("Failed to save history: {}", e);
        }
    }
}
