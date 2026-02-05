use std::fs;
use traybattery::config::{get_config_dir, load_config, validate_config, AppConfig};

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert_eq!(config.polling_interval, 60);
    assert_eq!(config.red_threshold, 20);
    assert_eq!(config.yellow_threshold, 30);
    assert_eq!(config.disable_red, false);
    assert_eq!(config.disable_yellow, false);
}

#[test]
fn test_config_dir_path() {
    let config_dir = get_config_dir().expect("Failed to get config dir");

    #[cfg(target_os = "windows")]
    {
        let userprofile = std::env::var("USERPROFILE").expect("USERPROFILE not set");
        let expected = std::path::PathBuf::from(userprofile)
            .join(".config")
            .join("traybattery");
        assert_eq!(config_dir, expected);
    }

    #[cfg(not(target_os = "windows"))]
    {
        assert!(config_dir.ends_with("traybattery"));
    }
}

#[test]
fn test_config_creation() {
    // Test that config is created if it doesn't exist
    let config = load_config().expect("Failed to load config");

    // Should have valid defaults
    assert!(config.polling_interval >= 10);
    assert!(config.polling_interval <= 3600);
    assert!(config.red_threshold < config.yellow_threshold);

    // Config file should exist now
    let config_path = get_config_dir()
        .expect("Failed to get config dir")
        .join("config.toml");
    assert!(config_path.exists());
}

#[test]
fn test_config_parsing() {
    let toml_str = r#"
        polling_interval = 120
        red_threshold = 15
        yellow_threshold = 40
        disable_red = true
        disable_yellow = false
    "#;

    let config: AppConfig = toml::from_str(toml_str).expect("Failed to parse config");
    assert_eq!(config.polling_interval, 120);
    assert_eq!(config.red_threshold, 15);
    assert_eq!(config.yellow_threshold, 40);
    assert_eq!(config.disable_red, true);
    assert_eq!(config.disable_yellow, false);
}

#[test]
fn test_invalid_polling_interval_too_low() {
    let result = validate_config(&AppConfig {
        polling_interval: 5,
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("at least 10 seconds"));
}

#[test]
fn test_invalid_polling_interval_too_high() {
    let result = validate_config(&AppConfig {
        polling_interval: 5000,
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("3600 seconds"));
}

#[test]
fn test_invalid_threshold_order() {
    let result = validate_config(&AppConfig {
        red_threshold: 50,
        yellow_threshold: 30,
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be less than"));
}

#[test]
fn test_config_roundtrip() {
    let original = AppConfig {
        polling_interval: 90,
        red_threshold: 25,
        yellow_threshold: 50,
        disable_red: false,
        disable_yellow: true,
    };

    // Serialize
    let toml_str = toml::to_string(&original).expect("Failed to serialize");

    // Deserialize
    let deserialized: AppConfig = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert_eq!(original.polling_interval, deserialized.polling_interval);
    assert_eq!(original.red_threshold, deserialized.red_threshold);
    assert_eq!(original.yellow_threshold, deserialized.yellow_threshold);
}
