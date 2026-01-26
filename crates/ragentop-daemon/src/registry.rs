use ragentop_core::AgentAdapter;
use std::sync::Arc;

pub struct AdapterRegistry { adapters: Vec<Arc<dyn AgentAdapter>> }
impl AdapterRegistry {
    pub fn new() -> Self { Self { adapters: vec![] } }
    pub fn register(&mut self, adapter: Arc<dyn AgentAdapter>) { self.adapters.push(adapter); }
}
impl Default for AdapterRegistry { fn default() -> Self { Self::new() } }

