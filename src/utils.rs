//! Utility Functions
//!
//! Helper functions for auth, URL handling, and header processing.

use base64::prelude::*;
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
        !headers
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .any(|l| l.trim_start().to_lowercase().starts_with("content-type:"))
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

/// Generate Bearer Auth header value (Bearer <token>)
pub fn generate_bearer_auth(token: &str) -> String {
    format!("Bearer {}", token)
}

// ============================================================================
// Query Parameter Utilities
// ============================================================================

/// Query parameter with enabled state for toggle functionality
#[derive(Clone, Debug, PartialEq)]
pub struct QueryParam {
    pub enabled: bool,
    pub key: String,
    pub value: String,
}

impl QueryParam {
    pub fn new(key: String, value: String) -> Self {
        Self {
            enabled: true,
            key,
            value,
        }
    }
}

/// Parse query parameters from a URL string
/// Decodes URL-encoded values (e.g., %20 -> space)
pub fn parse_query_params(url: &str) -> Vec<QueryParam> {
    // Find the query string start
    let query_start = match url.find('?') {
        Some(idx) => idx + 1,
        None => return Vec::new(),
    };

    // Get query string, excluding fragment if present
    let query_part = &url[query_start..];
    let query_str = query_part.split('#').next().unwrap_or("");

    if query_str.is_empty() {
        return Vec::new();
    }

    query_str
        .split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| {
            let (key, value) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, ""), // Key-only param like ?flag
            };
            QueryParam::new(url_decode(key), url_decode(value))
        })
        .collect()
}

/// Build a URL from base URL and query parameters
/// Only includes enabled parameters
/// Encodes values but preserves {{variable}} syntax
pub fn build_url_with_params(base_url: &str, params: &[QueryParam]) -> String {
    // Extract base URL without existing query string
    let base = get_base_url(base_url);

    // Filter to enabled params with non-empty keys
    let enabled_params: Vec<_> = params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .collect();

    if enabled_params.is_empty() {
        return base;
    }

    let query_string: String = enabled_params
        .iter()
        .map(|p| {
            let encoded_key = url_encode_preserve_vars(&p.key);
            let encoded_value = url_encode_preserve_vars(&p.value);
            if p.value.is_empty() {
                encoded_key
            } else {
                format!("{}={}", encoded_key, encoded_value)
            }
        })
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", base, query_string)
}

/// Extract base URL without query string or fragment
pub fn get_base_url(url: &str) -> String {
    url.split('?').next().unwrap_or(url).to_string()
}

/// Count enabled query parameters
pub fn count_enabled_params(params: &[QueryParam]) -> usize {
    params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .count()
}

/// URL decode a string (e.g., %20 -> space)
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Try to parse hex pair
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            // Failed to parse, keep original
            result.push('%');
            result.push_str(&hex);
        } else if c == '+' {
            result.push(' '); // + is space in query strings
        } else {
            result.push(c);
        }
    }

    result
}

/// URL encode a string, but preserve {{variable}} syntax
fn url_encode_preserve_vars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        // Check for {{ variable }} syntax
        if c == '{' && chars.peek() == Some(&'{') {
            // Preserve the entire {{...}} block
            result.push(c);
            result.push(chars.next().unwrap()); // second {

            // Copy until we find }}
            while let Some(inner) = chars.next() {
                result.push(inner);
                if inner == '}' && chars.peek() == Some(&'}') {
                    result.push(chars.next().unwrap()); // second }
                    break;
                }
            }
        } else if should_encode(c) {
            // URL encode the character
            for byte in c.to_string().as_bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Check if a character should be URL encoded
fn should_encode(c: char) -> bool {
    // RFC 3986: unreserved characters don't need encoding
    !matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~')
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
        // Should add if Content-Type is only in comment
        assert!(should_add_json_header(
            "{}",
            "# Content-Type: text/plain\nOther: Value"
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
    }

    #[test]
    fn test_generate_bearer_auth() {
        assert_eq!(generate_bearer_auth("token123"), "Bearer token123");
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

    // ========================================================================
    // Query Parameter Tests
    // ========================================================================

    #[test]
    fn test_parse_query_params_basic() {
        let params = parse_query_params("https://api.com/search?q=test&page=1");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].key, "q");
        assert_eq!(params[0].value, "test");
        assert!(params[0].enabled);
        assert_eq!(params[1].key, "page");
        assert_eq!(params[1].value, "1");
    }

    #[test]
    fn test_parse_query_params_empty() {
        // No query string
        let params = parse_query_params("https://api.com/users");
        assert!(params.is_empty());

        // Empty query string
        let params = parse_query_params("https://api.com?");
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_query_params_encoded() {
        let params = parse_query_params("https://api.com?name=John%20Doe&city=New+York");
        assert_eq!(params[0].value, "John Doe");
        assert_eq!(params[1].value, "New York");
    }

    #[test]
    fn test_parse_query_params_key_only() {
        let params = parse_query_params("https://api.com?flag&debug");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].key, "flag");
        assert_eq!(params[0].value, "");
        assert_eq!(params[1].key, "debug");
    }

    #[test]
    fn test_parse_query_params_with_fragment() {
        let params = parse_query_params("https://api.com?q=test#section");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].key, "q");
        assert_eq!(params[0].value, "test");
    }

    #[test]
    fn test_parse_query_params_duplicate_keys() {
        let params = parse_query_params("https://api.com?tag=a&tag=b&tag=c");
        assert_eq!(params.len(), 3);
        assert!(params.iter().all(|p| p.key == "tag"));
    }

    #[test]
    fn test_build_url_basic() {
        let params = vec![
            QueryParam::new("q".to_string(), "test".to_string()),
            QueryParam::new("page".to_string(), "1".to_string()),
        ];
        let url = build_url_with_params("https://api.com/search", &params);
        assert_eq!(url, "https://api.com/search?q=test&page=1");
    }

    #[test]
    fn test_build_url_disabled_params() {
        let params = vec![
            QueryParam::new("q".to_string(), "test".to_string()),
            QueryParam {
                enabled: false,
                key: "page".to_string(),
                value: "1".to_string(),
            },
        ];
        let url = build_url_with_params("https://api.com/search", &params);
        assert_eq!(url, "https://api.com/search?q=test");
    }

    #[test]
    fn test_build_url_empty_params() {
        let params: Vec<QueryParam> = vec![];
        let url = build_url_with_params("https://api.com/users", &params);
        assert_eq!(url, "https://api.com/users");
    }

    #[test]
    fn test_build_url_replaces_existing_params() {
        let params = vec![QueryParam::new("new".to_string(), "value".to_string())];
        let url = build_url_with_params("https://api.com?old=param", &params);
        assert_eq!(url, "https://api.com?new=value");
    }

    #[test]
    fn test_build_url_preserves_variables() {
        let params = vec![QueryParam::new(
            "token".to_string(),
            "{{API_KEY}}".to_string(),
        )];
        let url = build_url_with_params("https://api.com", &params);
        assert_eq!(url, "https://api.com?token={{API_KEY}}");
    }

    #[test]
    fn test_build_url_encodes_special_chars() {
        let params = vec![QueryParam::new("q".to_string(), "hello world".to_string())];
        let url = build_url_with_params("https://api.com", &params);
        assert_eq!(url, "https://api.com?q=hello%20world");
    }

    #[test]
    fn test_get_base_url() {
        assert_eq!(
            get_base_url("https://api.com/users"),
            "https://api.com/users"
        );
        assert_eq!(get_base_url("https://api.com?q=test"), "https://api.com");
        assert_eq!(get_base_url("localhost:3000/api?x=1"), "localhost:3000/api");
    }

    #[test]
    fn test_count_enabled_params() {
        let params = vec![
            QueryParam::new("a".to_string(), "1".to_string()),
            QueryParam {
                enabled: false,
                key: "b".to_string(),
                value: "2".to_string(),
            },
            QueryParam::new("c".to_string(), "3".to_string()),
            QueryParam::new("".to_string(), "empty_key".to_string()), // Empty key, shouldn't count
        ];
        assert_eq!(count_enabled_params(&params), 2);
    }
}
