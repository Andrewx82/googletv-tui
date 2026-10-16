use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::state::MacroModalState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, modal: &MacroModalState, area: Rect) {
    let popup_area = centered_rect(75, 60, area);
    frame.render_widget(Clear, popup_area);

    let title = if modal.is_editing {
        " ✏️ Edit Automation Macro "
    } else {
        " ⚡ Add Custom Automation Macro "
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
            Span::styled(" Macro Name:   ", field_border(0)),
            Span::styled(" [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.name_input.is_empty() {
                    "e.g. Watch Movie"
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
            Span::styled(" Description:  ", field_border(1)),
            Span::styled(" [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.description_input.is_empty() {
                    "e.g. Power on, wait 2s, launch Netflix"
                } else {
                    &modal.description_input
                },
                if modal.description_input.is_empty() {
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
            Span::styled(" Steps Sequence:", field_border(2)),
        ]),
        Line::from(vec![
            Span::styled(" [ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(
                if modal.steps_input.is_empty() {
                    "e.g. key:KEYCODE_WAKEUP, wait:2000, app:netflix://"
                } else {
                    &modal.steps_input
                },
                if modal.steps_input.is_empty() {
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
        Line::from(vec![Span::styled(
            " Supported Step Actions (separated by commas):",
            Style::default().fg(Theme::GOOGLE_YELLOW).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  • key:<NAME>  ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("key:KEYCODE_POWER, key:KEYCODE_HOME, key:VOLUME_UP, key:DPAD_UP"),
        ]),
        Line::from(vec![
            Span::styled("  • wait:<ms>   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("wait:1500 (delays execution for 1.5 seconds)"),
        ]),
        Line::from(vec![
            Span::styled("  • app:<URI>   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("app:netflix://, app:https://youtube.com, app:spotify://"),
        ]),
        Line::from(vec![
            Span::styled("  • text:<TXT>  ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("text:Synthwave Chill (types query into active search field)"),
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
            "[Tab/↓] Next field  |  [S-Tab/↑] Prev field  |  [Enter] Save Macro  |  [Esc] Cancel",
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
