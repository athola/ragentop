//! AgentAdapter trait - the primary PORT for agent integrations.

use crate::{AgentSession, AgentType, Command, HistoryDepth, Result, SessionId, SessionMetrics};
use std::path::PathBuf;

/// Capabilities that an adapter supports.
#[derive(Debug, Clone, Default)]
pub struct AdapterCapabilities {
    pub tokens: bool,
    pub cost: bool,
    pub commands: bool,
    pub model_info: bool,
    pub session_replay: bool,
}

/// Trait for agent-specific data extraction.
/// This is a DRIVEN PORT - adapters implement this trait.
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn config_dir(&self) -> PathBuf;
    fn detect_sessions(&self) -> Result<Vec<AgentSession>>;
    fn poll_metrics(&self, session_id: &SessionId) -> Result<SessionMetrics>;
    fn get_command_history(&self, session_id: &SessionId, depth: HistoryDepth, limit: usize) -> Result<Vec<Command>>;
    fn capabilities(&self) -> AdapterCapabilities;
}
