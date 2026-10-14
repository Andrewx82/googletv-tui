use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::{AppState, LogLevel};
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(3)])
        .split(area);

    let log_block = Block::default()
        .title(format!(
            " 📜 Protocol & Activity Inspector ({} events) ",
            state.logs.len()
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE))
        .style(Style::default().bg(Theme::BG_DARK));

    let visible_height = chunks[0].height.saturating_sub(2) as usize;
    let total_logs = state.logs.len();
    let scroll_offset = state.logs_scroll;

    let skip_count = if total_logs > visible_height {
        total_logs.saturating_sub(visible_height + scroll_offset)
    } else {
        0
    };

    let log_lines: Vec<Line> = state
        .logs
        .iter()
        .skip(skip_count)
        .take(visible_height)
        .map(|entry| {
            let (level_badge, level_style) = match entry.level {
                LogLevel::Info => (" [INFO] ", Style::default().fg(Theme::GOOGLE_BLUE)),
                LogLevel::Success => (" [ OK ] ", Style::default().fg(Theme::GOOGLE_GREEN)),
                LogLevel::Warning => (" [WARN] ", Style::default().fg(Theme::GOOGLE_YELLOW)),
                LogLevel::Error => (" [ERR!] ", Style::default().fg(Theme::GOOGLE_RED).add_modifier(Modifier::BOLD)),
                LogLevel::Protocol => (" [PROT] ", Style::default().fg(Theme::ACCENT_CYAN)),
            };

            Line::from(vec![
                Span::styled(
                    entry.time.format("%H:%M:%S%.3f").to_string(),
                    Style::default().fg(Theme::TEXT_MUTED),
                ),
                Span::styled(level_badge, level_style),
                Span::styled(&entry.message, Style::default().fg(Theme::TEXT_PRIMARY)),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(log_lines).block(log_block), chunks[0]);

    // Footer hints
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::SURFACE));

    let hints = Line::from(vec![
        Span::styled("  [↑/↓] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Scroll Logs   "),
        Span::styled("[PgUp/PgDn] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Page Scroll   "),
        Span::styled("[c] ", Style::default().fg(Theme::GOOGLE_RED).add_modifier(Modifier::BOLD)),
        Span::raw("Clear Logs"),
    ]);

    frame.render_widget(Paragraph::new(hints).block(footer_block), chunks[1]);
}
