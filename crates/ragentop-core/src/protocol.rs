//! Protocol types for daemon-client communication.
//!
//! These types define the request/response protocol used between
//! the daemon and its clients (TUI, web UI, CLI).

use crate::{AgentSession, Command, HistoryDepth, SessionId, SessionMetrics};
use serde::{Deserialize, Serialize};

/// Request from client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// List all active agent sessions.
    ListSessions,
    /// Get metrics for a specific session.
    GetMetrics {
        /// The session to get metrics for.
        session_id: SessionId,
    },
    /// Get command history for a session.
    GetHistory {
        /// The session to get history for.
        session_id: SessionId,
        /// How much detail to include in the history.
        depth: HistoryDepth,
        /// Maximum number of commands to return.
        limit: usize,
    },
    /// Subscribe to real-time session updates.
    Subscribe,
}

/// Response from daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// List of active sessions.
    Sessions {
        /// The active sessions.
        sessions: Vec<AgentSession>,
    },
    /// Metrics for a session.
    Metrics {
        /// The session ID.
        session_id: SessionId,
        /// Current metrics.
        metrics: SessionMetrics,
    },
    /// Command history for a session.
    History {
        /// The session ID.
        session_id: SessionId,
        /// The commands in chronological order.
        commands: Vec<Command>,
    },
    /// Subscription confirmed.
    Subscribed,
    /// Error response.
    Error {
        /// Error message.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentType, CommandStatus, SessionStatus};
    use std::time::SystemTime;

    fn make_session(id: &str) -> AgentSession {
        AgentSession {
            id: SessionId::new_unchecked(id),
            agent_type: AgentType::Claude,
            model: Some("opus-4".to_string()),
            session_name: Some("test-project".to_string()),
            working_dir: Some("/home/user/project".into()),
            pane_id: Some("%1".to_string()),
            pid: Some(12345),
            started_at: Some(SystemTime::now()),
            status: SessionStatus::Active,
            last_event_at: None,
            telemetry_status: crate::TelemetryStatus::default(),
        }
    }

    fn make_command(tool: &str) -> Command {
        Command {
            timestamp: SystemTime::UNIX_EPOCH,
            tool: tool.to_string(),
            args: "--flag".to_string(),
            status: CommandStatus::Success,
            result_summary: Some("ok".to_string()),
        }
    }

    #[test]
    fn test_request_list_sessions_serde() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::ListSessions;
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("list_sessions"));
        let parsed: Request = serde_json::from_str(&json)?;
        assert!(matches!(parsed, Request::ListSessions));
        Ok(())
    }

    #[test]
    fn test_request_get_metrics_serde() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::GetMetrics {
            session_id: SessionId::new_unchecked("abc123"),
        };
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("get_metrics"));
        assert!(json.contains("abc123"));

        let parsed: Request = serde_json::from_str(&json)?;
        match parsed {
            Request::GetMetrics { session_id } => assert_eq!(session_id.as_str(), "abc123"),
            _ => return Err("expected GetMetrics".into()),
        }
        Ok(())
    }

    #[test]
    fn test_request_get_history_serde() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::GetHistory {
            session_id: SessionId::new_unchecked("sess-1"),
            depth: HistoryDepth::ToolCallsOnly,
            limit: 50,
        };
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("get_history"));
        assert!(json.contains("tool_calls_only"));
        assert!(json.contains("\"limit\":50"));

        let parsed: Request = serde_json::from_str(&json)?;
        match parsed {
            Request::GetHistory {
                session_id,
                depth,
                limit,
            } => {
                assert_eq!(session_id.as_str(), "sess-1");
                assert_eq!(depth, HistoryDepth::ToolCallsOnly);
                assert_eq!(limit, 50);
            }
            _ => return Err("expected GetHistory".into()),
        }
        Ok(())
    }

    #[test]
    fn test_response_metrics_serde() -> Result<(), Box<dyn std::error::Error>> {
        let resp = Response::Metrics {
            session_id: SessionId::new_unchecked("sess-1"),
            metrics: SessionMetrics {
                token_count: 5000,
                cost_usd: Some(0.15),
                cpu_percent: Some(25.5),
                duration: Some(std::time::Duration::from_secs(120)),
                command_count: 42,
                lines_added: 0,
                lines_removed: 0,
                cache_hit_rate: None,
            },
        };
        let json = serde_json::to_string(&resp)?;
        assert!(json.contains("metrics"));
        assert!(json.contains("5000"));

        let parsed: Response = serde_json::from_str(&json)?;
        match parsed {
            Response::Metrics {
                session_id,
                metrics,
            } => {
                assert_eq!(session_id.as_str(), "sess-1");
                assert_eq!(metrics.token_count, 5000);
            }
            _ => return Err("expected Metrics".into()),
        }
        Ok(())
    }

    #[test]
    fn test_response_history_serde() -> Result<(), Box<dyn std::error::Error>> {
        let resp = Response::History {
            session_id: SessionId::new_unchecked("sess-1"),
            commands: vec![make_command("read"), make_command("write")],
        };
        let json = serde_json::to_string(&resp)?;
        assert!(json.contains("history"));

        let parsed: Response = serde_json::from_str(&json)?;
        match parsed {
            Response::History {
                session_id,
                commands,
            } => {
                assert_eq!(session_id.as_str(), "sess-1");
                assert_eq!(commands.len(), 2);
            }
            _ => return Err("expected History".into()),
        }
        Ok(())
    }

    #[test]
    fn test_response_sessions_serde() -> Result<(), Box<dyn std::error::Error>> {
        let resp = Response::Sessions {
            sessions: vec![make_session("s1")],
        };
        let json = serde_json::to_string(&resp)?;
        let parsed: Response = serde_json::from_str(&json)?;
        assert!(matches!(parsed, Response::Sessions { .. }));
        Ok(())
    }

    #[test]
    fn test_response_error_serde() -> Result<(), Box<dyn std::error::Error>> {
        let resp = Response::Error {
            message: "not found".to_string(),
        };
        let json = serde_json::to_string(&resp)?;
        assert!(json.contains("error"));
        assert!(json.contains("not found"));
        Ok(())
    }

    #[test]
    fn test_request_from_invalid_json() {
        // Malformed JSON - missing closing brace
        let malformed = r#"{"type": "list_sessions""#;
        let result: Result<Request, _> = serde_json::from_str(malformed);
        assert!(result.is_err());

        // Invalid JSON syntax - trailing comma
        let invalid = r#"{"type": "list_sessions",}"#;
        let result: Result<Request, _> = serde_json::from_str(invalid);
        assert!(result.is_err());

        // Empty string
        let empty = "";
        let result: Result<Request, _> = serde_json::from_str(empty);
        assert!(result.is_err());
    }

    #[test]
    fn test_response_from_invalid_json() {
        // Malformed JSON - unclosed string
        let malformed = r#"{"type": "error", "message": "oops}"#;
        let result: Result<Response, _> = serde_json::from_str(malformed);
        assert!(result.is_err());

        // Invalid JSON - missing colon
        let invalid = r#"{"type" "subscribed"}"#;
        let result: Result<Response, _> = serde_json::from_str(invalid);
        assert!(result.is_err());

        // Just whitespace
        let whitespace = "   ";
        let result: Result<Response, _> = serde_json::from_str(whitespace);
        assert!(result.is_err());
    }

    #[test]
    fn test_request_from_wrong_type() {
        // Valid JSON but unknown variant type
        let unknown_type = r#"{"type": "unknown_request"}"#;
        let result: Result<Request, _> = serde_json::from_str(unknown_type);
        assert!(result.is_err());

        // Valid JSON but missing required fields for variant
        let missing_fields = r#"{"type": "get_metrics"}"#;
        let result: Result<Request, _> = serde_json::from_str(missing_fields);
        assert!(result.is_err());

        // Valid JSON but wrong field types
        let wrong_field_type =
            r#"{"type": "get_history", "session_id": 123, "depth": "full", "limit": "ten"}"#;
        let result: Result<Request, _> = serde_json::from_str(wrong_field_type);
        assert!(result.is_err());
    }
}
