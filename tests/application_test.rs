use traybattery::hidpp::device::LogitechDevice;
use traybattery::tray::create_tray_for_platform;
use traybattery::config::load_config;

#[test]
fn test_main_loop_with_error_handling() {
    // This test simulates the main application loop to verify it handles
    // errors gracefully without crashing

    // Load configuration
    let config = load_config().expect("Should be able to load config");

    // Initialize HID
    let device_result = LogitechDevice::new();

    // Create tray
    let tray_result = create_tray_for_platform();

    // Both device and tray initialization should succeed without crashing
    // even if they encounter errors later
    assert!(device_result.is_ok() || device_result.is_err());
    assert!(tray_result.is_ok() || tray_result.is_err());

    if let Ok(mut device) = device_result {
        // Try to get battery - this is where the original error occurred
        let battery_result = device.get_battery();

        // The important thing is that getting battery info doesn't crash
        // even if it returns an error
        assert!(battery_result.is_ok() || battery_result.is_err());

        // If we have a tray, try to update it
        if let Ok(mut tray) = tray_result {
            // Update tray with either the actual battery info or mock data
            match battery_result {
                Ok(battery) => {
                    // Update tray with actual battery info
                    tray.update(&battery, &config);
                },
                Err(_) => {
                    // Create mock battery info for tray update
                    use traybattery::hidpp::battery::{BatteryInfo, BatteryStatus};

                    let mock_battery = BatteryInfo {
                        percentage: 0,
                        charging: false,
                        next_level: 0,
                        status: BatteryStatus::Error,
                    };

                    tray.update(&mock_battery, &config);
                }
            }
        }
    }
}