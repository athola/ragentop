//! Google Gemini CLI adapter for ragentop.

pub mod detector;

use ragentop_core::{
    AdapterCapabilities, AgentAdapter, AgentSession, AgentType, Command, HistoryDepth, Result,
    SessionId, SessionMetrics,
};
use std::path::PathBuf;

pub struct GeminiAdapter {
    config_dir: PathBuf,
}

impl GeminiAdapter {
    #[must_use]
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
        detector::detect_sessions(&self.config_dir)
    }
    fn poll_metrics(&self, _: &SessionId) -> Result<SessionMetrics> {
        Ok(SessionMetrics::default())
    }
    fn get_command_history(
        &self,
        session_id: &SessionId,
        _: HistoryDepth,
        limit: usize,
    ) -> Result<Vec<Command>> {
        detector::parse_history(&self.config_dir, session_id.as_str(), limit)
    }
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            commands: true,
            ..AdapterCapabilities::default()
        }
    }
}
