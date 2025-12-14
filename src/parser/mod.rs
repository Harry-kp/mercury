//! Parser Module
//!
//! Parsers for different file formats: .http, cURL commands, .env files.

pub mod curl;
pub mod env;
pub mod http;

// Re-export commonly used items
pub use curl::parse_curl;
pub use env::{parse_env_file, substitute_variables};
pub use http::{parse_http_file, HttpMethod, HttpRequest};
