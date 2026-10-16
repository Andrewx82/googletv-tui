use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::state::{AppState, TextInputMode};
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Typing box + mode & status
            Constraint::Min(6),     // History and suggestions
            Constraint::Length(3),  // Footer hints
        ])
        .split(area);

    // 1. Text Entry Box with Mode & Status
    let entry_block = Block::default()
        .title(" ⌨ TV Text Input & Search Injection ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE))
        .style(Style::default().bg(Theme::BG_DARK));

    let mode_style = match state.text_input_mode {
        TextInputMode::Smart => Style::default().fg(Color::Black).bg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD),
        TextInputMode::Keystroke => Style::default().fg(Color::Black).bg(Theme::GOOGLE_GREEN).add_modifier(Modifier::BOLD),
        TextInputMode::Ime => Style::default().fg(Color::Black).bg(Theme::GOOGLE_YELLOW).add_modifier(Modifier::BOLD),
    };

    let status_line = if state.ime_active && state.ime_field_counter > 0 {
        Line::from(vec![
            Span::styled(" ● ", Style::default().fg(Theme::GOOGLE_GREEN)),
            Span::styled("TV Input Field Detected: ", Style::default().fg(Theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled(
                if state.ime_field_label.is_empty() {
                    format!("Active (ID #{})", state.ime_field_counter)
                } else {
                    format!("\"{}\" (ID #{})", state.ime_field_label, state.ime_field_counter)
                },
                Style::default().fg(Theme::ACCENT_CYAN),
            ),
            Span::styled(" — IME ready", Style::default().fg(Theme::TEXT_MUTED)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" ○ ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled("TV Input Status: ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled("Standby / App Canvas Keyboard (YouTube, Netflix, Prime Video, etc.)", Style::default().fg(Theme::TEXT_MUTED)),
        ])
    };

    let mode_line = Line::from(vec![
        Span::styled(" Mode: ", Style::default().fg(Theme::TEXT_PRIMARY)),
        Span::styled(format!(" {} ", state.text_input_mode.label()), mode_style),
        Span::styled("  [Tab to cycle: Smart ➔ Keystrokes ➔ IME]", Style::default().fg(Theme::TEXT_MUTED)),
    ]);

    let entry_lines = vec![
        status_line,
        mode_line,
        Line::from(vec![]),
        Line::from(vec![
            Span::styled(" > [ ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(
                if state.text_input_buffer.is_empty() {
                    "Type query or text here..."
                } else {
                    &state.text_input_buffer
                },
                if state.text_input_buffer.is_empty() {
                    Style::default().fg(Theme::TEXT_MUTED)
                } else {
                    Style::default().fg(Theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD)
                },
            ),
            Span::styled(" █ ]", Style::default().fg(Theme::ACCENT_CYAN)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" Buffer length: {} characters", state.text_input_buffer.len()),
                Style::default().fg(Theme::TEXT_MUTED),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(entry_lines).block(entry_block), chunks[0]);

    // 2. Query History and Presets
    let hist_block = Block::default()
        .title(" 🕒 Search History & Quick Queries (Use ↑/↓ and Enter to re-send) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG_DARK));

    let items: Vec<ListItem> = state
        .text_input_history
        .iter()
        .enumerate()
        .map(|(i, query)| {
            let is_selected = state.selected_history_idx == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Theme::GOOGLE_BLUE)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::TEXT_PRIMARY)
            };

            ListItem::new(vec![Line::from(vec![
                Span::styled(format!("  {} ", if is_selected { "▶" } else { " " }), style),
                Span::styled(format!("\"{}\"", query), style),
            ])])
        })
        .collect();

    let list = List::new(items).block(hist_block);
    frame.render_widget(list, chunks[1]);

    // 3. Footer hints
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::SURFACE));

    let hints = Line::from(vec![
        Span::styled("  [Enter] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Send to TV   "),
        Span::styled("[Tab] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Cycle Mode   "),
        Span::styled("[Backspace] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Delete   "),
        Span::styled("[Esc] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Clear Buffer   "),
        Span::styled("[↑/↓] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Select History"),
    ]);

    frame.render_widget(Paragraph::new(hints).block(footer_block), chunks[2]);
}
