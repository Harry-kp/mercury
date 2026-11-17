//! Sending requests and classifying/formatting responses.

use crate::model::RequestFile;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Only connecting is timed out: a slow endpoint (a report, an LLM) may take
/// minutes to answer, and Esc cancels a request that never will.
pub const CONNECT_TIMEOUT_SECS: u64 = 30;
/// Starts every message `error_message` gives an untrusted certificate.
pub const CERT_ERROR: &str = "SSL/TLS error talking to";
/// Bodies larger than this are not downloaded ("Response Too Large").
pub const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

/// How the response panel renders a body. Serialized by name in history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseType {
    Json,
    Xml,
    Html,
    PlainText,
    Image,
    Binary,
    /// Exceeded MAX_RESPONSE_SIZE; body not downloaded.
    TooLarge,
    /// Written by Mercury < 0.4 for text over 100 KB, which it wouldn't
    /// show. Kept so old history still loads; it is shown as plain text.
    LargeText,
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub cookies: Vec<String>,
    pub body: String,
    /// Raw bytes for Image/Binary so "Save" writes the real file. Not persisted.
    #[serde(skip)]
    pub raw_bytes: Option<Vec<u8>>,
    pub duration_ms: u128,
    pub size_bytes: usize,
    pub content_type: String,
    pub response_type: ResponseType,
    /// The pretty-printed body (JSON/XML), made on the request thread so a
    /// multi-megabyte body doesn't stall the first frame. Not persisted.
    #[serde(skip)]
    pub pretty: Option<String>,
}

/// Whether these headers (as sent: substituted, one value per name) make
/// the body Multipart. The one test the app, the sender and Copy as cURL
/// all use: deciding it from different inputs is how a variable slipped a
/// file upload past the per-field substitution.
pub fn is_multipart(headers: &std::collections::BTreeMap<String, String>) -> bool {
    headers
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("content-type") && v.contains("multipart/form-data"))
}

/// A Multipart body with `{{variables}}` filled in, field by field: the
/// lines are split first and whether a field is a file is decided from what
/// the user wrote, so a variable's value can never add a field or turn text
/// into an upload (an imported collection ships its own `.env`). A text
/// value that comes out starting with `@` is escaped as `@@`.
pub fn substitute_multipart(
    text: &str,
    vars: &std::collections::HashMap<String, String>,
) -> String {
    let sub = |s: &str| crate::vars::substitute(s, vars).replace(['\r', '\n'], " ");
    crate::kv::parse_lines(text, "=")
        .into_iter()
        .filter(|r| r.enabled && !r.key.is_empty())
        .map(|r| {
            let key = sub(&r.key).replace('=', "%3D");
            let value = if let Some(rest) = r.value.strip_prefix("@@") {
                format!("@@{}", sub(rest))
            } else if let Some(path) = r.value.strip_prefix('@') {
                // a file path is taken as written: a variable choosing which
                // file to upload is a variable choosing what leaves the machine
                format!("@{}", path.replace(['\r', '\n'], " "))
            } else {
                let text = sub(&r.value);
                if text.starts_with('@') {
                    format!("@{text}")
                } else {
                    text
                }
            };
            format!("{key}={value}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Encode a Multipart body: one `name=value` field per line, `name=@path`
/// for a file read from disk, `name=@@x` for the text `@x`. `#` lines are
/// switched off. Returns the boundary and the bytes.
pub fn multipart_body(text: &str) -> Result<(String, Vec<u8>), String> {
    let boundary = format!(
        "mercury-{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    );
    let quote = |s: &str| s.replace('"', "%22").replace(['\r', '\n'], " ");
    let mut out = Vec::new();
    for row in crate::kv::parse_lines(text, "=") {
        if !row.enabled || row.key.is_empty() {
            continue;
        }
        out.extend(format!("--{boundary}\r\n").as_bytes());
        let name = quote(&row.key);
        let file = row.value.strip_prefix('@').filter(|p| !p.starts_with('@'));
        let text = row.value.strip_prefix('@').unwrap_or(&row.value);
        match file {
            Some(path) => {
                let path = std::path::Path::new(path.trim());
                let bytes = std::fs::read(path)
                    .map_err(|e| format!("Can't attach {}: {e}", path.display()))?;
                let file = quote(&path.file_name().unwrap_or_default().to_string_lossy());
                out.extend(
                    format!(
                        "Content-Disposition: form-data; name=\"{name}\"; filename=\"{file}\"\r\n\
                         Content-Type: application/octet-stream\r\n\r\n"
                    )
                    .as_bytes(),
                );
                out.extend(bytes);
            }
            None => out.extend(
                format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n{text}").as_bytes(),
            ),
        }
        out.extend(b"\r\n");
    }
    out.extend(format!("--{boundary}--\r\n").as_bytes());
    Ok((boundary, out))
}

impl HttpResponse {
    /// Stands in for the response of a send that got none, in history.
    /// Status 0 marks it.
    pub fn failed() -> Self {
        Self {
            status: 0,
            status_text: "Failed".into(),
            headers: Vec::new(),
            cookies: Vec::new(),
            body: String::new(),
            raw_bytes: None,
            duration_ms: 0,
            size_bytes: 0,
            content_type: String::new(),
            response_type: ResponseType::Empty,
            pretty: None,
        }
    }
}

#[cfg(test)]
pub fn client() -> reqwest::blocking::Client {
    client_with(true)
}

/// `check_certs: false` is for hosts the user chose to trust (a dev server's
/// self-signed certificate). That client keeps its own cookies, and follows
/// redirects only on the same host, so the skipped check reaches nothing else.
pub fn client_with(check_certs: bool) -> reqwest::blocking::Client {
    // reqwest is built without a crypto provider (see Cargo.toml); use ring.
    // Err only means one is already installed, which is fine.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let redirects = if check_certs {
        reqwest::redirect::Policy::limited(10)
    } else {
        reqwest::redirect::Policy::custom(|a| {
            let same = a
                .previous()
                .first()
                .map(|u| (u.host_str(), u.port_or_known_default()))
                == Some((a.url().host_str(), a.url().port_or_known_default()));
            if same && a.previous().len() < 10 {
                a.follow()
            } else {
                a.stop()
            }
        })
    };
    reqwest::blocking::Client::builder()
        .cookie_store(true)
        .redirect(redirects)
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        // the blocking client's default is a 30 s total
        .timeout(None::<Duration>)
        .danger_accept_invalid_certs(!check_certs)
        .build()
        .expect("HTTP client builds with static config")
}

/// `host[:port]` of a URL as typed (no scheme needed).
pub fn host(url: &str) -> Option<String> {
    let u = reqwest::Url::parse(&with_scheme(url)).ok()?;
    Some(format!(
        "{}{}",
        u.host_str()?,
        u.port().map(|p| format!(":{p}")).unwrap_or_default()
    ))
}

/// `localhost:3000/users` is what people type into a URL bar. Give a
/// scheme-less URL one: http for loopback, https for anything else.
pub fn with_scheme(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() || url.contains("://") {
        return url.to_string();
    }
    if let Some(rest) = url.strip_prefix("//") {
        return format!("https://{rest}");
    }
    let host = url.split(['/', ':', '?', '#']).next().unwrap_or(url);
    let loopback = matches!(host, "localhost" | "127.0.0.1" | "0.0.0.0" | "[::1]")
        || host.ends_with(".localhost");
    let scheme = if loopback { "http" } else { "https" };
    format!("{scheme}://{url}")
}

/// Blocking; call from a background thread.
pub fn execute(
    client: &reqwest::blocking::Client,
    request: &RequestFile,
) -> Result<HttpResponse, String> {
    let start = Instant::now();
    let method = reqwest::Method::from_bytes(request.method.as_str().as_bytes())
        .expect("HttpMethod names are valid HTTP methods");
    let mut builder = client.request(method, with_scheme(&request.url));
    let multipart = is_multipart(&request.headers);
    for (key, value) in &request.headers {
        // the boundary has to be in it, and only the encoder knows it
        if !(multipart && key.eq_ignore_ascii_case("content-type")) {
            builder = builder.header(key, value);
        }
    }
    if multipart {
        let (boundary, body) = multipart_body(&request.body)?;
        builder = builder
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body);
    } else if !request.body.is_empty() {
        builder = builder.body(request.body.clone());
    }

    let response = builder.send().map_err(|e| error_message(&e))?;
    let status = response.status();
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
        .collect();
    let cookies = extract_cookies(&headers);
    let content_type = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();

    // check Content-Length before downloading
    let declared = response.content_length().unwrap_or(0) as usize;
    let (bytes, response_type, size_bytes) = if declared > MAX_RESPONSE_SIZE {
        (Vec::new(), ResponseType::TooLarge, declared)
    } else {
        // a body without Content-Length (chunked, or a stream that never
        // ends) is read only one byte past the limit, not into memory whole
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(
            &mut std::io::Read::take(response, MAX_RESPONSE_SIZE as u64 + 1),
            &mut bytes,
        )
        .map_err(|e| format!("Failed to read response body: {e}"))?;
        let kind = detect_response_type(&content_type, &bytes, status.as_u16());
        let len = bytes.len();
        (bytes, kind, len)
    };

    let (body, raw_bytes) = match response_type {
        ResponseType::Image | ResponseType::Binary => {
            (format!("[Binary data: {size_bytes} bytes]"), Some(bytes))
        }
        ResponseType::TooLarge => (String::new(), None),
        _ => (text_of(&bytes, &content_type), None),
    };

    // pretty-print here, on the request thread, not in the first frame
    let pretty = match response_type {
        ResponseType::Json => Some(format_json(&body)),
        ResponseType::Xml => Some(format_xml(&body)),
        _ => None,
    };
    Ok(HttpResponse {
        status: status.as_u16(),
        status_text: status.to_string(),
        headers,
        cookies,
        body,
        raw_bytes,
        duration_ms: start.elapsed().as_millis(),
        size_bytes,
        content_type,
        response_type,
        pretty,
    })
}

/// What happened and what to try, then the full cause. reqwest's own text is
/// only the outer layer ("error sending request for url (…)"); the reason
/// (refused, DNS, certificate) is at the bottom of the `source()` chain.
fn error_message(e: &reqwest::Error) -> String {
    let mut text = e.to_string();
    let mut source = std::error::Error::source(e);
    while let Some(cause) = source {
        text = format!("{text}: {cause}");
        source = cause.source();
    }
    let lower = text.to_lowercase();
    let host = e
        .url()
        .and_then(|u| host(u.as_str()))
        .unwrap_or_else(|| "the server".into());
    let summary = if e.is_timeout() {
        format!("Could not connect to {host} within {CONNECT_TIMEOUT_SECS}s.")
    } else if e.is_connect()
        && ["certificate", "ssl", "tls", "handshake"]
            .iter()
            .any(|w| lower.contains(w))
    {
        format!("{CERT_ERROR} {host}. Its certificate may not be trusted.")
    } else if e.is_connect() && lower.contains("refused") {
        format!("Nothing is listening on {host}. Is the server running?")
    } else if lower.contains("dns") || lower.contains("lookup") || lower.contains("nodename") {
        format!("Can't find the host {host}. Check the spelling or your network.")
    } else if e.is_connect() {
        format!("Could not connect to {host}.")
    } else if e.is_builder() {
        "Invalid URL.".to_string()
    } else {
        "Request failed.".to_string()
    };
    format!("{summary}\n\n{text}")
}

fn extract_cookies(headers: &[(String, String)]) -> Vec<String> {
    headers
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, v)| v.clone())
        .collect()
}

/// A text body as a string. UTF-8, unless the server declares Latin-1,
/// whose bytes are exactly the first 256 code points.
fn text_of(bytes: &[u8], content_type: &str) -> String {
    let ct = content_type.to_lowercase().replace('"', "");
    if ["charset=iso-8859-1", "charset=latin1"]
        .iter()
        .any(|c| ct.contains(c))
    {
        bytes.iter().map(|&b| b as char).collect()
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

fn detect_response_type(content_type: &str, body: &[u8], status: u16) -> ResponseType {
    if status == 204 || body.is_empty() {
        return ResponseType::Empty;
    }
    if body.len() > MAX_RESPONSE_SIZE {
        return ResponseType::TooLarge;
    }
    let ct = content_type.to_lowercase();
    // text first: the binary words below are substrings of text types too
    // (GitHub's `application/vnd.github.star+json` has "tar" in it)
    if ct.contains("application/json") || ct.contains("+json") {
        return ResponseType::Json;
    }
    // SVG (`image/svg+xml`) is text too
    if ct.contains("application/xml") || ct.contains("text/xml") || ct.contains("+xml") {
        return ResponseType::Xml;
    }
    if ct.contains("text/html") {
        return ResponseType::Html;
    }
    if ct.starts_with("text/") {
        return ResponseType::PlainText;
    }
    if ct.starts_with("image/") {
        return ResponseType::Image;
    }
    if ct.starts_with("application/octet-stream")
        || ct.starts_with("application/pdf")
        || ct.starts_with("audio/")
        || ct.starts_with("video/")
        || ["zip", "tar", "gzip"].iter().any(|w| ct.contains(w))
    {
        return ResponseType::Binary;
    }
    // no useful content-type: sniff
    let Ok(text) = std::str::from_utf8(body) else {
        return ResponseType::Binary;
    };
    let text = text.trim_start();
    if text.starts_with('{') || text.starts_with('[') {
        ResponseType::Json
    } else if text.starts_with('<') {
        let head = text
            .char_indices()
            .nth(512)
            .map_or(text, |(i, _)| &text[..i]);
        let head = head.to_lowercase();
        if head.contains("<!doctype html") || head.contains("<html") {
            ResponseType::Html
        } else {
            ResponseType::Xml
        }
    } else {
        ResponseType::PlainText
    }
}

/// Pretty-print JSON; returns the input unchanged if it isn't valid JSON.
/// Indent valid JSON exactly as written: only whitespace between tokens
/// changes. Parsing into a `serde_json::Value` and printing it back sorted
/// keys, rewrote numbers and dropped duplicate keys, so the "Pretty" view
/// (and Format body) didn't show what was actually sent.
/// `"token": "abc",`, one line of formatted JSON, as `("token", "abc")`.
/// `None` for a line that opens an object or array, or has no key.
pub fn json_field(line: &str) -> Option<(String, String)> {
    let line = line.trim().trim_end_matches(',');
    let map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&format!("{{{line}}}")).ok()?;
    let (key, value) = map.into_iter().next()?;
    let value = match value {
        serde_json::Value::String(s) => s,
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => return None,
        other => other.to_string(),
    };
    Some((key, value))
}

pub fn format_json(body: &str) -> String {
    if serde_json::from_str::<serde::de::IgnoredAny>(body).is_err() {
        return body.to_string();
    }
    let mut out = String::with_capacity(body.len() * 2);
    let mut depth = 0usize;
    let (mut in_string, mut escaped) = (false, false);
    let mut chars = body.trim().chars().peekable();
    let newline = |out: &mut String, depth: usize| {
        out.push('\n');
        out.push_str(&"  ".repeat(depth));
    };
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            match (escaped, c) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            '{' | '[' => {
                while chars.peek().is_some_and(|c| c.is_whitespace()) {
                    chars.next();
                }
                out.push(c);
                // an empty container stays on one line: `{}` / `[]`
                if chars.peek() == Some(&if c == '{' { '}' } else { ']' }) {
                    out.extend(chars.next());
                } else {
                    depth += 1;
                    newline(&mut out, depth);
                }
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                newline(&mut out, depth);
                out.push(c);
            }
            ',' => {
                out.push(',');
                newline(&mut out, depth);
            }
            ':' => out.push_str(": "),
            c if c.is_whitespace() => {}
            c => out.push(c),
        }
    }
    out
}

/// Indent XML one level per open tag. Naive by design: good enough to read.
pub fn format_xml(body: &str) -> String {
    let mut out = String::new();
    let mut indent = 0usize;
    let mut token = String::new();
    // the last tag opened an element: its closing tag stays on its line
    // (`<name>Ada</name>`, not `</name>` alone on the next)
    let mut after_open = false;
    for ch in body.chars() {
        match ch {
            '<' => {
                out.push_str(token.trim());
                token.clear();
                token.push(ch);
            }
            '>' => {
                token.push(ch);
                let tag = token.trim();
                let closing = tag.starts_with("</");
                let opens = !closing
                    && !tag.ends_with("/>")
                    && !tag.starts_with("<?")
                    && !tag.starts_with("<!");
                if closing {
                    indent = indent.saturating_sub(1);
                }
                if !(closing && after_open) {
                    if !out.is_empty() && !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str(&"  ".repeat(indent));
                }
                out.push_str(tag);
                if opens {
                    indent += 1;
                }
                after_open = opens;
                token.clear();
            }
            _ => token.push(ch),
        }
    }
    out.push_str(token.trim());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An imported collection ships its own `.env`. A variable's value
    /// must not add a field, rename one into a file, or make text an upload.
    #[test]
    fn variables_cannot_turn_a_field_into_an_upload() {
        let vars = std::collections::HashMap::from([
            ("V".to_string(), "@/etc/passwd".to_string()),
            ("N".to_string(), "x\nb=@/etc/hosts".to_string()),
            ("K".to_string(), "k=@/etc/group".to_string()),
            ("P".to_string(), "/tmp/avatar.png".to_string()),
        ]);
        // ({{P}} stays as written: a variable never picks the file to upload)
        let body = "a={{V}}\nc={{N}}\n{{K}}=1\nf=@{{P}}\nt=@@{{V}}\n# off=@{{V}}";
        assert_eq!(
            substitute_multipart(body, &vars),
            "a=@@/etc/passwd\nc=x b=@/etc/hosts\nk%3D@/etc/group=1\nf=@{{P}}\nt=@@@/etc/passwd"
        );
        // and the encoder sends `@@x` as the text `@x`
        let (_, bytes) = multipart_body("a=@@/etc/passwd").unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("name=\"a\"\r\n\r\n@/etc/passwd\r\n") && !text.contains("filename"));
    }

    #[test]
    fn multipart_encodes_fields_and_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, "hello").unwrap();
        let body = format!("note=hi\n# off=1\nup=@{}", file.display());
        let (b, bytes) = multipart_body(&body).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert_eq!(
            text,
            format!(
                "--{b}\r\nContent-Disposition: form-data; name=\"note\"\r\n\r\nhi\r\n\
                 --{b}\r\nContent-Disposition: form-data; name=\"up\"; filename=\"a.txt\"\r\n\
                 Content-Type: application/octet-stream\r\n\r\nhello\r\n--{b}--\r\n"
            )
        );
        let missing = multipart_body("up=@/no/such/file").unwrap_err();
        assert!(
            missing.starts_with("Can't attach /no/such/file"),
            "{missing}"
        );
    }
    use ResponseType::*;

    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Serves one gzip-encoded JSON response. Hand-rolled so the test needs
    /// no extra dependency: a gzip header, one stored deflate block, CRC32.
    fn gzip_server(payload: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let _ = stream.read(&mut [0; 4096]);
                let raw = payload.as_bytes();
                let mut body = vec![0x1f, 0x8b, 0x08, 0, 0, 0, 0, 0, 0, 0xff];
                body.push(0x01);
                body.extend((raw.len() as u16).to_le_bytes());
                body.extend((!(raw.len() as u16)).to_le_bytes());
                body.extend_from_slice(raw);
                let mut crc = 0xffff_ffff_u32;
                for b in raw {
                    crc ^= *b as u32;
                    for _ in 0..8 {
                        crc = if crc & 1 == 1 {
                            (crc >> 1) ^ 0xEDB8_8320
                        } else {
                            crc >> 1
                        };
                    }
                }
                body.extend((!crc).to_le_bytes());
                body.extend((raw.len() as u32).to_le_bytes());
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(&body);
            }
        });
        format!("http://{addr}")
    }

    fn get(url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, String> {
        let mut map = std::collections::BTreeMap::new();
        for (k, v) in headers {
            map.insert((*k).to_string(), (*v).to_string());
        }
        execute(
            &client(),
            &RequestFile {
                method: crate::model::HttpMethod::GET,
                url: url.to_string(),
                headers: map,
                ..Default::default()
            },
        )
    }

    /// Any server with compression on — and any cURL pasted from a browser,
    /// which always asks for gzip — used to render as mojibake.
    #[test]
    fn compressed_responses_are_decoded() {
        let base = gzip_server(r#"{"ok":true}"#);
        let plain = get(&base, &[]).expect("request");
        assert_eq!(plain.body, r#"{"ok":true}"#);
        assert_eq!(plain.response_type, Json);

        let asked = get(&base, &[("Accept-Encoding", "gzip, deflate, br")]).expect("request");
        assert_eq!(
            asked.body, r#"{"ok":true}"#,
            "an explicit Accept-Encoding must still decode"
        );
    }

    /// "error sending request for url (…)" says nothing about why. The
    /// cause is deeper in the chain, and the user needs a next step.
    #[test]
    fn refused_connection_says_why_and_what_to_try() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port(); // listener dropped: nothing listens there now
        let err = get(&format!("http://127.0.0.1:{port}/x"), &[]).unwrap_err();
        assert!(
            err.starts_with(&format!(
                "Nothing is listening on 127.0.0.1:{port}. Is the server running?"
            )),
            "{err}"
        );
        assert!(err.to_lowercase().contains("refused"), "{err}");
    }

    /// A real TLS handshake against the OS trust store. Needs the network,
    /// so it is opt-in: `cargo test -- --ignored` after touching TLS.
    #[test]
    #[ignore]
    fn https_handshake_with_system_roots() {
        let ok = get("https://example.com", &[]).expect("https request");
        assert_eq!(ok.status, 200);
        let bad = get("https://self-signed.badssl.com", &[]).unwrap_err();
        assert!(bad.starts_with(CERT_ERROR), "{bad}");
        // the host the user chose to trust
        let trusted = execute(
            &client_with(false),
            &RequestFile {
                url: "https://self-signed.badssl.com".into(),
                ..Default::default()
            },
        )
        .expect("trusted host skips the certificate check");
        assert_eq!(trusted.status, 200);
    }

    /// The skipped certificate check must not follow a redirect to another host.
    #[test]
    fn trusted_host_client_stays_on_that_host() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let _ = stream.read(&mut [0; 4096]);
                let to = format!("http://localhost:{port}/elsewhere");
                let _ = stream.write_all(
                    format!("HTTP/1.1 302 Found\r\nLocation: {to}\r\nContent-Length: 0\r\n\r\n")
                        .as_bytes(),
                );
            }
        });
        let request = RequestFile {
            url: format!("http://127.0.0.1:{port}/"),
            ..Default::default()
        };
        let stopped = execute(&client_with(false), &request).unwrap();
        assert_eq!(stopped.status, 302);
    }

    #[test]
    fn host_of_a_typed_url() {
        assert_eq!(host("localhost:8443/x").as_deref(), Some("localhost:8443"));
        assert_eq!(host("https://api.dev/v1").as_deref(), Some("api.dev"));
    }

    #[test]
    fn a_formatted_json_line_gives_its_field() {
        let f = |l| json_field(l);
        assert_eq!(
            f(r#"    "token": "a\"b","#),
            Some(("token".into(), "a\"b".into()))
        );
        assert_eq!(f(r#"  "id": 42"#), Some(("id".into(), "42".into())));
        assert_eq!(f(r#"  "ok": true,"#), Some(("ok".into(), "true".into())));
        assert_eq!(f(r#"  "user": {"#), None);
        assert_eq!(f(r#"  "tags": [],"#), None);
        assert_eq!(f(r#"  "plain","#), None);
        assert_eq!(f("}"), None);
    }

    #[test]
    fn scheme_less_urls_get_one() {
        assert_eq!(
            with_scheme("localhost:3000/api"),
            "http://localhost:3000/api"
        );
        assert_eq!(with_scheme("127.0.0.1:8080"), "http://127.0.0.1:8080");
        assert_eq!(
            with_scheme("api.example.com/v1"),
            "https://api.example.com/v1"
        );
        assert_eq!(
            with_scheme("//cdn.example.com/x"),
            "https://cdn.example.com/x"
        );
        // already fine, and left exactly as typed
        assert_eq!(with_scheme("http://a.test/x"), "http://a.test/x");
        assert_eq!(with_scheme("https://a.test/x"), "https://a.test/x");
        assert_eq!(with_scheme(""), "");
        // an unresolved variable is still invalid, just as before
        assert_eq!(with_scheme("{{BASE}}/users"), "https://{{BASE}}/users");
    }

    #[test]
    fn detects_by_content_type() {
        assert_eq!(detect_response_type("application/json", b"{}", 200), Json);
        assert_eq!(
            detect_response_type("application/vnd+json", b"{}", 200),
            Json
        );
        assert_eq!(detect_response_type("application/xml", b"<a/>", 200), Xml);
        assert_eq!(detect_response_type("text/html", b"<html>", 200), Html);
        assert_eq!(detect_response_type("text/plain", b"hi", 200), PlainText);
        assert_eq!(detect_response_type("image/png", b"\x89PNG", 200), Image);
        assert_eq!(detect_response_type("image/svg+xml", b"<svg/>", 200), Xml);
        assert_eq!(
            detect_response_type("application/pdf", b"%PDF", 200),
            Binary
        );
        assert_eq!(
            detect_response_type("application/octet-stream", b"\0", 200),
            Binary
        );
    }

    /// The words that mark an archive are inside text types too: GitHub's
    /// `application/vnd.github.star+json` showed as an "Archive".
    #[test]
    fn text_types_are_not_taken_for_binary() {
        let star = "application/vnd.github.star+json; charset=utf-8";
        assert_eq!(detect_response_type(star, b"[]", 200), Json);
        assert_eq!(
            detect_response_type("text/x-tar-list", b"a", 200),
            PlainText
        );
        assert_eq!(
            detect_response_type("application/x-gzip", b"\x1f", 200),
            Binary
        );
    }

    #[test]
    fn latin1_bodies_are_decoded() {
        let body = b"caf\xe9";
        assert_eq!(text_of(body, "text/plain; charset=ISO-8859-1"), "café");
        assert_eq!(text_of(body, "text/plain; charset=\"latin1\""), "café");
        assert_eq!(text_of("café".as_bytes(), "text/plain"), "café");
    }

    /// Without a Content-Length the size is only known once the body has
    /// been read; it used to be read whole, however big, and then dropped.
    #[test]
    fn an_undeclared_body_stops_at_the_limit() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Some(mut stream) = listener.incoming().flatten().next() {
                let _ = stream.read(&mut [0; 4096]);
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n");
                let chunk = vec![b'a'; 1024 * 1024];
                for _ in 0..30 {
                    if stream.write_all(&chunk).is_err() {
                        break;
                    }
                }
            }
        });
        let r = get(&format!("http://{addr}"), &[]).unwrap();
        assert_eq!(r.response_type, TooLarge);
        assert_eq!(r.size_bytes, MAX_RESPONSE_SIZE + 1);
    }

    #[test]
    fn detects_empty() {
        assert_eq!(detect_response_type("application/json", b"", 200), Empty);
        assert_eq!(detect_response_type("", b"x", 204), Empty);
    }

    #[test]
    fn sniffs_without_content_type() {
        assert_eq!(detect_response_type("", b"{\"k\":1}", 200), Json);
        assert_eq!(detect_response_type("", b"<root/>", 200), Xml);
        assert_eq!(
            detect_response_type("", b"<!DOCTYPE html><html>", 200),
            Html
        );
        assert_eq!(detect_response_type("", b"hello", 200), PlainText);
        assert_eq!(detect_response_type("", b"\xff\xfe", 200), Binary);
    }

    #[test]
    fn sniffing_is_utf8_safe() {
        let body = format!("<{}é</x>", "a".repeat(510));
        assert_eq!(detect_response_type("", body.as_bytes(), 200), Xml);
    }

    #[test]
    fn size_limits() {
        // a big body keeps its type: Mercury shows text of any size now
        // (it used to become `LargeText`, "too large to show", past 100 KB)
        let big = vec![b'a'; 5 * 1024 * 1024];
        assert_eq!(detect_response_type("application/json", &big, 200), Json);
        assert_eq!(detect_response_type("image/svg+xml", &big, 200), Xml);
        assert_eq!(detect_response_type("image/jpeg", &big, 200), Image);
        assert_eq!(
            detect_response_type("application/json", &[b'{'; 100], 200),
            Json
        );
    }

    #[test]
    fn cookies_are_case_insensitive() {
        let h = vec![
            ("set-cookie".to_string(), "a=1".to_string()),
            ("Content-Type".to_string(), "x".to_string()),
            ("SET-COOKIE".to_string(), "b=2; HttpOnly".to_string()),
        ];
        assert_eq!(extract_cookies(&h), vec!["a=1", "b=2; HttpOnly"]);
    }

    #[test]
    fn formats() {
        assert_eq!(format_json("{\"a\":1}"), "{\n  \"a\": 1\n}");
        assert_eq!(format_json("not json"), "not json");
        assert_eq!(format_xml("<r><i>t</i></r>"), "<r>\n  <i>t</i>\n</r>");
        assert_eq!(
            format_xml("<a><b></b><c x=\"1\"/>\n  <d>y</d></a>"),
            "<a>\n  <b></b>\n  <c x=\"1\"/>\n  <d>y</d>\n</a>"
        );
    }

    /// Pretty view has to show what the server sent. Going through
    /// `serde_json::Value` sorted every object's keys, turned integers past
    /// u64 into floats, rewrote `1.10` as `1.1`, and dropped duplicate keys.
    #[test]
    fn pretty_json_keeps_the_server_s_order_and_numbers() {
        assert_eq!(
            format_json(r#"{"id":1,"name":"Ada","active":true}"#),
            "{\n  \"id\": 1,\n  \"name\": \"Ada\",\n  \"active\": true\n}"
        );
        let numbers = format_json(r#"{"big":123456789012345678901234,"price":1.10,"exp":1e3}"#);
        assert!(
            numbers.contains("123456789012345678901234")
                && numbers.contains("1.10")
                && numbers.contains("1e3"),
            "{numbers}"
        );
        assert_eq!(format_json(r#"{"a":1,"a":2}"#).matches("\"a\"").count(), 2);
        assert_eq!(
            format_json(r#"{"x":[],"y":{},"s":"a, {b}: [c] \"q\""}"#),
            "{\n  \"x\": [],\n  \"y\": {},\n  \"s\": \"a, {b}: [c] \\\"q\\\"\"\n}"
        );
        assert_eq!(format_json("[1,  2]"), "[\n  1,\n  2\n]");
    }

    #[test]
    fn response_type_serializes_by_name() {
        // history.json stores the variant name; keep it that way
        assert_eq!(serde_json::to_string(&LargeText).unwrap(), "\"LargeText\"");
    }
}
