//! HID++ Diagnostic Tool
//!
//! This tool helps diagnose why battery events are not being received
//! from Logitech devices.
//!
//! Usage: cargo run --bin diagnostic

use hidapi::HidApi;
use std::thread;
use std::time::Duration;

const LOGITECH_VENDOR_ID: u16 = 0x046D;
const UNIFIED_BATTERY_FEATURE: u16 = 0x1004;
const BATTERY_STATUS_FEATURE: u16 = 0x1000;
const BATTERY_VOLTAGE_FEATURE: u16 = 0x1001;
const HIDPP_LONG_REPORT: u8 = 0x11;
const UNIFIED_BATTERY_GET_STATUS: u8 = 0x10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("LOGITECH HID++ DIAGNOSTIC TOOL");
    println!("========================================\n");

    // Step 1: Scan for all Logitech devices
    println!("[STEP 1] Scanning for Logitech HID devices...");
    let api = HidApi::new()?;

    let mut hidpp_devices = Vec::new();

    for device_info in api.device_list() {
        if device_info.vendor_id() == LOGITECH_VENDOR_ID {
            println!("  ✓ Found device:");
            println!("    Product: {:?}", device_info.product_string());
            println!(
                "    VID:PID: 0x{:04X}:0x{:04X}",
                device_info.vendor_id(),
                device_info.product_id()
            );
            println!(
                "    Usage Page/Usage: 0x{:04X}/0x{:04X}",
                device_info.usage_page(),
                device_info.usage()
            );
            println!("    Interface: {}", device_info.interface_number());
            println!("    Path: {:?}", device_info.path());

            // Identify HID++ version
            if device_info.usage_page() == 0xFF00 {
                if device_info.usage() == 0x0001 {
                    println!("    Type: HID++ 1.0 interface");
                    hidpp_devices.push((device_info.clone(), "HID++ 1.0"));
                } else if device_info.usage() == 0x0002 {
                    println!("    Type: HID++ 2.0 interface ⭐");
                    hidpp_devices.push((device_info.clone(), "HID++ 2.0"));
                } else {
                    println!("    Type: HID++ Unknown interface");
                    hidpp_devices.push((device_info.clone(), "HID++ Unknown"));
                }
            } else {
                println!("    Type: Other interface (Mouse/Keyboard)");
                hidpp_devices.push((device_info.clone(), "Other"));
            }
            println!();
        }
    }

    if hidpp_devices.is_empty() {
        println!("  ✗ No Logitech devices found!");
        return Ok(());
    }

    // Find the best device to test (prefer HID++ 2.0)
    let (device_info, version) = hidpp_devices
        .iter()
        .find(|(_, v)| *v == "HID++ 2.0")
        .or_else(|| hidpp_devices.iter().find(|(_, v)| *v == "HID++ 1.0"))
        .or_else(|| hidpp_devices.first())
        .expect("No devices");

    // Step 2: Open selected device
    println!("[STEP 2] Opening {} interface...", version);

    let device = api.open_path(device_info.path())?;
    println!("  ✓ Opened: {:?}\n", device_info.path());

    // Step 3: Test battery feature support
    println!("[STEP 3] Testing battery feature support...");
    test_battery_features(&device)?;

    // Step 4: Read current battery
    println!("\n[STEP 4] Reading current battery status...");
    test_battery_read(&device)?;

    // Step 5: Listen for events
    println!("\n[STEP 5] Testing passive event listening (20 attempts, 100ms each)...");
    println!("  👉 PLEASE MOVE THE MOUSE during this test!");
    test_event_listening(&device)?;

    // Step 6: Try to enable notifications
    println!("\n[STEP 6] Trying to enable battery notifications...");
    test_enable_notifications(&device)?;

    // Step 7: Listen again after enabling
    println!("\n[STEP 7] Testing event listening AFTER enabling notifications...");
    println!("  👉 PLEASE MOVE THE MOUSE during this test!");
    test_event_listening_after_enabling(&device)?;

    // Final diagnosis
    println!("\n[DIAGNOSIS]");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("See results above to determine the cause.");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

fn test_battery_features(device: &hidapi::HidDevice) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Testing device indices: 0xFF, 0x01, 0x02, 0x03...");

    let device_indices = [0xFF, 0x01, 0x02, 0x03];
    let features = [
        (UNIFIED_BATTERY_FEATURE, "UNIFIED_BATTERY (0x1004)"),
        (BATTERY_STATUS_FEATURE, "BATTERY_STATUS (0x1000)"),
        (BATTERY_VOLTAGE_FEATURE, "BATTERY_VOLTAGE (0x1001)"),
    ];

    for device_idx in device_indices {
        println!("\n    Testing device index 0x{:02X}:", device_idx);

        for (feature, name) in &features {
            match get_feature_index_for_device(device, device_idx, *feature) {
                Ok(feature_idx) => {
                    println!(
                        "      ✓ Feature 0x{:04X} ({}) found at index 0x{:02X}",
                        feature, name, feature_idx
                    );

                    // Test reading battery status for this feature
                    match feature {
                        0x1004 => {
                            // UNIFIED_BATTERY
                            if let Ok(response) = feature_request_for_device(
                                device,
                                device_idx,
                                feature_idx,
                                UNIFIED_BATTERY_GET_STATUS,
                                &[],
                            ) {
                                println!("        Response: {:02X?}", response);
                                parse_battery_response(&response);
                            }
                        }
                        0x1000 => {
                            // BATTERY_STATUS
                            if let Ok(response) = feature_request_for_device(
                                device,
                                device_idx,
                                feature_idx,
                                0x00,
                                &[],
                            ) {
                                println!("        Response: {:02X?}", response);
                                parse_battery_response(&response);
                            }
                        }
                        0x1001 => {
                            // BATTERY_VOLTAGE
                            if let Ok(response) = feature_request_for_device(
                                device,
                                device_idx,
                                feature_idx,
                                0x00,
                                &[],
                            ) {
                                println!("        Response: {:02X?}", response);
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    println!(
                        "      ✗ Feature 0x{:04X} ({}) not supported - {}",
                        feature, name, e
                    );
                }
            }
        }
    }

    Ok(())
}

fn get_feature_index_for_device(
    device: &hidapi::HidDevice,
    device_idx: u8,
    feature: u16,
) -> Result<u8, Box<dyn std::error::Error>> {
    let mut request = [0u8; 20];
    request[0] = HIDPP_LONG_REPORT;
    request[1] = device_idx;
    request[2] = 0x00; // ROOT feature
    request[3] = 0x00; // GetFeature function
    request[4] = (feature >> 8) as u8;
    request[5] = (feature & 0xFF) as u8;

    device.write(&request)?;

    let mut response = [0u8; 20];
    let bytes_read = device.read_timeout(&mut response, 1000)?;

    if bytes_read < 7 {
        return Err(format!("Short response: {} bytes", bytes_read).into());
    }

    // Check for HID++ error
    if response[2] == 0x8F || response[2] == 0xFF {
        let error_code = response[3];
        return Err(format!("HID++ error 0x{:02X}", error_code).into());
    }

    let feature_index = response[4];

    if feature_index == 0 || feature_index == 0xFF {
        return Err(format!("Invalid feature index: 0x{:02X}", feature_index).into());
    }

    Ok(feature_index)
}

fn feature_request_for_device(
    device: &hidapi::HidDevice,
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

    device.write(&request)?;

    let mut response = [0u8; 20];
    let bytes_read = device.read_timeout(&mut response, 1000)?;

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

fn parse_battery_response(response: &[u8]) {
    if response.len() >= 3 {
        let level = response[0];
        let next_level = response[1];
        let status_raw = response[2];

        if level <= 100 {
            let status_str = match status_raw {
                0x00 => "Discharging",
                0x01 => "Slow Charging",
                0x02 => "Fast Charging",
                0x03 => "Full Charging",
                0x04 => "Charging Error",
                0x05 => "Charging",
                0x06 => "Fast Charging (alt)",
                _ => "Unknown Status",
            };

            println!(
                "        Battery: {}% (next: {}, status: {})",
                level, next_level, status_str
            );
        } else {
            println!("        Raw data: {:02X?}", response);
        }
    } else {
        println!("        Insufficient data: {:02X?}", response);
    }
}

fn test_battery_read(device: &hidapi::HidDevice) -> Result<(), Box<dyn std::error::Error>> {
    let device_indices = [0xFF, 0x01];

    for device_idx in device_indices {
        match get_feature_index_for_device(device, device_idx, UNIFIED_BATTERY_FEATURE) {
            Ok(feature_idx) => {
                println!(
                    "    Device 0x{:02X}, Feature 0x{:02X}, Function 0x{:02X} (GetStatus):",
                    device_idx, feature_idx, UNIFIED_BATTERY_GET_STATUS
                );

                match feature_request_for_device(
                    device,
                    device_idx,
                    feature_idx,
                    UNIFIED_BATTERY_GET_STATUS,
                    &[],
                ) {
                    Ok(response) => {
                        println!("      Response: {:02X?}", response);

                        if !response.is_empty() && response[0] <= 100 {
                            parse_battery_response(&response);
                        } else {
                            println!("      Raw data: {:02X?}", response);
                        }
                    }
                    Err(e) => {
                        println!("      Error reading battery: {}", e);
                    }
                }

                // Exit early after successful read
                break;
            }
            Err(_) => continue,
        }
    }

    Ok(())
}

fn test_event_listening(device: &hidapi::HidDevice) -> Result<(), Box<dyn std::error::Error>> {
    let mut events_received = 0;

    for i in 1..=20 {
        let mut buffer = [0u8; 20];
        match device.read_timeout(&mut buffer, 100) {
            Ok(bytes_read) if bytes_read > 0 => {
                events_received += 1;
                println!(
                    "    Attempt {}: ✓ Received {} bytes: {:02X?}",
                    i,
                    bytes_read,
                    &buffer[..bytes_read]
                );

                // Analyze the received packet
                if bytes_read >= 4 {
                    let report_id = buffer[0];
                    let device_idx = buffer[1];
                    let feature_idx = buffer[2];
                    let function_id = buffer[3];

                    println!("      Report ID: 0x{:02X}, Device: 0x{:02X}, Feature: 0x{:02X}, Function: 0x{:02X}", 
                             report_id, device_idx, feature_idx, function_id);

                    // Check if this looks like a battery event
                    if feature_idx == 0x04 || feature_idx == 0x00 {
                        // Common battery feature indexes
                        println!("      🎯 Potential battery event!");
                    }
                }
            }
            Ok(_) => {
                print!("\r    Attempt {}/20: No data (timeout)...", i);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            Err(e) => {
                println!("    Attempt {}: Error: {}", i, e);
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    println!();

    if events_received > 0 {
        println!(
            "    ✓ Received {} events out of 20 attempts",
            events_received
        );
    } else {
        println!("    ✗ No events received in 2 seconds");
    }

    Ok(())
}

fn test_event_listening_after_enabling(
    device: &hidapi::HidDevice,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut events_received = 0;

    for i in 1..=20 {
        let mut buffer = [0u8; 20];
        match device.read_timeout(&mut buffer, 100) {
            Ok(bytes_read) if bytes_read > 0 => {
                events_received += 1;
                println!(
                    "    Attempt {}: ✓ Received {} bytes: {:02X?}",
                    i,
                    bytes_read,
                    &buffer[..bytes_read]
                );

                // Analyze the received packet
                if bytes_read >= 4 {
                    let report_id = buffer[0];
                    let device_idx = buffer[1];
                    let feature_idx = buffer[2];
                    let function_id = buffer[3];

                    println!("      Report ID: 0x{:02X}, Device: 0x{:02X}, Feature: 0x{:02X}, Function: 0x{:02X}", 
                             report_id, device_idx, feature_idx, function_id);
                }
            }
            Ok(_) => {
                print!("\r    Attempt {}/20: No data (timeout)...", i);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            Err(e) => {
                println!("    Attempt {}: Error: {}", i, e);
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    println!();

    if events_received > 0 {
        println!(
            "    ✓ Received {} events out of 20 attempts",
            events_received
        );
    } else {
        println!("    ✗ No events received in 2 seconds");
    }

    Ok(())
}

fn test_enable_notifications(device: &hidapi::HidDevice) -> Result<(), Box<dyn std::error::Error>> {
    let mut success = false;

    // Try different device indices
    for device_idx in [0xFF, 0x01] {
        if let Ok(feature_idx) =
            get_feature_index_for_device(device, device_idx, UNIFIED_BATTERY_FEATURE)
        {
            println!(
                "    Testing device 0x{:02X}, feature 0x{:02X}",
                device_idx, feature_idx
            );

            // Method 1: Try to enable battery notifications (ShowBatteryStatus function 0x20)
            println!("    Method 1: ShowBatteryStatus (0x20)");
            let request = [
                HIDPP_LONG_REPORT,
                device_idx,
                feature_idx,
                0x20,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ];

            match device.write(&request) {
                Ok(bytes_written) => {
                    println!("      Request sent ({} bytes)", bytes_written);

                    let mut response = [0u8; 20];
                    match device.read_timeout(&mut response, 1000) {
                        Ok(bytes_read) => {
                            println!(
                                "      Response: {:02X?} ({} bytes)",
                                &response[..bytes_read],
                                bytes_read
                            );

                            if bytes_read >= 4 && response[2] == 0x8F {
                                let error = response[3];
                                println!("      Error: HID++ error 0x{:02X}", error);
                            } else if bytes_read >= 4 {
                                println!("      ✓ Success - notifications enabled?");
                                success = true;
                            }
                        }
                        Err(e) => {
                            println!("      No response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("      Error sending request: {}", e);
                }
            }

            // If method 1 worked, try other methods too
            if success {
                break;
            }
        }
    }

    if !success {
        println!("    ✗ Could not enable notifications using standard methods");
        println!(
            "    This may be normal - many devices don't require explicit notification enabling"
        );
    } else {
        println!("    ✓ Notifications possibly enabled");
    }

    Ok(())
}
