//! Error types for the core module.

use thiserror::Error;

/// Errors that can occur during app ID sanitization or Sunshine config operations.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("apps.json path is empty")]
    EmptyPath,

    #[error("failed to read apps.json: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("failed to parse apps.json: {0}")]
    ParseError(#[from] serde_json::Error),

    // MarshalError removed — serde_json::Error already covered by ParseError
    #[error("apps.json is neither an object map nor an {{\"apps\": [...]}} array")]
    InvalidFormat,

    #[error("plugin {0} is already registered")]
    DuplicatePlugin(String),

    #[error("plugin cannot be nil")]
    NilPlugin,
}
