//! Sled-based DAG storage implementation.

use ragentop_core::dag::{DagStore, Hash, HistoryWalker, StateNode};
use ragentop_core::{Error, Result};
use std::path::Path;

/// Sled-backed implementation of `DagStore`.
pub struct SledDagStore {
    db: sled::Db,
}

impl SledDagStore {
    /// Opens or creates a DAG store at the given path.
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened.
    pub fn open(path: &Path) -> Result<Self> {
        let db = sled::open(path.join("dag.sled")).map_err(|e| Error::Storage(e.to_string()))?;
        Ok(Self { db })
    }

    /// Returns an iterator that walks the history chain from a node back to root.
    #[must_use]
    pub fn walk_history(&self, from: &Hash) -> HistoryWalker<'_> {
        HistoryWalker::new(self, from)
    }
}

impl DagStore for SledDagStore {
    fn store(&self, node: &StateNode) -> Result<Hash> {
        let data = serde_json::to_vec(node)?;
        let hash = Hash::from_bytes(&data);
        self.db
            .insert(hash.0.as_bytes(), data)
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(hash)
    }

    fn load(&self, hash: &Hash) -> Result<Option<StateNode>> {
        match self
            .db
            .get(hash.0.as_bytes())
            .map_err(|e| Error::Storage(e.to_string()))?
        {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ragentop_core::{Command, CommandStatus};
    use std::time::SystemTime;
    use tempfile::TempDir;

    fn make_command(tool: &str) -> Command {
        Command {
            timestamp: SystemTime::now(),
            tool: tool.to_string(),
            args: String::new(),
            status: CommandStatus::Success,
            result_summary: None,
        }
    }

    #[test]
    fn test_sled_dag_store_open() {
        let tmp = TempDir::new().expect("create temp dir");
        let store = SledDagStore::open(tmp.path()).expect("open store");
        drop(store);
    }

    #[test]
    fn test_sled_dag_store_store_and_load() {
        let tmp = TempDir::new().expect("create temp dir");
        let store = SledDagStore::open(tmp.path()).expect("open store");

        let node = StateNode::new(vec![], None);
        let hash = store.store(&node).expect("store node");

        let loaded = store.load(&hash).expect("load node").expect("node exists");
        assert_eq!(loaded.commands.len(), 0);
        assert!(loaded.parent.is_none());
    }

    #[test]
    fn test_sled_dag_store_load_nonexistent() {
        let tmp = TempDir::new().expect("create temp dir");
        let store = SledDagStore::open(tmp.path()).expect("open store");

        let hash = Hash("nonexistent".to_string());
        let result = store.load(&hash).expect("load should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_sled_dag_store_walk_history() {
        let tmp = TempDir::new().expect("create temp dir");
        let store = SledDagStore::open(tmp.path()).expect("open store");

        let root = StateNode::new(vec![make_command("tool1")], None);
        let root_hash = store.store(&root).expect("store root");

        let child = StateNode::new(vec![make_command("tool2")], Some(root_hash));
        let child_hash = store.store(&child).expect("store child");

        let grandchild = StateNode::new(vec![make_command("tool3")], Some(child_hash));
        let grandchild_hash = store.store(&grandchild).expect("store grandchild");

        let nodes: Vec<_> = store.walk_history(&grandchild_hash).collect();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].commands[0].tool, "tool3");
        assert_eq!(nodes[1].commands[0].tool, "tool2");
        assert_eq!(nodes[2].commands[0].tool, "tool1");
    }
}
