use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedDevice {
    pub name: String,
    pub ip: String,
    pub paired: bool,
    pub last_connected: Option<DateTime<Utc>>,
    pub vendor: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomApp {
    pub name: String,
    pub link: String,
    pub category: String,
    pub hotkey: Option<char>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroStep {
    pub action: String, // "key:KEYCODE_POWER", "wait:2000", "app:https://youtube.com", "text:Hello"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedMacro {
    pub name: String,
    pub description: String,
    pub steps: Vec<MacroStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_device_ip: Option<String>,
    pub devices: Vec<SavedDevice>,
    pub custom_apps: Vec<CustomApp>,
    pub macros: Vec<SavedMacro>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_device_ip: None,
            devices: Vec::new(),
            custom_apps: default_apps(),
            macros: default_macros(),
        }
    }
}

pub fn default_apps() -> Vec<CustomApp> {
    vec![
        CustomApp {
            name: "YouTube".into(),
            link: "https://www.youtube.com".into(),
            category: "Video".into(),
            hotkey: Some('y'),
        },
        CustomApp {
            name: "Netflix".into(),
            link: "netflix://".into(),
            category: "Video".into(),
            hotkey: Some('n'),
        },
        CustomApp {
            name: "Prime Video".into(),
            link: "https://app.primevideo.com".into(),
            category: "Video".into(),
            hotkey: Some('p'),
        },
        CustomApp {
            name: "Disney+".into(),
            link: "https://www.disneyplus.com".into(),
            category: "Video".into(),
            hotkey: Some('d'),
        },
        CustomApp {
            name: "Spotify".into(),
            link: "spotify://".into(),
            category: "Music".into(),
            hotkey: Some('s'),
        },
        CustomApp {
            name: "Plex".into(),
            link: "plex://".into(),
            category: "Media".into(),
            hotkey: Some('x'),
        },
        CustomApp {
            name: "Twitch".into(),
            link: "twitch://".into(),
            category: "Streaming".into(),
            hotkey: Some('t'),
        },
        CustomApp {
            name: "SmartTube".into(),
            link: "smarttube://".into(),
            category: "Video".into(),
            hotkey: None,
        },
        CustomApp {
            name: "Kodi".into(),
            link: "kodi://".into(),
            category: "Media".into(),
            hotkey: Some('k'),
        },
        CustomApp {
            name: "VLC".into(),
            link: "org.videolan.vlc://".into(),
            category: "Media".into(),
            hotkey: Some('v'),
        },
    ]
}

pub fn default_macros() -> Vec<SavedMacro> {
    vec![
        SavedMacro {
            name: "Movie Night".into(),
            description: "Wake TV, wait 2s, and launch Netflix".into(),
            steps: vec![
                MacroStep {
                    action: "key:KEYCODE_WAKEUP".into(),
                    description: "Wake TV".into(),
                },
                MacroStep {
                    action: "wait:2000".into(),
                    description: "Wait for system wake (2s)".into(),
                },
                MacroStep {
                    action: "app:netflix://".into(),
                    description: "Launch Netflix".into(),
                },
            ],
        },
        SavedMacro {
            name: "Chill Music".into(),
            description: "Wake TV, launch Spotify, set moderate volume".into(),
            steps: vec![
                MacroStep {
                    action: "key:KEYCODE_WAKEUP".into(),
                    description: "Wake TV".into(),
                },
                MacroStep {
                    action: "wait:1500".into(),
                    description: "Wait 1.5s".into(),
                },
                MacroStep {
                    action: "app:spotify://".into(),
                    description: "Launch Spotify".into(),
                },
            ],
        },
        SavedMacro {
            name: "Bedtime Sleep".into(),
            description: "Return to Home screen and put TV to sleep".into(),
            steps: vec![
                MacroStep {
                    action: "key:KEYCODE_HOME".into(),
                    description: "Go to Home screen".into(),
                },
                MacroStep {
                    action: "wait:1000".into(),
                    description: "Wait 1s".into(),
                },
                MacroStep {
                    action: "key:KEYCODE_SLEEP".into(),
                    description: "Put TV into Sleep mode".into(),
                },
            ],
        },
    ]
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "googletv", "googletv-tui")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home).join(".config").join("googletv-tui")
            })
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(cfg) = serde_json::from_slice::<AppConfig>(&bytes) {
                    return cfg;
                }
            }
        }
        let cfg = Self::default();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir).context("Failed to create config directory")?;
        let bytes = serde_json::to_vec_pretty(self).context("Failed to serialize config")?;
        fs::write(Self::config_path(), bytes).context("Failed to write config file")?;
        Ok(())
    }

    pub fn add_or_update_device(
        &mut self,
        name: &str,
        ip: &str,
        paired: bool,
        vendor: Option<String>,
        model: Option<String>,
    ) {
        if let Some(existing) = self.devices.iter_mut().find(|d| d.ip == ip) {
            existing.name = name.to_string();
            existing.paired = paired;
            if vendor.is_some() {
                existing.vendor = vendor;
            }
            if model.is_some() {
                existing.model = model;
            }
            existing.last_connected = Some(Utc::now());
        } else {
            self.devices.push(SavedDevice {
                name: name.to_string(),
                ip: ip.to_string(),
                paired,
                last_connected: Some(Utc::now()),
                vendor,
                model,
            });
        }
        let _ = self.save();
    }

    pub fn remove_device(&mut self, ip: &str) {
        self.devices.retain(|d| d.ip != ip);
        if self.active_device_ip.as_deref() == Some(ip) {
            self.active_device_ip = self.devices.first().map(|d| d.ip.clone());
        }
        let _ = self.save();
    }

    pub fn set_active_device(&mut self, ip: &str) {
        self.active_device_ip = Some(ip.to_string());
        if let Some(dev) = self.devices.iter_mut().find(|d| d.ip == ip) {
            dev.last_connected = Some(Utc::now());
        }
        let _ = self.save();
    }
}
