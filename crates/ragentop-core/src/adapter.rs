//! `AgentAdapter` trait - the primary PORT for agent integrations.

use crate::{AgentSession, AgentType, Command, HistoryDepth, Result, SessionId, SessionMetrics};
use std::path::PathBuf;

/// Capabilities that an adapter supports.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Capability flags are intentionally bools"
)]
pub struct Capabilities {
    pub commands: bool,
    pub cost: bool,
    pub model_info: bool,
    pub session_replay: bool,
    pub tokens: bool,
}

impl Capabilities {
    /// Creates a new Capabilities with all flags set to false.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            commands: false,
            cost: false,
            model_info: false,
            session_replay: false,
            tokens: false,
        }
    }

    /// Sets the commands capability.
    #[inline]
    #[must_use]
    pub const fn with_commands(mut self, value: bool) -> Self {
        self.commands = value;
        self
    }

    /// Sets the cost capability.
    #[inline]
    #[must_use]
    pub const fn with_cost(mut self, value: bool) -> Self {
        self.cost = value;
        self
    }

    /// Sets the `model_info` capability.
    #[inline]
    #[must_use]
    pub const fn with_model_info(mut self, value: bool) -> Self {
        self.model_info = value;
        self
    }

    /// Sets the `session_replay` capability.
    #[inline]
    #[must_use]
    pub const fn with_session_replay(mut self, value: bool) -> Self {
        self.session_replay = value;
        self
    }

    /// Sets the tokens capability.
    #[inline]
    #[must_use]
    pub const fn with_tokens(mut self, value: bool) -> Self {
        self.tokens = value;
        self
    }
}

/// Trait for agent-specific data extraction.
/// This is a DRIVEN PORT - adapters implement this trait.
pub trait Adapter: Send + Sync {
    /// Returns the agent type this adapter handles.
    fn agent_type(&self) -> AgentType;

    /// Returns the capabilities of this adapter.
    fn capabilities(&self) -> Capabilities;

    /// Returns the config directory for this agent.
    fn config_dir(&self) -> PathBuf;

    /// Discovers active sessions for this agent.
    ///
    /// # Errors
    /// Returns an error if session detection fails (e.g., I/O errors).
    fn detect_sessions(&self) -> Result<Vec<AgentSession>>;

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

    /// Polls current metrics for a session.
    ///
    /// # Errors
    /// Returns an error if metrics cannot be read.
    fn poll_metrics(&self, session_id: &SessionId) -> Result<SessionMetrics>;
}
