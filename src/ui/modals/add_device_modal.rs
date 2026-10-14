use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::state::AddDeviceModalState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, modal: &AddDeviceModalState, area: Rect) {
    let popup_area = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" ➕ Add Google TV Device Manually ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Theme::SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = vec![
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Enter the local IP address of your Google TV / Android TV:",
            Style::default().fg(Theme::TEXT_PRIMARY),
        )]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled(" IP Address: [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.ip_input.is_empty() {
                    "e.g. 192.168.1.100"
                } else {
                    &modal.ip_input
                },
                if modal.ip_input.is_empty() {
                    Style::default().fg(Theme::TEXT_MUTED)
                } else {
                    Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)
                },
            ),
            Span::styled(" █ ]", Style::default().fg(Theme::ACCENT_CYAN)),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled(" Name (opt): [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.name_input.is_empty() {
                    "Living Room TV"
                } else {
                    &modal.name_input
                },
                if modal.name_input.is_empty() {
                    Style::default().fg(Theme::TEXT_MUTED)
                } else {
                    Style::default().fg(Theme::TEXT_PRIMARY)
                },
            ),
            Span::styled(" ]", Style::default().fg(Theme::TEXT_MUTED)),
        ]),
        Line::from(vec![]),
    ];

    if let Some(err) = &modal.error {
        lines.push(Line::from(vec![Span::styled(
            format!("✕ Error: {}", err),
            Style::default().fg(Theme::GOOGLE_RED).add_modifier(Modifier::BOLD),
        )]));
    } else {
        lines.push(Line::from(vec![Span::styled(
            "Press [Enter] to Save & Pair | Press [Esc] to cancel",
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
