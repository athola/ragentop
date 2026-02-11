//! Configuration types (pure data structures).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default socket path when runtime dir is unavailable.
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/ragentop.sock";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(default)]
pub struct Config {
    pub daemon: Daemon,
    pub tui: Tui,
    pub web: Web,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(default)]
pub struct Daemon {
    pub poll_interval_ms: u64,
    pub socket_path: PathBuf,
}

impl Default for Daemon {
    #[inline]
    fn default() -> Self {
        Self {
            poll_interval_ms: 2000,
            socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(default)]
pub struct Tui {
    pub ascii_mode: bool,
    pub default_depth: u8,
    pub mouse_enabled: bool,
}

impl Default for Tui {
    #[inline]
    fn default() -> Self {
        Self {
            ascii_mode: false,
            default_depth: 2,
            mouse_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(default)]
pub struct Web {
    pub bind_address: String,
    pub port: u16,
}

impl Default for Web {
    #[inline]
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_owned(),
            port: 8080,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_config_default() {
        let config = Daemon::default();
        assert_eq!(config.poll_interval_ms, 2000);
        assert_eq!(config.socket_path, PathBuf::from(DEFAULT_SOCKET_PATH));
    }

    #[test]
    fn test_tui_config_default() {
        let config = Tui::default();
        assert_eq!(config.default_depth, 2);
        assert!(config.mouse_enabled);
        assert!(!config.ascii_mode);
    }

    #[test]
    fn test_web_config_default() {
        let config = Web::default();
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.tui.default_depth, 2);
        assert_eq!(config.web.port, 8080);
    }

    #[test]
    fn test_config_toml_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let toml_str = toml::to_string(&config)?;
        let parsed: Config = toml::from_str(&toml_str)?;
        assert_eq!(parsed.tui.default_depth, config.tui.default_depth);
        assert_eq!(parsed.web.port, config.web.port);
        Ok(())
    }

    #[test]
    fn test_config_partial_toml() -> Result<(), Box<dyn std::error::Error>> {
        let toml_str = r"
            [tui]
            default_depth = 5
        ";
        let config: Config = toml::from_str(toml_str)?;
        assert_eq!(config.tui.default_depth, 5);
        assert_eq!(config.web.port, 8080);
        Ok(())
    }
}
