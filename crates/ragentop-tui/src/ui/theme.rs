//! Color theme and styling.

use ragentop_core::AgentType;
use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub const BG_SELECTED: Color = Color::Rgb(38, 38, 38);
    pub const TEXT_PRIMARY: Color = Color::Rgb(229, 229, 229);
    pub const TEXT_SECONDARY: Color = Color::Rgb(163, 163, 163);
    pub const TEXT_MUTED: Color = Color::Rgb(82, 82, 82);
    pub const STATUS_ACTIVE: Color = Color::Green;
    pub const STATUS_IDLE: Color = Color::Yellow;
    pub const STATUS_PAUSED: Color = Color::DarkGray;

    #[must_use]
    pub const fn agent_color(agent: AgentType) -> Color {
        match agent {
            AgentType::Claude => Color::Rgb(217, 119, 6),
            AgentType::Codex => Color::Rgb(34, 197, 94),
            AgentType::Gemini => Color::Rgb(59, 130, 246),
            AgentType::Copilot => Color::Rgb(139, 92, 246),
            AgentType::Qwen => Color::Rgb(6, 182, 212),
            AgentType::Glm => Color::Rgb(236, 72, 153),
        }
    }

    #[must_use]
    pub const fn default_style() -> Style {
        Style::new().fg(Self::TEXT_PRIMARY)
    }

    #[must_use]
    pub const fn selected_style() -> Style {
        Style::new()
            .fg(Self::TEXT_PRIMARY)
            .bg(Self::BG_SELECTED)
            .add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub const fn header_style() -> Style {
        Style::new()
            .fg(Self::TEXT_SECONDARY)
            .add_modifier(Modifier::UNDERLINED)
    }

    #[must_use]
    pub const fn border_style() -> Style {
        Style::new().fg(Self::TEXT_MUTED)
    }
}
