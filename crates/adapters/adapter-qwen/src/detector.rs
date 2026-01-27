//! Session detection for Qwen CLI.

use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct QwenLogEntry {
    session_id: Option<String>,
    model: Option<String>,
}

/// Detects Qwen sessions in the given config directory.
///
/// Qwen stores logs in ~/.qwen/logs/openai/*.json
///
/// # Errors
/// Returns an error if directory reading fails.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let logs_dir = config_dir.join("logs").join("openai");
    if !logs_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for entry in std::fs::read_dir(&logs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<QwenLogEntry>(&contents) {
                    let id = data.session_id.unwrap_or_else(|| {
                        path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default()
                    });
                    if seen_ids.insert(id.clone()) {
                        sessions.push(AgentSession {
                            id: SessionId::new_unchecked(id),
                            agent_type: AgentType::Qwen,
                            model: data.model.or_else(|| Some("qwen-coder".to_string())),
                            session_name: None,
                            working_dir: None,
                            pane_id: None,
                            pid: None,
                            started_at: None,
                            status: SessionStatus::Idle,
                        });
                    }
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
    fn test_detect_sessions_finds_qwen_sessions() {
        let dir = tempdir().unwrap();
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir).unwrap();
        fs::write(
            logs_dir.join("log1.json"),
            r#"{"session_id": "qwen-123", "model": "qwen2.5-coder"}"#,
        )
        .unwrap();

        let sessions = detect_sessions(&qwen_dir).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "qwen-123");
    }

    #[test]
    fn test_detect_sessions_empty_when_no_logs() {
        let dir = tempdir().unwrap();
        let qwen_dir = dir.path().join(".qwen");
        fs::create_dir_all(&qwen_dir).unwrap();

        let sessions = detect_sessions(&qwen_dir).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_detect_sessions_nonexistent_returns_empty() {
        let dir = tempdir().unwrap();
        let nonexistent = dir.path().join("does-not-exist");

        let sessions = detect_sessions(&nonexistent).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_detect_sessions_ignores_non_target_files() {
        let dir = tempdir().unwrap();
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir).unwrap();

        // Non-json files should be ignored
        fs::write(logs_dir.join("log.txt"), "not json").unwrap();
        fs::write(logs_dir.join("data.yaml"), "key: value").unwrap();

        // Subdirectory should be ignored
        let subdir = logs_dir.join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        let sessions = detect_sessions(&qwen_dir).unwrap();
        assert!(sessions.is_empty());
    }
}
