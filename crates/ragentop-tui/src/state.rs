//! Application state management.

use ragentop_core::{AgentSession, HistoryDepth, SessionMetrics};

/// Active UI panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    SessionList,
    Detail,
}

/// Application state.
#[derive(Debug)]
pub struct AppState {
    /// Currently selected session index
    pub selected_index: usize,
    /// Total number of sessions
    pub session_count: usize,
    /// Currently active panel
    pub active_panel: Panel,
    /// Whether detail panel is expanded
    pub detail_expanded: bool,
    /// Current history depth level (1-3)
    pub history_depth: HistoryDepth,
    /// Sessions data
    pub sessions: Vec<AgentSession>,
    /// Metrics for selected session
    pub selected_metrics: Option<SessionMetrics>,
    /// Whether to quit
    pub should_quit: bool,
}

impl AppState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_index: 0,
            session_count: 0,
            active_panel: Panel::SessionList,
            detail_expanded: false,
            history_depth: HistoryDepth::WithResponses,
            sessions: Vec::new(),
            selected_metrics: None,
            should_quit: false,
        }
    }

    pub const fn navigate_up(&mut self) {
        if self.session_count == 0 {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.session_count - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub const fn navigate_down(&mut self) {
        if self.session_count == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.session_count;
    }

    pub const fn toggle_expand(&mut self) {
        self.detail_expanded = !self.detail_expanded;
    }

    pub const fn cycle_depth(&mut self) {
        self.history_depth = match self.history_depth {
            HistoryDepth::ToolCallsOnly => HistoryDepth::WithResponses,
            HistoryDepth::WithResponses => HistoryDepth::FullConversation,
            HistoryDepth::FullConversation => HistoryDepth::ToolCallsOnly,
        };
    }

    #[must_use]
    pub fn selected_session(&self) -> Option<&AgentSession> {
        self.sessions.get(self.selected_index)
    }

    pub fn update_sessions(&mut self, sessions: Vec<AgentSession>) {
        self.session_count = sessions.len();
        self.sessions = sessions;
        if self.selected_index >= self.session_count && self.session_count > 0 {
            self.selected_index = self.session_count - 1;
        }
    }

    pub const fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ragentop_core::{AgentType, SessionId, SessionStatus};

    fn make_session(name: &str) -> AgentSession {
        AgentSession {
            id: SessionId::new_unchecked(name),
            agent_type: AgentType::Claude,
            model: None,
            session_name: Some(name.to_string()),
            working_dir: None,
            pane_id: None,
            pid: None,
            started_at: None,
            status: SessionStatus::Active,
        }
    }

    #[test]
    fn new_state_defaults() {
        let s = AppState::new();
        assert_eq!(s.selected_index, 0);
        assert_eq!(s.session_count, 0);
        assert_eq!(s.active_panel, Panel::SessionList);
        assert!(!s.detail_expanded);
        assert!(!s.should_quit);
        assert!(s.sessions.is_empty());
        assert!(s.selected_session().is_none());
    }

    #[test]
    fn navigate_down_wraps_around() {
        let mut s = AppState::new();
        s.update_sessions(vec![make_session("a"), make_session("b")]);

        s.navigate_down(); // 0 -> 1
        assert_eq!(s.selected_index, 1);
        s.navigate_down(); // 1 -> 0 (wrap)
        assert_eq!(s.selected_index, 0);
    }

    #[test]
    fn navigate_up_wraps_around() {
        let mut s = AppState::new();
        s.update_sessions(vec![
            make_session("a"),
            make_session("b"),
            make_session("c"),
        ]);

        s.navigate_up(); // 0 -> 2 (wrap)
        assert_eq!(s.selected_index, 2);
        s.navigate_up(); // 2 -> 1
        assert_eq!(s.selected_index, 1);
    }

    #[test]
    fn navigate_empty_is_noop() {
        let mut s = AppState::new();
        s.navigate_down();
        assert_eq!(s.selected_index, 0);
        s.navigate_up();
        assert_eq!(s.selected_index, 0);
    }

    #[test]
    fn toggle_expand() {
        let mut s = AppState::new();
        assert!(!s.detail_expanded);
        s.toggle_expand();
        assert!(s.detail_expanded);
        s.toggle_expand();
        assert!(!s.detail_expanded);
    }

    #[test]
    fn cycle_depth() {
        let mut s = AppState::new();
        assert_eq!(s.history_depth, HistoryDepth::WithResponses);
        s.cycle_depth();
        assert_eq!(s.history_depth, HistoryDepth::FullConversation);
        s.cycle_depth();
        assert_eq!(s.history_depth, HistoryDepth::ToolCallsOnly);
        s.cycle_depth();
        assert_eq!(s.history_depth, HistoryDepth::WithResponses);
    }

    #[test]
    fn update_sessions_clamps_index() {
        let mut s = AppState::new();
        s.update_sessions(vec![
            make_session("a"),
            make_session("b"),
            make_session("c"),
        ]);
        s.selected_index = 2;

        // Shrink to 2 sessions: index 2 is out of bounds, should clamp to 1
        s.update_sessions(vec![make_session("x"), make_session("y")]);
        assert_eq!(s.selected_index, 1);
    }

    #[test]
    fn selected_session_returns_correct() -> Result<(), Box<dyn std::error::Error>> {
        let mut s = AppState::new();
        s.update_sessions(vec![make_session("a"), make_session("b")]);
        s.navigate_down();
        assert_eq!(
            s.selected_session()
                .ok_or("no session selected")?
                .session_name
                .as_deref(),
            Some("b")
        );
        Ok(())
    }

    #[test]
    fn quit_sets_flag() {
        let mut s = AppState::new();
        s.quit();
        assert!(s.should_quit);
    }
}
