//! Parser Module
//!
//! Parsers for different file formats: JSON request files, cURL commands, .env files.

pub mod curl;
pub mod env;
pub mod request_file;

// Re-export commonly used items
pub use curl::parse_curl;
pub use env::{parse_env_file, substitute_variables};
pub use request_file::{parse_request_file, serialize_request_file};

// Re-export HttpMethod from types for backward compatibility with existing imports
pub use crate::core::types::HttpMethod;
