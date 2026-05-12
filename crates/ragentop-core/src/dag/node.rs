//! DAG node representation for session state snapshots.

use crate::Command;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A node in the Merkle DAG representing a session state snapshot.
///
/// Fields are ordered alphabetically as required by clippy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StateNode {
    /// Commands executed in this state.
    pub commands: Vec<Command>,
    /// Optional metadata attached to the node.
    pub metadata: Option<serde_json::Value>,
    /// Parent node hash, if any.
    pub parent: Option<super::Hash>,
    /// Timestamp when this node was created.
    pub timestamp: SystemTime,
}

impl StateNode {
    /// Creates a new state node with the given commands, parent, and timestamp.
    ///
    /// The caller must provide the timestamp explicitly to keep the core pure
    /// (no I/O or side effects). Typically pass `SystemTime::now()` from the
    /// imperative shell layer.
    #[must_use]
    #[inline]
    pub const fn new(
        commands: Vec<Command>,
        parent: Option<super::Hash>,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            commands,
            metadata: None,
            parent,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, CommandStatus};

    fn make_command(tool: &str) -> Command {
        Command {
            timestamp: SystemTime::UNIX_EPOCH,
            tool: tool.to_string(),
            args: "arg1".to_string(),
            status: CommandStatus::Success,
            result_summary: Some("done".to_string()),
        }
    }

    #[test]
    fn test_state_node_new_without_parent() {
        let node = StateNode::new(vec![], None, SystemTime::now());
        assert!(node.commands.is_empty());
        assert!(node.parent.is_none());
        assert!(node.metadata.is_none());
    }

    #[test]
    fn test_state_node_new_with_commands() {
        let cmds = vec![make_command("read"), make_command("write")];
        let node = StateNode::new(cmds, None, SystemTime::now());
        assert_eq!(node.commands.len(), 2);
        assert_eq!(node.commands[0].tool, "read");
        assert_eq!(node.commands[1].tool, "write");
    }

    #[test]
    fn test_state_node_new_with_parent() {
        let parent_hash = super::super::Hash("abc123".to_string());
        let node = StateNode::new(vec![], Some(parent_hash.clone()), SystemTime::now());
        assert_eq!(node.parent, Some(parent_hash));
    }

    #[test]
    fn test_state_node_timestamp_is_exact() {
        let ts = SystemTime::now();
        let node = StateNode::new(vec![], None, ts);
        assert_eq!(node.timestamp, ts);
    }

    #[test]
    fn test_state_node_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let node = StateNode::new(vec![make_command("test")], None, SystemTime::now());
        let json = serde_json::to_string(&node)?;
        let parsed: StateNode = serde_json::from_str(&json)?;

        assert_eq!(parsed.commands.len(), 1);
        assert_eq!(parsed.commands[0].tool, "test");
        assert!(parsed.parent.is_none());
        Ok(())
    }

    #[test]
    fn test_state_node_with_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let mut node = StateNode::new(vec![], None, SystemTime::now());
        node.metadata = Some(serde_json::json!({"key": "value", "count": 42}));

        let json = serde_json::to_string(&node)?;
        let parsed: StateNode = serde_json::from_str(&json)?;

        let meta = parsed.metadata.ok_or("metadata present")?;
        assert_eq!(meta["key"], "value");
        assert_eq!(meta["count"], 42);
        Ok(())
    }

    #[test]
    fn test_state_node_clone() {
        let node = StateNode::new(vec![make_command("clone_test")], None, SystemTime::now());
        let cloned = node.clone();

        assert_eq!(cloned.commands.len(), node.commands.len());
        assert_eq!(cloned.timestamp, node.timestamp);
    }
}
