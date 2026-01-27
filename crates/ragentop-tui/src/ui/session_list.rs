//! Session list panel.

use crate::state::AppState;
use crate::ui::theme::Theme;
use ragentop_core::SessionStatus;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state
        .sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let (status_symbol, status_color) = match session.status {
                SessionStatus::Active => ("●", Theme::STATUS_ACTIVE),
                SessionStatus::Idle => ("◐", Theme::STATUS_IDLE),
                SessionStatus::Paused => ("○", Theme::STATUS_PAUSED),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{status_symbol} "),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("{:<8}", session.agent_type),
                    Style::default().fg(Theme::agent_color(session.agent_type)),
                ),
                Span::styled(
                    session.session_name.as_deref().unwrap_or("—").to_string(),
                    Style::default().fg(Theme::TEXT_PRIMARY),
                ),
            ]);

            let style = if i == state.selected_index {
                Theme::selected_style()
            } else {
                Theme::default_style()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Sessions ")
            .borders(Borders::ALL)
            .border_style(Theme::border_style()),
    );

    frame.render_widget(list, area);
}
