//! Session tracking and state management.

use ragentop_core::{AgentSession, SessionId};
use std::collections::HashMap;

/// Tracks active agent sessions.
pub struct SessionTracker {
    sessions: HashMap<String, AgentSession>,
}

impl SessionTracker {
    #[must_use]
    pub fn new() -> Self {
        Self { sessions: HashMap::new() }
    }

    pub fn update(&mut self, session: AgentSession) {
        self.sessions.insert(session.id.0.clone(), session);
    }

    pub fn remove(&mut self, id: &SessionId) {
        self.sessions.remove(&id.0);
    }

    #[must_use]
    pub fn get(&self, id: &SessionId) -> Option<&AgentSession> {
        self.sessions.get(&id.0)
    }

    #[must_use]
    pub fn all(&self) -> Vec<&AgentSession> {
        self.sessions.values().collect()
    }
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ragentop_core::{AgentType, SessionStatus};

    fn make_session(id: &str) -> AgentSession {
        AgentSession {
            id: SessionId::new(id),
            agent_type: AgentType::Claude,
            model: None,
            session_name: None,
            working_dir: None,
            pane_id: None,
            pid: None,
            started_at: None,
            status: SessionStatus::Active,
        }
    }

    #[test]
    fn test_tracker_update_and_get() {
        let mut tracker = SessionTracker::new();
        tracker.update(make_session("s1"));
        assert!(tracker.get(&SessionId::new("s1")).is_some());
    }

    #[test]
    fn test_tracker_remove() {
        let mut tracker = SessionTracker::new();
        tracker.update(make_session("s1"));
        tracker.remove(&SessionId::new("s1"));
        assert!(tracker.get(&SessionId::new("s1")).is_none());
    }
}
