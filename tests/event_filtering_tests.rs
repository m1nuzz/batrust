#[cfg(test)]
mod event_filtering_tests {
    use traybattery::hidpp::battery::decipher_battery_unified;

    #[test]
    fn test_filter_garbage_0xff() {
        // Wireless disconnect event може мати 0xFF
        let response = vec![0xFF, 0xFF, 0x00];
        assert!(decipher_battery_unified(&response).is_none(), 
                "Should reject 0xFF as garbage");
    }

    #[test]
    fn test_filter_garbage_0x80() {
        // Link status events
        let response = vec![0x80, 0x00, 0x00];
        assert!(decipher_battery_unified(&response).is_none(), 
                "Should reject 0x80 as garbage");
    }

    #[test]
    fn test_filter_128_percent() {
        // Bug: 64% → 128% jump (bit flip?)
        let response = vec![128, 50, 0x02];
        assert!(decipher_battery_unified(&response).is_none(), 
                "Should reject 128% as invalid");
    }

    #[test]
    fn test_filter_invalid_status_0x80() {
        // Status byte має бути 0x00-0x07
        let response = vec![50, 30, 0x80];
        assert!(decipher_battery_unified(&response).is_none(), 
                "Should reject status 0x80");
    }

    #[test]
    fn test_accept_valid_after_garbage() {
        // Після garbage має прийняти валідні дані
        let garbage = vec![255, 255, 0xFF];
        assert!(decipher_battery_unified(&garbage).is_none());

        let valid = vec![47, 30, 0x00];
        assert!(decipher_battery_unified(&valid).is_some(), 
                "Should accept valid data after garbage");
    }
}