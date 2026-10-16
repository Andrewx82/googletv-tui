use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::state::AppModalState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, modal: &AppModalState, area: Rect) {
    let popup_area = centered_rect(65, 55, area);
    frame.render_widget(Clear, popup_area);

    let title = if modal.is_editing {
        " ✏️ Edit TV App "
    } else {
        " 📱 Add Custom TV App "
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Theme::SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let field_style = |field_idx: usize| {
        if modal.active_field == field_idx {
            Style::default()
                .fg(Color::Black)
                .bg(Theme::ACCENT_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::TEXT_PRIMARY)
        }
    };

    let field_border = |field_idx: usize| {
        if modal.active_field == field_idx {
            Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::TEXT_MUTED)
        }
    };

    let mut lines = vec![
        Line::from(vec![]),
        Line::from(vec![
            Span::styled(" App Name:     ", field_border(0)),
            Span::styled(" [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.name_input.is_empty() {
                    "e.g. SmartTube"
                } else {
                    &modal.name_input
                },
                if modal.name_input.is_empty() {
                    Style::default().fg(Theme::TEXT_MUTED)
                } else {
                    field_style(0)
                },
            ),
            if modal.active_field == 0 {
                Span::styled(" █", Style::default().fg(Theme::ACCENT_CYAN))
            } else {
                Span::raw("")
            },
            Span::styled(" ]", Style::default().fg(Theme::TEXT_MUTED)),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled(" App Link/URI: ", field_border(1)),
            Span::styled(" [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.link_input.is_empty() {
                    "e.g. https://... or netflix:// or intent:..."
                } else {
                    &modal.link_input
                },
                if modal.link_input.is_empty() {
                    Style::default().fg(Theme::TEXT_MUTED)
                } else {
                    field_style(1)
                },
            ),
            if modal.active_field == 1 {
                Span::styled(" █", Style::default().fg(Theme::ACCENT_CYAN))
            } else {
                Span::raw("")
            },
            Span::styled(" ]", Style::default().fg(Theme::TEXT_MUTED)),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled(" Category:     ", field_border(2)),
            Span::styled(" [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.category_input.is_empty() {
                    "Video / Music / Media / Tools"
                } else {
                    &modal.category_input
                },
                if modal.category_input.is_empty() {
                    Style::default().fg(Theme::TEXT_MUTED)
                } else {
                    field_style(2)
                },
            ),
            if modal.active_field == 2 {
                Span::styled(" █", Style::default().fg(Theme::ACCENT_CYAN))
            } else {
                Span::raw("")
            },
            Span::styled(" ]", Style::default().fg(Theme::TEXT_MUTED)),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled(" Hotkey (opt): ", field_border(3)),
            Span::styled(" [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.hotkey_input.is_empty() {
                    "None (e.g. y, n, x, t)"
                } else {
                    &modal.hotkey_input
                },
                if modal.hotkey_input.is_empty() {
                    Style::default().fg(Theme::TEXT_MUTED)
                } else {
                    field_style(3)
                },
            ),
            if modal.active_field == 3 {
                Span::styled(" █", Style::default().fg(Theme::ACCENT_CYAN))
            } else {
                Span::raw("")
            },
            Span::styled(" ]", Style::default().fg(Theme::TEXT_MUTED)),
            Span::raw(" (Single letter hotkey on Remote tab)"),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled(" Quick Launch: ", field_border(4)),
            Span::styled(
                if modal.is_quick_app {
                    " [X] Show on Remote Control Quick Apps Pad "
                } else {
                    " [ ] Do not show on Remote Control Quick Apps Pad "
                },
                if modal.active_field == 4 {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Theme::ACCENT_CYAN)
                        .add_modifier(Modifier::BOLD)
                } else if modal.is_quick_app {
                    Style::default().fg(Theme::GOOGLE_GREEN).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::TEXT_MUTED)
                },
            ),
            Span::raw(" (Press Space to toggle)"),
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
            "[Tab/↓] Next field  |  [S-Tab/↑] Prev field  |  [Enter] Save App  |  [Esc] Cancel",
            Style::default().fg(Theme::TEXT_MUTED),
        )]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
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
