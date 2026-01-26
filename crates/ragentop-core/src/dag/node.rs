use crate::Command;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNode {
    pub commands: Vec<Command>,
    pub parent: Option<super::Hash>,
    pub timestamp: SystemTime,
    pub metadata: Option<serde_json::Value>,
}

impl StateNode {
    #[must_use]
    pub fn new(commands: Vec<Command>, parent: Option<super::Hash>) -> Self {
        Self { commands, parent, timestamp: SystemTime::now(), metadata: None }
    }
}
