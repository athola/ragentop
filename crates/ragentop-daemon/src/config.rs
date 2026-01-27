//! Configuration loading (I/O operations).

use directories::ProjectDirs;
use ragentop_core::{Config, Result};
use std::path::PathBuf;

/// Returns the default config directory path.
#[must_use]
pub fn config_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "ragentop", "ragentop").map(|p| p.config_dir().to_path_buf())
}

/// Loads config from the default path, or returns default if not found.
///
/// # Errors
/// Returns an error if the config file exists but cannot be read or parsed.
pub fn load_config() -> Result<Config> {
    if let Some(path) = config_dir().map(|p| p.join("config.toml")) {
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            return Ok(toml::from_str(&contents)?);
        }
    }
    Ok(Config::default())
}
