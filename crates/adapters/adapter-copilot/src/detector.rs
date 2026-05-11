//! Session detection for GitHub Copilot CLI.

use adapter_common::{is_process_running, is_recently_modified, ACTIVE_THRESHOLD};
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
/// # Errors
/// Returns an error if the filesystem cannot be read.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let config_file = config_dir.join("config.json");
    if !config_file.exists() {
        return Ok(vec![]);
    }

    let process_active = is_process_running("copilot");
    let recently_modified = is_recently_modified(&config_file, ACTIVE_THRESHOLD);

    let contents = std::fs::read_to_string(&config_file)?;
    let config = match serde_json::from_str::<CopilotConfig>(&contents) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                path = %config_file.display(),
                error = %e,
                "skipping malformed copilot config.json"
            );
            return Ok(vec![]);
        }
    };
    let Some(session_id) = config.session_id else {
        return Ok(vec![]);
    };

    let status = if process_active || recently_modified {
        SessionStatus::Active
    } else {
        SessionStatus::Idle
    };
    let started_at = config_file.metadata().ok().and_then(|m| m.modified().ok());

    let mut session = AgentSession::new(
        SessionId::new_unchecked(session_id),
        AgentType::Copilot,
        status,
    );
    session.model = Some("gpt-4".to_string());
    session.started_at = started_at;
    Ok(vec![session])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_sessions_finds_copilot_session(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let copilot_dir = dir.path().join(".copilot");
        fs::create_dir_all(&copilot_dir)?;
        fs::write(
            copilot_dir.join("config.json"),
            r#"{"sessionId": "copilot-sess-123"}"#,
        )?;

        let sessions = detect_sessions(&copilot_dir)?;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "copilot-sess-123");
        Ok(())
    }

    #[test]
    fn test_detect_sessions_empty_when_no_config(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let copilot_dir = dir.path().join(".copilot");
        fs::create_dir_all(&copilot_dir)?;
        let sessions = detect_sessions(&copilot_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_detect_sessions_nonexistent_returns_empty(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let nonexistent = dir.path().join("does-not-exist");
        let sessions = detect_sessions(&nonexistent)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_detect_sessions_ignores_non_target_files(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let copilot_dir = dir.path().join(".copilot");
        fs::create_dir_all(&copilot_dir)?;
        fs::write(copilot_dir.join("settings.json"), r#"{"theme": "dark"}"#)?;
        let sessions = detect_sessions(&copilot_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }
}
