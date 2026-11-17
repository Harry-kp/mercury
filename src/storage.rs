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
/// A body bigger than this keeps only its status and headers in history:
/// the whole file is rewritten on every send, and 50 × 10 MB made that slow.
pub const HISTORY_BODY_LIMIT: usize = 1024 * 1024;
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

/// History file plus the entries sent but not yet written to it.
///
/// Rewriting the file (up to 50 full responses) took ~60 ms per send, so it
/// happens on a background thread. A send is queued first, instantly, and
/// every read sees queued + on-disk entries under one lock, so a history
/// row clicked right after a send is never missing and two quick sends
/// never lose each other.
struct History {
    pending: Mutex<Vec<HistoryEntry>>,
}

static HISTORY: History = History {
    pending: Mutex::new(Vec::new()),
};

impl History {
    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<HistoryEntry>> {
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn all(&self, dir: &Path) -> Vec<HistoryEntry> {
        let pending = self.lock();
        let mut entries: Vec<HistoryEntry> = load_from(dir, "history.json").unwrap_or_default();
        entries.extend(pending.iter().cloned());
        entries
    }

    fn entry(&self, dir: &Path, timestamp: f64) -> Option<HistoryEntry> {
        let pending = self.lock();
        pending
            .iter()
            .find(|e| e.timestamp == timestamp)
            .cloned()
            .or_else(|| {
                load_from::<Vec<HistoryEntry>>(dir, "history.json")?
                    .into_iter()
                    .find(|e| e.timestamp == timestamp)
            })
    }

    fn queue(&self, entry: HistoryEntry) {
        self.lock().push(entry);
    }

    fn flush(&self, dir: &Path) {
        let mut pending = self.lock();
        if pending.is_empty() {
            return;
        }
        let mut entries: Vec<HistoryEntry> = load_from(dir, "history.json").unwrap_or_default();
        entries.append(&mut pending);
        save_to(dir, "history.json", &prune(entries, now()));
    }

    fn take(&self, dir: &Path) -> Vec<HistoryEntry> {
        let mut pending = self.lock();
        let mut entries: Vec<HistoryEntry> = load_from(dir, "history.json").unwrap_or_default();
        entries.append(&mut pending);
        let _ = fs::remove_file(dir.join("history.json"));
        entries
    }

    fn restore(&self, dir: &Path, mut old: Vec<HistoryEntry>) {
        let _pending = self.lock();
        old.extend(load_from::<Vec<HistoryEntry>>(dir, "history.json").unwrap_or_default());
        save_to(dir, "history.json", &prune(old, now()));
    }
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
    prune(HISTORY.all(&dir()), now())
        .iter()
        .map(HistorySummary::from)
        .collect()
}

/// Full entry (with response body) for a summary the user clicked.
pub fn load_history_entry(timestamp: f64) -> Option<HistoryEntry> {
    HISTORY.entry(&dir(), timestamp)
}

/// Record a send. Instant: visible to every history read from now on. Call
/// [`flush_history`] off the UI thread to write it.
pub fn queue_history(entry: HistoryEntry) {
    HISTORY.queue(entry);
}

/// Write queued entries to disk. Slow with a big history: background only.
pub fn flush_history() {
    HISTORY.flush(&dir());
}

/// Delete the history file, returning what it held so the clear can be undone.
pub fn take_history() -> Vec<HistoryEntry> {
    HISTORY.take(&dir())
}

/// Undo [`take_history`]: put `old` back in front of anything sent since.
pub fn restore_history(old: Vec<HistoryEntry>) {
    HISTORY.restore(&dir(), old);
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
                pretty: None,
            },
            env: Some(".env.dev".into()),
            file: None,
            error: None,
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
        assert_eq!(entries[0].error, None);
    }

    fn history() -> History {
        History {
            pending: Mutex::new(Vec::new()),
        }
    }

    /// A send is visible to history reads at once, before the background
    /// write lands (a row clicked right after sending used to be missing).
    #[test]
    fn a_queued_entry_is_readable_before_it_is_written() {
        let dir = tempfile::TempDir::new().unwrap();
        let h = history();
        let t = now();
        h.queue(entry(t));
        assert!(h.entry(dir.path(), t).is_some());
        assert_eq!(h.all(dir.path()).len(), 1);
        assert!(!dir.path().join("history.json").exists(), "not written yet");
        h.flush(dir.path());
        assert!(
            h.entry(dir.path(), t).is_some(),
            "and still there after the write"
        );
    }

    /// Entries are found by timestamp. Without serde_json's exact float
    /// parsing, about one in eight came back from disk a hair off, so a
    /// history row clicked after its write opened nothing.
    #[test]
    fn timestamps_survive_the_file_exactly() {
        let base = now();
        for t in (0..10_000).map(|i| base + i as f64 * 0.000_123_7) {
            let back: f64 = serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap();
            assert_eq!(back, t);
        }
    }

    /// Flushes run on background threads; concurrent ones must not lose
    /// each other's entries.
    #[test]
    fn concurrent_sends_keep_every_entry() {
        let dir = tempfile::TempDir::new().unwrap();
        let h = std::sync::Arc::new(history());
        let now = now();
        let threads: Vec<_> = (0..20)
            .map(|i| {
                let (dir, h) = (dir.path().to_path_buf(), h.clone());
                std::thread::spawn(move || {
                    h.queue(entry(now - i as f64));
                    h.flush(&dir);
                })
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
