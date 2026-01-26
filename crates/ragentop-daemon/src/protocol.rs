use serde::{Deserialize, Serialize};
use ragentop_core::{AgentSession, Command, SessionMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request { ListSessions }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response { Sessions { sessions: Vec<AgentSession> } }

