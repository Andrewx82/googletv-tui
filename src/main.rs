#![allow(dead_code, unused_assignments)]

mod app;
mod config;
mod net;
mod proto;
mod ui;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

use crate::app::state::{
    ActiveModal, ActiveTab, AddDeviceModalState, AppModalState, AppState, ConnectionStatus,
    LaunchUrlModalState, LogLevel, MacroModalState, PairingModalState, RemoteButton, TextInputMode,
};
use crate::config::{AppConfig, CustomApp, SavedMacro};
use crate::net::client::{TvConnection, TvEvent};
use crate::net::discovery::{DiscoveredDevice, DiscoveryManager};
use crate::net::pairing::{PairingSession, PAIRING_PORT};
use crate::net::CertManager;
use crate::proto::parse_keycode;
use crate::proto::remote::{RemoteDirection, RemoteKeyCode};

#[derive(Parser, Debug)]
#[command(
    name = "googletv-tui",
    author = "Google TV Remote TUI Developer",
    version = "0.2.1",
    about = "Feature-rich Google TV & Android TV remote control TUI and CLI in Rust"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Discover Google TV and Android TV devices on local Wi-Fi via mDNS
    Scan {
        #[arg(short, long, default_value_t = 5)]
        timeout: u64,
    },
    /// Pair with a Google TV device at the specified IP address
    Pair {
        ip: String,
        #[arg(short, long, default_value = "Google TV TUI")]
        client_name: String,
    },
    /// Send a key command to a paired Google TV
    Key {
        ip: String,
        /// e.g. KEYCODE_HOME, POWER, DPAD_UP, PLAY_PAUSE, VOLUME_UP
        keycode: String,
    },
    /// Launch an app or deep link URL on the TV
    Launch {
        ip: String,
        /// App link or URL (e.g. https://www.youtube.com, netflix://)
        app_link: String,
    },
    /// Send text to active search/input field on TV
    Text {
        ip: String,
        text: String,
        /// Transmission mode: smart (default), keys, or ime
        #[arg(short, long, default_value = "smart")]
        mode: String,
        /// Press Enter on TV after sending text
        #[arg(short, long)]
        enter: bool,
        /// Trigger TV global search overlay before sending text
        #[arg(short, long)]
        search: bool,
    },
    /// List saved and paired devices
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan { timeout }) => run_cli_scan(timeout).await,
        Some(Commands::Pair { ip, client_name }) => run_cli_pair(&ip, &client_name).await,
        Some(Commands::Key { ip, keycode }) => run_cli_key(&ip, &keycode).await,
        Some(Commands::Launch { ip, app_link }) => run_cli_launch(&ip, &app_link).await,
        Some(Commands::Text { ip, text, mode, enter, search }) => {
            run_cli_text(&ip, &text, &mode, enter, search).await
        }
        Some(Commands::List) => run_cli_list().await,
        None => run_tui().await,
    }
}

// ============================================================================
// CLI Subcommands
// ============================================================================

async fn run_cli_scan(timeout_secs: u64) -> Result<()> {
    println!("🔍 Scanning local Wi-Fi for Google TV / Android TV devices ({}s)...", timeout_secs);
    let (tx, mut rx) = mpsc::channel(32);
    DiscoveryManager::start_discovery(tx, Duration::from_secs(timeout_secs)).await?;

    let mut found = 0;
    while let Some(dev) = rx.recv().await {
        found += 1;
        println!("  • Discovered: {} at {}:{} ({})", dev.name, dev.ip, dev.port, dev.service_type);
    }
    if found == 0 {
        println!("No devices discovered during the scan period.");
    }
    Ok(())
}

async fn run_cli_list() -> Result<()> {
    let config = AppConfig::load();
    println!("📋 Saved & Paired Google TV Devices ({}):", config.devices.len());
    for dev in &config.devices {
        println!(
            "  • {} - IP: {} [{}] (Last seen: {:?})",
            dev.name,
            dev.ip,
            if dev.paired { "PAIRED" } else { "UNPAIRED" },
            dev.last_connected
        );
    }
    Ok(())
}

async fn run_cli_pair(ip_str: &str, client_name: &str) -> Result<()> {
    let ip = ip_str.parse::<std::net::IpAddr>()?;
    let target = SocketAddr::new(ip, PAIRING_PORT);
    let cert_mgr = CertManager::new(&AppConfig::config_dir());
    let (cert, key) = cert_mgr.load_or_generate(client_name)?;

    println!("Connecting to {} on port 6467 for pairing...", target);
    let session = PairingSession::start(target, client_name, cert, key).await?;
    println!("Handshake successful! Look at your TV screen for a 6-character code.");
    print!("Enter the 6-character PIN shown on your TV: ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let pin = input.trim();

    session.finish(pin).await?;
    println!("✅ Pairing completed successfully! Certificate is permanently authorized on this TV.");

    let mut config = AppConfig::load();
    config.add_or_update_device("Google TV", ip_str, true, None, None);
    config.set_active_device(ip_str);
    Ok(())
}

async fn run_cli_key(ip_str: &str, key_str: &str) -> Result<()> {
    let keycode = parse_keycode(key_str).ok_or_else(|| anyhow::anyhow!("Unknown keycode: {}", key_str))?;
    let ip = ip_str.parse::<std::net::IpAddr>()?;
    let target = SocketAddr::new(ip, 6466);
    let cert_mgr = CertManager::new(&AppConfig::config_dir());
    let (cert, key) = cert_mgr.load_or_generate("googletv-tui")?;

    let (event_tx, _event_rx) = mpsc::channel(32);
    let client = TvConnection::connect(target, cert, key, event_tx).await?;

    // Wait for connection handshake
    tokio::time::sleep(Duration::from_millis(500)).await;
    client.send_key(keycode, RemoteDirection::Short).await?;
    println!("Sent key {:?} to {}", keycode, ip_str);
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _ = client.disconnect().await;
    Ok(())
}

async fn run_cli_launch(ip_str: &str, app_link: &str) -> Result<()> {
    let ip = ip_str.parse::<std::net::IpAddr>()?;
    let target = SocketAddr::new(ip, 6466);
    let cert_mgr = CertManager::new(&AppConfig::config_dir());
    let (cert, key) = cert_mgr.load_or_generate("googletv-tui")?;

    let (event_tx, _event_rx) = mpsc::channel(32);
    let client = TvConnection::connect(target, cert, key, event_tx).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;
    client.launch_app(app_link.to_string()).await?;
    println!("Launched app link '{}' on {}", app_link, ip_str);
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _ = client.disconnect().await;
    Ok(())
}

async fn run_cli_text(ip_str: &str, text: &str, mode: &str, enter: bool, search: bool) -> Result<()> {
    let ip = ip_str.parse::<std::net::IpAddr>()?;
    let target = SocketAddr::new(ip, 6466);
    let cert_mgr = CertManager::new(&AppConfig::config_dir());
    let (cert, key) = cert_mgr.load_or_generate("googletv-tui")?;

    let (event_tx, _event_rx) = mpsc::channel(32);
    let client = TvConnection::connect(target, cert, key, event_tx).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;
    if search {
        client.search_on_tv(text.to_string()).await?;
        println!("Triggered TV search and typed '{}' on {}", text, ip_str);
    } else {
        match mode.to_lowercase().as_str() {
            "keys" | "key" | "keystroke" | "keystrokes" => {
                client.send_text_keys(text.to_string(), enter).await?;
                println!("Sent keystroke text '{}' to {} (enter={})", text, ip_str, enter);
            }
            "ime" => {
                client.send_text_ime(text.to_string(), enter).await?;
                println!("Sent IME text '{}' to {} (enter={})", text, ip_str, enter);
            }
            _ => {
                if enter {
                    client.send_text_with_enter(text.to_string()).await?;
                    println!("Sent text '{}' + Enter to {}", text, ip_str);
                } else {
                    client.send_text(text.to_string()).await?;
                    println!("Sent text '{}' to {}", text, ip_str);
                }
            }
        }
    }
    tokio::time::sleep(Duration::from_millis(400)).await;
    let _ = client.disconnect().await;
    Ok(())
}

// ============================================================================
// Interactive TUI Application
// ============================================================================

async fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Load config and certificates
    let config = AppConfig::load();
    let cert_mgr = CertManager::new(&AppConfig::config_dir());
    let (client_cert, client_key) = cert_mgr.load_or_generate("Google TV Remote TUI")?;

    let mut state = AppState::new(config);

    // Event channels
    let (tv_event_tx, mut tv_event_rx) = mpsc::channel::<TvEvent>(32);
    let (disc_tx, mut disc_rx) = mpsc::channel::<DiscoveredDevice>(32);

    // Initial mDNS discovery
    state.is_scanning = true;
    let _ = DiscoveryManager::start_discovery(disc_tx.clone(), Duration::from_secs(10)).await;

    // Auto-connect to last TV on startup
    if let Some((target_name, target_ip)) = state.config.get_last_tv() {
        if let Ok(ip) = target_ip.parse::<std::net::IpAddr>() {
            let target = SocketAddr::new(ip, 6466);
            state.connection_status = ConnectionStatus::Connecting(target_ip.clone());
            state.config.set_active_device(&target_ip);
            state.log(
                LogLevel::Info,
                format!("Attempting auto-connect to last TV: {} ({})", target_name, target_ip),
            );

            let cert_clone = client_cert.clone();
            let key_clone = client_key.clone();
            let tx_clone = tv_event_tx.clone();
            let name_for_task = target_name.clone();
            let ip_for_task = target_ip.clone();

            tokio::spawn(async move {
                match TvConnection::connect(target, cert_clone, key_clone, tx_clone.clone()).await {
                    Ok(_handle) => {
                        // Handshake started and client task will send TvEvent::Connected
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(TvEvent::ConnectionFailed(format!(
                                "Failed to connect to {} ({}): {}",
                                name_for_task, ip_for_task, e
                            )))
                            .await;
                    }
                }
            });
        }
    } else {
        state.log(
            LogLevel::Info,
            "No previous TV found. Press [2] for Devices to scan and pair your TV.",
        );
    }

    // Run main application loop
    let res = run_app_loop(
        &mut terminal,
        &mut state,
        client_cert,
        client_key,
        tv_event_tx,
        &mut tv_event_rx,
        disc_tx,
        &mut disc_rx,
    )
    .await;

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Application error: {:?}", e);
    }

    Ok(())
}

async fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut AppState,
    client_cert: openssl::x509::X509,
    client_key: openssl::pkey::PKey<openssl::pkey::Private>,
    tv_event_tx: mpsc::Sender<TvEvent>,
    tv_event_rx: &mut mpsc::Receiver<TvEvent>,
    disc_tx: mpsc::Sender<DiscoveredDevice>,
    disc_rx: &mut mpsc::Receiver<DiscoveredDevice>,
) -> Result<()> {
    let mut event_stream = crossterm::event::EventStream::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(50));

    loop {
        if state.should_quit {
            break;
        }

        terminal.draw(|frame| {
            ui::render(frame, state);
        })?;

        tokio::select! {
            _ = ticker.tick() => {
                // Regular tick for clock and animations
            }

            // Incoming mDNS discovery events
            Some(dev) = disc_rx.recv() => {
                if !state.discovered_devices.iter().any(|d| d.ip == dev.ip) {
                    state.log(LogLevel::Success, format!("mDNS Discovered TV: {} at {}", dev.name, dev.ip));
                    state.discovered_devices.push(dev);
                }
            }

            // Incoming TV client events (port 6466)
            Some(event) = tv_event_rx.recv() => {
                match event {
                    TvEvent::Connected { details, handle } => {
                        state.client_handle = Some(handle);
                        let ip = state.config.active_device_ip.clone().unwrap_or_else(|| "TV".into());
                        state.connection_status = ConnectionStatus::Connected {
                            ip: ip.clone(),
                            name: format!("{} {}", details.vendor, details.model),
                        };
                        state.device_details = Some(details.clone());
                        state.log(LogLevel::Success, format!("Connected to {} {} (v{})", details.vendor, details.model, details.app_version));
                        state.config.add_or_update_device("Google TV", &ip, true, Some(details.vendor), Some(details.model));
                    }
                    TvEvent::ConnectionFailed(reason) => {
                        state.connection_status = ConnectionStatus::Disconnected;
                        state.client_handle = None;
                        state.log(LogLevel::Warning, format!("Auto-connect failed: {}. Press [2] for Devices.", reason));
                    }
                    TvEvent::Disconnected(reason) => {
                        state.connection_status = ConnectionStatus::Disconnected;
                        state.client_handle = None;
                        state.ime_active = false;
                        state.ime_field_counter = 0;
                        state.ime_field_label.clear();
                        state.log(LogLevel::Warning, format!("TV Disconnected: {}", reason));
                    }
                    TvEvent::PowerStateChanged(on) => {
                        state.is_on = Some(on);
                        state.log(LogLevel::Info, format!("TV Power State: {}", if on { "ON" } else { "STANDBY" }));
                    }
                    TvEvent::VolumeChanged { level, max, muted } => {
                        state.volume_level = level;
                        state.volume_max = max;
                        state.volume_muted = muted;
                    }
                    TvEvent::ActiveAppChanged(pkg) => {
                        state.current_app = pkg.clone();
                        state.log(LogLevel::Info, format!("Current Running App: {}", pkg));
                    }
                    TvEvent::ImeStateChanged { active, label, field_counter } => {
                        state.ime_active = active;
                        state.ime_field_label = label.clone();
                        state.ime_field_counter = field_counter;
                        if active && !label.is_empty() {
                            state.log(LogLevel::Info, format!("TV Input Field Active: \"{}\"", label));
                        }
                    }
                    TvEvent::ProtocolLog(msg) => {
                        state.log(LogLevel::Protocol, msg);
                    }
                }
            }

            // Crossterm user input events
            Some(Ok(event)) = futures_util::StreamExt::next(&mut event_stream) => {
                match event {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        handle_key_event(
                            key,
                            state,
                            &client_cert,
                            &client_key,
                            &tv_event_tx,
                            &disc_tx,
                        ).await?;
                    }
                    Event::Mouse(_mouse) => {
                        // Mouse click support can trigger tab switching or remote buttons
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

async fn handle_key_event(
    key: crossterm::event::KeyEvent,
    state: &mut AppState,
    client_cert: &openssl::x509::X509,
    client_key: &openssl::pkey::PKey<openssl::pkey::Private>,
    tv_event_tx: &mpsc::Sender<TvEvent>,
    disc_tx: &mpsc::Sender<DiscoveredDevice>,
) -> Result<()> {
    // 1. If Modal is active, route keys to modal
    match &mut state.modal {
        ActiveModal::Help => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')) {
                state.modal = ActiveModal::None;
            }
            return Ok(());
        }
        ActiveModal::AddDevice(add_state) => match key.code {
            KeyCode::Esc => {
                state.modal = ActiveModal::None;
                return Ok(());
            }
            KeyCode::Backspace => {
                add_state.ip_input.pop();
                return Ok(());
            }
            KeyCode::Char(c) if c.is_ascii_graphic() => {
                add_state.ip_input.push(c);
                return Ok(());
            }
            KeyCode::Enter => {
                let ip_str = add_state.ip_input.trim().to_string();
                if ip_str.parse::<std::net::IpAddr>().is_ok() {
                    let name = if add_state.name_input.trim().is_empty() {
                        "Google TV".to_string()
                    } else {
                        add_state.name_input.trim().to_string()
                    };
                    state.config.add_or_update_device(&name, &ip_str, false, None, None);
                    state.config.set_active_device(&ip_str);
                    state.log(LogLevel::Success, format!("Added device: {} ({})", name, ip_str));
                    state.modal = ActiveModal::None;
                } else {
                    add_state.error = Some("Invalid IP address format".into());
                }
                return Ok(());
            }
            _ => return Ok(()),
        },
        ActiveModal::LaunchUrl(launch_state) => match key.code {
            KeyCode::Esc => {
                state.modal = ActiveModal::None;
                return Ok(());
            }
            KeyCode::Backspace => {
                launch_state.url_input.pop();
                return Ok(());
            }
            KeyCode::Char(c) => {
                launch_state.url_input.push(c);
                return Ok(());
            }
            KeyCode::Enter => {
                let url = launch_state.url_input.trim().to_string();
                if !url.is_empty() {
                    if let Some(client) = &state.client_handle {
                        let _ = client.launch_app(url.clone()).await;
                        state.log(LogLevel::Success, format!("Launched URL: {}", url));
                    }
                    state.modal = ActiveModal::None;
                }
                return Ok(());
            }
            _ => return Ok(()),
        },
        ActiveModal::Pairing(pairing_state) => {
            let mut pin_to_submit = None;
            match key.code {
                KeyCode::Esc => {
                    state.modal = ActiveModal::None;
                    return Ok(());
                }
                KeyCode::Backspace => {
                    pairing_state.pin_input.pop();
                    return Ok(());
                }
                KeyCode::Char(c) if c.is_ascii_hexdigit() && pairing_state.pin_input.len() < 6 => {
                    pairing_state.pin_input.push(c.to_ascii_uppercase());
                    if pairing_state.pin_input.len() == 6 {
                        pairing_state.is_submitting = true;
                        pin_to_submit = Some((
                            pairing_state.ip.clone(),
                            pairing_state.name.clone(),
                            pairing_state.pin_input.clone(),
                            pairing_state.session.clone(),
                        ));
                    }
                }
                _ => return Ok(()),
            }

            if let Some((ip, name, pin, Some(arc_session))) = pin_to_submit {
                let mut lock = arc_session.lock().await;
                if let Some(session) = lock.take() {
                    match session.finish(&pin).await {
                        Ok(()) => {
                            state.modal = ActiveModal::None;
                            state.log(LogLevel::Success, "Pairing handshake successful!");
                            state.config.add_or_update_device(&name, &ip, true, None, None);
                            state.config.set_active_device(&ip);

                            // Auto connect to remote port 6466
                            if let Ok(ip_addr) = ip.parse::<std::net::IpAddr>() {
                                let target = SocketAddr::new(ip_addr, 6466);
                                let cert_c = client_cert.clone();
                                let key_c = client_key.clone();
                                let tx_c = tv_event_tx.clone();
                                state.connection_status = ConnectionStatus::Connecting(ip.clone());
                                tokio::spawn(async move {
                                    match TvConnection::connect(target, cert_c, key_c, tx_c.clone()).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            let _ = tx_c.send(TvEvent::ConnectionFailed(format!("{}", e))).await;
                                        }
                                    }
                                });
                            }
                        }
                        Err(e) => {
                            if let ActiveModal::Pairing(ref mut ps) = state.modal {
                                ps.error = Some(format!("{}", e));
                                ps.is_submitting = false;
                            }
                        }
                    }
                }
            }
            return Ok(());
        }
        ActiveModal::AppEditor(app_state) => {
            let mut save_app = None;
            match key.code {
                KeyCode::Esc => {
                    state.modal = ActiveModal::None;
                    return Ok(());
                }
                KeyCode::Tab | KeyCode::Down => {
                    app_state.active_field = (app_state.active_field + 1) % 5;
                    return Ok(());
                }
                KeyCode::BackTab | KeyCode::Up => {
                    app_state.active_field = (app_state.active_field + 4) % 5;
                    return Ok(());
                }
                KeyCode::Char(' ') if app_state.active_field == 4 => {
                    app_state.is_quick_app = !app_state.is_quick_app;
                    return Ok(());
                }
                KeyCode::Backspace => {
                    match app_state.active_field {
                        0 => { app_state.name_input.pop(); }
                        1 => { app_state.link_input.pop(); }
                        2 => { app_state.category_input.pop(); }
                        3 => { app_state.hotkey_input.pop(); }
                        _ => {}
                    }
                    return Ok(());
                }
                KeyCode::Char(c) => {
                    match app_state.active_field {
                        0 => app_state.name_input.push(c),
                        1 => app_state.link_input.push(c),
                        2 => app_state.category_input.push(c),
                        3 => {
                            if c.is_alphanumeric() {
                                app_state.hotkey_input = c.to_lowercase().to_string();
                            }
                        }
                        4 => {
                            if c == ' ' {
                                app_state.is_quick_app = !app_state.is_quick_app;
                            }
                        }
                        _ => {}
                    }
                    return Ok(());
                }
                KeyCode::Enter => {
                    let name = app_state.name_input.trim().to_string();
                    let link = app_state.link_input.trim().to_string();
                    if name.is_empty() || link.is_empty() {
                        app_state.error = Some("App name and Link/URI are required".to_string());
                        return Ok(());
                    }
                    let category = if app_state.category_input.trim().is_empty() {
                        "Custom".to_string()
                    } else {
                        app_state.category_input.trim().to_string()
                    };
                    let hotkey = app_state.hotkey_input.trim().chars().next();
                    let new_app = CustomApp {
                        name,
                        link,
                        category,
                        hotkey,
                        is_quick_app: app_state.is_quick_app,
                    };
                    save_app = Some((app_state.is_editing, app_state.editing_index, new_app));
                }
                _ => return Ok(()),
            }

            if let Some((is_editing, editing_index, app)) = save_app {
                let app_name = app.name.clone();
                if is_editing {
                    if let Some(idx) = editing_index {
                        let _ = state.config.update_app(idx, app);
                        state.log(LogLevel::Success, format!("Updated app: {}", app_name));
                    }
                } else {
                    let _ = state.config.add_app(app);
                    state.log(LogLevel::Success, format!("Added app: {}", app_name));
                }
                state.modal = ActiveModal::None;
            }
            return Ok(());
        }
        ActiveModal::MacroEditor(macro_state) => {
            let mut save_macro = None;
            match key.code {
                KeyCode::Esc => {
                    state.modal = ActiveModal::None;
                    return Ok(());
                }
                KeyCode::Tab | KeyCode::Down => {
                    macro_state.active_field = (macro_state.active_field + 1) % 3;
                    return Ok(());
                }
                KeyCode::BackTab | KeyCode::Up => {
                    macro_state.active_field = (macro_state.active_field + 2) % 3;
                    return Ok(());
                }
                KeyCode::Backspace => {
                    match macro_state.active_field {
                        0 => { macro_state.name_input.pop(); }
                        1 => { macro_state.description_input.pop(); }
                        2 => { macro_state.steps_input.pop(); }
                        _ => {}
                    }
                    return Ok(());
                }
                KeyCode::Char(c) => {
                    match macro_state.active_field {
                        0 => macro_state.name_input.push(c),
                        1 => macro_state.description_input.push(c),
                        2 => macro_state.steps_input.push(c),
                        _ => {}
                    }
                    return Ok(());
                }
                KeyCode::Enter => {
                    let name = macro_state.name_input.trim().to_string();
                    let steps_str = macro_state.steps_input.trim().to_string();
                    if name.is_empty() {
                        macro_state.error = Some("Macro name is required".to_string());
                        return Ok(());
                    }
                    if steps_str.is_empty() {
                        macro_state.error = Some("Steps sequence is required".to_string());
                        return Ok(());
                    }
                    let steps = AppConfig::parse_macro_steps(&steps_str);
                    let description = macro_state.description_input.trim().to_string();
                    let new_macro = SavedMacro {
                        name,
                        description,
                        steps,
                    };
                    save_macro = Some((macro_state.is_editing, macro_state.editing_index, new_macro));
                }
                _ => return Ok(()),
            }

            if let Some((is_editing, editing_index, m)) = save_macro {
                let macro_name = m.name.clone();
                if is_editing {
                    if let Some(idx) = editing_index {
                        let _ = state.config.update_macro(idx, m);
                        state.log(LogLevel::Success, format!("Updated macro: {}", macro_name));
                    }
                } else {
                    let _ = state.config.add_macro(m);
                    state.log(LogLevel::Success, format!("Added macro: {}", macro_name));
                }
                state.modal = ActiveModal::None;
            }
            return Ok(());
        }
        ActiveModal::None => {}
    }

    // 2. Global Hotkeys
    match key.code {
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('?') => {
            state.modal = ActiveModal::Help;
            return Ok(());
        }
        KeyCode::Char('1') => {
            state.active_tab = ActiveTab::Remote;
            return Ok(());
        }
        KeyCode::Char('2') => {
            state.active_tab = ActiveTab::Devices;
            return Ok(());
        }
        KeyCode::Char('3') => {
            state.active_tab = ActiveTab::Apps;
            return Ok(());
        }
        KeyCode::Char('4') => {
            state.active_tab = ActiveTab::TextInput;
            return Ok(());
        }
        KeyCode::Char('5') => {
            state.active_tab = ActiveTab::Macros;
            return Ok(());
        }
        KeyCode::Char('6') => {
            state.active_tab = ActiveTab::Logs;
            return Ok(());
        }
        _ => {}
    }

    // 3. Tab-Specific Key Handling
    match state.active_tab {
        ActiveTab::Remote => handle_remote_keys(key, state, client_cert, client_key, tv_event_tx).await?,
        ActiveTab::Devices => handle_devices_keys(key, state, client_cert, client_key, tv_event_tx, disc_tx).await?,
        ActiveTab::Apps => handle_apps_keys(key, state).await?,
        ActiveTab::TextInput => handle_text_input_keys(key, state).await?,
        ActiveTab::Macros => handle_macros_keys(key, state).await?,
        ActiveTab::Logs => handle_logs_keys(key, state),
    }

    Ok(())
}

async fn send_remote_key(state: &mut AppState, code: RemoteKeyCode) {
    state.last_action = Some(format!("Key: {:?}", code));
    state.log(LogLevel::Info, format!("Sending Key: {:?}", code));
    if let Some(client) = &state.client_handle {
        let _ = client.send_key(code, RemoteDirection::Short).await;
    }
}

async fn handle_remote_keys(
    key: crossterm::event::KeyEvent,
    state: &mut AppState,
    _client_cert: &openssl::x509::X509,
    _client_key: &openssl::pkey::PKey<openssl::pkey::Private>,
    _tv_event_tx: &mpsc::Sender<TvEvent>,
) -> Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => send_remote_key(state, RemoteKeyCode::KeycodeDpadUp).await,
        KeyCode::Down | KeyCode::Char('j') => send_remote_key(state, RemoteKeyCode::KeycodeDpadDown).await,
        KeyCode::Left | KeyCode::Char('h') => send_remote_key(state, RemoteKeyCode::KeycodeDpadLeft).await,
        KeyCode::Right | KeyCode::Char('l') => send_remote_key(state, RemoteKeyCode::KeycodeDpadRight).await,
        KeyCode::Enter => send_remote_key(state, RemoteKeyCode::KeycodeDpadCenter).await,
        KeyCode::Esc | KeyCode::Char('b') => send_remote_key(state, RemoteKeyCode::KeycodeBack).await,
        KeyCode::Char('H') => send_remote_key(state, RemoteKeyCode::KeycodeHome).await,
        KeyCode::Char('p') | KeyCode::Char('P') => send_remote_key(state, RemoteKeyCode::KeycodePower).await,
        KeyCode::Char('+') | KeyCode::Char('=') => send_remote_key(state, RemoteKeyCode::KeycodeVolumeUp).await,
        KeyCode::Char('-') | KeyCode::Char('_') => send_remote_key(state, RemoteKeyCode::KeycodeVolumeDown).await,
        KeyCode::Char('m') => send_remote_key(state, RemoteKeyCode::KeycodeVolumeMute).await,
        KeyCode::Char(' ') => send_remote_key(state, RemoteKeyCode::KeycodeMediaPlayPause).await,
        KeyCode::Char('[') => send_remote_key(state, RemoteKeyCode::KeycodeMediaRewind).await,
        KeyCode::Char(']') => send_remote_key(state, RemoteKeyCode::KeycodeMediaFastForward).await,
        KeyCode::Char('/') => send_remote_key(state, RemoteKeyCode::KeycodeSearch).await,
        KeyCode::Char('i') => send_remote_key(state, RemoteKeyCode::KeycodeTvInput).await,
        KeyCode::Char('s') => send_remote_key(state, RemoteKeyCode::KeycodeSettings).await,
        // Dynamic Quick App Hotkeys
        KeyCode::Char(c) => {
            let matched_app = state
                .config
                .custom_apps
                .iter()
                .find(|a| a.is_quick_app && a.hotkey == Some(c.to_ascii_lowercase()))
                .map(|a| (a.name.clone(), a.link.clone()));

            if let Some((name, link)) = matched_app {
                if let Some(client) = &state.client_handle {
                    let _ = client.launch_app(link).await;
                    state.log(LogLevel::Success, format!("Launched {}", name));
                }
            }
        }
        KeyCode::Tab => {
            let total = RemoteButton::all().len();
            state.selected_remote_button = (state.selected_remote_button + 1) % total;
        }
        KeyCode::BackTab => {
            let total = RemoteButton::all().len();
            state.selected_remote_button = (state.selected_remote_button + total - 1) % total;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_devices_keys(
    key: crossterm::event::KeyEvent,
    state: &mut AppState,
    client_cert: &openssl::x509::X509,
    client_key: &openssl::pkey::PKey<openssl::pkey::Private>,
    tv_event_tx: &mpsc::Sender<TvEvent>,
    disc_tx: &mpsc::Sender<DiscoveredDevice>,
) -> Result<()> {
    match key.code {
        KeyCode::Tab => {
            state.devices_focus_saved = !state.devices_focus_saved;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if state.devices_focus_saved {
                if state.selected_saved_idx > 0 {
                    state.selected_saved_idx -= 1;
                }
            } else if state.selected_discovered_idx > 0 {
                state.selected_discovered_idx -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.devices_focus_saved {
                if state.selected_saved_idx + 1 < state.config.devices.len() {
                    state.selected_saved_idx += 1;
                }
            } else if state.selected_discovered_idx + 1 < state.discovered_devices.len() {
                state.selected_discovered_idx += 1;
            }
        }
        KeyCode::Char('s') => {
            state.is_scanning = true;
            state.discovered_devices.clear();
            state.log(LogLevel::Info, "Initiating mDNS discovery scan...");
            let _ = DiscoveryManager::start_discovery(disc_tx.clone(), Duration::from_secs(8)).await;
        }
        KeyCode::Char('a') => {
            state.modal = ActiveModal::AddDevice(AddDeviceModalState::default());
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if state.devices_focus_saved && !state.config.devices.is_empty() {
                let ip = state.config.devices[state.selected_saved_idx].ip.clone();
                state.config.remove_device(&ip);
                state.log(LogLevel::Warning, format!("Removed device {}", ip));
                if state.selected_saved_idx >= state.config.devices.len() && state.selected_saved_idx > 0 {
                    state.selected_saved_idx -= 1;
                }
            }
        }
        KeyCode::Enter | KeyCode::Char('p') => {
            let target_ip = if state.devices_focus_saved {
                state.config.devices.get(state.selected_saved_idx).map(|d| (d.name.clone(), d.ip.clone(), d.paired))
            } else {
                state.discovered_devices.get(state.selected_discovered_idx).map(|d| {
                    let paired = state.config.devices.iter().any(|s| s.ip == d.ip.to_string() && s.paired);
                    (d.name.clone(), d.ip.to_string(), paired)
                })
            };

            if let Some((name, ip_str, paired)) = target_ip {
                state.config.set_active_device(&ip_str);

                if paired && key.code == KeyCode::Enter {
                    // Connect directly
                    if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
                        let target = SocketAddr::new(ip, 6466);
                        let cert_c = client_cert.clone();
                        let key_c = client_key.clone();
                        let tx_c = tv_event_tx.clone();
                        state.connection_status = ConnectionStatus::Connecting(ip_str.clone());
                        state.log(LogLevel::Info, format!("Connecting to {} ({})", name, ip_str));

                        let handle_res = TvConnection::connect(target, cert_c, key_c, tx_c).await;
                        match handle_res {
                            Ok(handle) => {
                                state.client_handle = Some(handle);
                                state.active_tab = ActiveTab::Remote;
                            }
                            Err(e) => {
                                state.connection_status = ConnectionStatus::Error(format!("{}", e));
                                state.log(LogLevel::Error, format!("Failed to connect: {}", e));
                            }
                        }
                    }
                } else {
                    // Trigger Pairing Handshake!
                    if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
                        let target = SocketAddr::new(ip, PAIRING_PORT);
                        state.connection_status = ConnectionStatus::Pairing(ip_str.clone());
                        state.log(LogLevel::Info, format!("Starting pairing handshake with {} ({})", name, ip_str));

                        let cert_c = client_cert.clone();
                        let key_c = client_key.clone();
                        let modal_name = name.clone();
                        let modal_ip = ip_str.clone();

                        match PairingSession::start(target, "Google TV TUI", cert_c, key_c).await {
                            Ok(session) => {
                                state.modal = ActiveModal::Pairing(PairingModalState {
                                    ip: modal_ip,
                                    name: modal_name,
                                    pin_input: String::new(),
                                    error: None,
                                    session: Some(Arc::new(Mutex::new(Some(session)))),
                                    is_submitting: false,
                                });
                            }
                            Err(e) => {
                                state.log(LogLevel::Error, format!("Pairing handshake initiation failed: {}", e));
                                state.connection_status = ConnectionStatus::Error(format!("{}", e));
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_apps_keys(key: crossterm::event::KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        KeyCode::Up => {
            if state.selected_app_idx > 0 {
                state.selected_app_idx -= 1;
            }
        }
        KeyCode::Down => {
            let total = state.filtered_apps().len();
            if total > 0 && state.selected_app_idx + 1 < total {
                state.selected_app_idx += 1;
            }
        }
        KeyCode::Esc => {
            state.apps_search.clear();
            state.selected_app_idx = 0;
        }
        KeyCode::Backspace => {
            state.apps_search.pop();
            state.selected_app_idx = 0;
        }
        KeyCode::Char('u') if state.apps_search.is_empty() => {
            state.modal = ActiveModal::LaunchUrl(LaunchUrlModalState::default());
        }
        KeyCode::Char('a') if state.apps_search.is_empty() => {
            state.modal = ActiveModal::AppEditor(AppModalState {
                is_editing: false,
                editing_index: None,
                name_input: String::new(),
                link_input: String::new(),
                category_input: "Video".to_string(),
                hotkey_input: String::new(),
                is_quick_app: true,
                active_field: 0,
                error: None,
            });
        }
        KeyCode::Char('e') if state.apps_search.is_empty() => {
            let filtered = state.filtered_apps();
            if let Some(app) = filtered.get(state.selected_app_idx) {
                let real_idx = state.config.custom_apps.iter().position(|a| a.name == app.name && a.link == app.link);
                if let Some(idx) = real_idx {
                    let app = &state.config.custom_apps[idx];
                    state.modal = ActiveModal::AppEditor(AppModalState {
                        is_editing: true,
                        editing_index: Some(idx),
                        name_input: app.name.clone(),
                        link_input: app.link.clone(),
                        category_input: app.category.clone(),
                        hotkey_input: app.hotkey.map(|c| c.to_string()).unwrap_or_default(),
                        is_quick_app: app.is_quick_app,
                        active_field: 0,
                        error: None,
                    });
                }
            }
        }
        KeyCode::Char('d') if state.apps_search.is_empty() => {
            let filtered = state.filtered_apps();
            if let Some(app) = filtered.get(state.selected_app_idx) {
                let real_idx = state.config.custom_apps.iter().position(|a| a.name == app.name && a.link == app.link);
                if let Some(idx) = real_idx {
                    let name = state.config.custom_apps[idx].name.clone();
                    let _ = state.config.delete_app(idx);
                    state.log(LogLevel::Warning, format!("Deleted app: {}", name));
                    if state.selected_app_idx >= state.filtered_apps().len() && state.selected_app_idx > 0 {
                        state.selected_app_idx -= 1;
                    }
                }
            }
        }
        KeyCode::Delete => {
            let filtered = state.filtered_apps();
            if let Some(app) = filtered.get(state.selected_app_idx) {
                let real_idx = state.config.custom_apps.iter().position(|a| a.name == app.name && a.link == app.link);
                if let Some(idx) = real_idx {
                    let name = state.config.custom_apps[idx].name.clone();
                    let _ = state.config.delete_app(idx);
                    state.log(LogLevel::Warning, format!("Deleted app: {}", name));
                    if state.selected_app_idx >= state.filtered_apps().len() && state.selected_app_idx > 0 {
                        state.selected_app_idx -= 1;
                    }
                }
            }
        }
        KeyCode::Char(' ') if state.apps_search.is_empty() => {
            let filtered = state.filtered_apps();
            if let Some(app) = filtered.get(state.selected_app_idx) {
                let real_idx = state.config.custom_apps.iter().position(|a| a.name == app.name && a.link == app.link);
                if let Some(idx) = real_idx {
                    let _ = state.config.toggle_quick_app(idx);
                    let is_quick = state.config.custom_apps[idx].is_quick_app;
                    let name = &state.config.custom_apps[idx].name;
                    state.log(
                        LogLevel::Info,
                        format!(
                            "{} {} remote quick launch",
                            if is_quick { "Added" } else { "Removed" },
                            name
                        ),
                    );
                }
            }
        }
        KeyCode::Char(c) => {
            state.apps_search.push(c);
            state.selected_app_idx = 0;
        }
        KeyCode::Enter => {
            let filtered = state.filtered_apps();
            if let Some(app) = filtered.get(state.selected_app_idx) {
                let link = app.link.clone();
                let name = app.name.clone();
                if let Some(client) = &state.client_handle {
                    let _ = client.launch_app(link).await;
                    state.log(LogLevel::Success, format!("Launched {}", name));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_text_input_keys(key: crossterm::event::KeyEvent, state: &mut AppState) -> Result<()> {
    // 1. Control modifier shortcuts
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('s') => {
                // TV Global Search overlay
                let text = if !state.text_input_buffer.trim().is_empty() {
                    state.text_input_buffer.trim().to_string()
                } else if let Some(h) = state.text_input_history.get(state.selected_history_idx) {
                    h.clone()
                } else {
                    String::new()
                };
                if !text.is_empty() {
                    if let Some(client) = &state.client_handle {
                        let _ = client.search_on_tv(text.clone()).await;
                        state.log(
                            LogLevel::Success,
                            format!("Started TV Search for: \"{}\"", text),
                        );
                        state.add_search_history(&text);
                        state.text_input_buffer.clear();
                    } else {
                        state.log(LogLevel::Warning, "Not connected to TV (connect first in Devices tab)");
                    }
                }
                return Ok(());
            }
            KeyCode::Char('d') => {
                // Delete selected history item
                if let Some(removed) = state.delete_selected_history() {
                    state.log(LogLevel::Warning, format!("Deleted history item: \"{}\"", removed));
                    if let Some(h) = state.text_input_history.get(state.selected_history_idx) {
                        state.text_input_buffer = h.clone();
                    } else {
                        state.text_input_buffer.clear();
                    }
                }
                return Ok(());
            }
            KeyCode::Char('x') => {
                // Clear all history
                state.clear_all_history();
                state.text_input_buffer.clear();
                state.log(LogLevel::Warning, "Cleared all search history.");
                return Ok(());
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Delete => {
            // Delete selected history item
            if let Some(removed) = state.delete_selected_history() {
                state.log(LogLevel::Warning, format!("Deleted history item: \"{}\"", removed));
                if let Some(h) = state.text_input_history.get(state.selected_history_idx) {
                    state.text_input_buffer = h.clone();
                } else {
                    state.text_input_buffer.clear();
                }
            }
        }
        KeyCode::Tab => {
            state.text_input_mode = state.text_input_mode.next();
            state.log(
                LogLevel::Info,
                format!("Switched text mode: {}", state.text_input_mode.label()),
            );
        }
        KeyCode::Up => {
            if state.selected_history_idx > 0 {
                state.selected_history_idx -= 1;
                if let Some(h) = state.text_input_history.get(state.selected_history_idx) {
                    state.text_input_buffer = h.clone();
                }
            }
        }
        KeyCode::Down => {
            if state.selected_history_idx + 1 < state.text_input_history.len() {
                state.selected_history_idx += 1;
                if let Some(h) = state.text_input_history.get(state.selected_history_idx) {
                    state.text_input_buffer = h.clone();
                }
            }
        }
        KeyCode::Esc => {
            state.text_input_buffer.clear();
        }
        KeyCode::Backspace => {
            state.text_input_buffer.pop();
        }
        KeyCode::Enter => {
            let press_enter = key.modifiers.contains(KeyModifiers::SHIFT);
            let text = if !state.text_input_buffer.trim().is_empty() {
                state.text_input_buffer.trim().to_string()
            } else if let Some(h) = state.text_input_history.get(state.selected_history_idx) {
                h.clone()
            } else {
                String::new()
            };

            if !text.is_empty() {
                if let Some(client) = &state.client_handle {
                    match state.text_input_mode {
                        TextInputMode::Smart => {
                            if press_enter {
                                let _ = client.send_text_with_enter(text.clone()).await;
                            } else {
                                let _ = client.send_text(text.clone()).await;
                            }
                        }
                        TextInputMode::Keystroke => {
                            let _ = client.send_text_keys(text.clone(), press_enter).await;
                        }
                        TextInputMode::Ime => {
                            let _ = client.send_text_ime(text.clone(), press_enter).await;
                        }
                    }
                    state.log(
                        LogLevel::Success,
                        format!(
                            "Sent text [{}{}] to TV: \"{}\"",
                            state.text_input_mode.label(),
                            if press_enter { " + Enter" } else { "" },
                            text
                        ),
                    );
                    state.add_search_history(&text);
                    state.text_input_buffer.clear();
                } else {
                    state.log(LogLevel::Warning, "Not connected to TV (connect first in Devices tab)");
                }
            }
        }
        KeyCode::Char(c) => {
            state.text_input_buffer.push(c);
        }
        _ => {}
    }
    Ok(())
}

async fn handle_macros_keys(key: crossterm::event::KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.selected_macro_idx > 0 {
                state.selected_macro_idx -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.selected_macro_idx + 1 < state.config.macros.len() {
                state.selected_macro_idx += 1;
            }
        }
        KeyCode::Char('a') => {
            state.modal = ActiveModal::MacroEditor(MacroModalState {
                is_editing: false,
                editing_index: None,
                name_input: String::new(),
                description_input: String::new(),
                steps_input: String::new(),
                active_field: 0,
                error: None,
            });
        }
        KeyCode::Char('e') => {
            if let Some(m) = state.config.macros.get(state.selected_macro_idx) {
                state.modal = ActiveModal::MacroEditor(MacroModalState {
                    is_editing: true,
                    editing_index: Some(state.selected_macro_idx),
                    name_input: m.name.clone(),
                    description_input: m.description.clone(),
                    steps_input: AppConfig::format_macro_steps(&m.steps),
                    active_field: 0,
                    error: None,
                });
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if !state.config.macros.is_empty() && state.selected_macro_idx < state.config.macros.len() {
                let name = state.config.macros[state.selected_macro_idx].name.clone();
                let _ = state.config.delete_macro(state.selected_macro_idx);
                state.log(LogLevel::Warning, format!("Deleted macro: {}", name));
                if state.selected_macro_idx >= state.config.macros.len() && state.selected_macro_idx > 0 {
                    state.selected_macro_idx -= 1;
                }
            }
        }
        KeyCode::Enter => {
            if let Some(m) = state.config.macros.get(state.selected_macro_idx).cloned() {
                state.log(LogLevel::Info, format!("Running macro: {}", m.name));
                state.running_macro = Some(m.name.clone());

                if let Some(client) = state.client_handle.clone() {
                    tokio::spawn(async move {
                        for step in m.steps {
                            if step.action.starts_with("key:") {
                                if let Some(code) = parse_keycode(&step.action[4..]) {
                                    let _ = client.send_key(code, RemoteDirection::Short).await;
                                }
                            } else if step.action.starts_with("wait:") {
                                if let Ok(ms) = step.action[5..].parse::<u64>() {
                                    tokio::time::sleep(Duration::from_millis(ms)).await;
                                }
                            } else if step.action.starts_with("app:") {
                                let _ = client.launch_app(step.action[4..].to_string()).await;
                            } else if step.action.starts_with("text:") {
                                let _ = client.send_text(step.action[5..].to_string()).await;
                            } else if step.action.starts_with("keys:") {
                                let _ = client.send_text_keys(step.action[5..].to_string(), false).await;
                            } else if step.action.starts_with("ime:") {
                                let _ = client.send_text_ime(step.action[4..].to_string(), false).await;
                            } else if step.action.starts_with("search:") {
                                let _ = client.search_on_tv(step.action[7..].to_string()).await;
                            }
                        }
                    });
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_logs_keys(key: crossterm::event::KeyEvent, state: &mut AppState) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.logs_scroll = state.logs_scroll.saturating_add(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.logs_scroll = state.logs_scroll.saturating_sub(1);
        }
        KeyCode::PageUp => {
            state.logs_scroll = state.logs_scroll.saturating_add(10);
        }
        KeyCode::PageDown => {
            state.logs_scroll = state.logs_scroll.saturating_sub(10);
        }
        KeyCode::Char('c') => {
            state.logs.clear();
            state.logs_scroll = 0;
            state.log(LogLevel::Info, "Logs cleared.");
        }
        _ => {}
    }
}
