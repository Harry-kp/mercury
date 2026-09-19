//! Application Constants
//!
//! Centralized configuration values: URLs, limits, and magic numbers.

pub const ISSUES_URL: &str = "https://github.com/Harry-kp/mercury/issues";
pub const RELEASES_URL: &str = "https://github.com/Harry-kp/mercury/releases";
pub const DOCS_URL: &str = "https://harry-kp.github.io/mercury/docs/getting-started";

pub const MAX_TIMELINE_ENTRIES: usize = 50;
pub const URL_TRUNCATE_LENGTH: usize = 35;
pub const HISTORY_URL_TRUNCATE_LENGTH: usize = 25;
pub const STATUS_MSG_TRUNCATE_LENGTH: usize = 60;
pub const COPY_CONFIRM_DURATION_SECONDS: f64 = 1.0;
pub const FADE_DURATION_SECONDS: f64 = 5.0; // Increased from 3.0 for better readability
pub const HISTORY_EXPIRY_SECONDS: f64 = 7.0 * 24.0 * 60.0 * 60.0; // 7 days

// Response Size Limits
// ---------------------
// These limits protect the UI from freezing when handling large API responses.
// The issue: rendering text requires cloning + formatting + syntax highlighting,
// which creates thousands of text spans per frame - too expensive for large data.

/// Maximum response size we'll even attempt to load into memory.
/// Responses larger than this show "Response Too Large" placeholder.
pub const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Maximum size for text display with syntax highlighting.
/// Responses larger than this skip highlighting and show plain text.
/// This keeps the UI at 60fps - character-by-character highlighting is expensive.
/// Also used as the threshold for ResponseType::LargeText classification.
pub const MAX_HIGHLIGHT_SIZE: usize = 100_000; // 100KB
