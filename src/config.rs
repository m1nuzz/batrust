use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub update_interval: u64,      // Interval in seconds for fallback polling
    pub red_threshold: u8,         // Below this value, show as red
    pub yellow_threshold: u8,      // Below this value, show as yellow
    pub disable_red: bool,         // Whether to disable red color
    pub disable_yellow: bool,      // Whether to disable yellow color
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            update_interval: 600,    // 10 minutes for fallback polling
            red_threshold: 20,
            yellow_threshold: 30,
            disable_red: false,
            disable_yellow: false,
        }
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    // Determine config directory based on OS
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("batrust");

    let config_path = config_dir.join("config.toml");

    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir)?;

    // If config file doesn't exist, create with defaults
    if !config_path.exists() {
        let default_config = AppConfig::default();
        let config_str = toml::to_string(&default_config)?;
        fs::write(&config_path, config_str)?;
        println!("Created default config file at: {:?}", config_path);
        return Ok(default_config);
    }

    // Read and parse existing config
    let config_str = fs::read_to_string(&config_path)?;
    let config: AppConfig = toml::from_str(&config_str)?;
    Ok(config)
}