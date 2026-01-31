use ragentop_core::Adapter;
use std::sync::Arc;

#[non_exhaustive]
pub struct AdapterRegistry {
    adapters: Vec<Arc<dyn Adapter>>,
}

impl AdapterRegistry {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self { adapters: vec![] }
    }

    #[inline]
    pub fn register(&mut self, adapter: Arc<dyn Adapter>) {
        self.adapters.push(adapter);
    }

    #[inline]
    #[must_use]
    pub fn adapters(&self) -> &[Arc<dyn Adapter>] {
        &self.adapters
    }
}

impl Default for AdapterRegistry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ragentop_core::{
        AgentSession, AgentType, Capabilities, Command, HistoryDepth, SessionId, SessionMetrics,
    };
    use std::path::PathBuf;

    struct StubAdapter(AgentType);

    impl Adapter for StubAdapter {
        fn agent_type(&self) -> AgentType {
            self.0
        }

        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }

        fn config_dir(&self) -> PathBuf {
            PathBuf::from("/tmp/stub")
        }

        fn detect_sessions(&self) -> ragentop_core::Result<Vec<AgentSession>> {
            Ok(vec![])
        }

        fn get_command_history(
            &self,
            _session_id: &SessionId,
            _depth: HistoryDepth,
            _limit: usize,
        ) -> ragentop_core::Result<Vec<Command>> {
            Ok(vec![])
        }

        fn poll_metrics(&self, _session_id: &SessionId) -> ragentop_core::Result<SessionMetrics> {
            Ok(SessionMetrics::default())
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = AdapterRegistry::new();
        assert!(registry.adapters().is_empty());
    }

    #[test]
    fn register_and_retrieve() {
        let mut registry = AdapterRegistry::new();
        registry.register(Arc::new(StubAdapter(AgentType::Claude)));
        registry.register(Arc::new(StubAdapter(AgentType::Codex)));

        assert_eq!(registry.adapters().len(), 2);
        assert_eq!(registry.adapters()[0].agent_type(), AgentType::Claude);
        assert_eq!(registry.adapters()[1].agent_type(), AgentType::Codex);
    }

    #[test]
    fn default_is_empty() {
        let registry = AdapterRegistry::default();
        assert!(registry.adapters().is_empty());
    }
}
