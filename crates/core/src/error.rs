use std::fmt;
use thiserror::Error;

/// Core error type for Claude Code
#[derive(Error, Debug, Clone)]
pub enum ClaudeError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Cancelled")]
    Cancelled,

    #[error("Timeout")]
    Timeout,
}

impl From<std::io::Error> for ClaudeError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for ClaudeError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

pub type ClaudeResult<T> = Result<T, ClaudeError>;
