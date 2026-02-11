//! GitHub Copilot CLI adapter for ragentop.
//!
//! # Implementation Status
//!
//! | Feature | Status |
//! |---------|--------|
//! | Session detection | Implemented |
//! | Command history | Stub (not available from Copilot) |
//! | Metrics polling | Stub (returns defaults) |
//! | Cost tracking | Not available |

pub mod detector;

use core::clone::Clone;

use ragentop_core::{
    Adapter, AgentSession, AgentType, Capabilities, Command, HistoryDepth, Result, SessionId,
    SessionMetrics,
};
use std::path::PathBuf;

/// Adapter for monitoring GitHub Copilot CLI sessions.
#[non_exhaustive]
pub struct CopilotAdapter {
    config_dir: PathBuf,
}

impl CopilotAdapter {
    /// Creates a new Copilot adapter with default config directory.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            config_dir: dirs::home_dir()
                .map(|home| home.join(".copilot"))
                .unwrap_or_default(),
        }
    }
}

impl Default for CopilotAdapter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for CopilotAdapter {
    #[inline]
    fn agent_type(&self) -> AgentType {
        AgentType::Copilot
    }

    #[inline]
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
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
    fn get_command_history(
        &self,
        _session_id: &SessionId,
        _depth: HistoryDepth,
        _limit: usize,
    ) -> Result<Vec<Command>> {
        Ok(vec![])
    }

    #[inline]
    fn poll_metrics(&self, _session_id: &SessionId) -> Result<SessionMetrics> {
        Ok(SessionMetrics::default())
    }
}
