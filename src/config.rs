use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    /// Interval in seconds for polling when events are not available
    pub polling_interval: u64,
    pub red_threshold: u8,
    pub yellow_threshold: u8,
    pub disable_red: bool,
    pub disable_yellow: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            polling_interval: 60, // 1 minute for devices without events
            red_threshold: 20,
            yellow_threshold: 30,
            disable_red: false,
            disable_yellow: false,
        }
    }
}

/// Get config directory path: %USERPROFILE%\.config\traybattery
pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        let userprofile = std::env::var("USERPROFILE")
            .map_err(|_| "USERPROFILE environment variable not found")?;
        Ok(PathBuf::from(userprofile)
            .join(".config")
            .join("traybattery"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        dirs::config_dir()
            .ok_or("Could not determine config directory")?
            .join("traybattery")
            .into()
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");

    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir)?;

    // If config file doesn't exist, create with defaults
    if !config_path.exists() {
        let default_config = AppConfig::default();
        let config_str = toml::to_string(&default_config)?;
        fs::write(&config_path, config_str)?;
        println!("✅ Created default config file at: {:?}", config_path);
        return Ok(default_config);
    }

    // Read and parse existing config
    let config_str = fs::read_to_string(&config_path)?;
    let config: AppConfig = toml::from_str(&config_str)?;

    // Validate config
    validate_config(&config)?;

    Ok(config)
}

pub fn validate_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.polling_interval < 10 {
        return Err("polling_interval must be at least 10 seconds".into());
    }
    if config.polling_interval > 3600 {
        return Err("polling_interval must not exceed 3600 seconds (1 hour)".into());
    }
    if config.red_threshold > 100 {
        return Err("red_threshold must be between 0 and 100".into());
    }
    if config.yellow_threshold > 100 {
        return Err("yellow_threshold must be between 0 and 100".into());
    }
    if config.red_threshold >= config.yellow_threshold {
        return Err("red_threshold must be less than yellow_threshold".into());
    }
    Ok(())
}
