//! Codex CLI adapter for ragentop.
//!
//! # Implementation Status
//!
//! | Feature | Status |
//! |---------|--------|
//! | Session detection | Implemented |
//! | Command history | Implemented |
//! | Metrics polling | Stub (returns defaults) |
//! | Cost tracking | Not available |

pub mod detector;

use ragentop_core::{
    Adapter, AgentSession, AgentType, Capabilities, Command, HistoryDepth, Result, SessionId,
    SessionMetrics,
};
use std::path::PathBuf;

/// Adapter for monitoring Codex CLI sessions.
#[non_exhaustive]
pub struct CodexAdapter {
    /// Configuration directory path.
    pub config_dir: PathBuf,
}

impl CodexAdapter {
    /// Creates a new Codex adapter with default config directory.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let home_dir = dirs::home_dir()
            .map(|home| home.join(".codex"))
            .unwrap_or_default();
        Self {
            config_dir: home_dir,
        }
    }
}

impl Default for CodexAdapter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for CodexAdapter {
    #[inline]
    fn agent_type(&self) -> AgentType {
        AgentType::Codex
    }

    #[inline]
    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    #[inline]
    fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
        detector::detect_sessions(&self.config_dir)
    }

    #[inline]
    fn poll_metrics(&self, _session_id: &SessionId) -> Result<SessionMetrics> {
        Ok(SessionMetrics::default())
    }

    #[inline]
    fn get_command_history(
        &self,
        _session_id: &SessionId,
        _depth: HistoryDepth,
        max_count: usize,
    ) -> Result<Vec<Command>> {
        detector::parse_history(&self.config_dir, max_count)
    }

    #[inline]
    fn capabilities(&self) -> Capabilities {
        Capabilities::new().with_commands(true)
    }
}
