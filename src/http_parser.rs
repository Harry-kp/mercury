use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "PATCH" => Some(HttpMethod::PATCH),
            "DELETE" => Some(HttpMethod::DELETE),
            "HEAD" => Some(HttpMethod::HEAD),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            "CONNECT" => Some(HttpMethod::CONNECT),
            "TRACE" => Some(HttpMethod::TRACE),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::CONNECT => "CONNECT",
            HttpMethod::TRACE => "TRACE",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

pub fn parse_http_file(content: &str) -> Result<HttpRequest, String> {
    let mut request = HttpRequest::default();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Err("Empty file".to_string());
    }

    // Parse first line: METHOD URL
    let first_line = lines[0].trim();
    // Split only on the first whitespace to separate Method and URL,
    // allowing spaces in the URL (e.g., for handlebars syntax)
    let parts: Vec<&str> = first_line.splitn(2, |c: char| c.is_whitespace()).collect();

    if parts.len() < 2 {
        return Err("Invalid first line. Expected: METHOD URL".to_string());
    }

    request.method = HttpMethod::from_str(parts[0])
        .ok_or_else(|| format!("Invalid HTTP method: {}", parts[0]))?;
    request.url = parts[1].trim().to_string();

    // Parse headers and body
    let mut i = 1;
    let mut in_body = false;
    let mut body_lines = Vec::new();

    while i < lines.len() {
        let line = lines[i];

        // Empty line signals start of body
        if line.trim().is_empty() && !in_body {
            in_body = true;
            i += 1;
            continue;
        }

        if in_body {
            body_lines.push(line);
        } else if line.contains(':') {
            // Parse header
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                request
                    .headers
                    .insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }

        i += 1;
    }

    if !body_lines.is_empty() {
        request.body = Some(body_lines.join("\n"));
    }

    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get() {
        let content = "GET https://api.example.com/users";
        let request = parse_http_file(content).unwrap();

        assert!(matches!(request.method, HttpMethod::GET));
        assert_eq!(request.url, "https://api.example.com/users");
        assert!(request.headers.is_empty());
        assert!(request.body.is_none());
    }

    #[test]
    fn test_parse_with_headers() {
        let content = r#"POST https://api.example.com/users
Authorization: Bearer token123
Content-Type: application/json

{"name": "John"}"#;

        let request = parse_http_file(content).unwrap();

        assert!(matches!(request.method, HttpMethod::POST));
        assert_eq!(request.url, "https://api.example.com/users");
        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(request.body, Some(r#"{"name": "John"}"#.to_string()));
    }

    #[test]
    fn test_parse_url_with_spaces() {
        let content = "GET {{ base_url }}/users";
        let request = parse_http_file(content).unwrap();

        assert!(matches!(request.method, HttpMethod::GET));
        assert_eq!(request.url, "{{ base_url }}/users");
    }

    #[test]
    fn test_parse_connect_method() {
        let content = "CONNECT example.com:443";
        let request = parse_http_file(content).unwrap();
        assert!(matches!(request.method, HttpMethod::CONNECT));
        assert_eq!(request.url, "example.com:443");
    }

    #[test]
    fn test_parse_trace_method() {
        let content = "TRACE https://api.example.com/users";
        let request = parse_http_file(content).unwrap();
        assert!(matches!(request.method, HttpMethod::TRACE));
        assert_eq!(request.url, "https://api.example.com/users");
    }
}
