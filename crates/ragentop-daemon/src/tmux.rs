//! Tmux adapter implementing the Multiplexer trait.

use ragentop_core::multiplexer::{Multiplexer, PaneInfo};
use ragentop_core::Result;
use tmux_interface::{ListPanes, SelectPane, Tmux};

/// Shell metacharacters that could enable command injection.
const SHELL_METACHARACTERS: &[char] = &[';', '`', '$', '|', '&', '(', ')', '<', '>'];

/// Validates that a string contains no shell metacharacters.
fn validate_no_shell_metacharacters(s: &str, field_name: &str) -> Result<()> {
    if s.chars().any(|c| SHELL_METACHARACTERS.contains(&c)) {
        return Err(ragentop_core::Error::Validation(format!(
            "{field_name} contains invalid shell metacharacters"
        )));
    }
    Ok(())
}

/// Validates that a `pane_id` matches tmux format: % followed by digits.
fn validate_pane_id(pane_id: &str) -> Result<()> {
    if !pane_id.starts_with('%') {
        return Err(ragentop_core::Error::Validation(
            "pane_id must start with '%'".to_string(),
        ));
    }
    if pane_id.len() < 2 || !pane_id[1..].chars().all(|c| c.is_ascii_digit()) {
        return Err(ragentop_core::Error::Validation(
            "pane_id must be '%' followed by digits only".to_string(),
        ));
    }
    Ok(())
}

/// Adapter for interacting with tmux terminal multiplexer.
#[derive(Debug, Default)]
pub struct TmuxAdapter;

impl TmuxAdapter {
    /// Create a new `TmuxAdapter`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Multiplexer for TmuxAdapter {
    fn list_panes(&self) -> Result<Vec<PaneInfo>> {
        let list_panes = ListPanes::new().format("#{pane_id}:#{pane_title}:#{pane_active}");
        let output = Tmux::with_command(list_panes)
            .output()
            .map_err(|e| ragentop_core::Error::Adapter(format!("tmux command failed: {e}")))?;

        if !output.success() {
            return Err(ragentop_core::Error::Adapter(
                "tmux list-panes failed".to_string(),
            ));
        }

        let stdout_bytes = output.stdout();
        let stdout = String::from_utf8_lossy(&stdout_bytes);
        let panes = stdout
            .lines()
            .filter_map(|line: &str| {
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                if parts.len() >= 3 {
                    Some(PaneInfo {
                        id: parts[0].to_string(),
                        title: parts[1].to_string(),
                        active: parts[2] == "1",
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(panes)
    }

    fn rename_pane(&self, pane_id: &str, name: &str) -> Result<()> {
        // Validate inputs to prevent command injection
        validate_pane_id(pane_id)?;
        validate_no_shell_metacharacters(name, "name")?;

        let select_pane = SelectPane::new().title(name).target_pane(pane_id);
        let output = Tmux::with_command(select_pane)
            .output()
            .map_err(|e| ragentop_core::Error::Adapter(format!("tmux command failed: {e}")))?;

        if !output.success() {
            return Err(ragentop_core::Error::Adapter(
                "tmux select-pane failed".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_adapter() {
        let adapter = TmuxAdapter::new();
        assert_eq!(format!("{adapter:?}"), "TmuxAdapter");
    }

    #[test]
    fn test_default_creates_adapter() {
        let adapter = TmuxAdapter;
        assert_eq!(format!("{adapter:?}"), "TmuxAdapter");
    }

    #[test]
    fn test_validate_pane_id_valid() {
        assert!(validate_pane_id("%0").is_ok());
        assert!(validate_pane_id("%123").is_ok());
    }

    #[test]
    fn test_validate_pane_id_invalid() {
        assert!(validate_pane_id("0").is_err());
        assert!(validate_pane_id("%").is_err());
        assert!(validate_pane_id("%abc").is_err());
        assert!(validate_pane_id("%;rm -rf").is_err());
    }

    #[test]
    fn test_validate_no_shell_metacharacters_valid() {
        assert!(validate_no_shell_metacharacters("my-session", "test").is_ok());
        assert!(validate_no_shell_metacharacters("session_123", "test").is_ok());
    }

    #[test]
    fn test_validate_no_shell_metacharacters_invalid() {
        assert!(validate_no_shell_metacharacters("foo;rm -rf /", "test").is_err());
        assert!(validate_no_shell_metacharacters("$(whoami)", "test").is_err());
        assert!(validate_no_shell_metacharacters("`id`", "test").is_err());
        assert!(validate_no_shell_metacharacters("foo|bar", "test").is_err());
    }
}
