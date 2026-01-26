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
