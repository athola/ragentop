//! Session detection for GLM (Claude via `ANTHROPIC_BASE_URL` proxy).

use ragentop_core::{AgentSession, AgentType, Result, SessionId, SessionStatus};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct GlmSession {
    id: Option<String>,
    model: Option<String>,
    #[serde(rename = "projectPath")]
    project_path: Option<String>,
}

/// Detects GLM sessions in the given config directory.
///
/// GLM wraps Claude and stores sessions similar to Claude in ~/.glm/projects/
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
                if let Ok(data) = serde_json::from_str::<GlmSession>(&contents) {
                    let id = data
                        .id
                        .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
                    sessions.push(AgentSession {
                        id: SessionId::new_unchecked(id),
                        agent_type: AgentType::Glm,
                        model: data.model.or_else(|| Some("glm-4".to_string())),
                        session_name: None,
                        working_dir: data
                            .project_path
                            .map(std::path::PathBuf::from)
                            .or(Some(project_path)),
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
    fn test_detect_sessions_finds_glm_sessions() {
        let dir = tempdir().unwrap();
        let glm_dir = dir.path().join(".glm");
        let project_dir = glm_dir.join("projects").join("my-project");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(
            project_dir.join("session.json"),
            r#"{"id": "glm-session-1", "model": "glm-4-plus"}"#,
        )
        .unwrap();

        let sessions = detect_sessions(&glm_dir).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id.as_str(), "glm-session-1");
        assert_eq!(sessions[0].model.as_deref(), Some("glm-4-plus"));
    }

    #[test]
    fn test_detect_sessions_empty_when_no_projects() {
        let dir = tempdir().unwrap();
        let glm_dir = dir.path().join(".glm");
        fs::create_dir_all(&glm_dir).unwrap();

        let sessions = detect_sessions(&glm_dir).unwrap();
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
        let glm_dir = dir.path().join(".glm");
        let projects_dir = glm_dir.join("projects");
        fs::create_dir_all(&projects_dir).unwrap();

        // Create a file instead of a directory - should be ignored
        fs::write(projects_dir.join("not-a-dir.txt"), "random content").unwrap();

        // Create a directory without session.json - should be ignored
        let empty_project = projects_dir.join("empty-project");
        fs::create_dir_all(&empty_project).unwrap();
        fs::write(empty_project.join("other.json"), r#"{"foo": "bar"}"#).unwrap();

        let sessions = detect_sessions(&glm_dir).unwrap();
        assert!(sessions.is_empty());
    }
}
