use base64::prelude::*;

// Public Enum for Auth Mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    None,
    Basic,
    Bearer,
    Custom,
}

/// Infer Auth state from existing header text
pub fn infer_auth_config(auth_text: &str) -> (AuthMode, String, String, String) {
    let mut mode = AuthMode::None;
    let mut username = String::new();
    let mut password = String::new();
    let mut token = String::new();

    let text = auth_text.trim();
    if text.starts_with("Basic ") {
        mode = AuthMode::Basic;
        if let Some(encoded) = text.strip_prefix("Basic ") {
            if let Ok(decoded_bytes) = BASE64_STANDARD.decode(encoded.trim()) {
                if let Ok(decoded) = String::from_utf8(decoded_bytes) {
                    if let Some((u, p)) = decoded.split_once(':') {
                        username = u.to_string();
                        password = p.to_string();
                    }
                }
            }
        }
    } else if text.starts_with("Bearer ") {
        mode = AuthMode::Bearer;
        token = text.strip_prefix("Bearer ").unwrap_or(text).to_string();
    } else if !text.is_empty() {
        mode = AuthMode::Custom;
    }

    (mode, username, password, token)
}

/// Count non-empty, non-comment header lines
pub fn count_active_headers(headers_text: &str) -> usize {
    headers_text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .count()
}

/// Prepend http:// to url if protocol is missing
pub fn sanitize_url(url: &str) -> String {
    if !url.is_empty() && !url.starts_with("http://") && !url.starts_with("https://") {
        format!("http://{}", url)
    } else {
        url.to_string()
    }
}

/// Check if Content-Type: application/json should be added
pub fn should_add_json_header(body: &str, headers: &str) -> bool {
    let body = body.trim();
    if body.starts_with('{') || body.starts_with('[') {
        !headers.to_lowercase().contains("content-type")
    } else {
        false
    }
}

/// Generate Basic Auth header value (Basic <base64>)
pub fn generate_basic_auth(username: &str, password: &str) -> String {
    let creds = format!("{}:{}", username, password);
    let encoded = BASE64_STANDARD.encode(creds);
    format!("Basic {}", encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_url() {
        assert_eq!(sanitize_url("google.com"), "http://google.com");
        assert_eq!(sanitize_url("http://test.com"), "http://test.com");
        assert_eq!(sanitize_url("https://secure.com"), "https://secure.com");
        assert_eq!(sanitize_url("localhost:3000"), "http://localhost:3000");
        assert_eq!(sanitize_url(""), "");
    }

    #[test]
    fn test_should_add_json_header() {
        assert!(should_add_json_header("{\"a\":1}", ""));
        assert!(should_add_json_header("[1,2]", "Header: Value"));
        assert!(!should_add_json_header("not json", ""));
        assert!(!should_add_json_header("{}", "Content-Type: text/plain"));
        assert!(!should_add_json_header(
            "{}",
            "content-type: application/json"
        ));
    }

    #[test]
    fn test_generate_basic_auth() {
        // user:pass -> dXNlcjpwYXNz
        assert_eq!(generate_basic_auth("user", "pass"), "Basic dXNlcjpwYXNz");
        assert_eq!(
            generate_basic_auth("admin", "1234"),
            "Basic YWRtaW46MTIzNA=="
        );
        assert_eq!(
            generate_basic_auth("admin", "1234"),
            "Basic YWRtaW46MTIzNA=="
        );
    }

    #[test]
    fn test_infer_auth_config() {
        // Basic
        let (mode, u, p, t) = infer_auth_config("Basic dXNlcjpwYXNz");
        assert_eq!(mode, AuthMode::Basic);
        assert_eq!(u, "user");
        assert_eq!(p, "pass");
        assert_eq!(t, "");

        // Bearer
        let (mode, u, p, t) = infer_auth_config("Bearer secret_token");
        assert_eq!(mode, AuthMode::Bearer);
        assert_eq!(u, "");
        assert_eq!(p, "");
        assert_eq!(t, "secret_token");

        // Bearer with "Bearer " in token (recurse check)
        let (mode, _u, _p, t) = infer_auth_config("Bearer Bearer Token");
        assert_eq!(mode, AuthMode::Bearer);
        assert_eq!(t, "Bearer Token");

        // Custom
        let (mode, u, p, t) = infer_auth_config("X-Custom: Value");
        assert_eq!(mode, AuthMode::Custom);
        assert_eq!(u, "");
        assert_eq!(p, "");
        assert_eq!(t, "");

        // None
        let (mode, _u, _p, _t) = infer_auth_config("");
        assert_eq!(mode, AuthMode::None);
    }

    #[test]
    fn test_count_active_headers() {
        assert_eq!(count_active_headers("H: V"), 1);
        assert_eq!(count_active_headers("H: V\nH2: V2"), 2);
        assert_eq!(count_active_headers("# Comment\n\n"), 0);
        assert_eq!(count_active_headers("H: V\n# Disabled\nH3: V3"), 2);
    }
}
