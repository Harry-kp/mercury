//! Sending requests and classifying/formatting responses.

use crate::model::RequestFile;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub const TIMEOUT_SECS: u64 = 30;
/// Bodies larger than this are not downloaded ("Response Too Large").
pub const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;
/// Text bodies larger than this are not rendered inline: highlighting
/// thousands of spans per frame would drop the UI below 60fps.
pub const MAX_INLINE_SIZE: usize = 100 * 1024;

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
    /// Text over MAX_INLINE_SIZE; offered as a download instead.
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
}

/// One client for the whole session so cookies persist across requests.
pub fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()
        .expect("HTTP client builds with static config")
}

/// Blocking; call from a background thread.
pub fn execute(
    client: &reqwest::blocking::Client,
    request: &RequestFile,
) -> Result<HttpResponse, String> {
    let start = Instant::now();
    let method = reqwest::Method::from_bytes(request.method.as_str().as_bytes())
        .expect("HttpMethod names are valid HTTP methods");
    let mut builder = client.request(method, &request.url);
    for (key, value) in &request.headers {
        builder = builder.header(key, value);
    }
    if !request.body.is_empty() {
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
        let bytes = response
            .bytes()
            .map_err(|e| format!("Failed to read response body: {e}"))?
            .to_vec();
        let kind = detect_response_type(&content_type, &bytes, status.as_u16());
        let len = bytes.len();
        (bytes, kind, len)
    };

    let (body, raw_bytes) = match response_type {
        ResponseType::Image | ResponseType::Binary => {
            (format!("[Binary data: {size_bytes} bytes]"), Some(bytes))
        }
        ResponseType::TooLarge => (String::new(), None),
        _ => (String::from_utf8_lossy(&bytes).into_owned(), None),
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
    })
}

fn error_message(e: &reqwest::Error) -> String {
    let text = e.to_string();
    let lower = text.to_lowercase();
    if e.is_timeout() {
        format!("Request timed out after {TIMEOUT_SECS}s")
    } else if e.is_connect()
        && ["certificate", "ssl", "tls", "handshake"]
            .iter()
            .any(|w| lower.contains(w))
    {
        format!("SSL/TLS error: {text}")
    } else if e.is_connect() {
        format!("Connection failed: {text}")
    } else if e.is_builder() {
        format!("Invalid URL: {text}")
    } else {
        format!("Request failed: {text}")
    }
}

fn extract_cookies(headers: &[(String, String)]) -> Vec<String> {
    headers
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, v)| v.clone())
        .collect()
}

fn detect_response_type(content_type: &str, body: &[u8], status: u16) -> ResponseType {
    if status == 204 || body.is_empty() {
        return ResponseType::Empty;
    }
    if body.len() > MAX_RESPONSE_SIZE {
        return ResponseType::TooLarge;
    }
    let ct = content_type.to_lowercase();
    // SVG is text; let it fall through to XML
    if ct.starts_with("image/") && !ct.contains("svg") {
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
    // after Image/Binary so large images stay images
    if body.len() > MAX_INLINE_SIZE {
        return ResponseType::LargeText;
    }
    if ct.contains("application/json") || ct.contains("+json") {
        return ResponseType::Json;
    }
    if ct.contains("application/xml") || ct.contains("text/xml") || ct.contains("+xml") {
        return ResponseType::Xml;
    }
    if ct.contains("text/html") {
        return ResponseType::Html;
    }
    if ct.starts_with("text/") {
        return ResponseType::PlainText;
    }
    // no useful content-type: sniff
    let Ok(text) = std::str::from_utf8(body) else {
        return ResponseType::Binary;
    };
    let text = text.trim_start();
    if text.starts_with('{') || text.starts_with('[') {
        ResponseType::Json
    } else if text.starts_with('<') {
        let head = text[..text.len().min(512)].to_lowercase();
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
pub fn format_json(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| body.to_string())
}

/// Indent XML one level per open tag. Naive by design: good enough to read.
pub fn format_xml(body: &str) -> String {
    let mut out = String::new();
    let mut indent = 0usize;
    let mut token = String::new();
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
                if !out.is_empty() && !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&"  ".repeat(indent));
                out.push_str(tag);
                if opens {
                    indent += 1;
                }
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
    use ResponseType::*;

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
    fn size_limits() {
        let big = vec![b'a'; MAX_INLINE_SIZE + 1];
        assert_eq!(
            detect_response_type("application/json", &big, 200),
            LargeText
        );
        assert_eq!(detect_response_type("image/svg+xml", &big, 200), LargeText);
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
        assert_eq!(format_xml("<r><i>t</i></r>"), "<r>\n  <i>t\n  </i>\n</r>");
    }

    #[test]
    fn response_type_serializes_by_name() {
        // history.json stores the variant name; keep it that way
        assert_eq!(serde_json::to_string(&LargeText).unwrap(), "\"LargeText\"");
    }
}
