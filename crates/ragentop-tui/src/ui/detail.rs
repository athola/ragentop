//! Detail panel for selected session.

use crate::state::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = state.selected_session().map_or_else(
        || {
            Paragraph::new("No session selected").block(
                Block::default()
                    .title(" Detail ")
                    .borders(Borders::ALL)
                    .border_style(Theme::border_style()),
            )
        },
        |session| {
            let title = format!(
                " {}: {} ",
                session.agent_type,
                session.session_name.as_deref().unwrap_or("unnamed"),
            );

            let metrics_line = state.selected_metrics.as_ref().map_or_else(
                || "Loading metrics...".to_string(),
                |metrics| {
                    format!(
                        "Tokens: {} | Cost: ${:.2} | Commands: {}",
                        metrics.token_count,
                        metrics.cost_usd.unwrap_or(0.0),
                        metrics.command_count
                    )
                },
            );

            let lines = vec![
                Line::from(Span::styled(metrics_line, Theme::default_style())),
                Line::from(""),
                Line::from(Span::styled("Recent Commands", Theme::header_style())),
                Line::from(Span::styled(
                    "(no commands yet)",
                    Style::default().fg(Theme::TEXT_MUTED),
                )),
            ];

            Paragraph::new(lines).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Theme::border_style()),
            )
        },
    );

    frame.render_widget(content, area);
}
