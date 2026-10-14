use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(3)])
        .split(area);

    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[0]);

    // 1. Left Panel: Macros List
    let list_block = Block::default()
        .title(" ⚡ Automation Macros ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE))
        .style(Style::default().bg(Theme::BG_DARK));

    let items: Vec<ListItem> = state
        .config
        .macros
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_selected = state.selected_macro_idx == i;
            let is_running = state.running_macro.as_deref() == Some(&m.name);

            let status_badge = if is_running {
                Span::styled(
                    " [⏳ RUNNING...] ",
                    Style::default().fg(Theme::GOOGLE_YELLOW).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Theme::ACCENT_CYAN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::TEXT_PRIMARY)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("  {} ", m.name), style),
                    status_badge,
                ]),
                Line::from(vec![Span::styled(
                    format!("     {} ({} steps)", m.description, m.steps.len()),
                    Style::default().fg(Theme::TEXT_MUTED),
                )]),
            ])
        })
        .collect();

    let list = List::new(items).block(list_block);
    frame.render_widget(list, panel_chunks[0]);

    // 2. Right Panel: Macro Steps Breakdown
    let details_block = Block::default()
        .title(" 📋 Macro Sequence Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG_DARK));

    let current_macro = state.config.macros.get(state.selected_macro_idx);

    let details_lines: Vec<Line> = if let Some(m) = current_macro {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Macro: ", Style::default().fg(Theme::TEXT_MUTED)),
                Span::styled(&m.name, Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Theme::TEXT_MUTED)),
                Span::styled(&m.description, Style::default().fg(Theme::TEXT_PRIMARY)),
            ]),
            Line::from(vec![]),
            Line::from(vec![Span::styled(
                "Execution Steps:",
                Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD),
            )]),
        ];

        for (step_idx, step) in m.steps.iter().enumerate() {
            let action_color = if step.action.starts_with("key:") {
                Theme::GOOGLE_YELLOW
            } else if step.action.starts_with("wait:") {
                Theme::TEXT_MUTED
            } else if step.action.starts_with("app:") {
                Theme::GOOGLE_RED
            } else {
                Theme::GOOGLE_GREEN
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {}. ", step_idx + 1), Style::default().fg(Theme::TEXT_MUTED)),
                Span::styled(format!("{:<20} ", step.action), Style::default().fg(action_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("→ {}", step.description), Style::default().fg(Theme::TEXT_PRIMARY)),
            ]));
        }

        lines
    } else {
        vec![Line::from(vec![Span::styled(
            "No macro selected.",
            Style::default().fg(Theme::TEXT_MUTED),
        )])]
    };

    frame.render_widget(
        Paragraph::new(details_lines).block(details_block),
        panel_chunks[1],
    );

    // 3. Footer hints
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::SURFACE));

    let hints = Line::from(vec![
        Span::styled("  [Enter] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Run Macro Sequence   "),
        Span::styled("[↑/↓] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Select Macro"),
    ]);

    frame.render_widget(Paragraph::new(hints).block(footer_block), chunks[1]);
}
