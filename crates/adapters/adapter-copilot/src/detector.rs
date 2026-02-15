//! Session detection for GitHub Copilot CLI.

use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use serde::Deserialize;
use std::path::Path;
use std::time::{Duration, SystemTime};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

const ACTIVE_THRESHOLD: Duration = Duration::from_secs(300);

fn is_process_running(name: &str) -> bool {
    let s =
        System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    s.processes()
        .values()
        .any(|p| p.name().to_string_lossy().contains(name))
}

fn is_recently_modified(path: &Path, threshold: Duration) -> bool {
    path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < threshold)
}

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
    if let Ok(config) = serde_json::from_str::<CopilotConfig>(&contents) {
        if let Some(session_id) = config.session_id {
            let status = if process_active || recently_modified {
                SessionStatus::Active
            } else {
                SessionStatus::Idle
            };
            let started_at = config_file.metadata().ok().and_then(|m| m.modified().ok());

            return Ok(vec![AgentSession::new(
                SessionId::new_unchecked(session_id),
                AgentType::Copilot,
                status,
            )
            .with_model(Some("gpt-4".to_string()))
            .with_started_at(started_at)]);
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

    #[test]
    fn test_is_recently_modified_true() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file = dir.path().join("recent");
        fs::write(&file, "data")?;
        assert!(is_recently_modified(&file, Duration::from_secs(60)));
        Ok(())
    }

    #[test]
    fn test_is_recently_modified_nonexistent() {
        assert!(!is_recently_modified(
            Path::new("/nonexistent"),
            Duration::from_secs(60)
        ));
    }
}
