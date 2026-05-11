//! Session detection for Qwen CLI.

use adapter_common::{is_process_running, is_recently_modified, ACTIVE_THRESHOLD};
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
/// # Errors
/// Returns an error if the filesystem cannot be read.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let logs_dir = config_dir.join("logs").join("openai");
    if !logs_dir.exists() {
        return Ok(vec![]);
    }

    let process_active = is_process_running("qwen");
    let mut sessions = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for entry in std::fs::read_dir(&logs_dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(ext) = path.extension() else {
            continue;
        };
        if ext != "json" {
            continue;
        }
        let recently_modified = is_recently_modified(&path, ACTIVE_THRESHOLD);

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "skipping unreadable qwen log file"
                );
                continue;
            }
        };
        let data = match serde_json::from_str::<QwenLogEntry>(&contents) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "skipping malformed qwen log file"
                );
                continue;
            }
        };

        let id = data.session_id.unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        });
        if !seen_ids.insert(id.clone()) {
            continue;
        }
        let status = if process_active || recently_modified {
            SessionStatus::Active
        } else {
            SessionStatus::Idle
        };
        let started_at = path.metadata().ok().and_then(|m| m.modified().ok());

        sessions.push(
            AgentSession::new(SessionId::new_unchecked(id), AgentType::Qwen, status)
                .with_model(data.model.or_else(|| Some("qwen-coder".to_string())))
                .with_started_at(started_at),
        );
    }
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_sessions_finds_qwen_sessions(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir)?;
        fs::write(
            logs_dir.join("log1.json"),
            r#"{"session_id": "qwen-123", "model": "qwen2.5-coder"}"#,
        )?;

        let sessions = detect_sessions(&qwen_dir)?;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "qwen-123");
        Ok(())
    }

    #[test]
    fn test_detect_sessions_empty_when_no_logs(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let qwen_dir = dir.path().join(".qwen");
        fs::create_dir_all(&qwen_dir)?;
        let sessions = detect_sessions(&qwen_dir)?;
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
    fn test_detect_sessions_deduplicates_by_session_id(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir)?;
        fs::write(logs_dir.join("log1.json"), r#"{"session_id": "same-id"}"#)?;
        fs::write(logs_dir.join("log2.json"), r#"{"session_id": "same-id"}"#)?;

        let sessions = detect_sessions(&qwen_dir)?;
        assert_eq!(sessions.len(), 1);
        Ok(())
    }

    #[test]
    fn test_detect_sessions_ignores_non_target_files(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let qwen_dir = dir.path().join(".qwen");
        let logs_dir = qwen_dir.join("logs").join("openai");
        fs::create_dir_all(&logs_dir)?;
        fs::write(logs_dir.join("log.txt"), "not json")?;
        let sessions = detect_sessions(&qwen_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }
}
