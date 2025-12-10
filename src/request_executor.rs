use crate::constants::MAX_RESPONSE_SIZE;
use crate::http_parser::{HttpMethod, HttpRequest};
use serde_json::Value;
use std::time::Instant;

/// Classification of response content for rendering
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseType {
    Json,
    Xml,
    Html,
    PlainText,
    Image(Vec<u8>), // Raw image bytes for display
    Binary,         // Non-displayable binary data
    TooLarge,       // Exceeded MAX_RESPONSE_SIZE
    LargeText,      // Text content too large for inline display (>100KB)
    Empty,          // 204 No Content or empty body
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub raw_bytes: Option<Vec<u8>>, // For binary/image content
    pub duration_ms: u128,
    pub size_bytes: usize,
    pub content_type: String,
    pub response_type: ResponseType,
}

/// Detect ResponseType from Content-Type header
fn detect_response_type(content_type: &str, body: &[u8], status: u16) -> ResponseType {
    // Handle empty responses
    if status == 204 || body.is_empty() {
        return ResponseType::Empty;
    }

    // Check size limit - too large to process
    if body.len() > MAX_RESPONSE_SIZE {
        return ResponseType::TooLarge;
    }

    // Large responses (>100KB) are treated as LargeText to prevent UI hangs
    // The clone() + format_json() + syntax_highlight() is too expensive for large text
    if body.len() > crate::constants::MAX_TEXT_DISPLAY_SIZE {
        return ResponseType::LargeText;
    }

    let ct_lower = content_type.to_lowercase();

    // JSON
    if ct_lower.contains("application/json") || ct_lower.contains("+json") {
        return ResponseType::Json;
    }

    // XML (including SOAP, RSS, Atom)
    if ct_lower.contains("application/xml")
        || ct_lower.contains("text/xml")
        || ct_lower.contains("+xml")
    {
        return ResponseType::Xml;
    }

    // HTML
    if ct_lower.contains("text/html") {
        return ResponseType::Html;
    }

    // Images - store raw bytes for display
    if ct_lower.starts_with("image/") {
        return ResponseType::Image(body.to_vec());
    }

    // Binary types that can't be displayed as text
    if ct_lower.starts_with("application/octet-stream")
        || ct_lower.starts_with("application/pdf")
        || ct_lower.starts_with("audio/")
        || ct_lower.starts_with("video/")
        || ct_lower.contains("zip")
        || ct_lower.contains("tar")
        || ct_lower.contains("gzip")
    {
        return ResponseType::Binary;
    }

    // Plain text types
    if ct_lower.starts_with("text/") {
        return ResponseType::PlainText;
    }

    // Fallback: try to detect if it's valid UTF-8 text
    if std::str::from_utf8(body).is_ok() {
        // Try to sniff JSON or XML from content
        let trimmed = String::from_utf8_lossy(body);
        let trimmed = trimmed.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return ResponseType::Json;
        }
        if trimmed.starts_with('<') {
            if trimmed.to_lowercase().contains("<!doctype html")
                || trimmed.to_lowercase().contains("<html")
            {
                return ResponseType::Html;
            }
            return ResponseType::Xml;
        }
        return ResponseType::PlainText;
    }

    ResponseType::Binary
}

/// Extract Content-Type header value
fn get_content_type(headers: &[(String, String)]) -> String {
    headers
        .iter()
        .find(|(name, _)| name.to_lowercase() == "content-type")
        .map(|(_, value)| value.clone())
        .unwrap_or_default()
}

pub fn execute_request(request: &HttpRequest) -> Result<HttpResponse, String> {
    let start = Instant::now();

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut req_builder = match request.method {
        HttpMethod::GET => client.get(&request.url),
        HttpMethod::POST => client.post(&request.url),
        HttpMethod::PUT => client.put(&request.url),
        HttpMethod::PATCH => client.patch(&request.url),
        HttpMethod::DELETE => client.delete(&request.url),
        HttpMethod::HEAD => client.head(&request.url),
        HttpMethod::OPTIONS => {
            return Err("OPTIONS method not yet supported".to_string());
        }
    };

    for (key, value) in &request.headers {
        req_builder = req_builder.header(key, value);
    }

    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    let response = req_builder
        .send()
        .map_err(|e| format_request_error(e))?;

    let status = response.status().as_u16();
    let status_text = response.status().to_string();

    let mut headers = Vec::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.push((name.to_string(), value_str.to_string()));
        }
    }

    let content_type = get_content_type(&headers);

    // Check Content-Length before downloading
    if let Some(content_length) = response.content_length() {
        if content_length as usize > MAX_RESPONSE_SIZE {
            let duration_ms = start.elapsed().as_millis();
            return Ok(HttpResponse {
                status,
                status_text,
                headers,
                body: String::new(),
                raw_bytes: None,
                duration_ms,
                size_bytes: content_length as usize,
                content_type,
                response_type: ResponseType::TooLarge,
            });
        }
    }

    // Read response as bytes first
    let raw_bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let size_bytes = raw_bytes.len();

    // Detect response type
    let response_type = detect_response_type(&content_type, &raw_bytes, status);

    // Convert to string (lossy for encoding errors)
    let body = match &response_type {
        ResponseType::Image(_) | ResponseType::Binary => {
            format!("[Binary data: {} bytes]", size_bytes)
        }
        ResponseType::TooLarge => {
            format!("[Response too large: {} bytes]", size_bytes)
        }
        _ => String::from_utf8_lossy(&raw_bytes).into_owned(),
    };

    let duration_ms = start.elapsed().as_millis();

    // Store raw bytes only for image type
    let stored_bytes = match &response_type {
        ResponseType::Image(bytes) => Some(bytes.clone()),
        _ => None,
    };

    Ok(HttpResponse {
        status,
        status_text,
        headers,
        body,
        raw_bytes: stored_bytes,
        duration_ms,
        size_bytes,
        content_type,
        response_type,
    })
}

/// Format request errors with user-friendly messages
fn format_request_error(e: reqwest::Error) -> String {
    if e.is_timeout() {
        return "Request timed out after 30 seconds".to_string();
    }
    if e.is_connect() {
        return format!("Connection failed: {}", e);
    }
    if e.is_builder() {
        return format!("Invalid request: {}", e);
    }
    format!("Request failed: {}", e)
}

pub fn format_json(body: &str) -> String {
    match serde_json::from_str::<Value>(body) {
        Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string()),
        Err(_) => body.to_string(),
    }
}

/// Format XML with basic indentation
pub fn format_xml(body: &str) -> String {
    let mut result = String::new();
    let mut indent = 0usize;
    let mut _in_tag = false;
    let mut tag_content = String::new();

    for ch in body.chars() {
        match ch {
            '<' => {
                // Flush any text content
                let trimmed = tag_content.trim();
                if !trimmed.is_empty() {
                    result.push_str(trimmed);
                }
                tag_content.clear();
                _in_tag = true;
                tag_content.push(ch);
            }
            '>' => {
                tag_content.push(ch);
                _in_tag = false;

                let tag = tag_content.trim();
                let is_closing = tag.starts_with("</");
                let is_self_closing = tag.ends_with("/>");
                let is_declaration = tag.starts_with("<?") || tag.starts_with("<!");

                if is_closing {
                    indent = indent.saturating_sub(1);
                }

                // Add newline and indent
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push_str(&"  ".repeat(indent));
                result.push_str(tag);

                if !is_closing && !is_self_closing && !is_declaration {
                    indent += 1;
                }

                tag_content.clear();
            }
            _ => {
                tag_content.push(ch);
            }
        }
    }

    // Flush remaining content
    let trimmed = tag_content.trim();
    if !trimmed.is_empty() {
        result.push_str(trimmed);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_json() {
        let body = b"{\"key\": \"value\"}";
        let result = detect_response_type("application/json", body, 200);
        assert_eq!(result, ResponseType::Json);
    }

    #[test]
    fn test_detect_xml() {
        let body = b"<root><item>test</item></root>";
        let result = detect_response_type("application/xml", body, 200);
        assert_eq!(result, ResponseType::Xml);
    }

    #[test]
    fn test_detect_html() {
        let body = b"<!DOCTYPE html><html><body>Test</body></html>";
        let result = detect_response_type("text/html", body, 200);
        assert_eq!(result, ResponseType::Html);
    }

    #[test]
    fn test_detect_empty() {
        let body = b"";
        let result = detect_response_type("application/json", body, 204);
        assert_eq!(result, ResponseType::Empty);
    }

    #[test]
    fn test_detect_binary() {
        let body = b"\x00\x01\x02\x03";
        let result = detect_response_type("application/octet-stream", body, 200);
        assert_eq!(result, ResponseType::Binary);
    }

    #[test]
    fn test_format_xml() {
        let xml = "<root><item>test</item></root>";
        let formatted = format_xml(xml);
        assert!(formatted.contains('\n'));
        assert!(formatted.contains("  ")); // Has indentation
    }

    #[test]
    fn test_sniff_json_without_content_type() {
        let body = b"{\"key\": \"value\"}";
        let result = detect_response_type("", body, 200);
        assert_eq!(result, ResponseType::Json);
    }

    #[test]
    fn test_sniff_xml_without_content_type() {
        let body = b"<root>test</root>";
        let result = detect_response_type("", body, 200);
        assert_eq!(result, ResponseType::Xml);
    }
}
