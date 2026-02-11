//! Claude Code adapter for ragentop.

pub mod detector;
pub mod parser;

use ragentop_core::{
    Adapter, AgentSession, AgentType, Capabilities, Command, HistoryDepth, Result, SessionId,
    SessionMetrics,
};
use std::path::PathBuf;

/// Adapter for monitoring Claude Code CLI sessions.
#[non_exhaustive]
pub struct ClaudeAdapter {
    config_dir: PathBuf,
}

impl ClaudeAdapter {
    /// Creates a new Claude adapter with default config directory.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            config_dir: dirs::home_dir()
                .map(|home| home.join(".claude"))
                .unwrap_or_default(),
        }
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for ClaudeAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Claude
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
        detector::detect_sessions(&self.config_dir)
    }

    fn poll_metrics(&self, _session_id: &SessionId) -> Result<SessionMetrics> {
        Ok(SessionMetrics::default())
    }

    fn get_command_history(
        &self,
        _session_id: &SessionId,
        _depth: HistoryDepth,
        _limit: usize,
    ) -> Result<Vec<Command>> {
        Ok(vec![])
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
}
