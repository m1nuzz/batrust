#[cfg(test)]
mod battery_cache_tests {
    use traybattery::hidpp::device::LogitechDevice;
    use traybattery::hidpp::battery::{BatteryInfo, BatteryStatus};

    #[test]
    fn test_cache_stores_valid_battery() {
        // Створюємо пристрій (він буде з пустим кешем)
        let mut device = LogitechDevice::new().unwrap_or_else(|_| {
            eprintln!("⚠ No physical device found, skipping test");
            panic!("Physical device required for this test");
        });
        
        // Симулюємо валідні дані батареї: 47% discharging
        let battery_info = device.simulate_battery_response(&[47, 30, 0x00]);
        
        // Перевіряємо, що симуляція повернула валідну батарею
        assert!(battery_info.is_some(), "Should return valid battery info");
        assert_eq!(battery_info.as_ref().unwrap().percentage, 47);
        
        // Перевіряємо, що кеш був оновлений
        let cached_battery = device.get_current_cached_value();
        assert!(cached_battery.is_some(), "Cache should be populated");
        assert_eq!(cached_battery.unwrap().percentage, 47);
    }

    #[test]
    fn test_cache_not_updated_on_garbage() {
        let mut device = LogitechDevice::new().unwrap_or_else(|_| {
            eprintln!("⚠ No physical device found, skipping test");
            panic!("Physical device required for this test");
        });
        
        // Спочатку заповнюємо кеш валідними даними: 47%
        device.simulate_battery_response(&[47, 30, 0x00]);
        
        // Перевіряємо, що кеш заповнений
        let initial_cached = device.get_current_cached_value();
        assert!(initial_cached.is_some(), "Initial cache should be populated");
        assert_eq!(initial_cached.unwrap().percentage, 47);
        
        // Тепер симулюємо garbage: 255% (неприпустиме значення)
        let garbage_result = device.simulate_battery_response(&[255, 255, 0xFF]);
        
        // Перевіряємо, що garbage не повернувся як валідна батарея
        assert!(garbage_result.is_none(), "Garbage should not return valid battery");
        
        // Перевіряємо, що кеш НЕ був оновлений і містить початкове значення
        let final_cached = device.get_current_cached_value();
        assert!(final_cached.is_some(), "Cache should still be populated");
        assert_eq!(final_cached.unwrap().percentage, 47, "Cache should not update on garbage");
    }

    #[test]
    fn test_cache_not_updated_on_invalid_change() {
        let mut device = LogitechDevice::new().unwrap_or_else(|_| {
            eprintln!("⚠ No physical device found, skipping test");
            panic!("Physical device required for this test");
        });
        
        // Створюємо початкову батарею з 50%
        let initial_battery = BatteryInfo {
            percentage: 50,
            charging: false,
            next_level: 0,
            status: BatteryStatus::Discharging,
        };
        
        // Зберігаємо її в кеш напряму
        device.update_cache_directly(initial_battery);
        
        // Тепер симулюємо підозрілу зміну: 50% -> 10% (великий стрибок)
        let suspicious_result = device.simulate_battery_response(&[10, 0, 0x00]);
        
        // Перевіряємо, що підозріла зміна не повернулася як валідна батарея
        assert!(suspicious_result.is_none(), "Suspicious change should not return valid battery");
        
        // Перевіряємо, що кеш НЕ був оновлений і містить початкове значення
        let cached = device.get_current_cached_value();
        assert!(cached.is_some(), "Cache should still be populated");
        assert_eq!(cached.unwrap().percentage, 50, "Cache should not update on suspicious change");
    }
    
    #[test]
    fn test_battery_change_validation_logic() {
        let mut device = LogitechDevice::new().unwrap_or_else(|_| {
            eprintln!("⚠ No physical device found, skipping test");
            panic!("Physical device required for this test");
        });
        
        // Створюємо початкову батарею
        let initial_battery = BatteryInfo {
            percentage: 50,
            charging: false,
            next_level: 0,
            status: BatteryStatus::Discharging,
        };
        
        // Зберігаємо її в кеш
        device.update_cache_directly(initial_battery.clone());
        
        // Тестуємо нормальну зміну: 50% -> 49%
        let normal_result = device.simulate_battery_response(&[49, 30, 0x00]);
        assert!(normal_result.is_some(), "Normal discharge should be valid");
        
        // Повертаємо початковий стан
        device.update_cache_directly(initial_battery.clone());
        
        // Тестуємо підозрілу зміну: 50% -> 10% (великий стрибок)
        let suspicious_result = device.simulate_battery_response(&[10, 0, 0x00]);
        assert!(suspicious_result.is_none(), "Large jump should be invalid");
        
        // Повертаємо початковий стан
        device.update_cache_directly(initial_battery.clone());
        
        // Тестуємо підозрілу зміну: 50% -> 1% charging (very low while charging)
        let suspicious_charging_result = device.simulate_battery_response(&[1, 0, 0x02]); // 1% charging fast
        assert!(suspicious_charging_result.is_none(), "Very low charging should be invalid");
    }
}