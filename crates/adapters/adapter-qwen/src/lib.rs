//! Qwen CLI adapter for ragentop.

pub mod detector;
pub mod parser;

use ragentop_core::{
    AdapterCapabilities, AgentAdapter, AgentSession, AgentType, Command, HistoryDepth, Result,
    SessionId, SessionMetrics,
};
use std::path::PathBuf;

pub struct QwenAdapter {
    config_dir: PathBuf,
}

impl QwenAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config_dir: dirs::home_dir()
                .map(|h| h.join(".qwen"))
                .unwrap_or_default(),
        }
    }
}

impl Default for QwenAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for QwenAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Qwen
    }
    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }
    fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
        detector::detect_sessions(&self.config_dir)
    }
    fn poll_metrics(&self, _: &SessionId) -> Result<SessionMetrics> {
        parser::aggregate_metrics(&self.config_dir)
    }
    fn get_command_history(
        &self,
        _: &SessionId,
        _: HistoryDepth,
        limit: usize,
    ) -> Result<Vec<Command>> {
        parser::parse_history(&self.config_dir, limit)
    }
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            tokens: true,
            commands: true,
            ..AdapterCapabilities::default()
        }
    }
}
