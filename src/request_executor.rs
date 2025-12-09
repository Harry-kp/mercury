use crate::http_parser::{HttpMethod, HttpRequest};
use serde_json::Value;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration_ms: u128,
    pub size_bytes: usize,
}

pub fn execute_request(request: &HttpRequest) -> Result<HttpResponse, String> {
    let start = Instant::now();

    let client = reqwest::blocking::Client::new();

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
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status().as_u16();
    let status_text = response.status().to_string();

    let mut headers = Vec::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.push((name.to_string(), value_str.to_string()));
        }
    }

    let body = response
        .text()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let size_bytes = body.len();
    let duration_ms = start.elapsed().as_millis();

    Ok(HttpResponse {
        status,
        status_text,
        headers,
        body,
        duration_ms,
        size_bytes,
    })
}

pub fn format_json(body: &str) -> String {
    match serde_json::from_str::<Value>(body) {
        Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string()),
        Err(_) => body.to_string(),
    }
}
