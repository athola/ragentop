//! Session detection for GitHub Copilot CLI.

use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct CopilotConfig {
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

/// Detects Copilot sessions in the given config directory.
///
/// Copilot stores config in ~/.copilot/config.json
///
/// # Errors
/// Returns an error if directory reading fails.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let config_file = config_dir.join("config.json");
    if !config_file.exists() {
        return Ok(vec![]);
    }

    let contents = std::fs::read_to_string(&config_file)?;
    if let Ok(config) = serde_json::from_str::<CopilotConfig>(&contents) {
        if let Some(session_id) = config.session_id {
            return Ok(vec![AgentSession {
                id: SessionId::new_unchecked(session_id),
                agent_type: AgentType::Copilot,
                model: Some("gpt-4".to_string()),
                session_name: None,
                working_dir: None,
                pane_id: None,
                pid: None,
                started_at: None,
                status: SessionStatus::Idle,
            }]);
        }
    }
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_sessions_finds_copilot_session() {
        let dir = tempdir().unwrap();
        let copilot_dir = dir.path().join(".copilot");
        fs::create_dir_all(&copilot_dir).unwrap();
        fs::write(
            copilot_dir.join("config.json"),
            r#"{"sessionId": "copilot-sess-123"}"#,
        )
        .unwrap();

        let sessions = detect_sessions(&copilot_dir).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "copilot-sess-123");
    }

    #[test]
    fn test_detect_sessions_empty_when_no_config() {
        let dir = tempdir().unwrap();
        let copilot_dir = dir.path().join(".copilot");
        fs::create_dir_all(&copilot_dir).unwrap();

        let sessions = detect_sessions(&copilot_dir).unwrap();
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
        let copilot_dir = dir.path().join(".copilot");
        fs::create_dir_all(&copilot_dir).unwrap();

        // Write non-config files that should be ignored
        fs::write(copilot_dir.join("settings.json"), r#"{"theme": "dark"}"#).unwrap();
        fs::write(copilot_dir.join("random.txt"), "not json").unwrap();

        let sessions = detect_sessions(&copilot_dir).unwrap();
        assert!(sessions.is_empty());
    }
}
