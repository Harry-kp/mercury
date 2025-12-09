// curl_parser.rs - Parse cURL commands into HTTP requests
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
                    method = match tokens[i + 1].to_uppercase().as_str() {
                        "GET" => HttpMethod::GET,
                        "POST" => HttpMethod::POST,
                        "PUT" => HttpMethod::PUT,
                        "PATCH" => HttpMethod::PATCH,
                        "DELETE" => HttpMethod::DELETE,
                        _ => HttpMethod::GET,
                    };
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
            "-d" | "--data" | "--data-raw" | "--data-binary" => {
                if i + 1 < tokens.len() {
                    body = Some(tokens[i + 1].clone());
                    if method == HttpMethod::GET {
                        method = HttpMethod::POST;
                    }
                    i += 1;
                }
            }
            "--compressed" | "-s" | "--silent" | "-L" | "--location" | "-k" | "--insecure" => {
                // Ignore these flags
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
}
