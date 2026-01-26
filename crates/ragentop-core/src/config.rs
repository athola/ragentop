//! Configuration system.

use crate::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub tui: TuiConfig,
    pub web: WebConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    pub poll_interval_ms: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: dirs::runtime_dir().map_or_else(
                || PathBuf::from("/tmp/ragentop.sock"),
                |d| d.join("ragentop.sock"),
            ),
            poll_interval_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    pub default_depth: u8,
    pub mouse_enabled: bool,
    pub ascii_mode: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            default_depth: 2,
            mouse_enabled: true,
            ascii_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebConfig {
    pub bind_address: String,
    pub port: u16,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

impl Config {
    /// Returns the default config directory path.
    #[must_use]
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "ragentop", "ragentop").map(|p| p.config_dir().to_path_buf())
    }

    /// Loads config from the default path, or returns default if not found.
    ///
    /// # Errors
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self> {
        if let Some(path) = Self::config_dir().map(|p| p.join("config.toml")) {
            if path.exists() {
                let contents = std::fs::read_to_string(&path)?;
                return Ok(toml::from_str(&contents)?);
            }
        }
        Ok(Self::default())
    }
}
