use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;

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
    #[serde(default)]
    pub mpris: MprisConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "192.168.1.143".to_string(),
            port: 6676,
            selected_controllers: HashSet::new(),
            mpris: MprisConfig::default(),
        }
    }
}

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
        
        // Always try to save to ensure all fields (including new ones) are present
        // Only if it was successfully parsed!
        let _ = config.save_to_path(path);
        Ok(config)
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_invalid_json_does_not_overwrite() {
        let mut path = std::env::temp_dir();
        path.push("broadlink_test_config_invalid.json");
        
        let invalid_json = "{ \"invalid\": json }";
        fs::write(&path, invalid_json).unwrap();

        let result = Config::load_from_path(path.clone());
        assert!(result.is_err());

        // Verify file still contains invalid JSON and was not overwritten
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, invalid_json);

        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_load_non_existent_creates_default() {
        let mut path = std::env::temp_dir();
        path.push("broadlink_test_config_new.json");
        if path.exists() { let _ = fs::remove_file(&path); }

        let result = Config::load_from_path(path.clone());
        assert!(result.is_ok());
        
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("192.168.1.143")); // default host

        // Cleanup
        let _ = fs::remove_file(path);
    }
}
