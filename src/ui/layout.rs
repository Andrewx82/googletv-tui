use chrono::Local;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::state::{ActiveModal, ActiveTab, AppState, ConnectionStatus};
use crate::ui::modals::{
    add_device_modal, app_modal, help_modal, launch_url_modal, macro_modal, pairing_modal,
};
use crate::ui::theme::Theme;
use crate::ui::views::{
    apps_view, devices_view, logs_view, macros_view, remote_view, text_input_view,
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header bar
            Constraint::Length(3), // Tabs bar
            Constraint::Min(10),   // Active view body
        ])
        .split(size);

    render_header(frame, state, chunks[0]);
    render_tabs(frame, state, chunks[1]);
    render_active_view(frame, state, chunks[2]);
    render_modals(frame, state, size);
}

fn render_header(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::SURFACE));

    let (conn_badge, conn_color) = match &state.connection_status {
        ConnectionStatus::Connected { ip, name } => {
            (format!("● {} ({})", name, ip), Theme::GOOGLE_GREEN)
        }
        ConnectionStatus::Connecting(ip) => {
            (format!("◌ Connecting to {}...", ip), Theme::GOOGLE_YELLOW)
        }
        ConnectionStatus::Pairing(ip) => {
            (format!("◌ Pairing with {}...", ip), Theme::GOOGLE_BLUE)
        }
        ConnectionStatus::Error(e) => (format!("✕ {}", e), Theme::GOOGLE_RED),
        ConnectionStatus::Disconnected => ("○ Disconnected".to_string(), Theme::TEXT_MUTED),
    };

    let title_spans = vec![
        Span::raw("  "),
        Span::styled("G", Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD)),
        Span::styled("o", Style::default().fg(Theme::GOOGLE_RED).add_modifier(Modifier::BOLD)),
        Span::styled("o", Style::default().fg(Theme::GOOGLE_YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("g", Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD)),
        Span::styled("l", Style::default().fg(Theme::GOOGLE_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("e", Style::default().fg(Theme::GOOGLE_RED).add_modifier(Modifier::BOLD)),
        Span::styled(" TV Remote TUI ", Style::default().fg(Theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled("v0.1.0 ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::raw(" │  Status: "),
        Span::styled(conn_badge, Style::default().fg(conn_color).add_modifier(Modifier::BOLD)),
    ];

    let time_str = Local::now().format("%H:%M:%S").to_string();
    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(24)])
        .split(block.inner(area));

    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(Line::from(title_spans)), header_layout[0]);

    let right_spans = vec![
        Span::styled("[?] Help ", Style::default().fg(Theme::ACCENT_CYAN)),
        Span::styled(format!(" {} ", time_str), Style::default().fg(Theme::TEXT_MUTED)),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        header_layout[1],
    );
}

fn render_tabs(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG_DARK));

    let tab_titles: Vec<Line> = ActiveTab::titles()
        .iter()
        .enumerate()
        .map(|(i, &title)| {
            if i == state.active_tab.index() {
                Line::from(Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Theme::GOOGLE_BLUE)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {} ", title),
                    Style::default().fg(Theme::TEXT_MUTED),
                ))
            }
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(block)
        .select(state.active_tab.index())
        .style(Style::default().fg(Theme::TEXT_MUTED))
        .divider("│");

    frame.render_widget(tabs, area);
}

fn render_active_view(frame: &mut Frame, state: &AppState, area: Rect) {
    match state.active_tab {
        ActiveTab::Remote => remote_view::render(frame, state, area),
        ActiveTab::Devices => devices_view::render(frame, state, area),
        ActiveTab::Apps => apps_view::render(frame, state, area),
        ActiveTab::TextInput => text_input_view::render(frame, state, area),
        ActiveTab::Macros => macros_view::render(frame, state, area),
        ActiveTab::Logs => logs_view::render(frame, state, area),
    }
}

fn render_modals(frame: &mut Frame, state: &AppState, area: Rect) {
    match &state.modal {
        ActiveModal::None => {}
        ActiveModal::Help => help_modal::render(frame, area),
        ActiveModal::Pairing(pairing_state) => pairing_modal::render(frame, pairing_state, area),
        ActiveModal::AddDevice(add_dev_state) => {
            add_device_modal::render(frame, add_dev_state, area)
        }
        ActiveModal::LaunchUrl(launch_url_state) => {
            launch_url_modal::render(frame, launch_url_state, area)
        }
        ActiveModal::AppEditor(app_state) => {
            app_modal::render(frame, app_state, area)
        }
        ActiveModal::MacroEditor(macro_state) => {
            macro_modal::render(frame, macro_state, area)
        }
    }
}
