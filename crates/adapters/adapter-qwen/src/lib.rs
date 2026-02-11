//! Qwen CLI adapter for ragentop.
//!
//! # Implementation Status
//!
//! | Feature | Status |
//! |---------|--------|
//! | Session detection | Implemented |
//! | Command history | Implemented |
//! | Metrics polling | Implemented (token aggregation) |
//! | Cost tracking | Not available |

pub mod detector;
pub mod parser;

use ragentop_core::{
    Adapter, AgentSession, AgentType, Capabilities, Command, HistoryDepth, Result, SessionId,
    SessionMetrics,
};
use std::path::PathBuf;

/// Adapter for monitoring Qwen CLI sessions.
#[non_exhaustive]
pub struct QwenAdapter {
    pub config_dir: PathBuf,
}

impl QwenAdapter {
    /// Creates a new Qwen adapter with default config directory.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config_dir: dirs::home_dir()
                .map(|home| home.join(".qwen"))
                .unwrap_or_default(),
        }
    }
}

impl Default for QwenAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for QwenAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Qwen
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
        detector::detect_sessions(&self.config_dir)
    }

    fn poll_metrics(&self, _session_id: &SessionId) -> Result<SessionMetrics> {
        parser::aggregate_metrics(&self.config_dir)
    }

    fn get_command_history(
        &self,
        _session_id: &SessionId,
        _depth: HistoryDepth,
        limit: usize,
    ) -> Result<Vec<Command>> {
        parser::parse_history(&self.config_dir, limit)
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::new().with_tokens(true).with_commands(true)
    }
}
