//! Error Module
//!
//! Centralized error types for Mercury with structured variants.
//! Uses `thiserror` for ergonomic Error trait implementations.

use thiserror::Error;

/// Centralized error type for Mercury application.
/// Each variant represents a distinct error category with user-friendly messages.
#[derive(Error, Debug)]
pub enum MercuryError {
    // =========================================================================
    // Network Errors
    // =========================================================================
    /// Connection to server failed (DNS, refused, unreachable)
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Request exceeded timeout duration
    #[error("Request timed out after {0}ms")]
    Timeout(u64),

    /// SSL/TLS certificate or handshake error
    #[error("SSL/TLS error: {0}")]
    TlsError(String),

    /// Generic request error (redirect limits, protocol issues)
    #[error("Request failed: {0}")]
    RequestFailed(String),

    // =========================================================================
    // File/IO Errors
    // =========================================================================
    /// Failed to read from file
    #[error("Failed to read file '{path}': {reason}")]
    FileRead { path: String, reason: String },

    /// Failed to write to file
    #[error("Failed to write file '{path}': {reason}")]
    FileWrite { path: String, reason: String },

    /// File or directory not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// File or folder already exists
    #[error("{kind} already exists: {name}")]
    AlreadyExists { kind: String, name: String },

    /// Failed to delete file or folder
    #[error("Failed to delete '{path}': {reason}")]
    DeleteFailed { path: String, reason: String },

    /// Failed to rename file or folder
    #[error("Failed to rename '{from}' to '{to}': {reason}")]
    RenameFailed {
        from: String,
        to: String,
        reason: String,
    },

    // =========================================================================
    // Parse Errors
    // =========================================================================
    /// Invalid JSON request file format
    #[error("Invalid HTTP file: {0}")]
    HttpParseError(String),

    /// Invalid JSON content
    #[error("Invalid JSON: {0}")]
    JsonError(String),

    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Invalid cURL command
    #[error("Invalid cURL command: {0}")]
    CurlParseError(String),

    // =========================================================================
    // Import Errors
    // =========================================================================
    /// Postman collection import failed
    #[error("Postman import failed: {0}")]
    PostmanImportError(String),

    /// Insomnia collection import failed
    #[error("Insomnia import failed: {0}")]
    InsomniaImportError(String),

    // =========================================================================
    // Workspace Errors
    // =========================================================================
    /// No workspace is currently loaded
    #[error("No workspace loaded")]
    NoWorkspace,

    /// Workspace directory not found
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    // =========================================================================
    // Environment Errors
    // =========================================================================
    /// Environment already exists
    #[error("Environment already exists: {0}")]
    EnvironmentExists(String),

    /// Failed to create environment
    #[error("Failed to create environment: {0}")]
    EnvironmentCreateFailed(String),

    // =========================================================================
    // File System Watcher Errors
    // =========================================================================
    /// File watcher error
    #[error("File watcher error: {0}")]
    FileWatcherError(String),
}

// =============================================================================
// Conversions from external errors
// =============================================================================

impl From<std::io::Error> for MercuryError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => MercuryError::FileNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => MercuryError::FileRead {
                path: String::new(),
                reason: "Permission denied".to_string(),
            },
            _ => MercuryError::FileRead {
                path: String::new(),
                reason: err.to_string(),
            },
        }
    }
}

impl From<serde_json::Error> for MercuryError {
    fn from(err: serde_json::Error) -> Self {
        MercuryError::JsonError(err.to_string())
    }
}

impl From<reqwest::Error> for MercuryError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            MercuryError::Timeout(30000) // Default timeout
        } else if err.is_connect() {
            MercuryError::ConnectionFailed(err.to_string())
        } else if err.is_builder() {
            MercuryError::InvalidUrl(err.to_string())
        } else {
            MercuryError::RequestFailed(err.to_string())
        }
    }
}

// =============================================================================
// User-friendly message helpers
// =============================================================================

impl MercuryError {
    /// Get a user-friendly message for display in the UI.
    /// These are actionable hints that help users resolve the issue.
    pub fn user_message(&self) -> &'static str {
        match self {
            // Network
            MercuryError::ConnectionFailed(_) => {
                "Could not connect to the server. Check your internet connection and the URL."
            }
            MercuryError::Timeout(_) => {
                "The server took too long to respond. Try again or increase the timeout."
            }
            MercuryError::TlsError(_) => {
                "SSL/TLS certificate error. The server's certificate may be invalid or expired."
            }
            MercuryError::RequestFailed(_) => {
                "The request could not be completed. Check the URL and try again."
            }

            // File
            MercuryError::FileRead { .. } => {
                "Could not read the file. Check if it exists and you have permission."
            }
            MercuryError::FileWrite { .. } => {
                "Could not save the file. Check write permissions and available disk space."
            }
            MercuryError::FileNotFound(_) => {
                "The file was not found. It may have been moved or deleted."
            }
            MercuryError::AlreadyExists { .. } => {
                "An item with this name already exists. Choose a different name."
            }
            MercuryError::DeleteFailed { .. } => {
                "Could not delete the item. It may be in use or protected."
            }
            MercuryError::RenameFailed { .. } => {
                "Could not rename the item. Check if the new name is valid."
            }

            // Parse
            MercuryError::HttpParseError(_) => {
                "Invalid HTTP file format. Expected: METHOD URL on the first line."
            }
            MercuryError::JsonError(_) => "The content is not valid JSON. Check for syntax errors.",
            MercuryError::InvalidUrl(_) => {
                "Please enter a valid URL (e.g., https://api.example.com)."
            }
            MercuryError::CurlParseError(_) => {
                "Could not parse the cURL command. Ensure it's a valid cURL command."
            }

            // Import
            MercuryError::PostmanImportError(_) => {
                "Could not import the Postman collection. Ensure it's a valid export file."
            }
            MercuryError::InsomniaImportError(_) => {
                "Could not import the Insomnia collection. Ensure it's a valid export file."
            }

            // Workspace
            MercuryError::NoWorkspace => "No workspace is open. Create or open a workspace first.",
            MercuryError::WorkspaceNotFound(_) => {
                "The workspace folder was not found. It may have been moved or deleted."
            }

            // Environment
            MercuryError::EnvironmentExists(_) => "An environment with this name already exists.",
            MercuryError::EnvironmentCreateFailed(_) => {
                "Could not create the environment file. Check write permissions."
            }

            // File Watcher
            MercuryError::FileWatcherError(_) => {
                "File system watcher encountered an error. Changes may not auto-refresh."
            }
        }
    }

    /// Returns true if this error is recoverable (user can retry)
    /// Future use: show "Retry" button on recoverable errors
    #[allow(dead_code)]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            MercuryError::ConnectionFailed(_)
                | MercuryError::Timeout(_)
                | MercuryError::RequestFailed(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MercuryError::ConnectionFailed("refused".to_string());
        assert_eq!(err.to_string(), "Connection failed: refused");
    }

    fn test_file_error_display() {
        let err = MercuryError::FileRead {
            path: "/tmp/test.json".to_string(),
            reason: "not found".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Failed to read file '/tmp/test.json': not found"
        );
    }

    #[test]
    fn test_already_exists_display() {
        let err = MercuryError::AlreadyExists {
            kind: "File".to_string(),
            name: "test.json".to_string(),
        };
        assert_eq!(err.to_string(), "File already exists: test.json");
    }

    #[test]
    fn test_user_message() {
        let err = MercuryError::Timeout(30000);
        assert!(err.user_message().contains("took too long"));
    }

    #[test]
    fn test_is_recoverable() {
        assert!(MercuryError::Timeout(1000).is_recoverable());
        assert!(MercuryError::ConnectionFailed("test".to_string()).is_recoverable());
        assert!(!MercuryError::FileNotFound("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let mercury_err: MercuryError = io_err.into();
        assert!(matches!(mercury_err, MercuryError::FileNotFound(_)));
    }

    #[test]
    fn test_from_json_error() {
        let json_str = "{ invalid }";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let mercury_err: MercuryError = json_err.into();
        assert!(matches!(mercury_err, MercuryError::JsonError(_)));
    }
}
