//! Error types for Nest API

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Sandbox error: {0}")]
    Sandbox(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Message bus error: {0}")]
    MessageBus(String),

    #[error("Audit log error: {0}")]
    Audit(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Sandbox(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
