//! Pure types for content-addressed hashing.

use serde::{Deserialize, Serialize};

/// Content-addressed hash for DAG nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub String);

impl Hash {
    /// Creates a hash from raw bytes using blake3.
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(blake3::hash(data).to_hex().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_from_bytes_deterministic() {
        let data = b"test data";
        let hash1 = Hash::from_bytes(data);
        let hash2 = Hash::from_bytes(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_from_bytes_different_inputs() {
        let hash1 = Hash::from_bytes(b"data1");
        let hash2 = Hash::from_bytes(b"data2");
        assert_ne!(hash1, hash2);
    }
}
