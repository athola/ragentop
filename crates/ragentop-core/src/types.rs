//! Core domain types - pure data structures.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Supported AI coding agent types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
    Gemini,
    Copilot,
    Qwen,
    Glm,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
            Self::Gemini => write!(f, "gemini"),
            Self::Copilot => write!(f, "copilot"),
            Self::Qwen => write!(f, "qwen"),
            Self::Glm => write!(f, "glm"),
        }
    }
}

/// Unique identifier for an agent session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Current status of an agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Idle,
    Paused,
}

/// Information about a detected agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: SessionId,
    pub agent_type: AgentType,
    pub model: Option<String>,
    pub session_name: Option<String>,
    pub working_dir: Option<PathBuf>,
    pub pane_id: Option<String>,
    pub pid: Option<u32>,
    pub started_at: Option<SystemTime>,
    pub status: SessionStatus,
}

/// Metrics for a session at a point in time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub token_count: u64,
    pub cost_usd: Option<f64>,
    pub cpu_percent: Option<f32>,
    pub duration: Option<Duration>,
    pub command_count: u64,
}

/// A command executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub timestamp: SystemTime,
    pub tool: String,
    pub args: String,
    pub status: CommandStatus,
    pub result_summary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Success,
    Failed,
    Running,
}

/// History depth for command queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryDepth {
    /// Tool calls only (level 1)
    ToolCallsOnly,
    /// Tool calls with abbreviated responses (level 2)
    #[default]
    WithResponses,
    /// Full conversation turns (level 3)
    FullConversation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Claude.to_string(), "claude");
        assert_eq!(AgentType::Codex.to_string(), "codex");
    }

    #[test]
    fn test_agent_type_serde() {
        let json = serde_json::to_string(&AgentType::Claude).unwrap();
        assert_eq!(json, "\"claude\"");
        let parsed: AgentType = serde_json::from_str("\"gemini\"").unwrap();
        assert_eq!(parsed, AgentType::Gemini);
    }

    #[test]
    fn test_session_id() {
        let id = SessionId::new("session-123");
        assert_eq!(id.0, "session-123");
    }

    #[test]
    fn test_history_depth_default() {
        assert_eq!(HistoryDepth::default(), HistoryDepth::WithResponses);
    }
}
