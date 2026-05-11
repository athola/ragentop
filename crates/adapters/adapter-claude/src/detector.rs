//! Session detection for Claude Code.

use adapter_common::ACTIVE_THRESHOLD;
use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use std::path::Path;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

/// Get working directories of running Claude processes.
///
/// Uses sysinfo for cross-platform process detection (Linux, macOS, Windows).
fn get_active_claude_dirs() -> std::collections::HashSet<String> {
    let mut active_dirs = std::collections::HashSet::new();
    let s = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );
    for process in s.processes().values() {
        let name = process.name().to_string_lossy();
        if name.contains("claude") {
            if let Some(cwd) = process.cwd() {
                active_dirs.insert(cwd.to_string_lossy().to_string());
            }
        }
    }
    active_dirs
}

/// Converts a Claude project directory name back to a path.
///
/// # Security Assumptions
///
/// This function reconstructs paths from directory names in `~/.claude/projects/`,
/// which is the user's own filesystem. The reconstructed path is used for **display
/// purposes only** (showing which project a session belongs to) and is **not used
/// for file I/O operations** on the reconstructed path.
///
/// The `working_dir` field in `AgentSession` is informational metadata for the TUI
/// and does not authorize or enable subsequent file operations.
///
/// # Rules
/// - `--` is converted to `/.` (hidden directories)
/// - Finds longest existing parent path
fn project_dir_to_path(name: &str) -> String {
    // -- means dot (hidden dirs)
    let name = name.replace("--", "/.");
    let base = name.replacen('-', "/", 1);

    if let Some(rest) = base.strip_prefix("/home-") {
        if let Some(idx) = rest.find('-') {
            let user = &rest[..idx];
            let project = &rest[idx + 1..];
            let home = format!("/home/{user}");

            // Split project into parts, find longest existing prefix
            let parts: Vec<&str> = project.split('-').collect();
            for iter in (1..=parts.len()).rev() {
                let prefix = parts[..iter].join("-");
                let candidate = format!("{home}/{prefix}");
                if std::path::Path::new(&candidate).exists() {
                    // Found existing dir - remaining parts are subdirs
                    let suffix = parts[iter..].join("/");
                    return if suffix.is_empty() {
                        candidate
                    } else {
                        format!("{candidate}/{suffix}")
                    };
                }
            }
            return format!("{}/{}", home, project.replace('-', "/"));
        }
    }
    base.replace('-', "/")
}

/// Detects Claude sessions in the given config directory.
///
/// Claude Code stores sessions as `.jsonl` files in project directories.
/// Each project dir is named like `-home-alext-myproject` (path with slashes as dashes).
/// Session files are UUIDs like `08183e2d-3def-465c-a576-dc79a868c1f2.jsonl`.
///
/// # Errors
/// Returns an error if directory reading fails.
#[inline]
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let projects_dir = config_dir.join("projects");
    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    // Get directories with active Claude processes
    let active_dirs = get_active_claude_dirs();

    let mut sessions = Vec::new();
    for project_entry in std::fs::read_dir(&projects_dir)? {
        let project_entry = project_entry?;
        let project_path = project_entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let project_name = project_entry.file_name().to_string_lossy().to_string();
        let working_dir = project_dir_to_path(&project_name);

        // Find all .jsonl session files in this project
        if let Ok(entries) = std::fs::read_dir(&project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "jsonl") {
                    let session_id = path
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Use file metadata for timestamps and activity detection
                    let metadata = path.metadata().ok();
                    let started_at = metadata.as_ref().and_then(|meta| meta.modified().ok());

                    // Active if: running process OR modified within threshold
                    let is_process_active = active_dirs.contains(&working_dir);
                    let is_recently_modified =
                        adapter_common::is_recently_modified(&path, ACTIVE_THRESHOLD);

                    let status = if is_process_active || is_recently_modified {
                        SessionStatus::Active
                    } else {
                        SessionStatus::Idle
                    };

                    let mut session = AgentSession::new(
                        SessionId::new_unchecked(session_id),
                        AgentType::Claude,
                        status,
                    );
                    session.session_name = Some(project_name.clone());
                    session.started_at = started_at;
                    session.working_dir = Some(working_dir.clone().into());
                    sessions.push(session);
                }
            }
        }
    }
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_sessions_finds_jsonl_files(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let claude_dir = dir.path().join(".claude");
        let project_dir = claude_dir.join("projects").join("-home-user-myproject");
        fs::create_dir_all(&project_dir)?;
        fs::write(
            project_dir.join("abc123-def456.jsonl"),
            r#"{"type":"message"}"#,
        )?;

        let sessions = detect_sessions(&claude_dir)?;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "abc123-def456");
        assert_eq!(
            sessions[0].session_name.as_deref(),
            Some("-home-user-myproject")
        );
        Ok(())
    }

    #[test]
    fn test_detect_sessions_multiple_sessions_per_project(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let claude_dir = dir.path().join(".claude");
        let project_dir = claude_dir.join("projects").join("-home-user-project");
        fs::create_dir_all(&project_dir)?;
        fs::write(project_dir.join("session-1.jsonl"), "{}")?;
        fs::write(project_dir.join("session-2.jsonl"), "{}")?;

        let sessions = detect_sessions(&claude_dir)?;
        assert_eq!(sessions.len(), 2);
        Ok(())
    }

    #[test]
    fn test_detect_sessions_empty_when_no_projects(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir)?;

        let sessions = detect_sessions(&claude_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_project_dir_to_path() {
        assert_eq!(
            project_dir_to_path("-home-user-project"),
            "/home/user/project"
        );
        assert_eq!(
            project_dir_to_path("-home-alext-ragentop"),
            "/home/alext/ragentop"
        );
    }

    #[test]
    fn test_detect_sessions_ignores_non_jsonl_files(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let claude_dir = dir.path().join(".claude");
        let project_dir = claude_dir.join("projects").join("-home-user-project");
        fs::create_dir_all(&project_dir)?;
        fs::write(project_dir.join("readme.txt"), "not a session")?;
        fs::write(project_dir.join("config.json"), "{}")?;

        let sessions = detect_sessions(&claude_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_detect_sessions_handles_empty_jsonl(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let claude_dir = dir.path().join(".claude");
        let project_dir = claude_dir.join("projects").join("-home-user-project");
        fs::create_dir_all(&project_dir)?;
        fs::write(project_dir.join("empty-session.jsonl"), "")?;

        let sessions = detect_sessions(&claude_dir)?;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "empty-session");
        Ok(())
    }

    #[test]
    fn test_detect_sessions_nonexistent_dir_returns_empty(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let nonexistent = dir.path().join("does-not-exist");
        let sessions = detect_sessions(&nonexistent)?;
        assert!(sessions.is_empty());
        Ok(())
    }
}
