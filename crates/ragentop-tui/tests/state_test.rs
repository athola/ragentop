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
