use super::StateNode;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub String);

impl Hash {
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(blake3::hash(data).to_hex().to_string())
    }
}

pub struct DagStore {
    db: sled::Db,
}

impl DagStore {
    /// Opens or creates a DAG store at the given path.
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened.
    pub fn open(path: &Path) -> Result<Self> {
        let db = sled::open(path.join("dag.sled")).map_err(|e| Error::Storage(e.to_string()))?;
        Ok(Self { db })
    }

    /// Stores a node and returns its content-addressed hash.
    ///
    /// # Errors
    /// Returns an error if serialization or database insertion fails.
    pub fn store(&self, node: &StateNode) -> Result<Hash> {
        let data = serde_json::to_vec(node)?;
        let hash = Hash::from_bytes(&data);
        self.db
            .insert(hash.0.as_bytes(), data)
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(hash)
    }

    /// Loads a node by its hash.
    ///
    /// # Errors
    /// Returns an error if the database read or deserialization fails.
    pub fn load(&self, hash: &Hash) -> Result<Option<StateNode>> {
        match self
            .db
            .get(hash.0.as_bytes())
            .map_err(|e| Error::Storage(e.to_string()))?
        {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Returns an iterator that walks the history chain from a node back to root.
    #[must_use]
    pub fn walk_history(&self, from: &Hash) -> HistoryWalker<'_> {
        HistoryWalker {
            store: self,
            current: Some(from.clone()),
        }
    }
}

pub struct HistoryWalker<'a> {
    store: &'a DagStore,
    current: Option<Hash>,
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
