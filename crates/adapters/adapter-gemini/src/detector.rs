//! Session detection for Google Gemini CLI.

use adapter_common::{is_process_running, is_recently_modified, ACTIVE_THRESHOLD};
use ragentop_core::{
    AgentSession, AgentType, Command, CommandStatus, Result, SessionId, SessionStatus,
};
use std::path::Path;
use std::time::SystemTime;

/// Detects Gemini sessions in the given config directory.
///
/// # Errors
/// Returns an error if the filesystem cannot be read.
pub fn detect_sessions(config_dir: &Path) -> Result<Vec<AgentSession>> {
    let tmp_dir = config_dir.join("tmp");
    if !tmp_dir.exists() {
        return Ok(vec![]);
    }

    let process_active = is_process_running("gemini");

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&tmp_dir)? {
        let entry = entry?;
        let session_dir = entry.path();
        if !session_dir.is_dir() {
            continue;
        }

        let history_file = session_dir.join("shell_history");
        if history_file.exists() {
            let id = entry.file_name().to_string_lossy().to_string();
            let recently_modified = is_recently_modified(&history_file, ACTIVE_THRESHOLD);
            let status = if process_active || recently_modified {
                SessionStatus::Active
            } else {
                SessionStatus::Idle
            };
            let started_at = history_file.metadata().ok().and_then(|m| m.modified().ok());

            sessions.push(
                AgentSession::new(SessionId::new_unchecked(id), AgentType::Gemini, status)
                    .with_model(Some("gemini-2.0-flash".to_string()))
                    .with_working_dir(Some(session_dir))
                    .with_started_at(started_at),
            );
        }
    }
    Ok(sessions)
}

/// Parses command history from a Gemini `shell_history` file.
/// Format: plain text, one command per line.
///
/// # Errors
/// Returns an error if the history file cannot be read.
pub fn parse_history(config_dir: &Path, session_id: &str, limit: usize) -> Result<Vec<Command>> {
    let history_file = config_dir
        .join("tmp")
        .join(session_id)
        .join("shell_history");
    if !history_file.exists() {
        return Ok(vec![]);
    }

    let contents = std::fs::read_to_string(&history_file)?;
    let commands: Vec<Command> = contents
        .lines()
        .rev()
        .filter(|l| !l.trim().is_empty())
        .take(limit)
        .map(|line| Command {
            timestamp: SystemTime::now(),
            tool: "shell".to_string(),
            args: line.trim().to_string(),
            status: CommandStatus::Success,
            result_summary: None,
        })
        .collect();

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_sessions_finds_gemini_sessions(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let gemini_dir = dir.path().join(".gemini");
        let session_dir = gemini_dir.join("tmp").join("abc123hash");
        fs::create_dir_all(&session_dir)?;
        fs::write(session_dir.join("shell_history"), "ls\ncat file.txt\n")?;

        let sessions = detect_sessions(&gemini_dir)?;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "abc123hash");
        assert_eq!(sessions[0].agent_type, AgentType::Gemini);
        Ok(())
    }

    #[test]
    fn test_detect_sessions_empty_when_no_tmp_dir(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let gemini_dir = dir.path().join(".gemini");
        fs::create_dir_all(&gemini_dir)?;
        let sessions = detect_sessions(&gemini_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_detect_sessions_ignores_dirs_without_history(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let gemini_dir = dir.path().join(".gemini");
        let session_dir = gemini_dir.join("tmp").join("nohist");
        fs::create_dir_all(&session_dir)?;
        let sessions = detect_sessions(&gemini_dir)?;
        assert!(sessions.is_empty());
        Ok(())
    }

    #[test]
    fn test_detect_sessions_ignores_files_in_tmp(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let gemini_dir = dir.path().join(".gemini");
        let tmp_dir = gemini_dir.join("tmp");
        fs::create_dir_all(&tmp_dir)?;
        fs::write(tmp_dir.join("some_file.txt"), "not a dir")?;
        let sessions = detect_sessions(&gemini_dir)?;
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
    fn test_parse_history_returns_commands() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempdir()?;
        let gemini_dir = dir.path().join(".gemini");
        let session_dir = gemini_dir.join("tmp").join("sess1");
        fs::create_dir_all(&session_dir)?;
        fs::write(
            session_dir.join("shell_history"),
            "ls -la\ncat foo.txt\npwd\n",
        )?;

        let cmds = parse_history(&gemini_dir, "sess1", 10)?;
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].args, "pwd");
        assert_eq!(cmds[2].args, "ls -la");
        Ok(())
    }

    #[test]
    fn test_parse_history_respects_limit() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let gemini_dir = dir.path().join(".gemini");
        let session_dir = gemini_dir.join("tmp").join("sess1");
        fs::create_dir_all(&session_dir)?;
        fs::write(session_dir.join("shell_history"), "a\nb\nc\nd\n")?;

        let cmds = parse_history(&gemini_dir, "sess1", 2)?;
        assert_eq!(cmds.len(), 2);
        Ok(())
    }

    #[test]
    fn test_parse_history_missing_file() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let cmds = parse_history(dir.path(), "nonexistent", 10)?;
        assert!(cmds.is_empty());
        Ok(())
    }
}
