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
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(6),    // Apps list
            Constraint::Length(3), // Footer hints
        ])
        .split(area);

    // 1. Search Bar
    let search_block = Block::default()
        .title(" 🔍 Filter Apps (Type to filter, Backspace to delete, Esc to clear) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE))
        .style(Style::default().bg(Theme::BG_DARK));

    let search_line = Line::from(vec![
        Span::styled(" Search: ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if state.apps_search.is_empty() {
                "All apps (start typing to filter...)"
            } else {
                &state.apps_search
            },
            if state.apps_search.is_empty() {
                Style::default().fg(Theme::TEXT_MUTED)
            } else {
                Style::default()
                    .fg(Theme::ACCENT_CYAN)
                    .add_modifier(Modifier::BOLD)
            },
        ),
    ]);

    frame.render_widget(Paragraph::new(search_line).block(search_block), chunks[0]);

    // 2. Apps List
    let apps_block = Block::default()
        .title(" 📱 Installed & Popular TV Apps ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG_DARK));

    let filtered = state.filtered_apps();
    let items: Vec<ListItem> = if filtered.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  No apps match your filter.", Style::default().fg(Theme::TEXT_MUTED)),
        ]))]
    } else {
        filtered
            .iter()
            .enumerate()
            .map(|(i, app)| {
                let is_selected = state.selected_app_idx == i;
                let category_color = match app.category.as_str() {
                    "Video" => Theme::GOOGLE_RED,
                    "Music" => Theme::GOOGLE_GREEN,
                    "Media" => Theme::GOOGLE_YELLOW,
                    _ => Theme::GOOGLE_BLUE,
                };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Theme::ACCENT_CYAN)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::TEXT_PRIMARY)
                };

                let hotkey_span = if let Some(hk) = app.hotkey {
                    Span::styled(
                        format!(" [{}] ", hk),
                        Style::default().fg(Theme::GOOGLE_YELLOW).add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::raw("     ")
                };

                ListItem::new(vec![
                    Line::from(vec![
                        hotkey_span,
                        Span::styled(format!(" {:<18} ", app.name), style),
                        Span::styled(
                            format!(" {:<10} ", app.category),
                            Style::default().fg(category_color),
                        ),
                        Span::styled(&app.link, Style::default().fg(Theme::TEXT_MUTED)),
                    ]),
                ])
            })
            .collect()
    };

    let list = List::new(items).block(apps_block);
    frame.render_widget(list, chunks[1]);

    // 3. Footer hints
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::SURFACE));

    let hints = Line::from(vec![
        Span::styled("  [Enter] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Launch Selected App   "),
        Span::styled("[u] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Launch Custom URL / Intent   "),
        Span::styled("[↑/↓] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Navigate List   "),
        Span::styled("[Esc] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Clear Search Filter"),
    ]);

    frame.render_widget(Paragraph::new(hints).block(footer_block), chunks[2]);
}
