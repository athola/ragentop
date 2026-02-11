//! Google Gemini CLI adapter for ragentop.
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

use core::clone::Clone;
use core::default::Default;

use ragentop_core::{
    Adapter, AgentSession, AgentType, Capabilities, Command, HistoryDepth, Result, SessionId,
    SessionMetrics,
};
use std::path::PathBuf;

/// Adapter for monitoring Google Gemini CLI sessions.
#[non_exhaustive]
pub struct GeminiAdapter {
    config_dir: PathBuf,
}

impl GeminiAdapter {
    /// Creates a new Gemini adapter with default config directory.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let config_dir = dirs::home_dir()
            .map(|home| home.join(".gemini"))
            .unwrap_or_default();
        Self { config_dir }
    }
}

impl Default for GeminiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for GeminiAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Gemini
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
        session_id: &SessionId,
        _depth: HistoryDepth,
        limit: usize,
    ) -> Result<Vec<Command>> {
        detector::parse_history(&self.config_dir, session_id.as_str(), limit)
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::new().with_commands(true)
    }
}
