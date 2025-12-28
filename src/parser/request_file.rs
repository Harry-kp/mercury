//! Request File Parser Module
//!
//! Parses and serializes JSON request files for collection storage.

use crate::core::error::MercuryError;
use crate::core::types::JsonRequest;

/// Parse a JSON request file content
pub fn parse_request_file(content: &str) -> Result<JsonRequest, MercuryError> {
    serde_json::from_str(content)
        .map_err(|e| MercuryError::HttpParseError(format!("Invalid JSON request file: {}", e)))
}

/// Serialize a request to JSON format for saving to file
pub fn serialize_request_file(request: &JsonRequest) -> Result<String, MercuryError> {
    serde_json::to_string_pretty(request)
        .map_err(|e| MercuryError::HttpParseError(format!("Failed to serialize request: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::HttpMethod;
    use std::collections::HashMap;

    #[test]
    fn test_parse_simple_get() {
        let json = r#"{"method": "GET", "url": "https://api.example.com/users"}"#;
        let request = parse_request_file(json).unwrap();

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.url, "https://api.example.com/users");
        assert!(request.headers.is_empty());
        assert!(request.body.is_empty());
    }

    #[test]
    fn test_parse_with_headers_and_body() {
        let json = r#"{
            "method": "POST",
            "url": "https://api.example.com/users",
            "headers": {
                "Content-Type": "application/json",
                "Authorization": "Bearer token123"
            },
            "body": "{\"name\": \"John\"}"
        }"#;
        let request = parse_request_file(json).unwrap();

        assert_eq!(request.method, HttpMethod::POST);
        assert_eq!(request.url, "https://api.example.com/users");
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(request.body, "{\"name\": \"John\"}");
    }

    #[test]
    fn test_serialize_request() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let request = JsonRequest {
            method: HttpMethod::POST,
            url: "https://api.example.com/users".to_string(),
            headers,
            body: "{\"name\": \"John\"}".to_string(),
        };

        let json = serialize_request_file(&request).unwrap();
        assert!(json.contains("\"method\": \"POST\""));
        assert!(json.contains("\"url\": \"https://api.example.com/users\""));
        assert!(json.contains("\"Content-Type\": \"application/json\""));
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_request_file("NOT JSON");
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer {{token}}".to_string());

        let original = JsonRequest {
            method: HttpMethod::DELETE,
            url: "https://api.example.com/users/1".to_string(),
            headers,
            body: String::new(),
        };

        let json = serialize_request_file(&original).unwrap();
        let parsed = parse_request_file(&json).unwrap();

        assert_eq!(parsed.method, original.method);
        assert_eq!(parsed.url, original.url);
        assert_eq!(parsed.headers, original.headers);
        assert_eq!(parsed.body, original.body);
    }
}
