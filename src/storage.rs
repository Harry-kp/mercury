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

fn save<T: Serialize + ?Sized>(name: &str, value: &T) -> Result<(), String> {
    save_to(&dir(), name, value)
}

/// Written beside the file and then renamed over it: a full disk or a crash
/// mid-write used to leave a cut-off file, which reads back as empty (the
/// whole history gone).
fn save_to<T: Serialize + ?Sized>(dir: &Path, name: &str, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).expect("app data serializes");
    fs::create_dir_all(dir)
        .and_then(|_| write_atomic(&dir.join(name), json.as_bytes()))
        .map_err(|e| format!("Could not save {name} in {}: {e}", dir.display()))
}

/// Replace `path`'s contents all at once: written beside it, then renamed
/// over it, so a full disk or a crash never leaves it cut short. A symlink
/// is written through (its target is replaced, the link stays), and the
/// file keeps its permissions (an `.env` may be private; a new file is
/// owner-only).
pub fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let target = match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => fs::canonicalize(path)?,
        _ => path.to_path_buf(),
    };
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    let permissions = fs::metadata(&target).ok().map(|m| m.permissions());
    // hidden, so a workspace never lists it; unguessable and created
    // exclusively (O_EXCL never follows a planted symlink), and never more
    // readable than the file it replaces, not even for a moment
    let (tmp, mut file) = loop {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = target.with_file_name(format!(".{name}.{}.{nanos}.{n}.tmp", std::process::id()));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            options.mode(permissions.as_ref().map_or(0o600, |p| p.mode() & 0o777));
        }
        match options.open(&tmp) {
            Ok(file) => break (tmp, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    };
    let result = permissions
        .map_or(Ok(()), |p| file.set_permissions(p))
        .and_then(|_| file.write_all(data))
        .and_then(|_| file.sync_all())
        .and_then(|_| fs::rename(&tmp, &target));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

pub fn load_state() -> Option<AppState> {
    load("state.json")
}

pub fn save_state(state: &AppState) -> Result<(), String> {
    save("state.json", state)
}

pub fn load_recent() -> Vec<RecentRequest> {
    load("recent.json").unwrap_or_default()
}

pub fn save_recent(requests: &[RecentRequest]) -> Result<(), String> {
    let skip = requests.len().saturating_sub(MAX_RECENT);
    save("recent.json", &requests[skip..])
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

    fn flush(&self, dir: &Path) -> Result<(), String> {
        let mut pending = self.lock();
        if pending.is_empty() {
            return Ok(());
        }
        let _other_windows = file_lock(dir);
        let mut entries: Vec<HistoryEntry> = load_from(dir, "history.json").unwrap_or_default();
        entries.extend(pending.iter().cloned());
        let saved = save_to(dir, "history.json", &prune(entries, now()));
        // unwritten, it stays queued: still in the list this session
        *pending = match saved {
            Ok(()) => Vec::new(),
            Err(_) => prune(std::mem::take(&mut *pending), now()),
        };
        saved
    }

    fn take(&self, dir: &Path) -> Result<Vec<HistoryEntry>, String> {
        let mut pending = self.lock();
        let _other_windows = file_lock(dir);
        let mut entries: Vec<HistoryEntry> = load_from(dir, "history.json").unwrap_or_default();
        match fs::remove_file(dir.join("history.json")) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(format!("Could not clear history: {e}"))
            }
            _ => {}
        }
        entries.append(&mut pending);
        Ok(entries)
    }

    fn restore(&self, dir: &Path, mut old: Vec<HistoryEntry>) -> Result<(), String> {
        let _pending = self.lock();
        let _other_windows = file_lock(dir);
        old.extend(load_from::<Vec<HistoryEntry>>(dir, "history.json").unwrap_or_default());
        save_to(dir, "history.json", &prune(old, now()))
    }
}

/// Held across a read-modify-write of history.json, so two Mercury windows
/// can't each write back a list without the other's send. Released on drop.
/// Without it (no lock support) writes go ahead as before.
fn file_lock(dir: &Path) -> Option<fs::File> {
    fs::create_dir_all(dir).ok()?;
    // never truncated: Windows refuses that while another window holds it
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(dir.join("history.lock"))
        .ok()?;
    file.lock().ok()?;
    Some(file)
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
pub fn flush_history() -> Result<(), String> {
    HISTORY.flush(&dir())
}

/// Delete the history file, returning what it held so the clear can be undone.
pub fn take_history() -> Result<Vec<HistoryEntry>, String> {
    HISTORY.take(&dir())
}

/// Undo [`take_history`]: put `old` back in front of anything sent since.
pub fn restore_history(old: Vec<HistoryEntry>) -> Result<(), String> {
    HISTORY.restore(&dir(), old)
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
        h.flush(dir.path()).unwrap();
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
                    h.flush(&dir).unwrap();
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        let kept: Vec<HistoryEntry> = load_from(dir.path(), "history.json").unwrap();
        assert_eq!(kept.len(), 20);
    }

    /// Two Mercury windows share one history file: each one's write used to
    /// be built from what it read before the other's landed, so one lost
    /// the other's sends.
    #[test]
    fn two_processes_keep_each_others_entries() {
        let dir = tempfile::TempDir::new().unwrap();
        let (a, b) = (
            std::sync::Arc::new(history()),
            std::sync::Arc::new(history()),
        );
        let now = now();
        let threads: Vec<_> = (0..40)
            .map(|i| {
                let dir = dir.path().to_path_buf();
                let h = if i % 2 == 0 { a.clone() } else { b.clone() };
                std::thread::spawn(move || {
                    h.queue(entry(now - i as f64));
                    h.flush(&dir).unwrap();
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        let kept: Vec<HistoryEntry> = load_from(dir.path(), "history.json").unwrap();
        assert_eq!(kept.len(), 40);
    }

    /// An app-data folder Mercury can't write must say so: history and the
    /// unsaved list used to vanish on relaunch with only a line on stderr.
    #[cfg(unix)]
    #[test]
    fn an_unwritable_home_is_an_error() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::TempDir::new().unwrap();
        save_to(dir.path(), "recent.json", &[1]).unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
        let result = save_to(dir.path(), "recent.json", &[2]);
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        let error = result.unwrap_err();
        assert!(error.starts_with("Could not save recent.json"), "{error}");
        // and the old file is untouched, not half-written
        assert_eq!(
            load_from::<Vec<i32>>(dir.path(), "recent.json"),
            Some(vec![1])
        );
    }

    /// Request and env files are saved this way too: a symlinked request
    /// stays a link to the file that changed, and a private `.env` stays
    /// private.
    #[cfg(unix)]
    #[test]
    fn write_atomic_goes_through_links_and_keeps_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::TempDir::new().unwrap();
        let real = dir.path().join("shared.json");
        fs::write(&real, "old").unwrap();
        fs::set_permissions(&real, fs::Permissions::from_mode(0o600)).unwrap();
        let link = dir.path().join("link.json");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        write_atomic(&link, b"new").unwrap();
        assert!(fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read_to_string(&real).unwrap(), "new");
        let mode = fs::metadata(&real).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        let left: Vec<_> = fs::read_dir(dir.path()).unwrap().flatten().collect();
        assert_eq!(left.len(), 2, "no temp file left behind");
    }

    /// The temp file is never world-readable and never written through a
    /// link planted at a guessable name.
    #[cfg(unix)]
    #[test]
    fn write_atomic_temp_file_is_private_and_unguessable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::TempDir::new().unwrap();
        let victim = dir.path().join("victim");
        fs::write(&victim, "keep").unwrap();
        let planted = dir
            .path()
            .join(format!(".history.json.{}.tmp", std::process::id()));
        std::os::unix::fs::symlink(&victim, &planted).unwrap();

        let new = dir.path().join("history.json");
        write_atomic(&new, b"secret").unwrap();
        assert_eq!(fs::read_to_string(&victim).unwrap(), "keep");
        assert_eq!(fs::read_to_string(&new).unwrap(), "secret");
        let mode = fs::metadata(&new).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "a new file is owner-only");
    }

    #[test]
    fn history_entry_roundtrips() {
        let json = serde_json::to_string(&entry(1.0)).unwrap();
        let back: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.request, entry(1.0).request);
        assert_eq!(back.response.status, 201);
    }
}
