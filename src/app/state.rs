use chrono::{DateTime, Local};
use std::collections::VecDeque;

use crate::config::{AppConfig, CustomApp};
use crate::net::client::{DeviceDetails, TvClientHandle};
use crate::net::discovery::DiscoveredDevice;
use crate::net::pairing::PairingSession;
use crate::proto::remote::RemoteKeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Remote,
    Devices,
    Apps,
    TextInput,
    Macros,
    Logs,
}

impl ActiveTab {
    pub fn index(&self) -> usize {
        match self {
            Self::Remote => 0,
            Self::Devices => 1,
            Self::Apps => 2,
            Self::TextInput => 3,
            Self::Macros => 4,
            Self::Logs => 5,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Remote,
            1 => Self::Devices,
            2 => Self::Apps,
            3 => Self::TextInput,
            4 => Self::Macros,
            _ => Self::Logs,
        }
    }

    pub fn titles() -> Vec<&'static str> {
        vec![
            "[1] Remote",
            "[2] Devices",
            "[3] Apps",
            "[4] Text Input",
            "[5] Macros",
            "[6] Logs",
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting(String),
    Connected {
        ip: String,
        name: String,
    },
    Pairing(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub time: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
    Protocol,
}

#[derive(Clone)]
pub struct PairingModalState {
    pub ip: String,
    pub name: String,
    pub pin_input: String,
    pub error: Option<String>,
    pub session: Option<std::sync::Arc<tokio::sync::Mutex<Option<PairingSession>>>>,
    pub is_submitting: bool,
}

impl std::fmt::Debug for PairingModalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingModalState")
            .field("ip", &self.ip)
            .field("name", &self.name)
            .field("pin_input", &self.pin_input)
            .field("error", &self.error)
            .field("is_submitting", &self.is_submitting)
            .finish()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AddDeviceModalState {
    pub ip_input: String,
    pub name_input: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LaunchUrlModalState {
    pub url_input: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AppModalState {
    pub is_editing: bool,
    pub editing_index: Option<usize>,
    pub name_input: String,
    pub link_input: String,
    pub category_input: String,
    pub hotkey_input: String,
    pub is_quick_app: bool,
    pub active_field: usize, // 0 = Name, 1 = Link, 2 = Category, 3 = Hotkey, 4 = QuickApp
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MacroModalState {
    pub is_editing: bool,
    pub editing_index: Option<usize>,
    pub name_input: String,
    pub description_input: String,
    pub steps_input: String,
    pub active_field: usize, // 0 = Name, 1 = Description, 2 = Steps
    pub error: Option<String>,
}

pub enum ActiveModal {
    None,
    Help,
    Pairing(PairingModalState),
    AddDevice(AddDeviceModalState),
    LaunchUrl(LaunchUrlModalState),
    AppEditor(AppModalState),
    MacroEditor(MacroModalState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteButton {
    Power,
    TvInput,
    Settings,
    Up,
    Left,
    Ok,
    Right,
    Down,
    Back,
    Home,
    Search,
    VolDown,
    Mute,
    VolUp,
    Rewind,
    PlayPause,
    FastForward,
    ChanDown,
    ChanUp,
    AppYouTube,
    AppNetflix,
    AppPrime,
    AppDisney,
    AppSpotify,
}

impl RemoteButton {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Power,
            Self::TvInput,
            Self::Settings,
            Self::Up,
            Self::Left,
            Self::Ok,
            Self::Right,
            Self::Down,
            Self::Back,
            Self::Home,
            Self::Search,
            Self::VolDown,
            Self::Mute,
            Self::VolUp,
            Self::Rewind,
            Self::PlayPause,
            Self::FastForward,
            Self::ChanDown,
            Self::ChanUp,
            Self::AppYouTube,
            Self::AppNetflix,
            Self::AppPrime,
            Self::AppDisney,
            Self::AppSpotify,
        ]
    }

    pub fn keycode(&self) -> Option<RemoteKeyCode> {
        match self {
            Self::Power => Some(RemoteKeyCode::KeycodePower),
            Self::TvInput => Some(RemoteKeyCode::KeycodeTvInput),
            Self::Settings => Some(RemoteKeyCode::KeycodeSettings),
            Self::Up => Some(RemoteKeyCode::KeycodeDpadUp),
            Self::Left => Some(RemoteKeyCode::KeycodeDpadLeft),
            Self::Ok => Some(RemoteKeyCode::KeycodeDpadCenter),
            Self::Right => Some(RemoteKeyCode::KeycodeDpadRight),
            Self::Down => Some(RemoteKeyCode::KeycodeDpadDown),
            Self::Back => Some(RemoteKeyCode::KeycodeBack),
            Self::Home => Some(RemoteKeyCode::KeycodeHome),
            Self::Search => Some(RemoteKeyCode::KeycodeSearch),
            Self::VolDown => Some(RemoteKeyCode::KeycodeVolumeDown),
            Self::Mute => Some(RemoteKeyCode::KeycodeVolumeMute),
            Self::VolUp => Some(RemoteKeyCode::KeycodeVolumeUp),
            Self::Rewind => Some(RemoteKeyCode::KeycodeMediaRewind),
            Self::PlayPause => Some(RemoteKeyCode::KeycodeMediaPlayPause),
            Self::FastForward => Some(RemoteKeyCode::KeycodeMediaFastForward),
            Self::ChanDown => Some(RemoteKeyCode::KeycodeChannelDown),
            Self::ChanUp => Some(RemoteKeyCode::KeycodeChannelUp),
            _ => None,
        }
    }

    pub fn app_link(&self) -> Option<&'static str> {
        match self {
            Self::AppYouTube => Some("https://www.youtube.com"),
            Self::AppNetflix => Some("netflix://"),
            Self::AppPrime => Some("https://app.primevideo.com"),
            Self::AppDisney => Some("https://www.disneyplus.com"),
            Self::AppSpotify => Some("spotify://"),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Power => "Power (p)",
            Self::TvInput => "Input (i)",
            Self::Settings => "Settings (s)",
            Self::Up => "▲ Up (k)",
            Self::Left => "◀ Left (h)",
            Self::Ok => "● OK (Enter)",
            Self::Right => "▶ Right (l)",
            Self::Down => "▼ Down (j)",
            Self::Back => "⎋ Back (Esc)",
            Self::Home => "⌂ Home (H)",
            Self::Search => "🔍 Search (/)",
            Self::VolDown => "🔉 Vol- (-)",
            Self::Mute => "🔇 Mute (m)",
            Self::VolUp => "🔊 Vol+ (+)",
            Self::Rewind => "⏪ Rewind ([)",
            Self::PlayPause => "⏯ Play/Pause (Space)",
            Self::FastForward => "⏩ Fwd (])",
            Self::ChanDown => "CH-",
            Self::ChanUp => "CH+",
            Self::AppYouTube => "YouTube (y)",
            Self::AppNetflix => "Netflix (n)",
            Self::AppPrime => "Prime (r)",
            Self::AppDisney => "Disney+ (d)",
            Self::AppSpotify => "Spotify",
        }
    }
}

pub struct AppState {
    pub config: AppConfig,
    pub active_tab: ActiveTab,
    pub modal: ActiveModal,
    pub should_quit: bool,

    // Connection
    pub connection_status: ConnectionStatus,
    pub client_handle: Option<TvClientHandle>,
    pub device_details: Option<DeviceDetails>,
    pub is_on: Option<bool>,
    pub volume_level: u32,
    pub volume_max: u32,
    pub volume_muted: bool,
    pub current_app: String,

    // Devices View
    pub discovered_devices: Vec<DiscoveredDevice>,
    pub is_scanning: bool,
    pub selected_discovered_idx: usize,
    pub selected_saved_idx: usize,
    pub devices_focus_saved: bool, // true = focusing saved list, false = discovered

    // Remote View
    pub selected_remote_button: usize,
    pub last_action: Option<String>,

    // Apps View
    pub apps_search: String,
    pub selected_app_idx: usize,

    // Text Input View
    pub text_input_buffer: String,
    pub text_input_history: Vec<String>,
    pub selected_history_idx: usize,

    // Macros View
    pub selected_macro_idx: usize,
    pub running_macro: Option<String>,

    // Logs View
    pub logs: VecDeque<LogEntry>,
    pub logs_scroll: usize,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let mut state = Self {
            config,
            active_tab: ActiveTab::Remote,
            modal: ActiveModal::None,
            should_quit: false,

            connection_status: ConnectionStatus::Disconnected,
            client_handle: None,
            device_details: None,
            is_on: None,
            volume_level: 10,
            volume_max: 25,
            volume_muted: false,
            current_app: String::new(),

            discovered_devices: Vec::new(),
            is_scanning: false,
            selected_discovered_idx: 0,
            selected_saved_idx: 0,
            devices_focus_saved: true,

            selected_remote_button: 5, // Default to OK button
            last_action: None,

            apps_search: String::new(),
            selected_app_idx: 0,

            text_input_buffer: String::new(),
            text_input_history: vec![
                "4K HDR OLED Demo".into(),
                "Synthwave chill mix".into(),
                "Nature documentary".into(),
                "Lo-fi hip hop radio".into(),
            ],
            selected_history_idx: 0,

            selected_macro_idx: 0,
            running_macro: None,

            logs: VecDeque::with_capacity(300),
            logs_scroll: 0,
        };

        state.log(LogLevel::Info, "Google TV Remote TUI initialized.");
        state
    }

    pub fn log(&mut self, level: LogLevel, msg: impl Into<String>) {
        if self.logs.len() >= 300 {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            time: Local::now(),
            level,
            message: msg.into(),
        });
    }

    pub fn filtered_apps(&self) -> Vec<&CustomApp> {
        let query = self.apps_search.trim().to_lowercase();
        if query.is_empty() {
            self.config.custom_apps.iter().collect()
        } else {
            self.config
                .custom_apps
                .iter()
                .filter(|a| {
                    a.name.to_lowercase().contains(&query)
                        || a.category.to_lowercase().contains(&query)
                })
                .collect()
        }
    }
}
