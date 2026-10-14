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
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    render_discovered_panel(frame, state, panel_chunks[0]);
    render_saved_panel(frame, state, panel_chunks[1]);
    render_hints_bar(frame, chunks[1]);
}

fn render_discovered_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = !state.devices_focus_saved;
    let block = Block::default()
        .title(if state.is_scanning {
            " 🌐 Discovered Devices (Scanning...) "
        } else {
            " 🌐 Discovered Devices (mDNS) "
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if is_focused {
            Theme::BORDER_FOCUS
        } else {
            Theme::BORDER
        }))
        .style(Style::default().bg(Theme::BG_DARK));

    let items: Vec<ListItem> = if state.discovered_devices.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled(
                if state.is_scanning {
                    "  Searching local Wi-Fi for Google TV devices..."
                } else {
                    "  No devices discovered. Press [s] to scan or [a] to add manual IP."
                },
                Style::default().fg(Theme::TEXT_MUTED),
            ),
        ]))]
    } else {
        state
            .discovered_devices
            .iter()
            .enumerate()
            .map(|(i, dev)| {
                let is_selected = is_focused && state.selected_discovered_idx == i;
                let already_paired = state
                    .config
                    .devices
                    .iter()
                    .any(|d| d.ip == dev.ip.to_string() && d.paired);

                let paired_badge = if already_paired {
                    Span::styled(" [● Paired] ", Style::default().fg(Theme::GOOGLE_GREEN))
                } else {
                    Span::styled(" [◌ Unpaired] ", Style::default().fg(Theme::GOOGLE_YELLOW))
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
                        Span::styled(format!("  {} ", dev.name), style),
                        paired_badge,
                    ]),
                    Line::from(vec![Span::styled(
                        format!("     IP: {}  Port: {}  ({})", dev.ip, dev.port, dev.service_type),
                        Style::default().fg(Theme::TEXT_MUTED),
                    )]),
                ])
            })
            .collect()
    };

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_saved_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = state.devices_focus_saved;
    let block = Block::default()
        .title(" 💾 Saved & Paired Devices ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if is_focused {
            Theme::BORDER_FOCUS
        } else {
            Theme::BORDER
        }))
        .style(Style::default().bg(Theme::BG_DARK));

    let items: Vec<ListItem> = if state.config.devices.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled(
                "  No saved devices yet. Discover devices or press [a] to add an IP.",
                Style::default().fg(Theme::TEXT_MUTED),
            ),
        ]))]
    } else {
        state
            .config
            .devices
            .iter()
            .enumerate()
            .map(|(i, dev)| {
                let is_selected = is_focused && state.selected_saved_idx == i;
                let is_active = state.config.active_device_ip.as_deref() == Some(&dev.ip);

                let status_badge = if dev.paired {
                    Span::styled(" [● Paired] ", Style::default().fg(Theme::GOOGLE_GREEN))
                } else {
                    Span::styled(" [◌ Unpaired] ", Style::default().fg(Theme::GOOGLE_YELLOW))
                };

                let active_badge = if is_active {
                    Span::styled(" [★ Active] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD))
                } else {
                    Span::raw("")
                };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Theme::GOOGLE_BLUE)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::TEXT_PRIMARY)
                };

                let hardware_info = format!(
                    "{} {}",
                    dev.vendor.as_deref().unwrap_or(""),
                    dev.model.as_deref().unwrap_or("")
                );

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!("  {} ", dev.name), style),
                        status_badge,
                        active_badge,
                    ]),
                    Line::from(vec![Span::styled(
                        format!(
                            "     IP: {}  Hardware: {}",
                            dev.ip,
                            if hardware_info.trim().is_empty() { "Unknown" } else { &hardware_info }
                        ),
                        Style::default().fg(Theme::TEXT_MUTED),
                    )]),
                ])
            })
            .collect()
    };

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_hints_bar(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::SURFACE));

    let hints = Line::from(vec![
        Span::styled("  [Tab] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Switch List   "),
        Span::styled("[Enter] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Connect/Select   "),
        Span::styled("[s] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Scan mDNS   "),
        Span::styled("[p] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Pair Device   "),
        Span::styled("[a] ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw("Add IP Manually   "),
        Span::styled("[d] ", Style::default().fg(Theme::GOOGLE_RED).add_modifier(Modifier::BOLD)),
        Span::raw("Delete Device"),
    ]);

    frame.render_widget(Paragraph::new(hints).block(block), area);
}
