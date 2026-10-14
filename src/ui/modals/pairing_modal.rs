use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::state::PairingModalState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, modal: &PairingModalState, area: Rect) {
    let popup_area = centered_rect(65, 45, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" 🔐 Pairing with Google TV ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Theme::SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let pin_chars: Vec<char> = modal.pin_input.chars().collect();
    let display_pin = format!(
        "{} {} {} {} {} {}",
        pin_chars.get(0).copied().unwrap_or('_'),
        pin_chars.get(1).copied().unwrap_or('_'),
        pin_chars.get(2).copied().unwrap_or('_'),
        pin_chars.get(3).copied().unwrap_or('_'),
        pin_chars.get(4).copied().unwrap_or('_'),
        pin_chars.get(5).copied().unwrap_or('_'),
    );

    let mut lines = vec![
        Line::from(vec![]),
        Line::from(vec![
            Span::styled("Device: ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(&modal.name, Style::default().fg(Theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({})", modal.ip), Style::default().fg(Theme::TEXT_MUTED)),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "A 6-character code is currently displayed on your TV screen.",
            Style::default().fg(Theme::TEXT_PRIMARY),
        )]),
        Line::from(vec![Span::styled(
            "Please type the 6 characters (0-9, A-F) below:",
            Style::default().fg(Theme::TEXT_MUTED),
        )]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled("        [  ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::styled(
                display_pin,
                Style::default()
                    .fg(Theme::GOOGLE_YELLOW)
                    .bg(Theme::BG_DARK)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ]        ", Style::default().fg(Theme::ACCENT_CYAN)),
        ]),
        Line::from(vec![]),
    ];

    if let Some(err) = &modal.error {
        lines.push(Line::from(vec![Span::styled(
            format!("✕ Error: {}", err),
            Style::default().fg(Theme::GOOGLE_RED).add_modifier(Modifier::BOLD),
        )]));
    } else if modal.is_submitting {
        lines.push(Line::from(vec![Span::styled(
            "◌ Submitting PIN to TV...",
            Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD),
        )]));
    } else {
        lines.push(Line::from(vec![Span::styled(
            "Press [Enter] to submit PIN | Press [Esc] to cancel",
            Style::default().fg(Theme::TEXT_MUTED),
        )]));
    }

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
