//! Input handling.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Actions that can be triggered by user input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    NavigateUp,
    NavigateDown,
    ToggleExpand,
    CycleDepth,
    Refresh,
    Help,
    SelectSession(usize),
}

/// Handles a key event and returns the corresponding action.
#[must_use]
pub fn handle_key_event(event: KeyEvent) -> Option<Action> {
    match event.code {
        KeyCode::Char('q') if event.modifiers == KeyModifiers::NONE => Some(Action::Quit),
        KeyCode::Char('c') if event.modifiers == KeyModifiers::CONTROL => Some(Action::Quit),
        KeyCode::Char('j') if event.modifiers == KeyModifiers::NONE => Some(Action::NavigateDown),
        KeyCode::Down => Some(Action::NavigateDown),
        KeyCode::Char('k') if event.modifiers == KeyModifiers::NONE => Some(Action::NavigateUp),
        KeyCode::Up => Some(Action::NavigateUp),
        KeyCode::Char(' ') if event.modifiers == KeyModifiers::NONE => Some(Action::ToggleExpand),
        KeyCode::Enter => Some(Action::ToggleExpand),
        KeyCode::Char('d') if event.modifiers == KeyModifiers::NONE => Some(Action::CycleDepth),
        KeyCode::Char('r') if event.modifiers == KeyModifiers::NONE => Some(Action::Refresh),
        KeyCode::F(5) => Some(Action::Refresh),
        KeyCode::Char('?') | KeyCode::F(1) => Some(Action::Help),
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' && event.modifiers == KeyModifiers::NONE => {
            Some(Action::SelectSession((c as u8 - b'1') as usize))
        }
        _ => None,
    }
}

/// Handles a mouse event and returns the corresponding action.
#[must_use]
pub const fn handle_mouse_event(event: MouseEvent) -> Option<Action> {
    match event.kind {
        MouseEventKind::ScrollUp => Some(Action::NavigateUp),
        MouseEventKind::ScrollDown => Some(Action::NavigateDown),
        _ => None,
    }
}
