use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;

// TUI-specific config (simplified, no MPRIS)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TuiConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub selected_controllers: HashSet<String>,
    #[serde(default)]
    pub last_selected_controller: String,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            host: "192.168.1.143".to_string(),
            port: 6676,
            selected_controllers: HashSet::new(),
            last_selected_controller: String::new(),
        }
    }
}

impl TuiConfig {
    pub fn get_path() -> PathBuf {
        let mut path = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".config");
        path.push("broadlink-remote");
        path.push("config-tui.json");
        path
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_path();
        log::debug!("Loading TUI config from {:?}", path);
        if !path.exists() {
            log::debug!("TUI config file not found. Creating default at {:?}", path);
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)?;
        let config = match serde_json::from_str::<Self>(&content) {
            Ok(config) => {
                log::debug!("Loaded TUI config from {:?}", path);
                config
            }
            Err(e) => {
                let msg = format!("Failed to parse TUI config file at {:?}: {}", path, e);
                log::error!("{}", msg);
                return Err(msg.into());
            }
        };
        Ok(config)
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::get_path();
        log::debug!("Saving TUI config to {:?}", path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(&path, content)
    }
}

// Full service config (kept for reference)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MprisCommands {
    #[serde(default, rename = "play-pause")]
    pub play_pause: String,
    #[serde(default)]
    pub previous: String,
    #[serde(default)]
    pub next: String,
}

impl Default for MprisCommands {
    fn default() -> Self {
        Self {
            play_pause: "".to_string(),
            previous: "".to_string(),
            next: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MprisConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub controller: String,
    #[serde(default)]
    pub device: String,
    #[serde(default)]
    pub commands: MprisCommands,
}

impl Default for MprisConfig {
    fn default() -> Self {
        Self {
            enable: false,
            controller: "".to_string(),
            device: "".to_string(),
            commands: MprisCommands::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub selected_controllers: HashSet<String>,
    #[serde(default = "default_tray_icon")]
    pub tray_icon: Option<String>,
    pub mpris: MprisConfig,
}

fn default_tray_icon() -> Option<String> {
    Some("preferences-desktop-peripherals".to_string())
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "192.168.1.143".to_string(),
            port: 6676,
            selected_controllers: HashSet::new(),
            tray_icon: Some("preferences-desktop-peripherals".to_string()),
            mpris: MprisConfig::default(),
        }
    }
}

#[allow(dead_code)]
impl Config {
    pub fn get_path() -> PathBuf {
        let mut path = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".config");
        path.push("broadlink-remote");
        path.push("config.json");
        path
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_from_path(Self::get_path())
    }

    pub fn load_from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        if !path.exists() {
            log::info!("Config file not found. Creating default at {:?}", path);
            let config = Self::default();
            config.save_to_path(path)?;
            return Ok(config);
        }

        let content = fs::read_to_string(path)?;
        let config = match serde_json::from_str::<Self>(&content) {
            Ok(config) => {
                log::info!("Loaded config from {:?}", path);
                config
            }
            Err(e) => {
                let msg = format!("Failed to parse config file at {:?}: {}", path, e);
                log::error!("{}", msg);
                return Err(msg.into());
            }
        };
        
        let _ = config.save_to_path(path);
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to_path(Self::get_path())
    }

    pub fn save_to_path<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, content)
    }
}