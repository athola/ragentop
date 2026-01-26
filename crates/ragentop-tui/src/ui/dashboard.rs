//! Main dashboard layout.

use crate::state::AppState;
use crate::ui::{detail, session_list, theme::Theme};
use ragentop_core::SessionStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_summary(frame, chunks[0], state);
    session_list::render(frame, chunks[1], state);
    detail::render(frame, chunks[2], state);
    render_keybindings(frame, chunks[3]);
}

fn render_summary(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let active_count = state
        .sessions
        .iter()
        .filter(|s| s.status == SessionStatus::Active)
        .count();

    let summary = format!(
        " {} active | {} sessions ",
        active_count, state.session_count
    );

    let paragraph = Paragraph::new(Line::from(Span::styled(summary, Theme::default_style())))
        .block(
            Block::default()
                .title(" ragentop ")
                .borders(Borders::ALL)
                .border_style(Theme::border_style()),
        );

    frame.render_widget(paragraph, area);
}

fn render_keybindings(frame: &mut Frame, area: ratatui::layout::Rect) {
    let hints = " [j/k] navigate  [Space] expand  [d] depth  [q] quit";
    let paragraph = Paragraph::new(Span::styled(hints, Style::default().fg(Theme::TEXT_MUTED)));
    frame.render_widget(paragraph, area);
}
