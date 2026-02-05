#[cfg(test)]
mod battery_parsing_tests {
    use traybattery::hidpp::battery::{decipher_battery_unified, BatteryStatus};

    #[test]
    fn test_valid_discharging() {
        let response = vec![85, 50, 0x00]; // 85%, discharging
        let battery = decipher_battery_unified(&response).unwrap();
        
        assert_eq!(battery.percentage, 85);
        assert_eq!(battery.charging, false);
        assert_eq!(battery.status, BatteryStatus::Discharging);
    }

    #[test]
    fn test_valid_charging_slow() {
        let response = vec![60, 30, 0x01]; // 60%, charging slow
        let battery = decipher_battery_unified(&response).unwrap();
        
        assert_eq!(battery.percentage, 60);
        assert_eq!(battery.charging, true);
        assert_eq!(battery.status, BatteryStatus::ChargingSlow);
    }

    #[test]
    fn test_valid_charging_fast() {
        let response = vec![64, 30, 0x02]; // 64%, charging fast
        let battery = decipher_battery_unified(&response).unwrap();
        
        assert_eq!(battery.percentage, 64);
        assert_eq!(battery.charging, true);
        assert_eq!(battery.status, BatteryStatus::ChargingFast);
    }

    #[test]
    fn test_reject_over_100_percent() {
        let response = vec![128, 50, 0x00]; // Invalid: 128%
        assert!(decipher_battery_unified(&response).is_none());
    }

    #[test]
    fn test_reject_255_percent() {
        let response = vec![255, 50, 0x00]; // Invalid: 255%
        assert!(decipher_battery_unified(&response).is_none());
    }

    #[test]
    fn test_reject_error_state() {
        let response = vec![0, 0, 0x05]; // Error state
        assert!(decipher_battery_unified(&response).is_none());
    }

    #[test]
    fn test_reject_invalid_status_byte() {
        let response = vec![50, 30, 0xFF]; // Invalid status
        assert!(decipher_battery_unified(&response).is_none());
    }

    #[test]
    fn test_suspicious_1_percent_charging() {
        let response = vec![1, 0, 0x01]; // 1% charging - suspicious
        assert!(decipher_battery_unified(&response).is_none());
    }

    #[test]
    fn test_edge_case_0_percent_discharging() {
        let response = vec![0, 0, 0x00]; // 0% discharging
        let battery = decipher_battery_unified(&response).unwrap();
        assert_eq!(battery.percentage, 0);
    }

    #[test]
    fn test_edge_case_100_percent() {
        let response = vec![100, 95, 0x00]; // 100% 
        let battery = decipher_battery_unified(&response).unwrap();
        assert_eq!(battery.percentage, 100);
    }

    #[test]
    fn test_short_response() {
        let response = vec![85, 50]; // Only 2 bytes - invalid
        assert!(decipher_battery_unified(&response).is_none());
    }

    #[test]
    fn test_empty_response() {
        let response = vec![];
        assert!(decipher_battery_unified(&response).is_none());
    }
}