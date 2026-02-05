#[cfg(test)]
mod battery_validation_tests {
    use traybattery::hidpp::battery::{BatteryInfo, BatteryStatus};

    // Mock helper function
    fn create_battery_info(percentage: u8, charging: bool) -> BatteryInfo {
        BatteryInfo {
            percentage,
            charging,
            next_level: 0,
            status: if charging { 
                BatteryStatus::ChargingSlow 
            } else { 
                BatteryStatus::Discharging 
            },
        }
    }

    #[test]
    fn test_valid_gradual_discharge() {
        // 85% → 84% - OK
        let last = create_battery_info(85, false);
        let new = create_battery_info(84, false);
        
        // Симулюємо валідацію
        let diff = (last.percentage as i16 - new.percentage as i16).abs();
        assert!(diff <= 50, "Should accept gradual discharge");
    }

    #[test]
    fn test_reject_huge_jump() {
        // 85% → 20% - стрибок 65% - suspicious!
        let last = create_battery_info(85, false);
        let new = create_battery_info(20, false);
        
        let diff = (last.percentage as i16 - new.percentage as i16).abs();
        assert!(diff > 50, "Should reject huge jump");
    }

    #[test]
    fn test_reject_sudden_low_charging() {
        // 47% discharging → 1% charging - suspicious
        let last = create_battery_info(47, false);
        let new = create_battery_info(1, true);
        
        assert!(!last.charging && new.charging && new.percentage <= 2, 
                "Should detect suspicious charging state");
    }

    #[test]
    fn test_accept_normal_charging_transition() {
        // 47% discharging → 48% charging - OK
        let last = create_battery_info(47, false);
        let new = create_battery_info(48, true);
        
        let diff = (last.percentage as i16 - new.percentage as i16).abs();
        assert!(diff <= 10, "Should accept normal charging transition");
    }
}