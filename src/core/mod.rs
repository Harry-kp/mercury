//! Core Module
//!
//! Core business logic: types, persistence, constants, and HTTP execution.

pub mod constants;
pub mod persistence;
pub mod request;
pub mod types;

// Re-export commonly used items
pub use request::{execute_request, format_json, format_xml, HttpResponse, ResponseType};
