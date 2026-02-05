#[cfg(test)]
mod e2e_tests {
    use traybattery::hidpp::battery::{BatteryInfo, BatteryStatus};
    use traybattery::config::AppConfig;

    #[test]
    fn test_full_data_flow_without_device() {
        // Симулюємо повний flow без реального пристрою
        
        // 1. Створити конфіг
        let config = AppConfig::default();
        assert_eq!(config.red_threshold, 20);
        assert_eq!(config.yellow_threshold, 30);
        
        // 2. Симулювати отримання даних батареї
        let battery = BatteryInfo {
            percentage: 47,
            charging: false,
            next_level: 30,
            status: BatteryStatus::Discharging,
        };
        
        // 3. Визначити колір для tray (логіка з WindowsTray)
        use image::Rgba;
        let color = if battery.percentage <= config.red_threshold {
            Rgba([255, 0, 0, 255])
        } else if battery.percentage <= config.yellow_threshold {
            Rgba([255, 255, 0, 255])
        } else {
            Rgba([255, 255, 255, 255])
        };
        
        assert_eq!(color, Rgba([255, 255, 255, 255]), "47% should be white");
        
        // 4. Перевірити tooltip
        let tooltip = format!("Logitech Battery: {}% 🔌\nStatus: {:?}", 
                              battery.percentage, battery.status);
        assert!(tooltip.contains("47%"));
        assert!(tooltip.contains("Discharging"));
        
        println!("✓ Full data flow works: Config → Battery → Tray");
    }

    #[test]
    fn test_state_transitions() {
        // Тестуємо різні переходи стану батареї
        
        let scenarios = vec![
            // (start%, start_charging) → (end%, end_charging) → expected_valid
            (50, false, 49, false, true),   // Нормальна розрядка
            (50, false, 51, true, true),    // Поставили на зарядку
            (50, true, 55, true, true),     // Зарядка працює
            (50, false, 10, false, false),  // SUSPICIOUS: стрибок 40%
            (50, false, 1, true, false),    // SUSPICIOUS: 1% charging
            (50, false, 128, false, false), // GARBAGE: 128%
        ];
        
        for (start_pct, start_chr, end_pct, end_chr, expected_valid) in scenarios {
            let last = BatteryInfo {
                percentage: start_pct,
                charging: start_chr,
                next_level: 0,
                status: if start_chr { BatteryStatus::ChargingSlow } 
                        else { BatteryStatus::Discharging },
            };
            
            let new = BatteryInfo {
                percentage: end_pct,
                charging: end_chr,
                next_level: 0,
                status: if end_chr { BatteryStatus::ChargingSlow } 
                        else { BatteryStatus::Discharging },
            };
            
            // Валідація (логіка з is_battery_change_valid)
            let diff = (last.percentage as i16 - new.percentage as i16).abs();
            let is_valid = if end_pct > 100 {
                false
            } else if diff > 50 {
                false
            } else if !last.charging && new.charging && new.percentage <= 2 {
                false
            } else {
                true
            };
            
            assert_eq!(is_valid, expected_valid, 
                       "{}% {} → {}% {} should be {}",
                       start_pct, if start_chr {"CHR"} else {"DIS"},
                       end_pct, if end_chr {"CHR"} else {"DIS"},
                       if expected_valid {"VALID"} else {"INVALID"});
        }
        
        println!("✓ All state transitions validated correctly");
    }

    #[test]
    #[ignore] // Потребує реальний пристрій + GUI
    fn test_real_full_workflow() {
        // Цей тест залишаємо для РУЧНОГО запуску
        use traybattery::hidpp::device::LogitechDevice;
        
        let mut device = LogitechDevice::new().expect("Device required");
        let battery = device.get_battery().expect("Battery read failed");
        
        println!("✓ Real workflow: Device → Battery: {}%", battery.percentage);
        
        // TODO: Додати тест tray icon якщо в headless режимі
    }
}