use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(80, 85, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" ❓ Google TV Remote TUI — Help & Shortcuts ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Theme::SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let lines = vec![
        Line::from(vec![Span::styled(
            "Global Navigation",
            Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  1 - 6        ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Switch tabs (Remote, Devices, Apps, TextInput, Macros, Logs)"),
        ]),
        Line::from(vec![
            Span::styled("  Tab / S-Tab  ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Switch between panels / focus areas"),
        ]),
        Line::from(vec![
            Span::styled("  ?            ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Show this Help screen"),
        ]),
        Line::from(vec![
            Span::styled("  q / Ctrl+C   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Quit the application"),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Remote Control Shortcuts (Tab 1)",
            Style::default().fg(Theme::GOOGLE_GREEN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ↑/k, ↓/j, ←/h, →/l  ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("D-Pad Up, Down, Left, Right"),
        ]),
        Line::from(vec![
            Span::styled("  Enter / Space       ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Enter: OK / Select | Space: Media Play/Pause"),
        ]),
        Line::from(vec![
            Span::styled("  Esc / b             ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Back key"),
        ]),
        Line::from(vec![
            Span::styled("  H                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Home key"),
        ]),
        Line::from(vec![
            Span::styled("  p / P               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Power toggle / Standby"),
        ]),
        Line::from(vec![
            Span::styled("  + / -               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Volume Up / Volume Down"),
        ]),
        Line::from(vec![
            Span::styled("  m                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Volume Mute toggle"),
        ]),
        Line::from(vec![
            Span::styled("  [ / ]               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Media Rewind / Media Fast Forward"),
        ]),
        Line::from(vec![
            Span::styled("  /                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Google Assistant / Voice Search"),
        ]),
        Line::from(vec![
            Span::styled("  i / s               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("i: Input Source | s: Settings"),
        ]),
        Line::from(vec![
            Span::styled("  Quick App Hotkeys   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Dynamic hotkeys (e.g. y=YouTube, n=Netflix, r=Prime, d=Disney+)"),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Devices Management (Tab 2)",
            Style::default().fg(Theme::GOOGLE_YELLOW).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  s                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Re-scan local Wi-Fi for Google TV devices via mDNS"),
        ]),
        Line::from(vec![
            Span::styled("  Enter               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Connect to selected TV (auto-prompts to pair if needed)"),
        ]),
        Line::from(vec![
            Span::styled("  p                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Start pairing handshake (enter 6-character TV code)"),
        ]),
        Line::from(vec![
            Span::styled("  a / d               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("a: Add manual IP | d: Delete/forget device"),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Apps & Quick Launch (Tab 3)",
            Style::default().fg(Theme::GOOGLE_BLUE).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Enter               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Launch selected app on TV"),
        ]),
        Line::from(vec![
            Span::styled("  a                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Add custom app (Name, URI/Link, Category, Hotkey, Quick App)"),
        ]),
        Line::from(vec![
            Span::styled("  e                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Edit selected app configuration"),
        ]),
        Line::from(vec![
            Span::styled("  d / Delete          ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Delete custom app from list"),
        ]),
        Line::from(vec![
            Span::styled("  Space               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Toggle app on/off Remote Control Quick Apps Pad"),
        ]),
        Line::from(vec![
            Span::styled("  u                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Quick launch custom URL or deep-link URI"),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Text Input & TV Search (Tab 4)",
            Style::default().fg(Theme::GOOGLE_YELLOW).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Enter / S-Enter     ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Enter: Send text | Shift+Enter: Send text + Submit (Enter)"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+S              ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Trigger TV Search overlay, type query, and submit"),
        ]),
        Line::from(vec![
            Span::styled("  Tab                 ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Cycle input mode: Smart IME ➔ Keystrokes ➔ Force IME"),
        ]),
        Line::from(vec![
            Span::styled("  ↑ / ↓               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Navigate and select from search history"),
        ]),
        Line::from(vec![
            Span::styled("  Del / Ctrl+D        ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Delete selected search history query"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+X              ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Clear all search history"),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Macros & Automation (Tab 5)",
            Style::default().fg(Theme::GOOGLE_GREEN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Enter               ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Execute selected automation macro sequence"),
        ]),
        Line::from(vec![
            Span::styled("  a                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Add custom automation macro with steps"),
        ]),
        Line::from(vec![
            Span::styled("  e                   ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Edit selected macro steps and properties"),
        ]),
        Line::from(vec![
            Span::styled("  d / Delete          ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Delete macro from list"),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Press [Esc] or [?] to close this help dialog",
            Style::default().fg(Theme::TEXT_MUTED),
        )]),
    ];

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
