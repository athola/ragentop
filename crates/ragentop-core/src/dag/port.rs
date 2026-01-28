//! Port definition for DAG storage.

use super::{Hash, StateNode};
use crate::Result;

/// Port trait for content-addressed DAG storage.
///
/// Implementations handle the actual I/O (sled, filesystem, etc.).
/// This trait lives in core but implementations live in the shell (daemon).
pub trait DagStore: Send + Sync {
    /// Stores a node and returns its content-addressed hash.
    ///
    /// # Errors
    /// Returns `Error::Io` if the underlying storage operation fails.
    fn store(&self, node: &StateNode) -> Result<Hash>;

    /// Loads a node by its hash.
    ///
    /// Returns `Ok(None)` if the hash is not found.
    ///
    /// # Errors
    /// Returns `Error::Io` if the underlying storage operation fails.
    fn load(&self, hash: &Hash) -> Result<Option<StateNode>>;
}

/// Iterator that walks the history chain from a node back to root.
pub struct HistoryWalker<'a> {
    store: &'a dyn DagStore,
    current: Option<Hash>,
}

impl<'a> HistoryWalker<'a> {
    /// Creates a new history walker starting from the given hash.
    #[must_use]
    pub fn new(store: &'a dyn DagStore, from: &Hash) -> Self {
        Self {
            store,
            current: Some(from.clone()),
        }
    }
}

impl Iterator for HistoryWalker<'_> {
    type Item = StateNode;

    fn next(&mut self) -> Option<Self::Item> {
        let hash = self.current.take()?;
        let node = self.store.load(&hash).ok()??;
        self.current.clone_from(&node.parent);
        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, CommandStatus};
    use std::collections::HashMap;
    use std::time::SystemTime;

    /// In-memory mock store for testing the walker.
    struct MockStore {
        nodes: HashMap<String, StateNode>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                nodes: HashMap::new(),
            }
        }

        fn insert(&mut self, key: &str, node: StateNode) {
            self.nodes.insert(key.to_string(), node);
        }
    }

    impl DagStore for MockStore {
        fn store(&self, _node: &StateNode) -> crate::Result<Hash> {
            unimplemented!("not needed for walker tests")
        }

        fn load(&self, hash: &Hash) -> crate::Result<Option<StateNode>> {
            Ok(self.nodes.get(&hash.0).cloned())
        }
    }

    fn make_node(cmds: &[&str], parent: Option<&str>) -> StateNode {
        StateNode {
            commands: cmds
                .iter()
                .map(|t| Command {
                    timestamp: SystemTime::UNIX_EPOCH,
                    tool: t.to_string(),
                    args: String::new(),
                    status: CommandStatus::Success,
                    result_summary: None,
                })
                .collect(),
            parent: parent.map(|p| Hash(p.to_string())),
            timestamp: SystemTime::UNIX_EPOCH,
            metadata: None,
        }
    }

    #[test]
    fn walker_empty_store_yields_nothing() {
        let store = MockStore::new();
        let start = Hash("nonexistent".to_string());
        let items: Vec<_> = HistoryWalker::new(&store, &start).collect();
        assert!(items.is_empty());
    }

    #[test]
    fn walker_single_node_no_parent() {
        let mut store = MockStore::new();
        store.insert("h1", make_node(&["read"], None));

        let items: Vec<_> = HistoryWalker::new(&store, &Hash("h1".into())).collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].commands[0].tool, "read");
    }

    #[test]
    fn walker_chain_of_three() {
        let mut store = MockStore::new();
        store.insert("h1", make_node(&["first"], None));
        store.insert("h2", make_node(&["second"], Some("h1")));
        store.insert("h3", make_node(&["third"], Some("h2")));

        let items: Vec<_> = HistoryWalker::new(&store, &Hash("h3".into())).collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].commands[0].tool, "third");
        assert_eq!(items[1].commands[0].tool, "second");
        assert_eq!(items[2].commands[0].tool, "first");
    }

    #[test]
    fn walker_stops_at_missing_parent() {
        let mut store = MockStore::new();
        // h2 points to h1 which doesn't exist
        store.insert("h2", make_node(&["orphan"], Some("h1")));

        let items: Vec<_> = HistoryWalker::new(&store, &Hash("h2".into())).collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].commands[0].tool, "orphan");
    }
}
