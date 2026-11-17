//! App data in `~/.mercury/`: session state, recent requests, history.
//!
//! Set `MERCURY_HOME` to use another directory (tests, or running a dev
//! build without touching your real data).

use crate::model::{AppState, HistoryEntry, HistorySummary, RecentRequest};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const MAX_HISTORY: usize = 50;
pub const MAX_RECENT: usize = 50;
const HISTORY_TTL_SECS: f64 = 7.0 * 24.0 * 60.0 * 60.0;

/// Seconds since the Unix epoch (what all stored timestamps use).
pub fn now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

fn dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("MERCURY_HOME") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".mercury")
}

fn load<T: DeserializeOwned>(name: &str) -> Option<T> {
    load_from(&dir(), name)
}

fn load_from<T: DeserializeOwned>(dir: &Path, name: &str) -> Option<T> {
    let content = fs::read_to_string(dir.join(name)).ok()?;
    serde_json::from_str(&content).ok()
}

fn save<T: Serialize + ?Sized>(name: &str, value: &T) {
    save_to(&dir(), name, value)
}

fn save_to<T: Serialize + ?Sized>(dir: &Path, name: &str, value: &T) {
    let result = fs::create_dir_all(dir).and_then(|_| {
        let json = serde_json::to_string_pretty(value).expect("app data serializes");
        fs::write(dir.join(name), json)
    });
    if let Err(e) = result {
        eprintln!("Failed to save {name}: {e}");
    }
}

pub fn load_state() -> Option<AppState> {
    load("state.json")
}

pub fn save_state(state: &AppState) {
    save("state.json", state);
}

pub fn load_recent() -> Vec<RecentRequest> {
    load("recent.json").unwrap_or_default()
}

pub fn save_recent(requests: &[RecentRequest]) {
    let skip = requests.len().saturating_sub(MAX_RECENT);
    save("recent.json", &requests[skip..]);
}

fn load_history() -> Vec<HistoryEntry> {
    load("history.json").unwrap_or_default()
}

/// History is appended from a background thread (rewriting a file of up to
/// 50 full responses took ~60 ms per send on the UI thread), so every
/// read-modify-write of it goes through this lock, or two quick sends would
/// lose one entry.
static HISTORY: Mutex<()> = Mutex::new(());

fn history_lock() -> std::sync::MutexGuard<'static, ()> {
    HISTORY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Drop expired entries and keep the newest MAX_HISTORY (oldest first).
fn prune(entries: Vec<HistoryEntry>, now: f64) -> Vec<HistoryEntry> {
    let mut kept: Vec<_> = entries
        .into_iter()
        .filter(|e| e.timestamp > now - HISTORY_TTL_SECS)
        .collect();
    let excess = kept.len().saturating_sub(MAX_HISTORY);
    kept.drain(..excess);
    kept
}

pub fn load_history_summaries() -> Vec<HistorySummary> {
    let _lock = history_lock();
    prune(load_history(), now())
        .iter()
        .map(HistorySummary::from)
        .collect()
}

/// Full entry (with response body) for a summary the user clicked.
pub fn load_history_entry(timestamp: f64) -> Option<HistoryEntry> {
    let _lock = history_lock();
    load_history()
        .into_iter()
        .find(|e| e.timestamp == timestamp)
}

/// Slow with a big history: call it off the UI thread (`MercuryApp::spawn`).
pub fn append_history(entry: HistoryEntry) {
    append_in(&dir(), entry);
}

fn append_in(dir: &Path, entry: HistoryEntry) {
    let _lock = history_lock();
    let mut entries: Vec<HistoryEntry> = load_from(dir, "history.json").unwrap_or_default();
    entries.push(entry);
    save_to(dir, "history.json", &prune(entries, now()));
}

/// Delete the history file, returning what it held so the clear can be undone.
pub fn take_history() -> Vec<HistoryEntry> {
    let _lock = history_lock();
    let entries = load_history();
    let _ = fs::remove_file(dir().join("history.json"));
    entries
}

/// Undo [`take_history`]: put `old` back in front of anything sent since.
pub fn restore_history(mut old: Vec<HistoryEntry>) {
    let _lock = history_lock();
    old.extend(load_history());
    save("history.json", &prune(old, now()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{HttpResponse, ResponseType};
    use crate::model::{HttpMethod, Request};

    fn entry(timestamp: f64) -> HistoryEntry {
        HistoryEntry {
            timestamp,
            request: Request {
                method: HttpMethod::POST,
                url: "https://a.com".into(),
                headers: "A: 1".into(),
                body: "{}".into(),
            },
            response: HttpResponse {
                status: 201,
                status_text: "201 Created".into(),
                headers: vec![],
                cookies: vec![],
                body: "{\"id\":1}".into(),
                raw_bytes: None,
                duration_ms: 12,
                size_bytes: 8,
                content_type: "application/json".into(),
                response_type: ResponseType::Json,
            },
            env: Some(".env.dev".into()),
            file: None,
        }
    }

    #[test]
    fn prune_drops_expired_and_keeps_newest() {
        let now = 1_000_000_000.0;
        let mut entries = vec![entry(now - HISTORY_TTL_SECS - 1.0)];
        entries.extend((0..MAX_HISTORY + 5).map(|i| entry(now - 100.0 + i as f64)));
        let kept = prune(entries, now);
        assert_eq!(kept.len(), MAX_HISTORY);
        assert_eq!(kept[0].timestamp, now - 100.0 + 5.0);
    }

    #[test]
    fn reads_v0_2_history_format() {
        // entries written before HttpResponse was stored directly (no cookies field)
        let old = r#"[{"timestamp":1702400000.0,
            "request":{"method":"GET","url":"u","headers":"","body":""},
            "response":{"status":204,"status_text":"No Content","headers":[],"body":"",
              "content_type":"","response_type":"Empty","size_bytes":0,"duration_ms":50}}]"#;
        let entries: Vec<HistoryEntry> = serde_json::from_str(old).unwrap();
        assert_eq!(entries[0].response.response_type, ResponseType::Empty);
        assert!(entries[0].response.cookies.is_empty());
        // 0.4 added which environment and file a send came from
        assert_eq!(entries[0].env, None);
        assert_eq!(entries[0].file, None);
    }

    /// Appends now run on background threads; concurrent ones must not
    /// lose each other's entries.
    #[test]
    fn concurrent_appends_keep_every_entry() {
        let dir = tempfile::TempDir::new().unwrap();
        let now = now();
        let threads: Vec<_> = (0..20)
            .map(|i| {
                let dir = dir.path().to_path_buf();
                std::thread::spawn(move || append_in(&dir, entry(now - i as f64)))
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        let kept: Vec<HistoryEntry> = load_from(dir.path(), "history.json").unwrap();
        assert_eq!(kept.len(), 20);
    }

    #[test]
    fn history_entry_roundtrips() {
        let json = serde_json::to_string(&entry(1.0)).unwrap();
        let back: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.request, entry(1.0).request);
        assert_eq!(back.response.status, 201);
    }
}
