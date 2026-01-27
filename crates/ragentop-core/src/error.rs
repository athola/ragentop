//! Error types for ragentop.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Adapter error: {0}")]
    Adapter(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Config error: {0}")]
    Config(String),
}
