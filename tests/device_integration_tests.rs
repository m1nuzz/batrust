// Ці тести працюють ТІЛЬКИ якщо миша підключена!

#[cfg(test)]
mod device_integration_tests {
    use traybattery::hidpp::device::LogitechDevice;
    
    #[test]
    #[ignore] // Запустити вручну: cargo test --test device_integration_tests -- --ignored
    fn test_device_detection() {
        let device = LogitechDevice::new();
        assert!(device.is_ok(), "Should detect Logitech device");
    }

    #[test]
    #[ignore]
    fn test_read_battery_once() {
        let mut device = LogitechDevice::new().expect("Device not found");
        let battery = device.get_battery();
        
        assert!(battery.is_ok(), "Should read battery");
        
        let battery = battery.unwrap();
        assert!(battery.percentage <= 100, "Battery should be 0-100%");
        println!("✓ Battery: {}% {:?}", battery.percentage, battery.status);
    }

    #[test]
    #[ignore]
    fn test_battery_feature_index() {
        let mut device = LogitechDevice::new().expect("Device not found");
        
        // Перевірити що можемо отримати батарею - це означає, що функція знайдена
        let battery_result = device.get_battery();
        assert!(battery_result.is_ok(), 
                "Should be able to read battery (meaning feature index was found)");
        
        if let Ok(battery) = battery_result {
            println!("✓ Battery feature working, got: {}% {:?}", battery.percentage, battery.status);
        }
    }
}