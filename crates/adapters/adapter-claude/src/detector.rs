//! Session detection for Claude Code.

use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct SessionFile {
    id: Option<String>,
    model: Option<String>,
    #[serde(rename = "sessionName")]
    session_name: Option<String>,
}

/// Detects Claude sessions in the given config directory.
///
/// # Errors
/// Returns an error if directory reading fails.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let projects_dir = config_dir.join("projects");
    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&projects_dir)? {
        let entry = entry?;
        let project_path = entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let session_file = project_path.join("session.json");
        if session_file.exists() {
            if let Ok(contents) = std::fs::read_to_string(&session_file) {
                if let Ok(data) = serde_json::from_str::<SessionFile>(&contents) {
                    let id = data.id.unwrap_or_else(|| {
                        entry.file_name().to_string_lossy().to_string()
                    });
                    sessions.push(AgentSession {
                        id: SessionId::new(id),
                        agent_type: AgentType::Claude,
                        model: data.model,
                        session_name: data.session_name,
                        working_dir: Some(project_path),
                        pane_id: None,
                        pid: None,
                        started_at: None,
                        status: SessionStatus::Idle,
                    });
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
    fn test_detect_sessions_finds_projects() {
        let dir = tempdir().unwrap();
        let claude_dir = dir.path().join(".claude");
        let project_dir = claude_dir.join("projects").join("test-project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(
            project_dir.join("session.json"),
            r#"{"id": "session-123", "model": "claude-sonnet-4-20250514"}"#,
        ).unwrap();

        let sessions = detect_sessions(&claude_dir).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.0, "session-123");
    }

    #[test]
    fn test_detect_sessions_empty_when_no_projects() {
        let dir = tempdir().unwrap();
        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        let sessions = detect_sessions(&claude_dir).unwrap();
        assert!(sessions.is_empty());
    }
}
