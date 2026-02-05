#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percentage: u8,      // 0-100
    pub charging: bool,
    pub next_level: u8,      // Warning level
    pub status: BatteryStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryStatus {
    Discharging,
    ChargingSlow,
    ChargingFast,
    ChargingWireless,
    Full,
    Error,
}

/// Feature constants for Logitech devices
pub const LOGITECH_VENDOR_ID: u16 = 0x046D;
pub const FEATURE_ROOT: u16 = 0x0000;
pub const FEATURE_FEATURE_SET: u16 = 0x0001;
pub const FEATURE_BATTERY_STATUS: u16 = 0x1000;
pub const FEATURE_BATTERY_VOLTAGE: u16 = 0x1001;
pub const FEATURE_UNIFIED_BATTERY: u16 = 0x1004; // Best option

// Feature 0x1004 (UNIFIED_BATTERY) functions
pub const UNIFIED_BATTERY_GET_CAPABILITIES: u8 = 0x00;  // GetCapabilities
pub const UNIFIED_BATTERY_GET_STATUS: u8 = 0x10;        // GetStatus - This is what we need!
pub const UNIFIED_BATTERY_SHOW_BATTERY_STATUS: u8 = 0x20; // ShowBatteryStatus (on some devices)

/// HID++ Report IDs
pub const HIDPP_SHORT_REPORT: u8 = 0x10;  // 7 bytes
pub const HIDPP_LONG_REPORT: u8 = 0x11;   // 20 bytes

/// Deciphers battery info from unified battery response
/// Feature 0x1004 function 0x00: GetStatus
///
/// Response format:
/// byte[0] = discharge_level (0-100%)
/// byte[1] = discharge_next_level
/// byte[2] = battery_status:
///           0x00 = discharging
///           0x01 = charging (slow)
///           0x02 = charging (fast)
///           0x03 = charging (wireless)
///           0x04-0x07 = error states
pub fn decipher_battery_unified(response: &[u8]) -> Option<BatteryInfo> {
    if response.len() < 3 {
        return None;
    }

    let battery_level = response[0];  // 0-100%
    let next_level = response[1];
    let status_byte = response[2];

    // 🛑 Жорстка фільтрація сміття (0xFF, 0x80 etc)
    if battery_level > 100 { return None; }

    // Якщо статус > 0x07 (невідомий), це сміття
    if status_byte > 0x07 { return None; }

    // Якщо 0% і Error - це може бути реальний event "Device disconnected"
    // АЛЕ ми не хочемо показувати це як 0% в tray.
    if status_byte >= 0x04 { // Error statuses
         return None; // Ігноруємо помилки як update
    }

    // 🛑 Фільтруємо підозрілі комбінації: 1% при зарядженні
    if battery_level == 1 && status_byte != 0x00 { // 1% але не Discharging
        return None;
    }

    // Determine status
    let (charging, status) = match status_byte {
        0x00 => (false, BatteryStatus::Discharging),
        0x01 => (true, BatteryStatus::ChargingSlow),
        0x02 => (true, BatteryStatus::ChargingFast),
        0x03 => (true, BatteryStatus::ChargingWireless),
        0x04..=0x07 => (false, BatteryStatus::Error),
        _ => (false, BatteryStatus::Error),
    };

    Some(BatteryInfo {
        percentage: battery_level,
        charging,
        next_level,
        status,
    })
}

/// Deciphers battery info from battery status response
/// Feature 0x1000 function 0x00: GetBatteryStatus
///
/// Response format:
/// byte[0] = battery level (0-100%)
/// byte[1] = battery next level
/// byte[2] = battery status flags
pub fn decipher_battery_status(response: &[u8]) -> Option<BatteryInfo> {
    if response.len() < 3 {
        return None;
    }

    let battery_level = response[0];  // 0-100%
    let next_level = response[1];
    let status_flags = response[2];

    // Determine status based on flags
    let (charging, status) = if status_flags & 0x80 != 0 {  // Charging bit
        (true, BatteryStatus::ChargingSlow)  // Simplified
    } else {
        (false, BatteryStatus::Discharging)
    };

    Some(BatteryInfo {
        percentage: battery_level,
        charging,
        next_level,
        status,
    })
}

/// Deciphers battery info from voltage response
/// Feature 0x1001 function 0x00: GetBatteryVoltage
///
/// Response format:
/// byte[0-1] = voltage in mV
/// byte[2] = battery level (0-100%)
/// byte[3] = battery next level
/// byte[4] = battery status flags
pub fn decipher_battery_voltage(response: &[u8]) -> Option<BatteryInfo> {
    if response.len() < 5 {
        return None;
    }

    let battery_level = response[2];  // 0-100%
    let next_level = response[3];
    let status_flags = response[4];

    // Determine status based on flags
    let (charging, status) = if status_flags & 0x80 != 0 {  // Charging bit
        (true, BatteryStatus::ChargingSlow)  // Simplified
    } else {
        (false, BatteryStatus::Discharging)
    };

    Some(BatteryInfo {
        percentage: battery_level,
        charging,
        next_level,
        status,
    })
}