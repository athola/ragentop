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
