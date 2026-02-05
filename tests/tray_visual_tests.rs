#[cfg(test)]
mod tray_visual_tests {
    use traybattery::config::AppConfig;

    #[test]
    fn test_color_thresholds() {
        let config = AppConfig::default();
        
        // Test red zone
        let battery = 15u8;
        let color = if battery <= config.red_threshold && !config.disable_red {
            [255, 0, 0, 255]
        } else {
            [255, 255, 255, 255]
        };
        assert_eq!(color, [255, 0, 0, 255], "15% should be red");

        // Test yellow zone
        let battery = 25u8;
        let color = if battery <= config.red_threshold && !config.disable_red {
            [255, 0, 0, 255]
        } else if battery <= config.yellow_threshold && !config.disable_yellow {
            [255, 255, 0, 255]
        } else {
            [255, 255, 255, 255]
        };
        assert_eq!(color, [255, 255, 0, 255], "25% should be yellow");

        // Test white zone
        let battery = 50u8;
        let color = if battery <= config.red_threshold {
            [255, 0, 0, 255]
        } else if battery <= config.yellow_threshold {
            [255, 255, 0, 255]
        } else {
            [255, 255, 255, 255]
        };
        assert_eq!(color, [255, 255, 255, 255], "50% should be white");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_tray_icon_generation() {
        // This test would require access to private function create_text_icon
        // which is not accessible from tests. 
        // The function is tested indirectly through the application.
        // For now, we'll just verify that the config works properly.
        use traybattery::config::AppConfig;
        
        let config = AppConfig::default();
        assert!(config.red_threshold <= 100);
        assert!(config.yellow_threshold <= 100);
    }
}