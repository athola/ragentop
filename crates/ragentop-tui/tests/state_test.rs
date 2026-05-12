use ragentop_tui::state::{AppState, Panel};

#[test]
fn test_initial_state() {
    let state = AppState::new();
    assert_eq!(state.selected_index, 0);
    assert_eq!(state.active_panel, Panel::SessionList);
}

#[test]
fn test_navigate_down() {
    let mut state = AppState::new();
    state.session_count = 5;

    state.navigate_down();
    assert_eq!(state.selected_index, 1);

    state.navigate_down();
    assert_eq!(state.selected_index, 2);
}

#[test]
fn test_navigate_wraps() {
    let mut state = AppState::new();
    state.session_count = 3;
    state.selected_index = 2;

    state.navigate_down();
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_navigate_up_wraps() {
    let mut state = AppState::new();
    state.session_count = 3;
    state.selected_index = 0;

    state.navigate_up();
    assert_eq!(state.selected_index, 2);
}

#[test]
fn test_toggle_expanded() {
    let mut state = AppState::new();
    assert!(!state.detail_expanded);

    state.toggle_expand();
    assert!(state.detail_expanded);

    state.toggle_expand();
    assert!(!state.detail_expanded);
}

#[test]
fn test_cycle_depth() {
    use ragentop_core::HistoryDepth;

    let mut state = AppState::new();
    assert_eq!(state.history_depth, HistoryDepth::WithResponses);

    state.cycle_depth();
    assert_eq!(state.history_depth, HistoryDepth::FullConversation);

    state.cycle_depth();
    assert_eq!(state.history_depth, HistoryDepth::ToolCallsOnly);

    state.cycle_depth();
    assert_eq!(state.history_depth, HistoryDepth::WithResponses);
}

#[test]
fn test_navigate_empty_state() {
    let mut state = AppState::new();
    // Should not panic on empty state
    state.navigate_up();
    state.navigate_down();
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_update_sessions_adjusts_index() {
    use ragentop_core::{AgentSession, AgentType, SessionId, SessionStatus};

    let mut state = AppState::new();
    state.selected_index = 5;
    state.session_count = 10;

    // Reduce to 3 sessions - index should adjust
    let sessions: Vec<AgentSession> = (0..3)
        .map(|i| {
            AgentSession::new(
                SessionId::new_unchecked(format!("s{i}")),
                AgentType::Claude,
                SessionStatus::Active,
            )
        })
        .collect();

    state.update_sessions(sessions);
    assert_eq!(state.session_count, 3);
    assert_eq!(state.selected_index, 2); // Adjusted to last valid index
}

#[test]
fn test_selected_session_returns_none_when_empty() {
    let state = AppState::new();
    assert!(state.selected_session().is_none());
}

#[test]
fn test_quit_sets_flag() {
    let mut state = AppState::new();
    assert!(!state.should_quit);
    state.quit();
    assert!(state.should_quit);
}

#[test]
fn test_default_matches_new() {
    let new_state = AppState::new();
    let default_state = AppState::default();
    assert_eq!(new_state.selected_index, default_state.selected_index);
    assert_eq!(new_state.active_panel, default_state.active_panel);
}
