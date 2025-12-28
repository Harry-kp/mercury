//! Persistence Module
//!
//! Handles saving and loading application state to disk.
//! All data is stored in ~/.mercury/ directory.

use super::constants::{HISTORY_EXPIRY_SECONDS, MAX_TIMELINE_ENTRIES};
use super::types::{AppState, RecentRequest, TimelineEntry};
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

pub fn load_recent_requests() -> Vec<RecentRequest> {
    let path = get_recent_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(requests) = serde_json::from_str::<Vec<RecentRequest>>(&content) {
                return requests;
            }
        }
    }
    Vec::new()
}

pub fn save_recent_requests(requests: &[RecentRequest]) {
    ensure_config_dir();
    let path = get_recent_file_path();

    let skip = requests.len().saturating_sub(50);
    let to_save: Vec<_> = requests.iter().skip(skip).cloned().collect();

    if let Ok(json) = serde_json::to_string_pretty(&to_save) {
        if let Err(e) = fs::write(&path, json) {
            eprintln!("Failed to save recent requests: {}", e);
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

/// Load only lightweight summaries for history list display.
/// Full entries are loaded on-demand via `load_history_entry()`.
pub fn load_history_summaries() -> Vec<super::types::TimelineSummary> {
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
                .iter()
                .filter(|e| e.timestamp > cutoff)
                .map(super::types::TimelineSummary::from)
                .collect();
        }
    }
    Vec::new()
}

/// Load a full history entry by timestamp (on-demand when user clicks).
/// Returns None if entry not found or file read fails.
pub fn load_history_entry(timestamp: f64) -> Option<TimelineEntry> {
    let path = get_history_file_path();
    if !path.exists() {
        return None;
    }
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(entries) = serde_json::from_str::<Vec<TimelineEntry>>(&content) {
            return entries.into_iter().find(|e| e.timestamp == timestamp);
        }
    }
    None
}
/// Append a new history entry to disk.
/// Loads existing history, adds new entry, enforces limits, and saves.
pub fn append_history_entry(entry: &TimelineEntry) {
    ensure_config_dir();
    let path = get_history_file_path();

    // Load existing entries
    let mut entries: Vec<TimelineEntry> = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Add new entry
    entries.push(entry.clone());

    // Apply expiry and limits
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let cutoff = now - HISTORY_EXPIRY_SECONDS;

    // Filter by expiry, take most recent entries, preserve chronological order
    let mut to_save: Vec<_> = entries
        .into_iter()
        .filter(|e| e.timestamp > cutoff)
        .collect();

    // Keep only the most recent MAX_TIMELINE_ENTRIES
    if to_save.len() > MAX_TIMELINE_ENTRIES {
        to_save = to_save.split_off(to_save.len() - MAX_TIMELINE_ENTRIES);
    }

    if let Ok(json) = serde_json::to_string_pretty(&to_save) {
        if let Err(e) = fs::write(&path, json) {
            eprintln!("Failed to save history: {}", e);
        }
    }
}

/// Clear all history entries from disk.
pub fn clear_history() {
    let path = get_history_file_path();
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
}
