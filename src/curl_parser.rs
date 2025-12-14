//! cURL Parser Module
//!
//! Parses cURL command strings into structured request objects.
//! Supports common flags like -X, -H, -d, -u, -A, -b, -I, -G, --json.

use crate::http_parser::HttpMethod;

#[derive(Debug)]
pub struct CurlRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// Parse a cURL command into a structured request
pub fn parse_curl(curl_cmd: &str) -> Result<CurlRequest, String> {
    let curl_cmd = curl_cmd.trim();

    // Remove leading 'curl' command
    let curl_cmd = curl_cmd.strip_prefix("curl").unwrap_or(curl_cmd).trim();

    let mut url = String::new();
    let mut method = HttpMethod::GET;
    let mut headers = Vec::new();
    let mut body = None;

    // Simple tokenizer for shell arguments
    let mut chars = curl_cmd.chars().peekable();
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    while let Some(ch) = chars.next() {
        match ch {
            '\'' | '"' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ' ' | '\t' | '\n' if !in_quotes => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            '\\' if in_quotes => {
                if let Some(next) = chars.next() {
                    current_token.push(next);
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    // Parse tokens
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        match token.as_str() {
            "-X" | "--request" => {
                if i + 1 < tokens.len() {
                    method = HttpMethod::from_str(&tokens[i + 1]).unwrap_or(HttpMethod::GET);
                    i += 1;
                }
            }
            "-H" | "--header" => {
                if i + 1 < tokens.len() {
                    let header = &tokens[i + 1];
                    if let Some(pos) = header.find(':') {
                        let name = header[..pos].trim().to_string();
                        let value = header[pos + 1..].trim().to_string();
                        headers.push((name, value));
                    }
                    i += 1;
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--json" => {
                if i + 1 < tokens.len() {
                    body = Some(tokens[i + 1].clone());
                    if method == HttpMethod::GET {
                        method = HttpMethod::POST;
                    }
                    // --json also adds Content-Type header
                    if token == "--json" {
                        headers.push(("Content-Type".to_string(), "application/json".to_string()));
                    }
                    i += 1;
                }
            }
            "-u" | "--user" => {
                // Basic auth: -u user:password
                if i + 1 < tokens.len() {
                    let credentials = &tokens[i + 1];
                    use base64::Engine;
                    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
                    headers.push(("Authorization".to_string(), format!("Basic {}", encoded)));
                    i += 1;
                }
            }
            "-A" | "--user-agent" => {
                if i + 1 < tokens.len() {
                    headers.push(("User-Agent".to_string(), tokens[i + 1].clone()));
                    i += 1;
                }
            }
            "-b" | "--cookie" => {
                if i + 1 < tokens.len() {
                    headers.push(("Cookie".to_string(), tokens[i + 1].clone()));
                    i += 1;
                }
            }
            "-I" | "--head" => {
                method = HttpMethod::HEAD;
            }
            "-G" | "--get" => {
                // Force GET even with data
                method = HttpMethod::GET;
            }
            "--compressed" | "-s" | "--silent" | "-L" | "--location" | "-k" | "--insecure"
            | "-v" | "--verbose" | "-p" | "-#" | "--progress-bar" | "-f" | "--fail" | "-S"
            | "--show-error" | "-N" | "--no-buffer" => {
                // Ignore these boolean flags
            }
            "-o" | "--output" | "-x" | "--proxy" | "-c" | "--cookie-jar" | "-j"
            | "--connect-timeout" | "--max-time" | "-m" | "-w" | "--write-out" | "--cacert"
            | "--cert" | "--key" | "-e" | "--referer" => {
                // Ignore these flags that take one argument
                i += 1; // Skip the argument
            }
            arg if !arg.starts_with('-') => {
                // Assume it's the URL
                if url.is_empty() {
                    url = arg.to_string();
                }
            }
            _ => {
                // Unknown flag, skip
            }
        }

        i += 1;
    }

    if url.is_empty() {
        return Err("No URL found in cURL command".to_string());
    }

    Ok(CurlRequest {
        method,
        url,
        headers,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_get() {
        let curl = "curl https://api.example.com/users";
        let req = parse_curl(curl).unwrap();
        assert_eq!(req.method, HttpMethod::GET);
        assert_eq!(req.url, "https://api.example.com/users");
    }

    #[test]
    fn test_post_with_data() {
        let curl = r#"curl -X POST https://api.example.com/users -H "Content-Type: application/json" -d '{"name":"test"}'"#;
        let req = parse_curl(curl).unwrap();
        assert_eq!(req.method, HttpMethod::POST);
        assert_eq!(req.url, "https://api.example.com/users");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.body, Some(r#"{"name":"test"}"#.to_string()));
    }

    #[test]
    fn test_basic_auth() {
        let curl = "curl -u admin:secret https://api.example.com/users";
        let req = parse_curl(curl).unwrap();
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].0, "Authorization");
        assert!(req.headers[0].1.starts_with("Basic "));
    }

    #[test]
    fn test_user_agent() {
        let curl = r#"curl -A "Mozilla/5.0" https://example.com"#;
        let req = parse_curl(curl).unwrap();
        assert_eq!(req.headers.len(), 1);
        assert_eq!(
            req.headers[0],
            ("User-Agent".to_string(), "Mozilla/5.0".to_string())
        );
    }

    #[test]
    fn test_cookie() {
        let curl = r#"curl -b "session=abc123" https://example.com"#;
        let req = parse_curl(curl).unwrap();
        assert_eq!(req.headers.len(), 1);
        assert_eq!(
            req.headers[0],
            ("Cookie".to_string(), "session=abc123".to_string())
        );
    }

    #[test]
    fn test_head_request() {
        let curl = "curl -I https://example.com";
        let req = parse_curl(curl).unwrap();
        assert_eq!(req.method, HttpMethod::HEAD);
    }

    #[test]
    fn test_json_flag() {
        let curl = r#"curl --json '{"key":"value"}' https://api.example.com"#;
        let req = parse_curl(curl).unwrap();
        assert_eq!(req.method, HttpMethod::POST);
        assert_eq!(req.body, Some(r#"{"key":"value"}"#.to_string()));
        assert!(req
            .headers
            .iter()
            .any(|(k, v)| k == "Content-Type" && v == "application/json"));
    }

    #[test]
    fn test_proxy_flags() {
        // This should parse successfully, ignoring the proxy flags
        let curl = "curl -v -p -x http://proxy.example.com:8080 https://httpbin.org/get";
        let req = parse_curl(curl).unwrap();
        assert_eq!(req.method, HttpMethod::GET);
        assert_eq!(req.url, "https://httpbin.org/get");
    }
}
