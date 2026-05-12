//! Configuration types (pure data structures).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::alert::AlertThresholds;
use crate::burnrate::Thresholds as CostThresholds;

/// Default socket path when runtime dir is unavailable.
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/ragentop.sock";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(default)]
pub struct Config {
    pub daemon: Daemon,
    pub tui: Tui,
    pub web: Web,
    pub alerts: AlertConfig,
    pub display: DisplayConfig,
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

/// Configuration for the alert subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(default)]
pub struct AlertConfig {
    /// Thresholds for triggering alerts.
    pub thresholds: AlertThresholds,
    /// Deduplication window in seconds.
    pub dedup_window_secs: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            thresholds: AlertThresholds::default(),
            dedup_window_secs: 300,
        }
    }
}

/// Configuration for display and UI behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(default)]
pub struct DisplayConfig {
    /// Maximum number of events to keep in the ring buffer.
    pub event_buffer_size: usize,
    /// UI refresh rate in milliseconds.
    pub refresh_rate_ms: u64,
    /// Thresholds for cost burn-rate color coding.
    pub cost_thresholds: CostThresholds,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            event_buffer_size: 1000,
            refresh_rate_ms: 500,
            cost_thresholds: CostThresholds::default(),
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

    // -- AlertConfig --

    #[test]
    fn test_alert_config_default() {
        let config = AlertConfig::default();
        assert_eq!(config.dedup_window_secs, 300);
        assert_eq!(config.thresholds.loop_threshold, 3);
        assert_eq!(config.thresholds.error_storm_count, 10);
    }

    #[test]
    fn test_alert_config_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let config = AlertConfig::default();
        let json = serde_json::to_string(&config)?;
        let parsed: AlertConfig = serde_json::from_str(&json)?;
        assert_eq!(parsed.dedup_window_secs, config.dedup_window_secs);
        assert_eq!(
            parsed.thresholds.loop_threshold,
            config.thresholds.loop_threshold
        );
        Ok(())
    }

    // -- DisplayConfig --

    #[test]
    fn test_display_config_default() {
        let config = DisplayConfig::default();
        assert_eq!(config.event_buffer_size, 1000);
        assert_eq!(config.refresh_rate_ms, 500);
        assert!((config.cost_thresholds.green_below - 0.50).abs() < f64::EPSILON);
        assert!((config.cost_thresholds.yellow_below - 2.00).abs() < f64::EPSILON);
    }

    #[test]
    fn test_display_config_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let config = DisplayConfig::default();
        let json = serde_json::to_string(&config)?;
        let parsed: DisplayConfig = serde_json::from_str(&json)?;
        assert_eq!(parsed.event_buffer_size, config.event_buffer_size);
        assert_eq!(parsed.refresh_rate_ms, config.refresh_rate_ms);
        Ok(())
    }

    // -- Full config with new sections --

    #[test]
    fn test_full_config_with_alerts_and_display() -> Result<(), Box<dyn std::error::Error>> {
        let toml_str = r"
            [alerts]
            dedup_window_secs = 600

            [alerts.thresholds]
            loop_threshold = 5

            [display]
            event_buffer_size = 500
            refresh_rate_ms = 1000
        ";
        let config: Config = toml::from_str(toml_str)?;
        assert_eq!(config.alerts.dedup_window_secs, 600);
        assert_eq!(config.alerts.thresholds.loop_threshold, 5);
        assert_eq!(config.display.event_buffer_size, 500);
        assert_eq!(config.display.refresh_rate_ms, 1000);
        // Other fields use defaults
        assert_eq!(config.alerts.thresholds.error_storm_count, 10);
        Ok(())
    }
}
