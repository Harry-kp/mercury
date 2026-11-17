//! cURL import (paste into the URL bar) and export (Copy as cURL).

use crate::kv::{basic_auth, url_encode};
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
    let mut chars = cmd.chars().peekable();
    while let Some(ch) = chars.next() {
        match (quote, ch) {
            // bash's $'…': like '…' but with C escapes
            (None, '$') if chars.peek() == Some(&'\'') => {
                chars.next();
                quote = Some('$');
            }
            (Some('$'), '\'') => quote = None,
            (Some('$'), '\\') => match chars.next() {
                Some('n') => current.push('\n'),
                Some('t') => current.push('\t'),
                Some('r') => current.push('\r'),
                Some(c) => current.push(c), // \\ \' \" and the rest: literal
                None => {}
            },
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
    let mut data: Vec<String> = Vec::new();
    let mut form: Vec<String> = Vec::new();
    let mut get = false;
    // a stack, so a word can be split and its rest read next
    let mut args: Vec<String> = tokens.into_iter().rev().collect();
    while let Some(mut token) = args.pop() {
        // `-XPOST`, `-sSL`, `-sX POST`: short flags share one dash, and the
        // first that takes a value takes the rest of the word
        let short = token.strip_prefix('-').filter(|r| !r.starts_with('-'));
        if let Some(c) = short
            .filter(|r| r.chars().count() > 1)
            .and_then(|r| r.chars().next())
        {
            let rest = token[1 + c.len_utf8()..].to_string();
            args.push(if "XHdubAFeoxcmw".contains(c) {
                rest
            } else {
                format!("-{rest}")
            });
            token = format!("-{c}");
        }
        match token.as_str() {
            "-X" | "--request" => {
                if let Some(m) = args.pop() {
                    req.method = HttpMethod::parse(&m).unwrap_or_default();
                }
            }
            "-H" | "--header" => {
                if let Some((k, v)) = args.pop().as_deref().and_then(|h| h.split_once(':')) {
                    req.headers
                        .push((k.trim().to_string(), v.trim().to_string()));
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii" | "--json" => {
                if let Some(body) = args.pop() {
                    data.push(body);
                    if token == "--json" {
                        req.headers
                            .push(("Content-Type".into(), "application/json".into()));
                    }
                }
            }
            "--data-urlencode" => {
                if let Some(arg) = args.pop() {
                    // `name=value` encodes only the value; `=value` and
                    // `value` encode the whole thing
                    data.push(match arg.split_once('=') {
                        Some(("", v)) => url_encode(v),
                        Some((k, v)) => format!("{k}={}", url_encode(v)),
                        None => url_encode(&arg),
                    });
                }
            }
            // Mercury's Multipart body is these lines as they are; drop
            // curl's `;type=…` / `;filename=…` modifiers on a file
            "-F" | "--form" | "--form-string" => {
                if let Some(field) = args.pop() {
                    // one -F is one body line, even if `$'…'` hid a newline
                    let field = &field.replace(['\r', '\n'], " ");
                    let literal = token == "--form-string";
                    form.push(match field.split_once("=@") {
                        // --form-string's `@` is text: Mercury writes that `@@`
                        Some((k, v)) if literal => format!("{k}=@@{v}"),
                        Some(_) => field.split(';').next().unwrap_or(field).to_string(),
                        None => field.clone(),
                    });
                }
            }
            "-u" | "--user" => {
                if let Some(creds) = args.pop() {
                    let (user, pass) = creds.split_once(':').unwrap_or((&creds, ""));
                    req.headers
                        .push(("Authorization".into(), basic_auth(user, pass)));
                }
            }
            "-A" | "--user-agent" => {
                if let Some(ua) = args.pop() {
                    req.headers.push(("User-Agent".into(), ua));
                }
            }
            "-b" | "--cookie" => {
                if let Some(c) = args.pop() {
                    req.headers.push(("Cookie".into(), c));
                }
            }
            "-I" | "--head" => req.method = HttpMethod::HEAD,
            "-G" | "--get" => get = true,
            // flags that take an argument we don't use
            "-o" | "--output" | "-x" | "--proxy" | "-c" | "--cookie-jar" | "--connect-timeout"
            | "--max-time" | "-m" | "-w" | "--write-out" | "--cacert" | "--cert" | "--key"
            | "-e" | "--referer" => {
                args.pop();
            }
            arg if !arg.starts_with('-') && req.url.is_empty() => req.url = arg.to_string(),
            _ => {} // boolean flags (-s, -L, -k, ...) and unknown flags
        }
    }

    if req.url.is_empty() {
        return Err("Invalid cURL command: no URL found".into());
    }
    // the Content-Type curl sends when the command doesn't set one
    let default_type = |req: &mut CurlRequest, ct: &str| {
        if !req
            .headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        {
            req.headers.push(("Content-Type".into(), ct.into()));
        }
    };
    if !form.is_empty() {
        req.body = Some(form.join("\n"));
        default_type(&mut req, "multipart/form-data");
        if req.method == HttpMethod::GET {
            req.method = HttpMethod::POST;
        }
    }
    // curl joins repeated data with `&`; -G sends it as the query string
    if !data.is_empty() {
        let data = data.join("&");
        if get {
            let sep = if req.url.contains('?') { '&' } else { '?' };
            req.url = format!("{}{sep}{data}", req.url);
        } else {
            req.body = Some(data);
            default_type(&mut req, "application/x-www-form-urlencoded");
            if req.method == HttpMethod::GET {
                req.method = HttpMethod::POST;
            }
        }
    }
    if get {
        req.method = HttpMethod::GET;
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
    let multipart = crate::http::is_multipart(headers);
    for (k, v) in headers {
        // curl writes its own, with the boundary
        if !(multipart && k.eq_ignore_ascii_case("content-type")) {
            curl.push_str(&format!(" \\\n  -H {}", shell_quote(&format!("{k}: {v}"))));
        }
    }
    if multipart {
        for row in crate::kv::parse_lines(body, "=") {
            if row.enabled && !row.key.is_empty() {
                // -F only for real files: curl reads `@path` and `<path` in a
                // -F value from disk, so every text field is --form-string
                // (`@@x` is the text `@x`)
                let (flag, value) = match row.value.strip_prefix('@') {
                    Some(rest) if rest.starts_with('@') => ("--form-string", rest.to_string()),
                    Some(_) => ("-F", row.value),
                    None => ("--form-string", row.value),
                };
                let field = format!("{}={value}", row.key);
                curl.push_str(&format!(" \\\n  {flag} {}", shell_quote(&field)));
            }
        }
    } else if !body.is_empty() {
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

    /// Pastes from browsers and docs that used to come out wrong.
    #[test]
    fn real_world_spellings() {
        // curl joins repeated data with &
        let r = parse("curl https://a.com -d a=1 -d b=2").unwrap();
        assert_eq!(r.body.as_deref(), Some("a=1&b=2"));
        // --data-urlencode encodes the value, not the name
        let r = parse("curl https://a.com --data-urlencode 'q=a b&c' --data-urlencode x").unwrap();
        assert_eq!(r.body.as_deref(), Some("q=a%20b%26c&x"));
        // a value glued to its flag
        let r = parse("curl -XPUT -HAccept:text/plain https://a.com").unwrap();
        assert_eq!(r.method, HttpMethod::PUT);
        assert_eq!(r.headers, vec![("Accept".into(), "text/plain".into())]);
        // -G sends the data as the query string, whatever order
        let r = parse("curl -G https://a.com/s?x=1 -d q=rust -d n=2").unwrap();
        assert_eq!(
            (r.method, r.url.as_str()),
            (HttpMethod::GET, "https://a.com/s?x=1&q=rust&n=2")
        );
        assert_eq!(r.body, None);
        // bash's $'…' quoting (Chrome uses it when the body has a quote or newline)
        let r = parse(r#"curl https://a.com --data-raw $'{\"a\":\"it\'s\n\"}'"#).unwrap();
        assert_eq!(r.body.as_deref(), Some("{\"a\":\"it's\n\"}"));
        assert!(
            parse("curl -é https://a.com").is_ok(),
            "no panic on odd flags"
        );
        // -F is a Multipart body: one line per field, curl's ;type= dropped
        let r =
            parse("curl -F name=Ada -F 'avatar=@/tmp/a.png;type=image/png' https://a.com").unwrap();
        assert_eq!(r.method, HttpMethod::POST);
        assert_eq!(r.body.as_deref(), Some("name=Ada\navatar=@/tmp/a.png"));
        assert!(r
            .headers
            .contains(&("Content-Type".into(), "multipart/form-data".into())));
        // and back: Copy as cURL writes -F, and lets curl set the boundary
        let headers = BTreeMap::from([("Content-Type".into(), "multipart/form-data".into())]);
        let cmd = generate(
            HttpMethod::POST,
            "https://a.com",
            &headers,
            "name=Ada\n# off=1",
        );
        assert!(
            cmd.contains("--form-string 'name=Ada'") && !cmd.contains("off") && !cmd.contains("-H")
        );
        // -F only for a real file: curl would read `<path` in a -F text value
        let cmd = generate(
            HttpMethod::POST,
            "https://a.com",
            &headers,
            "a=<x\nf=@/tmp/f",
        );
        assert!(
            cmd.contains("--form-string 'a=<x'") && cmd.contains("-F 'f=@/tmp/f'"),
            "{cmd}"
        );
        // --form-string's `@` is text both ways (Mercury writes it `@@`)
        let r = parse("curl --form-string 'a=@x' https://a.com").unwrap();
        assert_eq!(r.body.as_deref(), Some("a=@@x"));
        let cmd = generate(HttpMethod::POST, "https://a.com", &headers, "a=@@x");
        assert!(cmd.contains("--form-string 'a=@x'"), "{cmd}");
        // a newline hidden in one -F can't add a second (file) field
        let r = parse(r"curl -F $'a=1\nb=@/etc/passwd' https://a.com").unwrap();
        assert_eq!(r.body.as_deref(), Some("a=1 b=@/etc/passwd"));
    }

    #[test]
    fn short_flags_share_a_dash() {
        let r = parse("curl -sX POST https://a.com -sSLd x=1").unwrap();
        assert_eq!(
            (r.method, r.url.as_str()),
            (HttpMethod::POST, "https://a.com")
        );
        assert_eq!(r.body.as_deref(), Some("x=1"));
        assert_eq!(
            parse("curl -sI https://a.com").unwrap().method,
            HttpMethod::HEAD
        );
        // a value is never split, even one that looks like flags
        let r = parse("curl -H 'X: 1' -d -sX https://a.com").unwrap();
        assert_eq!(r.body.as_deref(), Some("-sX"));
    }

    #[test]
    fn data_without_a_type_is_sent_as_a_form_like_curl_does() {
        let r = parse("curl https://a.com -d a=1").unwrap();
        assert_eq!(
            r.headers,
            vec![(
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into()
            )]
        );
        let r = parse("curl https://a.com -H 'content-type: text/csv' -d a").unwrap();
        assert_eq!(r.headers.len(), 1);
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
        assert_eq!(back.headers[0], ("A".into(), "it's".into()));
        assert_eq!(back.body.as_deref(), Some("{\"a\":'b'}"));
    }
}
