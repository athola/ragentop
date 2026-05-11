//! Zellij multiplexer adapter implementation.

use ragentop_core::{Error, Multiplexer, PaneInfo, Result};
use std::process::Command;

/// Shell metacharacters that could enable command injection.
const SHELL_METACHARACTERS: &[char] = &[';', '`', '$', '|', '&', '(', ')', '<', '>'];

/// Validates that a string contains no shell metacharacters.
fn validate_no_shell_metacharacters(s: &str, field_name: &str) -> Result<()> {
    if s.chars().any(|c| SHELL_METACHARACTERS.contains(&c)) {
        return Err(Error::Validation(format!(
            "{field_name} contains invalid shell metacharacters"
        )));
    }
    Ok(())
}

/// Validates that a `pane_id` is a valid numeric identifier.
fn validate_pane_id(pane_id: &str) -> Result<()> {
    if pane_id.is_empty() {
        return Err(Error::Validation("pane_id cannot be empty".to_string()));
    }
    if !pane_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(Error::Validation(
            "pane_id must contain only digits".to_string(),
        ));
    }
    Ok(())
}

/// Zellij multiplexer adapter.
#[derive(Debug, Default)]
pub struct ZellijAdapter;

impl ZellijAdapter {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Multiplexer for ZellijAdapter {
    fn list_panes(&self) -> Result<Vec<PaneInfo>> {
        let output = Command::new("zellij")
            .args(["action", "dump-session"])
            .output()
            .map_err(|e| Error::Adapter(format!("Failed to run zellij: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Adapter(format!(
                "zellij dump-session failed: {stderr}"
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_zellij_session(&stdout))
    }

    fn rename_pane(&self, pane_id: &str, name: &str) -> Result<()> {
        // Validate inputs to prevent command injection
        validate_pane_id(pane_id)?;
        validate_no_shell_metacharacters(name, "name")?;

        // Zellij's CLI exposes no "focus pane by id" action — `rename-pane`
        // always operates on the currently-focused pane. We cycle through
        // `focus-next-pane` until the requested pane becomes active, then
        // rename. Aborts if cycling fails to advance focus (e.g. only one
        // pane exists and it isn't the target).
        let panes = self.list_panes()?;
        let target_exists = panes.iter().any(|p| p.id == pane_id);
        if !target_exists {
            return Err(Error::Adapter(format!(
                "zellij: pane {pane_id} not found ({} panes available)",
                panes.len()
            )));
        }

        let max_cycles = panes.len();
        let mut last_focused: Option<String> =
            panes.iter().find(|p| p.active).map(|p| p.id.clone());

        if last_focused.as_deref() != Some(pane_id) {
            for _ in 0..max_cycles {
                let output = Command::new("zellij")
                    .args(["action", "focus-next-pane"])
                    .output()
                    .map_err(|e| {
                        Error::Adapter(format!("Failed to run zellij focus-next-pane: {e}"))
                    })?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(Error::Adapter(format!(
                        "zellij focus-next-pane failed: {stderr}"
                    )));
                }

                let now_panes = self.list_panes()?;
                let now_focused = now_panes.iter().find(|p| p.active).map(|p| p.id.clone());
                if now_focused.as_deref() == Some(pane_id) {
                    last_focused = now_focused;
                    break;
                }
                if now_focused == last_focused {
                    return Err(Error::Adapter(format!(
                        "zellij: focus-next-pane did not advance focus from {last_focused:?}"
                    )));
                }
                last_focused = now_focused;
            }
        }

        if last_focused.as_deref() != Some(pane_id) {
            return Err(Error::Adapter(format!(
                "zellij: could not focus pane {pane_id} after {max_cycles} cycle(s)"
            )));
        }

        let output = Command::new("zellij")
            .args(["action", "rename-pane", name])
            .output()
            .map_err(|e| Error::Adapter(format!("Failed to run zellij: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Adapter(format!(
                "zellij rename-pane failed for {pane_id}: {stderr}"
            )));
        }

        Ok(())
    }
}

/// Parse Zellij dump-session output into `PaneInfo` structs.
fn parse_zellij_session(output: &str) -> Vec<PaneInfo> {
    // Zellij dump-session outputs KDL format
    // This is a simplified parser - full implementation would use kdl crate
    let mut panes = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pane") {
            let pane = parse_pane_line(trimmed);
            if let Some(p) = pane {
                panes.push(p);
            }
        }
    }

    panes
}

/// Parse a single pane line from KDL output.
fn parse_pane_line(line: &str) -> Option<PaneInfo> {
    // Example: pane id=1 name="editor" focus=true
    let mut id = String::new();
    let mut title = String::new();
    let mut active = false;

    for part in line.split_whitespace() {
        if let Some(val) = part.strip_prefix("id=") {
            id = val.to_string();
        } else if let Some(val) = part.strip_prefix("name=") {
            title = val.trim_matches('"').to_string();
        } else if part == "focus=true" {
            active = true;
        }
    }

    if id.is_empty() {
        return None;
    }

    Some(PaneInfo { id, title, active })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pane_line() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let line = r#"pane id=1 name="editor" focus=true"#;
        let pane = parse_pane_line(line).ok_or("failed to parse pane")?;
        assert_eq!(pane.id, "1");
        assert_eq!(pane.title, "editor");
        assert!(pane.active);
        Ok(())
    }

    #[test]
    fn test_parse_pane_line_no_focus() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let line = r#"pane id=2 name="terminal""#;
        let pane = parse_pane_line(line).ok_or("failed to parse pane")?;
        assert_eq!(pane.id, "2");
        assert_eq!(pane.title, "terminal");
        assert!(!pane.active);
        Ok(())
    }

    #[test]
    fn test_parse_empty_id() {
        let line = "pane name=\"test\"";
        assert!(parse_pane_line(line).is_none());
    }

    #[test]
    fn test_validate_pane_id_valid() {
        assert!(validate_pane_id("0").is_ok());
        assert!(validate_pane_id("123").is_ok());
    }

    #[test]
    fn test_validate_pane_id_invalid() {
        assert!(validate_pane_id("").is_err());
        assert!(validate_pane_id("abc").is_err());
        assert!(validate_pane_id("1;rm -rf").is_err());
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
