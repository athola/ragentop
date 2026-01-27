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
        KeyCode::Char(c)
            if c.is_ascii_digit() && c != '0' && event.modifiers == KeyModifiers::NONE =>
        {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn key_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_quit_on_q() {
        assert_eq!(
            handle_key_event(key(KeyCode::Char('q'))),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        assert_eq!(
            handle_key_event(key_ctrl(KeyCode::Char('c'))),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_navigate_down_j() {
        assert_eq!(
            handle_key_event(key(KeyCode::Char('j'))),
            Some(Action::NavigateDown)
        );
    }

    #[test]
    fn test_navigate_down_arrow() {
        assert_eq!(
            handle_key_event(key(KeyCode::Down)),
            Some(Action::NavigateDown)
        );
    }

    #[test]
    fn test_navigate_up_k() {
        assert_eq!(
            handle_key_event(key(KeyCode::Char('k'))),
            Some(Action::NavigateUp)
        );
    }

    #[test]
    fn test_navigate_up_arrow() {
        assert_eq!(handle_key_event(key(KeyCode::Up)), Some(Action::NavigateUp));
    }

    #[test]
    fn test_toggle_expand_space() {
        assert_eq!(
            handle_key_event(key(KeyCode::Char(' '))),
            Some(Action::ToggleExpand)
        );
    }

    #[test]
    fn test_toggle_expand_enter() {
        assert_eq!(
            handle_key_event(key(KeyCode::Enter)),
            Some(Action::ToggleExpand)
        );
    }

    #[test]
    fn test_cycle_depth() {
        assert_eq!(
            handle_key_event(key(KeyCode::Char('d'))),
            Some(Action::CycleDepth)
        );
    }

    #[test]
    fn test_refresh_r() {
        assert_eq!(
            handle_key_event(key(KeyCode::Char('r'))),
            Some(Action::Refresh)
        );
    }

    #[test]
    fn test_refresh_f5() {
        assert_eq!(handle_key_event(key(KeyCode::F(5))), Some(Action::Refresh));
    }

    #[test]
    fn test_help_question_mark() {
        assert_eq!(
            handle_key_event(key(KeyCode::Char('?'))),
            Some(Action::Help)
        );
    }

    #[test]
    fn test_help_f1() {
        assert_eq!(handle_key_event(key(KeyCode::F(1))), Some(Action::Help));
    }

    #[test]
    fn test_select_session_digits() {
        assert_eq!(
            handle_key_event(key(KeyCode::Char('1'))),
            Some(Action::SelectSession(0))
        );
        assert_eq!(
            handle_key_event(key(KeyCode::Char('5'))),
            Some(Action::SelectSession(4))
        );
        assert_eq!(
            handle_key_event(key(KeyCode::Char('9'))),
            Some(Action::SelectSession(8))
        );
    }

    #[test]
    fn test_digit_zero_ignored() {
        assert_eq!(handle_key_event(key(KeyCode::Char('0'))), None);
    }

    #[test]
    fn test_unhandled_key_returns_none() {
        assert_eq!(handle_key_event(key(KeyCode::Char('x'))), None);
        assert_eq!(handle_key_event(key(KeyCode::Tab)), None);
    }

    #[test]
    fn test_ctrl_q_not_quit() {
        // 'q' only quits without modifiers
        assert_eq!(handle_key_event(key_ctrl(KeyCode::Char('q'))), None);
    }

    #[test]
    fn test_mouse_scroll_up() {
        let event = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(handle_mouse_event(event), Some(Action::NavigateUp));
    }

    #[test]
    fn test_mouse_scroll_down() {
        let event = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(handle_mouse_event(event), Some(Action::NavigateDown));
    }

    #[test]
    fn test_mouse_click_ignored() {
        let event = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(handle_mouse_event(event), None);
    }
}
