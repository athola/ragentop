//! GLM adapter for ragentop (Claude via `ANTHROPIC_BASE_URL` proxy).

pub mod detector;
use ragentop_core::{
    AdapterCapabilities, AgentAdapter, AgentSession, AgentType, Command, HistoryDepth, Result,
    SessionId, SessionMetrics,
};
use std::path::PathBuf;

pub struct GlmAdapter {
    config_dir: PathBuf,
}

impl GlmAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config_dir: dirs::home_dir().map(|h| h.join(".glm")).unwrap_or_default(),
        }
    }
}

impl Default for GlmAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for GlmAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Glm
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
