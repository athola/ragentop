//! Session detection for Google Gemini CLI.

use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use std::path::Path;

/// Detects Gemini sessions in the given config directory.
///
/// Gemini CLI stores sessions in ~/.gemini/tmp/<hash>/ directories
/// with `shell_history` files indicating activity.
///
/// # Errors
/// Returns an error if directory reading fails.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let tmp_dir = config_dir.join("tmp");
    if !tmp_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&tmp_dir)? {
        let entry = entry?;
        let session_dir = entry.path();
        if !session_dir.is_dir() {
            continue;
        }

        // Check for shell_history file as indicator of session
        let history_file = session_dir.join("shell_history");
        if history_file.exists() {
            let id = entry.file_name().to_string_lossy().to_string();
            sessions.push(AgentSession {
                id: SessionId::new_unchecked(id),
                agent_type: AgentType::Gemini,
                model: Some("gemini-2.0-flash".to_string()),
                session_name: None,
                working_dir: Some(session_dir),
                pane_id: None,
                pid: None,
                started_at: None,
                status: SessionStatus::Idle,
            });
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
    fn test_detect_sessions_finds_gemini_sessions() {
        let dir = tempdir().unwrap();
        let gemini_dir = dir.path().join(".gemini");
        let session_dir = gemini_dir.join("tmp").join("abc123hash");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(session_dir.join("shell_history"), "ls\ncat file.txt\n").unwrap();

        let sessions = detect_sessions(&gemini_dir).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "abc123hash");
        assert_eq!(sessions[0].agent_type, AgentType::Gemini);
    }

    #[test]
    fn test_detect_sessions_empty_when_no_tmp_dir() {
        let dir = tempdir().unwrap();
        let gemini_dir = dir.path().join(".gemini");
        fs::create_dir_all(&gemini_dir).unwrap();

        let sessions = detect_sessions(&gemini_dir).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_detect_sessions_ignores_dirs_without_history() {
        let dir = tempdir().unwrap();
        let gemini_dir = dir.path().join(".gemini");
        let session_dir = gemini_dir.join("tmp").join("nohist");
        fs::create_dir_all(&session_dir).unwrap();

        let sessions = detect_sessions(&gemini_dir).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_detect_sessions_ignores_files_in_tmp() {
        let dir = tempdir().unwrap();
        let gemini_dir = dir.path().join(".gemini");
        let tmp_dir = gemini_dir.join("tmp");
        fs::create_dir_all(&tmp_dir).unwrap();
        fs::write(tmp_dir.join("some_file.txt"), "not a dir").unwrap();

        let sessions = detect_sessions(&gemini_dir).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_detect_sessions_nonexistent_returns_empty() {
        let dir = tempdir().unwrap();
        let nonexistent = dir.path().join("does-not-exist");
        let sessions = detect_sessions(&nonexistent).unwrap();
        assert!(sessions.is_empty());
    }
}
