use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

pub struct Theme;

impl Theme {
    // Primary Google Colors
    pub const GOOGLE_BLUE: Color = Color::Rgb(66, 133, 244);
    pub const GOOGLE_RED: Color = Color::Rgb(234, 67, 53);
    pub const GOOGLE_YELLOW: Color = Color::Rgb(251, 188, 5);
    pub const GOOGLE_GREEN: Color = Color::Rgb(52, 168, 83);

    // Dark Surface Colors
    pub const BG_DARK: Color = Color::Rgb(20, 22, 28);
    pub const SURFACE: Color = Color::Rgb(32, 35, 45);
    pub const SURFACE_HIGHLIGHT: Color = Color::Rgb(45, 50, 65);
    pub const BORDER: Color = Color::Rgb(70, 75, 90);
    pub const BORDER_FOCUS: Color = Color::Rgb(66, 133, 244);

    // Text & Accents
    pub const TEXT_PRIMARY: Color = Color::Rgb(245, 247, 250);
    pub const TEXT_MUTED: Color = Color::Rgb(140, 145, 160);
    pub const ACCENT_CYAN: Color = Color::Rgb(77, 208, 225);
    pub const ACCENT_PURPLE: Color = Color::Rgb(186, 104, 200);

    // Styles
    pub fn title() -> Style {
        Style::default()
            .fg(Self::GOOGLE_BLUE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn text() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY)
    }

    pub fn text_muted() -> Style {
        Style::default().fg(Self::TEXT_MUTED)
    }

    pub fn highlight() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Self::GOOGLE_BLUE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn button_normal() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY).bg(Self::SURFACE)
    }

    pub fn button_focused() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Self::ACCENT_CYAN)
            .add_modifier(Modifier::BOLD)
    }

    pub fn block(title: &'static str) -> Block<'static> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Self::BORDER))
            .style(Style::default().bg(Self::BG_DARK))
    }

    pub fn block_focused(title: &'static str) -> Block<'static> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Self::BORDER_FOCUS))
            .style(Style::default().bg(Self::BG_DARK))
    }
}
