//! Protocol types for daemon-client communication.

use ragentop_core::{AgentSession, Command, SessionMetrics};
use serde::{Deserialize, Serialize};

/// Request from client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    ListSessions,
    GetMetrics { session_id: String },
    GetHistory { session_id: String, depth: u8, limit: usize },
    Subscribe,
}

/// Response from daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Sessions { sessions: Vec<AgentSession> },
    Metrics { session_id: String, metrics: SessionMetrics },
    History { session_id: String, commands: Vec<Command> },
    Subscribed,
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serde() {
        let req = Request::ListSessions;
        let json = serde_json::to_string(&req).unwrap();
        let parsed: Request = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, Request::ListSessions));
    }

    #[test]
    fn test_response_serde() {
        let resp = Response::Sessions { sessions: vec![] };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("sessions"));
    }
}
