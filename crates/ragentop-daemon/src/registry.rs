use ragentop_core::AgentAdapter;
use std::sync::Arc;

pub struct AdapterRegistry {
    adapters: Vec<Arc<dyn AgentAdapter>>,
}
impl AdapterRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self { adapters: vec![] }
    }
    pub fn register(&mut self, adapter: Arc<dyn AgentAdapter>) {
        self.adapters.push(adapter);
    }

    #[must_use]
    pub fn adapters(&self) -> &[Arc<dyn AgentAdapter>] {
        &self.adapters
    }
}
impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ragentop_core::{
        AdapterCapabilities, AgentSession, AgentType, Command, HistoryDepth, SessionId,
        SessionMetrics,
    };
    use std::path::PathBuf;

    struct StubAdapter(AgentType);

    impl AgentAdapter for StubAdapter {
        fn agent_type(&self) -> AgentType {
            self.0
        }
        fn config_dir(&self) -> PathBuf {
            PathBuf::from("/tmp/stub")
        }
        fn detect_sessions(&self) -> ragentop_core::Result<Vec<AgentSession>> {
            Ok(vec![])
        }
        fn poll_metrics(&self, _: &SessionId) -> ragentop_core::Result<SessionMetrics> {
            Ok(SessionMetrics::default())
        }
        fn get_command_history(
            &self,
            _: &SessionId,
            _: HistoryDepth,
            _: usize,
        ) -> ragentop_core::Result<Vec<Command>> {
            Ok(vec![])
        }
        fn capabilities(&self) -> AdapterCapabilities {
            AdapterCapabilities::default()
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = AdapterRegistry::new();
        assert!(reg.adapters().is_empty());
    }

    #[test]
    fn register_and_retrieve() {
        let mut reg = AdapterRegistry::new();
        reg.register(Arc::new(StubAdapter(AgentType::Claude)));
        reg.register(Arc::new(StubAdapter(AgentType::Codex)));

        assert_eq!(reg.adapters().len(), 2);
        assert_eq!(reg.adapters()[0].agent_type(), AgentType::Claude);
        assert_eq!(reg.adapters()[1].agent_type(), AgentType::Codex);
    }

    #[test]
    fn default_is_empty() {
        let reg = AdapterRegistry::default();
        assert!(reg.adapters().is_empty());
    }
}
