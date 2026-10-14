use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::app::state::{AppState, ConnectionStatus, RemoteButton};
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(48), Constraint::Min(35)])
        .split(area);

    render_remote_body(frame, state, chunks[0]);
    render_status_and_telemetry(frame, state, chunks[1]);
}

fn render_remote_body(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(" 🎮 Google TV Remote ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::GOOGLE_BLUE))
        .style(Style::default().bg(Theme::BG_DARK));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout rows inside remote
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top row (Power, Input)
            Constraint::Length(1), // Spacer
            Constraint::Length(7), // D-Pad
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Nav row 1 (Back, Home, Search)
            Constraint::Length(1), // Nav row 2 (Vol-, Mute, Vol+)
            Constraint::Length(1), // Nav row 3 (Rewind, Play/Pause, FastForward)
            Constraint::Length(1), // Nav row 4 (Settings, Channels)
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Section header: Quick Apps
            Constraint::Length(1), // App row 1
            Constraint::Length(1), // App row 2
            Constraint::Length(1), // App row 3
            Constraint::Min(0),    // Remainder
        ])
        .split(inner);

    // Row 0: Power and Input
    let all_buttons = RemoteButton::all();
    let btn_style = |b: RemoteButton| {
        let is_selected = all_buttons.get(state.selected_remote_button).copied() == Some(b);

        if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Theme::ACCENT_CYAN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Theme::TEXT_PRIMARY)
                .bg(Theme::SURFACE)
        }
    };

    // Row 0: Power (p) and Input (i)
    let p_style = btn_style(RemoteButton::Power);
    let in_style = btn_style(RemoteButton::TvInput);
    let set_style = btn_style(RemoteButton::Settings);

    let row0 = Line::from(vec![
        Span::raw("  "),
        Span::styled(" [ ⏻ Power (p) ] ", p_style),
        Span::raw("   "),
        Span::styled(" [ 📺 Input (i) ] ", in_style),
    ]);
    frame.render_widget(Paragraph::new(row0).alignment(Alignment::Center), rows[0]);

    // D-Pad in rows[2] (height 7)
    let up_style = btn_style(RemoteButton::Up);
    let left_style = btn_style(RemoteButton::Left);
    let ok_style = btn_style(RemoteButton::Ok);
    let right_style = btn_style(RemoteButton::Right);
    let down_style = btn_style(RemoteButton::Down);

    let dpad_lines = vec![
        Line::from(vec![Span::raw("        ╭───────╮        ")]),
        Line::from(vec![
            Span::raw("        │ "),
            Span::styled("  ▲  ", up_style),
            Span::raw(" │        "),
        ]),
        Line::from(vec![Span::raw("    ╭───┴───────┴───╮    ")]),
        Line::from(vec![
            Span::raw("    │ "),
            Span::styled(" ◀ ", left_style),
            Span::raw(" "),
            Span::styled("  OK  ", ok_style),
            Span::raw(" "),
            Span::styled(" ▶ ", right_style),
            Span::raw(" │    "),
        ]),
        Line::from(vec![Span::raw("    ╰───┬───────┬───╯    ")]),
        Line::from(vec![
            Span::raw("        │ "),
            Span::styled("  ▼  ", down_style),
            Span::raw(" │        "),
        ]),
        Line::from(vec![Span::raw("        ╰───────╯        ")]),
    ];
    frame.render_widget(
        Paragraph::new(dpad_lines).alignment(Alignment::Center),
        rows[2],
    );

    // Nav Row 1: Back, Home, Search
    let back_style = btn_style(RemoteButton::Back);
    let home_style = btn_style(RemoteButton::Home);
    let search_style = btn_style(RemoteButton::Search);

    let row1 = Line::from(vec![
        Span::styled(" [ ⎋ Back ] ", back_style),
        Span::raw(" "),
        Span::styled(" [ ⌂ Home (H) ] ", home_style),
        Span::raw(" "),
        Span::styled(" [ 🔍 Search ] ", search_style),
    ]);
    frame.render_widget(Paragraph::new(row1).alignment(Alignment::Center), rows[4]);

    // Nav Row 2: Volume -, Mute, Volume +
    let voldown_style = btn_style(RemoteButton::VolDown);
    let mute_style = btn_style(RemoteButton::Mute);
    let volup_style = btn_style(RemoteButton::VolUp);

    let row2 = Line::from(vec![
        Span::styled(" [ 🔉 Vol- (-) ] ", voldown_style),
        Span::raw(" "),
        Span::styled(" [ 🔇 Mute (m) ] ", mute_style),
        Span::raw(" "),
        Span::styled(" [ 🔊 Vol+ (+) ] ", volup_style),
    ]);
    frame.render_widget(Paragraph::new(row2).alignment(Alignment::Center), rows[5]);

    // Nav Row 3: Rewind, Play/Pause, FastForward
    let rew_style = btn_style(RemoteButton::Rewind);
    let play_style = btn_style(RemoteButton::PlayPause);
    let ff_style = btn_style(RemoteButton::FastForward);

    let row3 = Line::from(vec![
        Span::styled(" [ ⏪ Rew ] ", rew_style),
        Span::raw(" "),
        Span::styled(" [ ⏯ Play/Pause ] ", play_style),
        Span::raw(" "),
        Span::styled(" [ ⏩ Fwd ] ", ff_style),
    ]);
    frame.render_widget(Paragraph::new(row3).alignment(Alignment::Center), rows[6]);

    // Nav Row 4: Settings & Channel +/-
    let chandown_style = btn_style(RemoteButton::ChanDown);
    let chanup_style = btn_style(RemoteButton::ChanUp);

    let row4 = Line::from(vec![
        Span::styled(" [ CH- ] ", chandown_style),
        Span::raw("  "),
        Span::styled(" [ ⚙ Settings (s) ] ", set_style),
        Span::raw("  "),
        Span::styled(" [ CH+ ] ", chanup_style),
    ]);
    frame.render_widget(Paragraph::new(row4).alignment(Alignment::Center), rows[7]);

    // Quick Apps header
    let header_line = Line::from(vec![
        Span::styled("── Quick Apps ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled("────────────────────────────────", Style::default().fg(Theme::BORDER)),
    ]);
    frame.render_widget(Paragraph::new(header_line), rows[9]);

    // App Row 1: YouTube, Netflix
    let yt_style = btn_style(RemoteButton::AppYouTube);
    let nf_style = btn_style(RemoteButton::AppNetflix);

    let app_row1 = Line::from(vec![
        Span::styled(" [ ▶ YouTube (y) ] ", yt_style),
        Span::raw("   "),
        Span::styled(" [ 🔴 Netflix (n) ] ", nf_style),
    ]);
    frame.render_widget(
        Paragraph::new(app_row1).alignment(Alignment::Center),
        rows[10],
    );

    // App Row 2: Prime, Disney+
    let prime_style = btn_style(RemoteButton::AppPrime);
    let disney_style = btn_style(RemoteButton::AppDisney);

    let app_row2 = Line::from(vec![
        Span::styled(" [ 🎬 Prime (r) ] ", prime_style),
        Span::raw("    "),
        Span::styled(" [ ✨ Disney+ (d) ] ", disney_style),
    ]);
    frame.render_widget(
        Paragraph::new(app_row2).alignment(Alignment::Center),
        rows[11],
    );

    // App Row 3: Spotify
    let spot_style = btn_style(RemoteButton::AppSpotify);
    let app_row3 = Line::from(vec![Span::styled(" [ 🎵 Spotify ] ", spot_style)]);
    frame.render_widget(
        Paragraph::new(app_row3).alignment(Alignment::Center),
        rows[12],
    );
}

fn render_status_and_telemetry(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Device Telemetry
            Constraint::Length(9),  // Shortcuts cheat sheet
            Constraint::Min(6),     // Activity Log
        ])
        .split(area);

    // 1. Device Telemetry Card
    let status_block = Block::default()
        .title(" 📡 Device Telemetry & Status ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG_DARK));

    let status_inner = status_block.inner(chunks[0]);
    frame.render_widget(status_block, chunks[0]);

    let (conn_badge, conn_color) = match &state.connection_status {
        ConnectionStatus::Connected { ip, name } => {
            (format!("● Connected to {} ({})", name, ip), Theme::GOOGLE_GREEN)
        }
        ConnectionStatus::Connecting(ip) => {
            (format!("◌ Connecting to {}...", ip), Theme::GOOGLE_YELLOW)
        }
        ConnectionStatus::Pairing(ip) => {
            (format!("◌ Pairing with {}...", ip), Theme::GOOGLE_BLUE)
        }
        ConnectionStatus::Error(e) => (format!("✕ Error: {}", e), Theme::GOOGLE_RED),
        ConnectionStatus::Disconnected => {
            ("○ Disconnected (Press [2] for Devices)".to_string(), Theme::TEXT_MUTED)
        }
    };

    let power_badge = match state.is_on {
        Some(true) => Span::styled(" [ ● ON ] ", Style::default().fg(Theme::GOOGLE_GREEN).add_modifier(Modifier::BOLD)),
        Some(false) => Span::styled(" [ ○ STANDBY ] ", Style::default().fg(Theme::GOOGLE_YELLOW)),
        None => Span::styled(" [ ? UNKNOWN ] ", Style::default().fg(Theme::TEXT_MUTED)),
    };

    let dev_model = state
        .device_details
        .as_ref()
        .map(|d| format!("{} {} (v{})", d.vendor, d.model, d.app_version))
        .unwrap_or_else(|| "Google TV / Android TV".to_string());

    let app_display = if state.current_app.is_empty() {
        "Home Screen / Unknown".to_string()
    } else {
        match state.current_app.as_str() {
            "com.google.android.youtube.tv" => "YouTube".to_string(),
            "com.netflix.ninja" => "Netflix".to_string(),
            "com.amazon.amazonvideo.livingroom" => "Prime Video".to_string(),
            "com.disney.disneyplus" => "Disney+".to_string(),
            "com.spotify.tv.android" => "Spotify".to_string(),
            "com.plexapp.android" => "Plex".to_string(),
            other => other.to_string(),
        }
    };

    let telemetry_lines = vec![
        Line::from(vec![
            Span::styled("Connection: ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(conn_badge, Style::default().fg(conn_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Power State: ", Style::default().fg(Theme::TEXT_MUTED)),
            power_badge,
            Span::raw("   "),
            Span::styled("Hardware: ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(dev_model, Style::default().fg(Theme::TEXT_PRIMARY)),
        ]),
        Line::from(vec![
            Span::styled("Active App:  ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled(app_display, Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let t_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(2), Constraint::Min(0)])
        .split(status_inner);

    frame.render_widget(Paragraph::new(telemetry_lines), t_chunks[0]);

    // Volume gauge
    let vol_percent = if state.volume_max > 0 {
        ((state.volume_level as f64 / state.volume_max as f64) * 100.0) as u16
    } else {
        40
    };

    let vol_label = format!(
        "Vol: {}/{} ({}{})",
        state.volume_level,
        state.volume_max,
        if state.volume_muted { "MUTED" } else { "Unmuted" },
        format!(" - {}%", vol_percent)
    );

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(
            Style::default()
                .fg(if state.volume_muted {
                    Theme::GOOGLE_RED
                } else {
                    Theme::GOOGLE_BLUE
                })
                .bg(Theme::SURFACE),
        )
        .percent(vol_percent)
        .label(vol_label);

    frame.render_widget(gauge, t_chunks[1]);

    // 2. Keyboard Shortcuts Panel
    let keys_block = Block::default()
        .title(" ⌨ Remote Shortcuts ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG_DARK));

    let keys_lines = vec![
        Line::from(vec![
            Span::styled(" ↑/k ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
            Span::raw("Up   "),
            Span::styled(" ↓/j ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
            Span::raw("Down "),
            Span::styled(" ←/h ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
            Span::raw("Left "),
            Span::styled(" →/l ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
            Span::raw("Right"),
            Span::styled(" Enter ", Style::default().fg(Theme::ACCENT_CYAN).add_modifier(Modifier::BOLD)),
            Span::raw("Select/OK"),
        ]),
        Line::from(vec![
            Span::styled(" Esc/b ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Back  "),
            Span::styled(" H ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Home  "),
            Span::styled(" p ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Power "),
            Span::styled(" + / - ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Vol +/-"),
            Span::styled(" m ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Mute"),
        ]),
        Line::from(vec![
            Span::styled(" Space ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Play/Pause "),
            Span::styled(" [ / ] ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Rewind/Fwd "),
            Span::styled(" / ", Style::default().fg(Theme::ACCENT_CYAN)),
            Span::raw("Search/Assistant"),
        ]),
        Line::from(vec![
            Span::styled(" y ", Style::default().fg(Theme::GOOGLE_RED)),
            Span::raw("YouTube  "),
            Span::styled(" n ", Style::default().fg(Theme::GOOGLE_RED)),
            Span::raw("Netflix  "),
            Span::styled(" r ", Style::default().fg(Theme::GOOGLE_BLUE)),
            Span::raw("Prime  "),
            Span::styled(" d ", Style::default().fg(Theme::GOOGLE_BLUE)),
            Span::raw("Disney+"),
        ]),
    ];

    frame.render_widget(Paragraph::new(keys_lines).block(keys_block), chunks[1]);

    // 3. Recent Activity Log
    let activity_block = Block::default()
        .title(" 📝 Recent Activity Stream ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG_DARK));

    let recent_logs: Vec<Line> = state
        .logs
        .iter()
        .rev()
        .take(12)
        .map(|entry| {
            let color = match entry.level {
                crate::app::state::LogLevel::Info => Theme::TEXT_PRIMARY,
                crate::app::state::LogLevel::Success => Theme::GOOGLE_GREEN,
                crate::app::state::LogLevel::Warning => Theme::GOOGLE_YELLOW,
                crate::app::state::LogLevel::Error => Theme::GOOGLE_RED,
                crate::app::state::LogLevel::Protocol => Theme::TEXT_MUTED,
            };
            Line::from(vec![
                Span::styled(
                    entry.time.format("%H:%M:%S ").to_string(),
                    Style::default().fg(Theme::TEXT_MUTED),
                ),
                Span::styled(&entry.message, Style::default().fg(color)),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(recent_logs).block(activity_block), chunks[2]);
}
