use base64::prelude::*;

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
    }

    #[test]
    fn test_should_add_json_header() {
        assert!(should_add_json_header("{\"a\":1}", ""));
        assert!(should_add_json_header("[1,2]", "Header: Value"));
        assert!(!should_add_json_header("not json", ""));
        assert!(!should_add_json_header("{}", "Content-Type: text/plain"));
        assert!(!should_add_json_header("{}", "content-type: application/json"));
    }

    #[test]
    fn test_generate_basic_auth() {
        // user:pass -> dXNlcjpwYXNz
        assert_eq!(generate_basic_auth("user", "pass"), "Basic dXNlcjpwYXNz");
        assert_eq!(generate_basic_auth("admin", "1234"), "Basic YWRtaW46MTIzNA==");
    }
}
