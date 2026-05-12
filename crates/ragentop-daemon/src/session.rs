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
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn update(&mut self, session: AgentSession) {
        self.sessions.insert(session.id.to_string(), session);
    }

    pub fn remove(&mut self, id: &SessionId) {
        self.sessions.remove(id.as_str());
    }

    #[must_use]
    pub fn get(&self, id: &SessionId) -> Option<&AgentSession> {
        self.sessions.get(id.as_str())
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
        AgentSession::new(
            SessionId::new_unchecked(id),
            AgentType::Claude,
            SessionStatus::Active,
        )
    }

    #[test]
    fn test_tracker_new_is_empty() {
        let tracker = SessionTracker::new();
        assert!(tracker.all().is_empty());
    }

    #[test]
    fn test_tracker_default_is_empty() {
        let tracker = SessionTracker::default();
        assert!(tracker.all().is_empty());
    }

    #[test]
    fn test_tracker_update_and_get() {
        let mut tracker = SessionTracker::new();
        tracker.update(make_session("s1"));
        assert!(tracker.get(&SessionId::new_unchecked("s1")).is_some());
    }

    #[test]
    fn test_tracker_remove() {
        let mut tracker = SessionTracker::new();
        tracker.update(make_session("s1"));
        tracker.remove(&SessionId::new_unchecked("s1"));
        assert!(tracker.get(&SessionId::new_unchecked("s1")).is_none());
    }

    #[test]
    fn test_tracker_all_returns_all_sessions() {
        let mut tracker = SessionTracker::new();
        tracker.update(make_session("s1"));
        tracker.update(make_session("s2"));
        tracker.update(make_session("s3"));

        let all = tracker.all();
        assert_eq!(all.len(), 3);

        // Check that all session IDs are present
        let ids: Vec<&str> = all.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"s1"));
        assert!(ids.contains(&"s2"));
        assert!(ids.contains(&"s3"));
    }

    #[test]
    fn test_tracker_update_replaces_existing() -> Result<(), Box<dyn std::error::Error>> {
        let mut tracker = SessionTracker::new();
        let mut session = make_session("s1");
        session.model = Some("opus".to_string());
        tracker.update(session);

        let mut updated = make_session("s1");
        updated.model = Some("sonnet".to_string());
        tracker.update(updated);

        // Should still have only one session
        assert_eq!(tracker.all().len(), 1);
        let stored = tracker
            .get(&SessionId::new_unchecked("s1"))
            .ok_or("session not found")?;
        assert_eq!(stored.model.as_deref(), Some("sonnet"));
        Ok(())
    }

    #[test]
    fn test_tracker_get_nonexistent_returns_none() {
        let tracker = SessionTracker::new();
        assert!(tracker
            .get(&SessionId::new_unchecked("nonexistent"))
            .is_none());
    }

    #[test]
    fn test_tracker_remove_nonexistent_is_safe() {
        let mut tracker = SessionTracker::new();
        tracker.update(make_session("s1"));
        tracker.remove(&SessionId::new_unchecked("nonexistent")); // Should not panic
        assert_eq!(tracker.all().len(), 1);
    }
}
