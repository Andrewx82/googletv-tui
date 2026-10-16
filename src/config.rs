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

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomApp {
    pub name: String,
    pub link: String,
    pub category: String,
    pub hotkey: Option<char>,
    #[serde(default = "default_true")]
    pub is_quick_app: bool,
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
            is_quick_app: true,
        },
        CustomApp {
            name: "Netflix".into(),
            link: "netflix://".into(),
            category: "Video".into(),
            hotkey: Some('n'),
            is_quick_app: true,
        },
        CustomApp {
            name: "Prime Video".into(),
            link: "https://app.primevideo.com".into(),
            category: "Video".into(),
            hotkey: Some('r'),
            is_quick_app: true,
        },
        CustomApp {
            name: "Disney+".into(),
            link: "https://www.disneyplus.com".into(),
            category: "Video".into(),
            hotkey: Some('d'),
            is_quick_app: true,
        },
        CustomApp {
            name: "Spotify".into(),
            link: "spotify://".into(),
            category: "Music".into(),
            hotkey: Some('o'),
            is_quick_app: true,
        },
        CustomApp {
            name: "Plex".into(),
            link: "plex://".into(),
            category: "Media".into(),
            hotkey: Some('x'),
            is_quick_app: true,
        },
        CustomApp {
            name: "Twitch".into(),
            link: "twitch://".into(),
            category: "Streaming".into(),
            hotkey: Some('t'),
            is_quick_app: false,
        },
        CustomApp {
            name: "SmartTube".into(),
            link: "smarttube://".into(),
            category: "Video".into(),
            hotkey: None,
            is_quick_app: false,
        },
        CustomApp {
            name: "Kodi".into(),
            link: "kodi://".into(),
            category: "Media".into(),
            hotkey: None,
            is_quick_app: false,
        },
        CustomApp {
            name: "VLC".into(),
            link: "org.videolan.vlc://".into(),
            category: "Media".into(),
            hotkey: None,
            is_quick_app: false,
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

    pub fn get_last_tv(&self) -> Option<(String, String)> {
        // 1. If active_device_ip is set, check if it matches a paired saved device
        if let Some(active_ip) = &self.active_device_ip {
            if let Some(dev) = self.devices.iter().find(|d| &d.ip == active_ip && d.paired) {
                return Some((dev.name.clone(), dev.ip.clone()));
            }
            if !active_ip.is_empty() {
                let name = self
                    .devices
                    .iter()
                    .find(|d| &d.ip == active_ip)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| "Google TV".to_string());
                return Some((name, active_ip.clone()));
            }
        }

        // 2. Fall back to the most recently connected paired device
        if let Some(dev) = self
            .devices
            .iter()
            .filter(|d| d.paired)
            .max_by_key(|d| d.last_connected)
        {
            return Some((dev.name.clone(), dev.ip.clone()));
        }

        // 3. Fall back to any paired device
        if let Some(dev) = self.devices.iter().find(|d| d.paired) {
            return Some((dev.name.clone(), dev.ip.clone()));
        }

        // 4. Fall back to any saved device in the list
        self.devices.first().map(|d| (d.name.clone(), d.ip.clone()))
    }

    pub fn add_app(&mut self, app: CustomApp) -> Result<()> {
        self.custom_apps.push(app);
        self.save()
    }

    pub fn update_app(&mut self, index: usize, app: CustomApp) -> Result<()> {
        if index < self.custom_apps.len() {
            self.custom_apps[index] = app;
            self.save()?;
        }
        Ok(())
    }

    pub fn delete_app(&mut self, index: usize) -> Result<()> {
        if index < self.custom_apps.len() {
            self.custom_apps.remove(index);
            self.save()?;
        }
        Ok(())
    }

    pub fn toggle_quick_app(&mut self, index: usize) -> Result<()> {
        if let Some(app) = self.custom_apps.get_mut(index) {
            app.is_quick_app = !app.is_quick_app;
            self.save()?;
        }
        Ok(())
    }

    pub fn add_macro(&mut self, m: SavedMacro) -> Result<()> {
        self.macros.push(m);
        self.save()
    }

    pub fn update_macro(&mut self, index: usize, m: SavedMacro) -> Result<()> {
        if index < self.macros.len() {
            self.macros[index] = m;
            self.save()?;
        }
        Ok(())
    }

    pub fn delete_macro(&mut self, index: usize) -> Result<()> {
        if index < self.macros.len() {
            self.macros.remove(index);
            self.save()?;
        }
        Ok(())
    }

    pub fn parse_macro_steps(input: &str) -> Vec<MacroStep> {
        input
            .split([',', ';', '\n'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|action| {
                let desc = if let Some(key) = action.strip_prefix("key:") {
                    format!("Key {}", key.trim())
                } else if let Some(wait) = action.strip_prefix("wait:") {
                    format!("Wait {}ms", wait.trim())
                } else if let Some(app) = action.strip_prefix("app:") {
                    format!("Launch {}", app.trim())
                } else if let Some(text) = action.strip_prefix("text:") {
                    format!("Type \"{}\"", text.trim())
                } else {
                    action.to_string()
                };
                MacroStep {
                    action: action.to_string(),
                    description: desc,
                }
            })
            .collect()
    }

    pub fn format_macro_steps(steps: &[MacroStep]) -> String {
        steps
            .iter()
            .map(|s| s.action.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_format_macro_steps() {
        let input = "key:KEYCODE_POWER, wait:1500, app:netflix://, text:Stranger Things";
        let steps = AppConfig::parse_macro_steps(input);
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[0].action, "key:KEYCODE_POWER");
        assert_eq!(steps[0].description, "Key KEYCODE_POWER");
        assert_eq!(steps[1].action, "wait:1500");
        assert_eq!(steps[1].description, "Wait 1500ms");
        assert_eq!(steps[2].action, "app:netflix://");
        assert_eq!(steps[2].description, "Launch netflix://");
        assert_eq!(steps[3].action, "text:Stranger Things");
        assert_eq!(steps[3].description, "Type \"Stranger Things\"");

        let formatted = AppConfig::format_macro_steps(&steps);
        assert_eq!(formatted, "key:KEYCODE_POWER, wait:1500, app:netflix://, text:Stranger Things");
    }

    #[test]
    fn test_custom_app_serde_default() {
        let json = r#"{
            "name": "SmartTube",
            "link": "smarttube://",
            "category": "Video"
        }"#;
        let app: CustomApp = serde_json::from_str(json).unwrap();
        assert_eq!(app.name, "SmartTube");
        assert_eq!(app.hotkey, None);
        assert!(app.is_quick_app);
    }

    #[test]
    fn test_get_last_tv() {
        let mut cfg = AppConfig::default();
        assert_eq!(cfg.get_last_tv(), None);

        cfg.devices.push(SavedDevice {
            name: "Living Room TV".to_string(),
            ip: "192.168.1.50".to_string(),
            paired: true,
            last_connected: None,
            vendor: None,
            model: None,
        });
        cfg.devices.push(SavedDevice {
            name: "Bedroom TV".to_string(),
            ip: "192.168.1.60".to_string(),
            paired: true,
            last_connected: Some(Utc::now()),
            vendor: None,
            model: None,
        });

        // With no active_device_ip, picks the one with most recent last_connected
        assert_eq!(
            cfg.get_last_tv(),
            Some(("Bedroom TV".to_string(), "192.168.1.60".to_string()))
        );

        // When active_device_ip is explicitly set, it takes priority
        cfg.active_device_ip = Some("192.168.1.50".to_string());
        assert_eq!(
            cfg.get_last_tv(),
            Some(("Living Room TV".to_string(), "192.168.1.50".to_string()))
        );
    }
}


