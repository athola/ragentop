//! Configuration types (pure data structures).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default socket path when runtime dir is unavailable.
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/ragentop.sock";

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
            socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_config_default() {
        let config = DaemonConfig::default();
        assert_eq!(config.poll_interval_ms, 2000);
        assert_eq!(config.socket_path, PathBuf::from(DEFAULT_SOCKET_PATH));
    }

    #[test]
    fn test_tui_config_default() {
        let config = TuiConfig::default();
        assert_eq!(config.default_depth, 2);
        assert!(config.mouse_enabled);
        assert!(!config.ascii_mode);
    }

    #[test]
    fn test_web_config_default() {
        let config = WebConfig::default();
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
    fn test_config_toml_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("serialize");
        let parsed: Config = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(parsed.tui.default_depth, config.tui.default_depth);
        assert_eq!(parsed.web.port, config.web.port);
    }

    #[test]
    fn test_config_partial_toml() {
        let toml_str = r"
            [tui]
            default_depth = 5
        ";
        let config: Config = toml::from_str(toml_str).expect("parse partial");
        assert_eq!(config.tui.default_depth, 5);
        assert_eq!(config.web.port, 8080);
    }
}
