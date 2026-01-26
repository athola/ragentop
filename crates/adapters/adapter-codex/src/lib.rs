//! codex adapter for ragentop.
use ragentop_core::{
    AdapterCapabilities, AgentAdapter, AgentSession, AgentType, Command, HistoryDepth, Result,
    SessionId, SessionMetrics,
};
use std::path::PathBuf;

pub struct CodexAdapter {
    config_dir: PathBuf,
}

impl CodexAdapter {
    pub fn new() -> Self {
        Self {
            config_dir: dirs::home_dir()
                .map(|h| h.join(".codex"))
                .unwrap_or_default(),
        }
    }
}

impl Default for CodexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for CodexAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Codex
    }
    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }
    fn detect_sessions(&self) -> Result<Vec<AgentSession>> {
        Ok(vec![])
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
