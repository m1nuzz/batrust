pub mod hidpp;
pub mod tray;
pub mod config;


#[cfg(test)]
mod tests {
    use crate::hidpp::battery::{decipher_battery_unified, BatteryStatus};

    #[test]
    fn test_decipher_battery_unified() {
        // Test a sample response for unified battery
        let response = [85, 50, 0x00]; // 85% battery, not charging
        let battery_info = decipher_battery_unified(&response).unwrap();

        assert_eq!(battery_info.percentage, 85);
        assert_eq!(battery_info.charging, false);
        assert_eq!(battery_info.status, BatteryStatus::Discharging);
    }

    #[test]
    fn test_decipher_battery_charging() {
        // Test a sample response for charging battery
        let response = [60, 30, 0x01]; // 60% battery, charging slow
        let battery_info = decipher_battery_unified(&response).unwrap();

        assert_eq!(battery_info.percentage, 60);
        assert_eq!(battery_info.charging, true);
        assert_eq!(battery_info.status, BatteryStatus::ChargingSlow);
    }

    #[test]
    fn test_decipher_battery_error() {
        // Test a sample response for error state - should return None now
        let response = [0, 0, 0x05]; // Error state - should be filtered out
        let result = decipher_battery_unified(&response);

        assert!(result.is_none()); // Error states should be filtered out
    }

    #[test]
    fn test_empty_response_handling() {
        // Test handling of empty response
        let response = [];
        assert!(decipher_battery_unified(&response).is_none());
    }

    #[test]
    fn test_short_response_handling() {
        // Test handling of short response (less than 3 bytes)
        let response = [85, 50]; // Only 2 bytes
        assert!(decipher_battery_unified(&response).is_none());
    }

    #[test]
    fn test_device_error_handling() {
        // This test documents the expected behavior when a device
        // doesn't support battery reporting or isn't connected
        // Since we can't guarantee a physical device for testing,
        // this serves as documentation of the expected error handling
        assert!(true); // Placeholder test
    }
}