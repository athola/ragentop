//! Multiplexer port for terminal multiplexer integrations.
//!
//! This trait defines the interface for interacting with terminal multiplexers
//! like Zellij, tmux, etc. Implementations live in the daemon crate.

use crate::Result;

/// Information about a terminal pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneInfo {
    /// Unique identifier for the pane.
    pub id: String,
    /// Display title of the pane.
    pub title: String,
    /// Whether this pane is currently active/focused.
    pub active: bool,
}

/// Port trait for terminal multiplexer operations.
///
/// This is a port in the hexagonal architecture sense - implementations
/// (adapters) live outside the core crate.
pub trait Multiplexer {
    /// List all panes in the current session.
    ///
    /// # Errors
    /// Returns an error if the multiplexer command fails or output cannot be parsed.
    fn list_panes(&self) -> Result<Vec<PaneInfo>>;

    /// Rename a pane by its ID.
    ///
    /// # Errors
    /// Returns an error if the multiplexer command fails.
    fn rename_pane(&self, pane_id: &str, name: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_info_equality() {
        let pane1 = PaneInfo {
            id: "1".to_string(),
            title: "test".to_string(),
            active: true,
        };
        let pane2 = PaneInfo {
            id: "1".to_string(),
            title: "test".to_string(),
            active: true,
        };
        assert_eq!(pane1, pane2);
    }

    #[test]
    fn test_pane_info_inequality() {
        let pane1 = PaneInfo {
            id: "1".to_string(),
            title: "test".to_string(),
            active: true,
        };
        let pane2 = PaneInfo {
            id: "2".to_string(),
            title: "test".to_string(),
            active: true,
        };
        assert_ne!(pane1, pane2);
    }

    #[test]
    fn test_pane_info_clone() {
        let pane = PaneInfo {
            id: "1".to_string(),
            title: "shell".to_string(),
            active: false,
        };
        let cloned = pane.clone();
        assert_eq!(pane, cloned);
    }

    #[test]
    fn test_pane_info_debug() {
        let pane = PaneInfo {
            id: "%0".to_string(),
            title: "zsh".to_string(),
            active: true,
        };
        let debug = format!("{pane:?}");
        assert!(debug.contains("%0"));
        assert!(debug.contains("zsh"));
    }
}
