//! Session detection for Codex CLI.

use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct CodexSession {
    id: Option<String>,
    model: Option<String>,
    #[serde(rename = "projectPath")]
    project_path: Option<String>,
}

/// Detects Codex sessions in the given config directory.
///
/// Codex stores sessions in ~/.codex/sessions/ as JSON files.
///
/// # Errors
/// Returns an error if directory reading fails.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let sessions_dir = config_dir.join("sessions");
    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&sessions_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<CodexSession>(&contents) {
                    let id = data.id.unwrap_or_else(|| {
                        path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default()
                    });
                    sessions.push(AgentSession {
                        id: SessionId::new_unchecked(id),
                        agent_type: AgentType::Codex,
                        model: data.model,
                        session_name: None,
                        working_dir: data.project_path.map(std::path::PathBuf::from),
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
    fn test_detect_sessions_finds_codex_sessions() {
        let dir = tempdir().unwrap();
        let codex_dir = dir.path().join(".codex");
        let sessions_dir = codex_dir.join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(
            sessions_dir.join("abc123.json"),
            r#"{"id": "abc123", "model": "gpt-4o", "projectPath": "/home/user/project"}"#,
        )
        .unwrap();

        let sessions = detect_sessions(&codex_dir).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "abc123");
        assert_eq!(sessions[0].model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn test_detect_sessions_empty_when_no_sessions_dir() {
        let dir = tempdir().unwrap();
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir).unwrap();

        let sessions = detect_sessions(&codex_dir).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_detect_sessions_uses_filename_as_fallback_id() {
        let dir = tempdir().unwrap();
        let codex_dir = dir.path().join(".codex");
        let sessions_dir = codex_dir.join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(
            sessions_dir.join("my-session.json"),
            r#"{"model": "gpt-4o"}"#,
        )
        .unwrap();

        let sessions = detect_sessions(&codex_dir).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "my-session");
    }

    #[test]
    fn test_detect_sessions_skips_malformed_json() {
        let dir = tempdir().unwrap();
        let codex_dir = dir.path().join(".codex");
        let sessions_dir = codex_dir.join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(sessions_dir.join("bad.json"), "not valid json {{{").unwrap();

        let sessions = detect_sessions(&codex_dir).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_detect_sessions_ignores_non_json_files() {
        let dir = tempdir().unwrap();
        let codex_dir = dir.path().join(".codex");
        let sessions_dir = codex_dir.join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(sessions_dir.join("readme.txt"), "text file").unwrap();

        let sessions = detect_sessions(&codex_dir).unwrap();
        assert!(sessions.is_empty());
    }
}
