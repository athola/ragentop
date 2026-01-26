//! gemini adapter for ragentop.
use ragentop_core::{
    AdapterCapabilities, AgentAdapter, AgentSession, AgentType, Command, HistoryDepth, Result,
    SessionId, SessionMetrics,
};
use std::path::PathBuf;

pub struct GeminiAdapter {
    config_dir: PathBuf,
}

impl GeminiAdapter {
    pub fn new() -> Self {
        Self {
            config_dir: dirs::home_dir()
                .map(|h| h.join(".gemini"))
                .unwrap_or_default(),
        }
    }
}

impl Default for GeminiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for GeminiAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Gemini
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
