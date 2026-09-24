//! cURL import (paste into the URL bar) and export (Copy as cURL).

use crate::kv::basic_auth;
use crate::model::HttpMethod;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct CurlRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// Split a command line into arguments like a POSIX shell: single quotes are
/// literal, backslash escapes outside quotes and inside double quotes, and
/// `\`-newline is a line continuation.
fn tokenize(cmd: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = cmd.chars();
    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (None, '\'' | '"') => quote = Some(ch),
            (Some(q), c) if c == q => quote = None,
            (None | Some('"'), '\\') => match chars.next() {
                Some('\n') | None => {}
                Some(c) => current.push(c),
            },
            (None, ' ' | '\t' | '\n' | '\r') => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub fn parse(cmd: &str) -> Result<CurlRequest, String> {
    let cmd = cmd.trim();
    let tokens = tokenize(cmd.strip_prefix("curl").unwrap_or(cmd));

    let mut req = CurlRequest {
        method: HttpMethod::GET,
        url: String::new(),
        headers: Vec::new(),
        body: None,
    };
    let mut args = tokens.iter();
    while let Some(token) = args.next() {
        match token.as_str() {
            "-X" | "--request" => {
                if let Some(m) = args.next() {
                    req.method = HttpMethod::parse(m).unwrap_or_default();
                }
            }
            "-H" | "--header" => {
                if let Some((k, v)) = args.next().and_then(|h| h.split_once(':')) {
                    req.headers
                        .push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--json" => {
                if let Some(body) = args.next() {
                    req.body = Some(body.clone());
                    if req.method == HttpMethod::GET {
                        req.method = HttpMethod::POST;
                    }
                    if token == "--json" {
                        req.headers
                            .push(("Content-Type".into(), "application/json".into()));
                    }
                }
            }
            "-u" | "--user" => {
                if let Some(creds) = args.next() {
                    let (user, pass) = creds.split_once(':').unwrap_or((creds, ""));
                    req.headers
                        .push(("Authorization".into(), basic_auth(user, pass)));
                }
            }
            "-A" | "--user-agent" => {
                if let Some(ua) = args.next() {
                    req.headers.push(("User-Agent".into(), ua.clone()));
                }
            }
            "-b" | "--cookie" => {
                if let Some(c) = args.next() {
                    req.headers.push(("Cookie".into(), c.clone()));
                }
            }
            "-I" | "--head" => req.method = HttpMethod::HEAD,
            "-G" | "--get" => req.method = HttpMethod::GET,
            // flags that take an argument we don't use
            "-o" | "--output" | "-x" | "--proxy" | "-c" | "--cookie-jar" | "--connect-timeout"
            | "--max-time" | "-m" | "-w" | "--write-out" | "--cacert" | "--cert" | "--key"
            | "-e" | "--referer" => {
                args.next();
            }
            arg if !arg.starts_with('-') && req.url.is_empty() => req.url = arg.to_string(),
            _ => {} // boolean flags (-s, -L, -k, ...) and unknown flags
        }
    }

    if req.url.is_empty() {
        return Err("Invalid cURL command: no URL found".into());
    }
    Ok(req)
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Build a copy-pasteable cURL command (variables already substituted).
pub fn generate(
    method: HttpMethod,
    url: &str,
    headers: &BTreeMap<String, String>,
    body: &str,
) -> String {
    let mut curl = format!("curl -X {} {}", method.as_str(), shell_quote(url));
    for (k, v) in headers {
        curl.push_str(&format!(" \\\n  -H {}", shell_quote(&format!("{k}: {v}"))));
    }
    if !body.is_empty() {
        curl.push_str(&format!(" \\\n  -d {}", shell_quote(body)));
    }
    curl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_get() {
        let r = parse("curl https://api.example.com/users").unwrap();
        assert_eq!(r.method, HttpMethod::GET);
        assert_eq!(r.url, "https://api.example.com/users");
    }

    #[test]
    fn post_with_header_and_data() {
        let r = parse(r#"curl -X POST https://a.com -H "Content-Type: application/json" -d '{"name":"test"}'"#).unwrap();
        assert_eq!(r.method, HttpMethod::POST);
        assert_eq!(
            r.headers,
            vec![("Content-Type".into(), "application/json".into())]
        );
        assert_eq!(r.body.as_deref(), Some(r#"{"name":"test"}"#));
    }

    #[test]
    fn data_implies_post_and_json_adds_content_type() {
        let r = parse(r#"curl --json '{"k":"v"}' https://a.com"#).unwrap();
        assert_eq!(r.method, HttpMethod::POST);
        assert!(r.headers.iter().any(|(k, _)| k == "Content-Type"));
    }

    #[test]
    fn user_agent_cookie_basic_auth_head() {
        let r = parse(&format!(
            r#"curl -A "Mozilla/5.0" -b "s=1" -u {} -I https://a.com"#,
            "u:p"
        ))
        .unwrap();
        assert_eq!(r.method, HttpMethod::HEAD);
        assert_eq!(r.headers[0], ("User-Agent".into(), "Mozilla/5.0".into()));
        assert_eq!(r.headers[1], ("Cookie".into(), "s=1".into()));
        assert_eq!(r.headers[2].1, basic_auth("u", "p"));
    }

    #[test]
    fn ignores_flags_and_their_arguments() {
        let r = parse("curl -v -s -x http://proxy:8080 -L https://httpbin.org/get").unwrap();
        assert_eq!(r.url, "https://httpbin.org/get");
    }

    #[test]
    fn multiline_with_continuations() {
        let r = parse("curl -X PUT \\\n  'https://a.com/1' \\\n  -H 'A: b'").unwrap();
        assert_eq!(r.method, HttpMethod::PUT);
        assert_eq!(r.url, "https://a.com/1");
        assert_eq!(r.headers.len(), 1);
    }

    #[test]
    fn single_quotes_are_literal() {
        // Chrome's "Copy as cURL" output: JSON with escaped quotes stays valid
        let r = parse(r#"curl 'https://a.com' --data-raw '{"a":"b\"c"}'"#).unwrap();
        assert_eq!(r.body.as_deref(), Some(r#"{"a":"b\"c"}"#));
    }

    #[test]
    fn missing_url_is_error() {
        assert!(parse("curl -X POST").is_err());
    }

    #[test]
    fn generate_quotes_and_roundtrips() {
        let mut h = BTreeMap::new();
        h.insert("A".to_string(), "it's".to_string());
        let cmd = generate(
            HttpMethod::POST,
            "https://a.com/?q=1&x=2",
            &h,
            "{\"a\":'b'}",
        );
        let back = parse(&cmd).unwrap();
        assert_eq!(back.method, HttpMethod::POST);
        assert_eq!(back.url, "https://a.com/?q=1&x=2");
        assert_eq!(back.headers, vec![("A".into(), "it's".into())]);
        assert_eq!(back.body.as_deref(), Some("{\"a\":'b'}"));
    }
}
