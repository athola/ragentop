//! GitHub Copilot CLI adapter for ragentop.
//!
//! # Implementation Status
//!
//! | Feature | Status |
//! |---------|--------|
//! | Session detection | ✅ Implemented |
//! | Command history | ❌ Stub (not available from Copilot) |
//! | Metrics polling | ❌ Stub (returns defaults) |
//! | Cost tracking | ❌ Not available |

pub mod detector;
use ragentop_core::{
    AdapterCapabilities, AgentAdapter, AgentSession, AgentType, Command, HistoryDepth, Result,
    SessionId, SessionMetrics,
};
use std::path::PathBuf;

pub struct CopilotAdapter {
    config_dir: PathBuf,
}

impl CopilotAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config_dir: dirs::home_dir()
                .map(|h| h.join(".copilot"))
                .unwrap_or_default(),
        }
    }
}

impl Default for CopilotAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for CopilotAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Copilot
    }
    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }
    fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
        detector::detect_sessions(&self.config_dir)
    }
    fn poll_metrics(&self, _: &SessionId) -> Result<SessionMetrics> {
        Ok(SessionMetrics::default())
    }
    fn get_command_history(
        &self,
        _: &SessionId,
        _: HistoryDepth,
        _: usize,
    ) -> Result<Vec<Command>> {
        Ok(vec![])
    }
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::default()
    }
}
