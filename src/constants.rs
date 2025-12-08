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
