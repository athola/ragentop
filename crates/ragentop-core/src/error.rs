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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        assert_eq!(
            Error::Storage("disk full".into()).to_string(),
            "Storage error: disk full"
        );
        assert_eq!(
            Error::Adapter("timeout".into()).to_string(),
            "Adapter error: timeout"
        );
        assert_eq!(
            Error::Validation("bad input".into()).to_string(),
            "Validation error: bad input"
        );
        assert_eq!(
            Error::SessionNotFound("abc".into()).to_string(),
            "Session not found: abc"
        );
        assert_eq!(
            Error::Config("missing key".into()).to_string(),
            "Config error: missing key"
        );
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err: Error = io_err.into();
        assert!(err.to_string().contains("gone"));
    }

    #[test]
    fn from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{{bad").unwrap_err();
        let err: Error = json_err.into();
        assert!(err.to_string().starts_with("JSON error:"));
    }

    #[test]
    fn result_alias_works() {
        let ok: Result<i32> = Ok(42);
        assert!(ok.is_ok());

        let err: Result<i32> = Err(Error::Config("test".into()));
        assert!(err.is_err());
    }
}
