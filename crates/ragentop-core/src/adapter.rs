//! `AgentAdapter` trait - the primary PORT for agent integrations.

use crate::{AgentSession, AgentType, Command, HistoryDepth, Result, SessionId, SessionMetrics};
use std::path::PathBuf;

/// Capabilities that an adapter supports.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)] // Capability flags are intentionally bools
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
    /// Returns the agent type this adapter handles.
    fn agent_type(&self) -> AgentType;

    /// Returns the config directory for this agent.
    fn config_dir(&self) -> PathBuf;

    /// Discovers active sessions for this agent.
    ///
    /// # Errors
    /// Returns an error if session detection fails (e.g., I/O errors).
    fn detect_sessions(&self) -> Result<Vec<AgentSession>>;

    /// Polls current metrics for a session.
    ///
    /// # Errors
    /// Returns an error if metrics cannot be read.
    fn poll_metrics(&self, session_id: &SessionId) -> Result<SessionMetrics>;

    /// Gets command history at the specified depth.
    ///
    /// # Errors
    /// Returns an error if history cannot be read.
    fn get_command_history(
        &self,
        session_id: &SessionId,
        depth: HistoryDepth,
        limit: usize,
    ) -> Result<Vec<Command>>;

    /// Returns the capabilities of this adapter.
    fn capabilities(&self) -> AdapterCapabilities;
}
