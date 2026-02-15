//! Session detection for Codex CLI.

use ragentop_core::{
    AgentSession, AgentType, Command, CommandStatus, Result, SessionId, SessionStatus,
};
use serde::Deserialize;
use std::path::Path;
use std::time::{Duration, SystemTime};

const ACTIVE_THRESHOLD: Duration = Duration::from_secs(300);

fn is_process_running(name: &str) -> bool {
    std::process::Command::new("pgrep")
        .args(["-f", name])
        .output()
        .is_ok_and(|o| o.status.success())
}

fn is_recently_modified(path: &Path, threshold: Duration) -> bool {
    path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < threshold)
}

#[derive(Deserialize)]
struct CodexSession {
    id: Option<String>,
    model: Option<String>,
    #[serde(rename = "projectPath")]
    project_path: Option<String>,
}

/// Detects Codex sessions in the given config directory.
///
/// # Errors
/// Returns an error if the filesystem cannot be read.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let sessions_dir = config_dir.join("sessions");
    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let process_active = is_process_running("codex");

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

                    let recently_modified = is_recently_modified(&path, ACTIVE_THRESHOLD);
                    let status = if process_active || recently_modified {
                        SessionStatus::Active
                    } else {
                        SessionStatus::Idle
                    };

                    let started_at = path.metadata().ok().and_then(|m| m.modified().ok());

                    sessions.push(
                        AgentSession::new(SessionId::new_unchecked(id), AgentType::Codex, status)
                            .with_model(data.model)
                            .with_working_dir(data.project_path.map(std::path::PathBuf::from))
                            .with_started_at(started_at),
                    );
                }
            }
        }
    }
    Ok(sessions)
}

/// A single entry in Codex's history.jsonl file.
#[derive(Deserialize)]
struct HistoryEntry {
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    args: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    timestamp: Option<f64>,
}

/// Parses command history from Codex's history.jsonl file.
///
/// # Errors
/// Returns an error if the history file cannot be read.
pub fn parse_history(config_dir: &Path, limit: usize) -> Result<Vec<Command>> {
    let history_file = config_dir.join("history.jsonl");
    if !history_file.exists() {
        return Ok(vec![]);
    }

    let contents = std::fs::read_to_string(&history_file)?;
    let mut commands: Vec<Command> = Vec::new();

    for line in contents.lines().rev() {
        if commands.len() >= limit {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<HistoryEntry>(line) {
            let tool = entry
                .tool
                .or(entry.command)
                .unwrap_or_else(|| "unknown".to_string());
            let args = entry.args.unwrap_or_default();
            let status = match entry.status.as_deref() {
                Some("failed" | "error") => CommandStatus::Failed,
                Some("running") => CommandStatus::Running,
                _ => CommandStatus::Success,
            };
            let timestamp = entry.timestamp.map_or_else(SystemTime::now, |ts| {
                SystemTime::UNIX_EPOCH + Duration::from_secs_f64(ts)
            });

            commands.push(Command {
                timestamp,
                tool,
                args,
                status,
                result_summary: entry.result,
            });
        }
    }
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_sessions_finds_codex_sessions(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let codex_dir = dir.path().join(".codex");
        let sessions_dir = codex_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)?;
        fs::write(
            sessions_dir.join("abc123.json"),
            r#"{"id": "abc123", "model": "gpt-4o", "projectPath": "/home/user/project"}"#,
        )?;

        let sessions = detect_sessions(&codex_dir)?;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "abc123");
        assert_eq!(sessions[0].model.as_deref(), Some("gpt-4o"));
        Ok(())
    }

    #[test]
    fn test_detect_sessions_empty_when_no_sessions_dir(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir)?;
        let sessions = detect_sessions(&codex_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_detect_sessions_uses_filename_as_fallback_id(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let codex_dir = dir.path().join(".codex");
        let sessions_dir = codex_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)?;
        fs::write(
            sessions_dir.join("my-session.json"),
            r#"{"model": "gpt-4o"}"#,
        )?;
        let sessions = detect_sessions(&codex_dir)?;
        assert_eq!(sessions[0].id.as_str(), "my-session");
        Ok(())
    }

    #[test]
    fn test_detect_sessions_skips_malformed_json(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let codex_dir = dir.path().join(".codex");
        let sessions_dir = codex_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)?;
        fs::write(sessions_dir.join("bad.json"), "not valid json {{{")?;
        let sessions = detect_sessions(&codex_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_detect_sessions_ignores_non_json_files(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let codex_dir = dir.path().join(".codex");
        let sessions_dir = codex_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)?;
        fs::write(sessions_dir.join("readme.txt"), "text file")?;
        let sessions = detect_sessions(&codex_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_history_returns_commands() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempdir()?;
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir)?;
        fs::write(
            codex_dir.join("history.jsonl"),
            r#"{"tool": "bash", "args": "ls -la", "status": "success", "timestamp": 1706000000.0}
{"tool": "write", "args": "file.txt", "status": "success", "timestamp": 1706000001.0}
"#,
        )?;

        let cmds = parse_history(&codex_dir, 10)?;
        assert_eq!(cmds.len(), 2);
        // Reversed order (most recent first)
        assert_eq!(cmds[0].tool, "write");
        assert_eq!(cmds[1].tool, "bash");
        Ok(())
    }

    #[test]
    fn test_parse_history_respects_limit() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir)?;
        fs::write(
            codex_dir.join("history.jsonl"),
            r#"{"tool": "a"}
{"tool": "b"}
{"tool": "c"}
"#,
        )?;

        let cmds = parse_history(&codex_dir, 2)?;
        assert_eq!(cmds.len(), 2);
        Ok(())
    }

    #[test]
    fn test_parse_history_missing_file() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let cmds = parse_history(dir.path(), 10)?;
        assert!(cmds.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_history_skips_blank_lines() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempdir()?;
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir)?;
        fs::write(
            codex_dir.join("history.jsonl"),
            "\n{\"tool\": \"bash\"}\n\n",
        )?;

        let cmds = parse_history(&codex_dir, 10)?;
        assert_eq!(cmds.len(), 1);
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
            Path::new("/nonexistent/file"),
            Duration::from_secs(60)
        ));
    }
}
