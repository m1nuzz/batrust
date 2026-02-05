use crate::hidpp::battery::{
    decipher_battery_status, decipher_battery_unified, decipher_battery_voltage, BatteryInfo,
    FEATURE_BATTERY_STATUS, FEATURE_BATTERY_VOLTAGE, FEATURE_UNIFIED_BATTERY, HIDPP_LONG_REPORT,
    LOGITECH_VENDOR_ID, UNIFIED_BATTERY_GET_STATUS,
};
use hidapi::{HidApi, HidDevice};
use std::collections::HashMap;

// Constant for pairing info register
const REG_PAIRING_INFO: u8 = 0xB5; // HID++ 1.0 register

use std::time::Instant;

pub struct LogitechDevice {
    hid_device: HidDevice,
    feature_cache: HashMap<u32, u8>, // (Device index << 16) | Feature ID -> Index
    active_device_index: Option<u8>, // To store the active device index
    last_valid_battery: Option<BatteryInfo>, // Cache of last valid battery reading
    last_update_time: Option<Instant>, // Time of last update
    battery_feature_index: Option<u8>, // Store the battery feature index
}

impl LogitechDevice {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api = HidApi::new()?;

        println!("=== Scanning for Logitech HID devices ===");
        let mut hidpp_candidates = Vec::new();

        for device_info in api.device_list() {
            if device_info.vendor_id() == LOGITECH_VENDOR_ID {
                println!("\n📱 Found Logitech device:");
                println!("   Product: {:?}", device_info.product_string());
                println!(
                    "   VID:PID = 0x{:04X}:0x{:04X}",
                    device_info.vendor_id(),
                    device_info.product_id()
                );
                println!("   Usage Page: 0x{:04X}", device_info.usage_page());
                println!("   Usage: 0x{:04X}", device_info.usage());
                println!("   Interface: {}", device_info.interface_number());

                // Identify HID++ interface type
                let hidpp_version = if device_info.usage_page() == 0xFF00 {
                    if device_info.usage() == 0x0001 {
                        Some("HID++ 1.0")
                    } else if device_info.usage() == 0x0002 {
                        Some("HID++ 2.0") // This is what we need for battery features
                    } else {
                        Some("HID++ Unknown")
                    }
                } else {
                    None
                };

                if let Some(version) = hidpp_version {
                    println!("   ✓ {} interface!", version);
                    hidpp_candidates.push((device_info.clone(), version.to_string()));
                } else {
                    println!("   ✗ Not HID++ (likely mouse/keyboard interface)");
                }
            }
        }

        if hidpp_candidates.is_empty() {
            return Err("No HID++ devices found".into());
        }

        println!(
            "\n=== Total HID++ devices found: {} ===",
            hidpp_candidates.len()
        );

        // PRIORITY 1: Try to open HID++ 2.0 first (needed for battery features)
        for (device_info, version) in &hidpp_candidates {
            if version == "HID++ 2.0" {
                println!("\n>>> Opening HID++ 2.0 device (for battery features)...");
                match api.open_path(device_info.path()) {
                    Ok(device) => {
                        println!("✓✓✓ Successfully opened HID++ 2.0 device!");
                        let mut logitech_device = LogitechDevice {
                            hid_device: device,
                            feature_cache: HashMap::new(),
                            active_device_index: None,
                            last_valid_battery: None,
                            last_update_time: None,
                            battery_feature_index: None,
                        };

                        // ⚡ ВАЖЛИВО: Спробувати увімкнути notifications при старті
                        // Це критично для отримання events!
                        println!(">>> Enabling battery notifications...");
                        let _ = logitech_device.enable_battery_notifications();

                        return Ok(logitech_device);
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to open HID++ 2.0: {}", e);
                    }
                }
            }
        }

        // PRIORITY 2: If HID++ 2.0 not found, try HID++ 1.0 as fallback
        for (device_info, version) in &hidpp_candidates {
            if version == "HID++ 1.0" {
                println!("\n>>> Opening HID++ 1.0 device (fallback)...");
                match api.open_path(device_info.path()) {
                    Ok(device) => {
                        println!("⚠ Opened HID++ 1.0 device (battery features may not work)");
                        let mut logitech_device = LogitechDevice {
                            hid_device: device,
                            feature_cache: HashMap::new(),
                            active_device_index: None,
                            last_valid_battery: None,
                            last_update_time: None,
                            battery_feature_index: None,
                        };

                        // ⚡ ВАЖЛИВО: Спробувати увімкнути notifications при старті
                        println!(">>> Enabling battery notifications...");
                        let _ = logitech_device.enable_battery_notifications();

                        return Ok(logitech_device);
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to open HID++ 1.0: {}", e);
                    }
                }
            }
        }

        // PRIORITY 3: If no HID++ interface works, try other interfaces
        for (device_info, _) in &hidpp_candidates {
            println!("\n>>> Trying alternative device opening method...");
            match api.open_path(device_info.path()) {
                Ok(device) => {
                    println!("✓ Opened device via alternative method");
                    let mut logitech_device = LogitechDevice {
                        hid_device: device,
                        feature_cache: HashMap::new(),
                        active_device_index: None,
                        last_valid_battery: None,
                        last_update_time: None,
                        battery_feature_index: None,
                    };

                    // ⚡ ВАЖЛИВО: Спробувати увімкнути notifications при старті
                    println!(">>> Enabling battery notifications...");
                    let _ = logitech_device.enable_battery_notifications();

                    return Ok(logitech_device);
                }
                Err(e) => {
                    eprintln!("✗ Failed to open: {}", e);
                }
            }
        }

        Err("Could not open any HID++ device".into())
    }

    /// Вмикає notifications для unified battery feature
    pub fn enable_battery_notifications(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Спробувати знайти unified battery feature
        // Для Lightspeed пристроїв часто це 0x01 або 0xFF
        let device_indices = [0x01, 0xFF];

        for device_idx in device_indices {
            // Feature 0x1004 (Unified Battery)
            if let Ok(feature_idx) =
                self.get_feature_index_for_device(device_idx, FEATURE_UNIFIED_BATTERY)
            {
                println!(
                    "   Found Unified Battery (0x1004) at index 0x{:02X} for device 0x{:02X}",
                    feature_idx, device_idx
                );

                // Зберегти індекс батареї для подальшої фільтрації подій
                self.battery_feature_index = Some(feature_idx);

                // Спробуємо виконати GetStatus щоб ініціювати можливість отримання подій
                // Це може підписати на події зміни батареї
                if let Ok(_response) = self.feature_request_for_device(
                    device_idx,
                    feature_idx,
                    UNIFIED_BATTERY_GET_STATUS,
                    &[],
                ) {
                    println!("   Successfully read battery status, notifications may be enabled");
                    self.active_device_index = Some(device_idx);
                    return Ok(());
                }
            }
        }

        // Fallback: спробувати Battery Status (0x1000)
        for device_idx in device_indices {
            if let Ok(feature_idx) =
                self.get_feature_index_for_device(device_idx, FEATURE_BATTERY_STATUS)
            {
                println!(
                    "   Found Battery Status (0x1000) at index 0x{:02X}",
                    feature_idx
                );

                // Зберегти індекс батареї для подальшої фільтрації подій
                self.battery_feature_index = Some(feature_idx);

                // Спробуємо виконати GetStatus щоб ініціювати можливість отримання подій
                if let Ok(_response) =
                    self.feature_request_for_device(device_idx, feature_idx, 0x00, &[])
                {
                    println!("   Successfully read battery status, notifications may be enabled");
                    self.active_device_index = Some(device_idx);
                    return Ok(());
                }
            }
        }

        Err("Could not find battery feature to enable notifications".into())
    }

    /// Active battery reading - only called ONCE at startup
    pub fn get_battery(&mut self) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        // If we already know the active device index, try it first
        if let Some(device_idx) = self.active_device_index {
            if let Ok(battery) = self.get_battery_for_device(device_idx) {
                if self.is_battery_change_valid(&battery) {
                    self.last_valid_battery = Some(battery.clone());
                    self.last_update_time = Some(Instant::now());
                    return Ok(battery);
                }
            } else {
                // Index no longer works, reset cache
                self.active_device_index = None;
            }
        }

        // Try common device indices
        for device_idx in [0x01, 0xFF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06] {
            if let Ok(battery) = self.get_battery_for_device(device_idx) {
                if self.is_battery_change_valid(&battery) {
                    self.active_device_index = Some(device_idx);
                    self.last_valid_battery = Some(battery.clone());
                    self.last_update_time = Some(Instant::now());
                    return Ok(battery);
                }
            }
        }

        // If no new valid reading, return cached value if available
        if let Some(ref cached) = self.last_valid_battery {
            eprintln!(
                "⚠ Failed to read new battery data, using cached: {}%",
                cached.percentage
            );
            return Ok(cached.clone());
        }

        Err("No devices with battery found (tried 0x01, 0xFF, 1-6)".into())
    }

    /// 🎧 Passive event listener - з суворою фільтрацією!
    pub fn listen_for_battery_events(&mut self) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 20];

        // NON-BLOCKING read
        match self.hid_device.read_timeout(&mut buffer, 10) {
            Ok(bytes_read) if bytes_read >= 7 => {
                // HID++ 2.0 Long Report format:
                // byte[0]: Report ID (0x11)
                // byte[1]: Device Index
                // byte[2]: Feature Index <-- КРИТИЧНО ВАЖЛИВО!
                // byte[3]: Function ID
                // byte[4..]: Data

                if buffer[0] == 0x11 {
                    let msg_feature_idx = buffer[2];

                    // Перевірка: чи це повідомлення від батареї?
                    if let Some(bat_idx) = self.battery_feature_index {
                        if msg_feature_idx != bat_idx {
                            return Err("Ignore non-battery event".into());
                        }
                    } else {
                        // Якщо ми ще не знаємо індекс батареї, ігноруємо події
                        return Err("Battery feature index not known yet".into());
                    }

                    // Тепер перевіряємо, чи це виглядає як батарейний репорт
                    // Перевіряємо довжину і можливий формат
                    if bytes_read >= 7 && buffer[4] <= 100 {
                        // battery level має бути <= 100
                        if let Some(battery) = decipher_battery_unified(&buffer[4..bytes_read]) {
                            // 🛑 СУВОРА ВАЛІДАЦІЯ:
                            // 1. Якщо статус "ChargingFast" (0x02) але % скаче 64 -> 0 -> 128 -> 255
                            //    Це явне сміття!

                            if battery.percentage > 100 {
                                return Err("Invalid percentage > 100".into());
                            }

                            // 2. Фільтрувати "стрибки"
                            if let Some(ref last) = self.last_valid_battery {
                                // Якщо % змінився більше ніж на 10% за раз - це підозріло для event
                                // (хіба що поставили на зарядку)
                                let diff =
                                    (last.percentage as i16 - battery.percentage as i16).abs();
                                if diff > 10 && !battery.charging {
                                    // Раптовий розряд на 10%? Сміття.
                                    return Err("Suspicious jump > 10%".into());
                                }
                            }

                            // Якщо пройшли перевірки - це валідний event
                            self.last_valid_battery = Some(battery.clone());
                            self.last_update_time = Some(Instant::now());
                            return Ok(battery);
                        }
                    }
                }
            }
            Ok(_) => {
                // No data available - normal for passive listening
            }
            Err(_) => {
                // Timeout - normal for passive listening
            }
        }

        Err("No battery event".into())
    }

    /// Check if battery change is logical
    fn is_battery_change_valid(&self, new_battery: &BatteryInfo) -> bool {
        if let Some(ref last) = self.last_valid_battery {
            let diff = (last.percentage as i16 - new_battery.percentage as i16).abs();

            // Reject suspicious jumps > 50% in one reading
            if diff > 50 {
                eprintln!(
                    "⚠ Suspicious battery jump: {}% → {}% (diff: {}%)",
                    last.percentage, new_battery.percentage, diff
                );
                return false;
            }

            // Reject sudden charging state with very low battery (likely invalid data)
            if !last.charging && new_battery.charging && new_battery.percentage <= 2 {
                eprintln!(
                    "⚠ Suspicious charging state: {}% discharging → {}% charging",
                    last.percentage, new_battery.percentage
                );
                return false;
            }
        }

        true
    }

    // New method with full debug output
    fn get_battery_for_device(
        &mut self,
        device_idx: u8,
    ) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        // UNIFIED_BATTERY (0x1004) - Feature 0x10 = GetStatus (correct function!)
        if let Ok(feature_idx) =
            self.get_feature_index_for_device(device_idx, FEATURE_UNIFIED_BATTERY)
        {
            if let Ok(response) = self.feature_request_for_device(
                device_idx,
                feature_idx,
                UNIFIED_BATTERY_GET_STATUS,
                &[],
            ) {
                if let Some(battery_info) = decipher_battery_unified(&response) {
                    return Ok(battery_info);
                }
            }
        }

        // BATTERY_STATUS (0x1000) - uses function 0x00
        if let Ok(feature_idx) =
            self.get_feature_index_for_device(device_idx, FEATURE_BATTERY_STATUS)
        {
            if let Ok(response) =
                self.feature_request_for_device(device_idx, feature_idx, 0x00, &[])
            {
                if let Some(battery_info) = decipher_battery_status(&response) {
                    return Ok(battery_info);
                }
            }
        }

        // BATTERY_VOLTAGE (0x1001) - uses function 0x00
        if let Ok(feature_idx) =
            self.get_feature_index_for_device(device_idx, FEATURE_BATTERY_VOLTAGE)
        {
            if let Ok(response) =
                self.feature_request_for_device(device_idx, feature_idx, 0x00, &[])
            {
                if let Some(battery_info) = decipher_battery_voltage(&response) {
                    return Ok(battery_info);
                }
            }
        }

        Err(format!(
            "Device 0x{:02X} does not support any known battery features",
            device_idx
        )
        .into())
    }

    fn get_feature_index_for_device_debug(
        &mut self,
        device_idx: u8,
        feature: u16,
    ) -> Result<u8, Box<dyn std::error::Error>> {
        let cache_key = ((device_idx as u32) << 16) | (feature as u32);

        if let Some(&index) = self.feature_cache.get(&cache_key) {
            return Ok(index);
        }

        // Try SHORT report first (HID++ 1.0 style) - might work better on some devices
        let mut short_request = [0u8; 7];
        short_request[0] = 0x10; // SHORT REPORT ID
        short_request[1] = device_idx;
        short_request[2] = 0x80; // GetLongParam (0x80) or GetFeature (0x81)
        short_request[3] = 0x00; // Function: GetFeature
        short_request[4] = (feature >> 8) as u8;
        short_request[5] = (feature & 0xFF) as u8;
        short_request[6] = 0x00;

        println!(
            "        Sending SHORT GetFeature request: {:02X?}",
            &short_request
        );

        match self.hid_device.write(&short_request) {
            Ok(bytes_written) => {
                println!("        SHORT request written {} bytes", bytes_written);

                let mut response = [0u8; 20];
                let bytes_read = self.hid_device.read_timeout(&mut response, 1000)?;

                println!(
                    "        SHORT Response ({} bytes): {:02X?}",
                    bytes_read,
                    &response[..bytes_read.min(10)]
                );

                if bytes_read >= 5 && response[2] != 0x8F {
                    // Successful response
                    let feature_index = response[4];
                    if feature_index != 0 && feature_index != 0xFF {
                        self.feature_cache.insert(cache_key, feature_index);
                        return Ok(feature_index);
                    }
                }
            }
            Err(e) => {
                println!("        SHORT request failed: {}", e);
            }
        }

        // If short report fails, try LONG report (HID++ 2.0 style)
        let mut long_request = [0u8; 20];
        long_request[0] = HIDPP_LONG_REPORT; // 0x11 - Report ID
        long_request[1] = device_idx;
        long_request[2] = 0x00; // ROOT feature
        long_request[3] = 0x00; // GetFeature function
        long_request[4] = (feature >> 8) as u8;
        long_request[5] = (feature & 0xFF) as u8;
        // Remaining bytes = 0

        println!(
            "        Sending LONG GetFeature request: {:02X?}",
            &long_request[..7]
        );

        match self.hid_device.write(&long_request) {
            Ok(bytes_written) => {
                println!("        LONG request written {} bytes", bytes_written);

                let mut response = [0u8; 20];
                let bytes_read = self.hid_device.read_timeout(&mut response, 1000)?;

                println!(
                    "        LONG Response ({} bytes): {:02X?}",
                    bytes_read,
                    &response[..bytes_read.min(10)]
                );

                if bytes_read < 7 {
                    return Err(format!("Short response: {} bytes", bytes_read).into());
                }

                // Response format: [Report ID, Device Index, Feature Index, Function, Data...]
                // Check for HID++ error
                if response[2] == 0x8F {
                    let error_code = response[3];
                    return Err(format!("HID++ error 0x{:02X}", error_code).into());
                }

                let feature_index = response[4];

                if feature_index == 0 || feature_index == 0xFF {
                    return Err(format!("Invalid feature index: 0x{:02X}", feature_index).into());
                }

                self.feature_cache.insert(cache_key, feature_index);
                Ok(feature_index)
            }
            Err(e) => Err(format!("LONG request failed: {}", e).into()),
        }
    }

    fn feature_request_for_device_debug(
        &self,
        device_idx: u8,
        feature_idx: u8,
        function: u8,
        params: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut request = [0u8; 20];
        request[0] = HIDPP_LONG_REPORT;
        request[1] = device_idx;
        request[2] = feature_idx;
        request[3] = function;

        for (i, &param) in params.iter().enumerate() {
            if i + 4 < request.len() {
                request[i + 4] = param;
            }
        }

        println!("        Sending feature request: {:02X?}", &request[..7]);

        // FIX: On Windows we need to write ALL 20 bytes including the Report ID
        let bytes_written = self.hid_device.write(&request)?;
        println!("        Written {} bytes", bytes_written);

        let mut response = [0u8; 20];
        let bytes_read = self.hid_device.read_timeout(&mut response, 1000)?;

        println!(
            "        Response ({} bytes): {:02X?}",
            bytes_read,
            &response[..bytes_read.min(10)]
        );

        if bytes_read < 4 {
            return Err(format!("Short response: {} bytes", bytes_read).into());
        }

        if response[2] == 0x8F {
            return Err(format!("HID++ error 0x{:02X}", response[3]).into());
        }

        Ok(response[4..bytes_read].to_vec())
    }

    fn get_feature_index_for_device(
        &mut self,
        device_idx: u8,
        feature: u16,
    ) -> Result<u8, Box<dyn std::error::Error>> {
        // Cache key: device_idx in high 16 bits + feature ID
        let cache_key = ((device_idx as u32) << 16) | (feature as u32);

        if let Some(&index) = self.feature_cache.get(&cache_key) {
            return Ok(index);
        }

        let mut request = [0u8; 20];
        request[0] = HIDPP_LONG_REPORT; // 0x11
        request[1] = device_idx; // SPECIFIC device index (NOT 0xFF!)
        request[2] = 0x00; // ROOT feature (always index 0)
        request[3] = 0x00; // Function: GetFeature
        request[4] = (feature >> 8) as u8;
        request[5] = (feature & 0xFF) as u8;

        self.hid_device.write(&request)?;

        let mut response = [0u8; 20];
        let bytes_read = self.hid_device.read_timeout(&mut response, 1000)?;

        if bytes_read < 7 {
            return Err(format!("Short response: {} bytes", bytes_read).into());
        }

        // Check for HID++ error
        // Error response: response[2] == 0x8F or 0xFF
        if response[2] == 0x8F || response[2] == 0xFF {
            // response[3] contains error code
            let error_code = response[3];
            return Err(format!(
                "Feature 0x{:04X} not supported on device {} (error: 0x{:02X})",
                feature, device_idx, error_code
            )
            .into());
        }

        // In HID++ 2.0 long report response:
        // [0] = Report ID (0x11)
        // [1] = Device Index
        // [2] = ROOT feature index (0x00)
        // [3] = Function ID (0x00)
        // [4] = Feature index (result!)
        // [5] = Feature type
        // [6] = Feature version

        let feature_index = response[4]; // IMPORTANT: byte 4, not 6!

        if feature_index == 0 || feature_index == 0xFF {
            return Err(format!("Invalid feature index: 0x{:02X}", feature_index).into());
        }

        self.feature_cache.insert(cache_key, feature_index);
        Ok(feature_index)
    }

    fn feature_request_for_device(
        &self,
        device_idx: u8,
        feature_idx: u8,
        function: u8,
        params: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut request = [0u8; 20];
        request[0] = HIDPP_LONG_REPORT;
        request[1] = device_idx; // Specific device
        request[2] = feature_idx;
        request[3] = function;

        for (i, &param) in params.iter().enumerate() {
            if i + 4 < request.len() {
                request[i + 4] = param;
            }
        }

        self.hid_device.write(&request)?;

        let mut response = [0u8; 20];
        let bytes_read = self.hid_device.read_timeout(&mut response, 1000)?;

        if bytes_read < 4 {
            return Err(format!("Short response: {} bytes", bytes_read).into());
        }

        // Check for error
        if response[2] == 0x8F || response[2] == 0xFF {
            return Err(format!("Feature request error: 0x{:02X}", response[3]).into());
        }

        // Return data from byte 4 onward
        Ok(response[4..bytes_read].to_vec())
    }

    /// Публічний метод для отримання battery feature index
    pub fn get_battery_feature_index(&self) -> Option<u8> {
        self.battery_feature_index
    }

    /// Для тестування: отримати збережений індекс фічі батареї
    #[cfg(test)]
    pub fn get_battery_feature_index_for_test(&self) -> Option<u8> {
        self.battery_feature_index
    }

    /// For testing: get cached battery value
    pub fn get_last_valid_battery(&self) -> Option<&BatteryInfo> {
        self.last_valid_battery.as_ref()
    }

    /// For testing: check if battery change is valid
    pub fn check_battery_change_validity(&self, new_battery: &BatteryInfo) -> bool {
        self.is_battery_change_valid(new_battery)
    }

    /// For testing: manually update cache
    pub fn set_last_valid_battery(&mut self, battery: BatteryInfo) {
        self.last_valid_battery = Some(battery);
    }

    /// For testing: check if cached value exists
    pub fn has_cached_battery(&self) -> bool {
        self.last_valid_battery.is_some()
    }
}

// Helper function for creating a dummy HidDevice for tests
#[cfg(test)]
fn create_dummy_hid_device() -> hidapi::HidDevice {
    // This is problematic as we can't create a HidDevice without a real device
    // So we'll need to restructure the code to allow testing of the caching logic
    // without requiring a real HID device
    unimplemented!("Cannot create a dummy HidDevice without a real device connected");
}
