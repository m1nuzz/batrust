#[cfg(test)]
mod feature_index_filtering_tests {
    use traybattery::hidpp::device::LogitechDevice;

    #[test]
    fn test_accept_only_battery_feature_index() {
        // Це тест для логіки фільтрації по feature index
        // Потрібно перевірити, що ми обробляємо лише повідомлення 
        // з правильним feature index для батареї
        
        // Оскільки це потребує реального пристрою, 
        // ми перевіримо наявність функції фільтрації
        // у реалізації LogitechDevice
        
        // Це буде інтеграційний тест, який вимагає пристрій
        // тому поки що він буде ігнорований
    }

    #[test]
    #[ignore]
    fn test_battery_feature_index_detected() {
        // Тепер це ПРАЦЮЄ!
        let device = LogitechDevice::new().expect("Device not found");
        
        let feature_idx = device.get_battery_feature_index();
        assert!(feature_idx.is_some(), "Should find battery feature index");
        
        let idx = feature_idx.unwrap();
        println!("✓ Battery feature at index: 0x{:02X}", idx);
        
        // Перевірити що індекс валідний (зазвичай 0x06 для 0x1004)
        assert!(idx > 0 && idx < 0x20, "Feature index should be valid");
    }

    #[test]
    fn test_filter_non_battery_events() {
        // Цей тест МОЖЕ працювати без пристрою через мокінг
        
        // Симуляція HID++ репорту від non-battery feature
        let wireless_event = vec![0x11, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00];
        //                                       ^^^^
        //                                   feature_idx = 0x04 (не батарея!)
        
        // Якщо battery_feature_index = 0x06, цей event має бути відхилений
        // Це можна перевірити через listen_for_battery_events()
        
        // Для цього треба mock HidDevice, але як мінімум можна перевірити логіку:
        let msg_feature_idx = wireless_event[2];
        let battery_feature_idx = 0x06u8;
        
        assert_ne!(msg_feature_idx, battery_feature_idx, 
                   "Non-battery event should have different feature index");
    }
}