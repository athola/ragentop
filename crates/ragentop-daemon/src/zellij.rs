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
        // rename. The decision logic (when to stop, when to error) is
        // extracted into `focus_pane_via_cycling` so it can be tested with
        // mock closures; this body only wires real zellij commands.
        let initial_panes = self.list_panes()?;
        focus_pane_via_cycling(
            pane_id,
            &initial_panes,
            || self.list_panes(),
            run_zellij_focus_next,
        )?;
        run_zellij_rename(pane_id, name)
    }
}

/// Cycles focus until `target_id` becomes the active pane.
///
/// Pure decision logic — takes `list_panes` and `focus_next` as injected
/// callables so tests can simulate a multi-pane zellij without spawning
/// the real binary. The four error paths (target missing, focus stalled,
/// not reached after `panes.len()` cycles, and the original "already
/// focused" fast-path) are all exercised by the unit tests below.
///
/// # Errors
/// - `Error::Adapter` if `target_id` is not in `initial_panes`
/// - `Error::Adapter` if `focus_next` fails to advance focus (cycle stalls)
/// - `Error::Adapter` if `target_id` is not reached after `initial_panes.len()` cycles
/// - Propagates errors from `list_panes` and `focus_next`
fn focus_pane_via_cycling<L, F>(
    target_id: &str,
    initial_panes: &[PaneInfo],
    mut list_panes: L,
    mut focus_next: F,
) -> Result<()>
where
    L: FnMut() -> Result<Vec<PaneInfo>>,
    F: FnMut() -> Result<()>,
{
    let target_exists = initial_panes.iter().any(|p| p.id == target_id);
    if !target_exists {
        return Err(Error::Adapter(format!(
            "zellij: pane {target_id} not found ({} panes available)",
            initial_panes.len()
        )));
    }

    let max_cycles = initial_panes.len();
    let mut last_focused: Option<String> = initial_panes
        .iter()
        .find(|p| p.active)
        .map(|p| p.id.clone());

    if last_focused.as_deref() != Some(target_id) {
        for _ in 0..max_cycles {
            focus_next()?;

            let now_panes = list_panes()?;
            let now_focused = now_panes.iter().find(|p| p.active).map(|p| p.id.clone());
            if now_focused.as_deref() == Some(target_id) {
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

    if last_focused.as_deref() != Some(target_id) {
        return Err(Error::Adapter(format!(
            "zellij: could not focus pane {target_id} after {max_cycles} cycle(s)"
        )));
    }

    Ok(())
}

/// Runs `zellij action focus-next-pane`. Shell-only; tested via integration.
fn run_zellij_focus_next() -> Result<()> {
    let output = Command::new("zellij")
        .args(["action", "focus-next-pane"])
        .output()
        .map_err(|e| Error::Adapter(format!("Failed to run zellij focus-next-pane: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Adapter(format!(
            "zellij focus-next-pane failed: {stderr}"
        )));
    }
    Ok(())
}

/// Runs `zellij action rename-pane <name>` against the currently-focused pane.
/// Shell-only; tested via integration.
fn run_zellij_rename(pane_id: &str, name: &str) -> Result<()> {
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

    // --- focus_pane_via_cycling tests ----------------------------------
    //
    // These tests exercise the extracted decision logic that was previously
    // inlined in `rename_pane`. Without the extraction the only path to
    // covering the cycle/stall/timeout branches was a full zellij integration
    // run — i.e. they were untested.

    use std::cell::RefCell;

    fn pane(id: &str, active: bool) -> PaneInfo {
        PaneInfo {
            id: id.to_owned(),
            title: format!("pane-{id}"),
            active,
        }
    }

    /// Mock state for a virtual zellij session. `focus_next` advances the
    /// active pane round-robin; `current` returns the current snapshot.
    struct FakeZellij {
        panes: RefCell<Vec<PaneInfo>>,
        focus_calls: RefCell<usize>,
    }

    impl FakeZellij {
        fn new(panes: Vec<PaneInfo>) -> Self {
            Self {
                panes: RefCell::new(panes),
                focus_calls: RefCell::new(0),
            }
        }

        fn snapshot(&self) -> Vec<PaneInfo> {
            self.panes.borrow().clone()
        }

        fn focus_next(&self) {
            *self.focus_calls.borrow_mut() += 1;
            let mut panes = self.panes.borrow_mut();
            let active_idx = panes.iter().position(|p| p.active);
            if let Some(i) = active_idx {
                panes[i].active = false;
                let next = (i + 1) % panes.len();
                panes[next].active = true;
            } else if !panes.is_empty() {
                panes[0].active = true;
            }
        }
    }

    /// Boxed result alias for tests that propagate via `?`.
    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn focus_pane_returns_err_when_target_not_in_panes() -> TestResult {
        let zellij = FakeZellij::new(vec![pane("1", true), pane("2", false)]);
        let initial = zellij.snapshot();

        let result = focus_pane_via_cycling(
            "999",
            &initial,
            || Ok(zellij.snapshot()),
            || {
                zellij.focus_next();
                Ok(())
            },
        );
        let Err(err) = result else {
            return Err("missing target should error".into());
        };
        let msg = format!("{err}");
        assert!(msg.contains("pane 999 not found"), "got: {msg}");
        assert_eq!(
            *zellij.focus_calls.borrow(),
            0,
            "should not cycle when target missing"
        );
        Ok(())
    }

    #[test]
    fn focus_pane_succeeds_immediately_when_already_focused() -> TestResult {
        let zellij = FakeZellij::new(vec![pane("1", true), pane("2", false)]);
        let initial = zellij.snapshot();

        focus_pane_via_cycling(
            "1",
            &initial,
            || Ok(zellij.snapshot()),
            || {
                zellij.focus_next();
                Ok(())
            },
        )?;
        assert_eq!(
            *zellij.focus_calls.borrow(),
            0,
            "fast-path: no focus_next calls when target already active"
        );
        Ok(())
    }

    #[test]
    fn focus_pane_cycles_until_target_active() -> TestResult {
        // 3 panes: 1 (active), 2, 3. Target = 3, reached after 2 cycles.
        let zellij = FakeZellij::new(vec![pane("1", true), pane("2", false), pane("3", false)]);
        let initial = zellij.snapshot();

        focus_pane_via_cycling(
            "3",
            &initial,
            || Ok(zellij.snapshot()),
            || {
                zellij.focus_next();
                Ok(())
            },
        )?;
        assert_eq!(*zellij.focus_calls.borrow(), 2);
        Ok(())
    }

    #[test]
    fn focus_pane_errors_when_cycle_stalls() -> TestResult {
        // focus_next is a no-op: pane 1 stays active forever. We're targeting
        // pane 2, which exists. After the first cycle, now_focused == "1",
        // last_focused == "1", so we bail with "did not advance".
        let zellij = FakeZellij::new(vec![pane("1", true), pane("2", false)]);
        let initial = zellij.snapshot();
        let no_op_focus = || -> Result<()> { Ok(()) };

        let result = focus_pane_via_cycling("2", &initial, || Ok(zellij.snapshot()), no_op_focus);
        let Err(err) = result else {
            return Err("stall should be detected".into());
        };
        assert!(format!("{err}").contains("did not advance"), "got: {err}");
        Ok(())
    }

    #[test]
    fn focus_pane_propagates_focus_next_error() -> TestResult {
        let zellij = FakeZellij::new(vec![pane("1", true), pane("2", false)]);
        let initial = zellij.snapshot();
        let failing_focus =
            || -> Result<()> { Err(Error::Adapter("simulated zellij crash".to_owned())) };

        let result = focus_pane_via_cycling("2", &initial, || Ok(zellij.snapshot()), failing_focus);
        let Err(err) = result else {
            return Err("focus_next error should bubble up".into());
        };
        assert!(format!("{err}").contains("simulated zellij crash"));
        Ok(())
    }

    #[test]
    fn focus_pane_propagates_list_panes_error() -> TestResult {
        let zellij = FakeZellij::new(vec![pane("1", true), pane("2", false)]);
        let initial = zellij.snapshot();
        let failing_list =
            || -> Result<Vec<PaneInfo>> { Err(Error::Adapter("snapshot failed".to_owned())) };

        let result = focus_pane_via_cycling("2", &initial, failing_list, || {
            zellij.focus_next();
            Ok(())
        });
        let Err(err) = result else {
            return Err("list error should bubble up".into());
        };
        assert!(format!("{err}").contains("snapshot failed"));
        Ok(())
    }
}
