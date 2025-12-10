pub const GITHUB_USERNAME: &str = "Harry-kp";
pub const GITHUB_REPO: &str = "mercury";

pub fn get_repo_url() -> String {
    format!("https://github.com/{}/{}", GITHUB_USERNAME, GITHUB_REPO)
}

pub fn get_issues_url() -> String {
    format!("{}/issues", get_repo_url())
}

pub fn get_releases_url() -> String {
    format!("{}/releases", get_repo_url())
}

pub const MAX_TIMELINE_ENTRIES: usize = 50;
pub const URL_TRUNCATE_LENGTH: usize = 35;
pub const FADE_DURATION_SECONDS: f64 = 3.0;

// Response Size Limits
// ---------------------
// These limits protect the UI from freezing when handling large API responses.
// The issue: rendering text requires cloning + formatting + syntax highlighting,
// which creates thousands of text spans per frame - too expensive for large data.

/// Maximum response size we'll even attempt to load into memory.
/// Responses larger than this show "Response Too Large" placeholder.
pub const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Maximum text size for inline display with formatting/highlighting.
/// Larger responses show "Large Response" placeholder with Save option.
/// User can download and view in their preferred editor.
pub const MAX_TEXT_DISPLAY_SIZE: usize = 1000 * 1024; // 1000KB
