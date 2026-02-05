use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::time::{interval, Duration};

mod hidpp;
mod tray;
mod config;

use hidpp::device::LogitechDevice;

#[tokio::main(flavor = "current_thread")] // Minimal runtime
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔋 batrust - Logitech Battery Monitor (Event-based with fallback)");

    // Load configuration
    let config = config::load_config()?;

    // 1. Initialize HID
    let mut device = LogitechDevice::new()?;

    // 2. Create tray - handle error gracefully
    let tray = match tray::create_tray_for_platform() {
        Ok(t) => {
            println!("System tray created successfully");
            Some(Arc::new(Mutex::new(t)))
        },
        Err(e) => {
            eprintln!("Warning: Could not create system tray. The app will run without a tray icon. Error: {}", e);
            None
        }
    };

    // ========================================
    // STEP 1: Read battery ONCE at startup
    // ========================================
    println!("🔍 Reading initial battery level...");
    match device.get_battery() {
        Ok(battery) => {
            println!("✓ Initial battery: {}% {:?}", battery.percentage, battery.status);

            // Update tray with initial value
            if let Some(ref t) = tray {
                if let Ok(mut tray_handle) = t.lock() {
                    tray_handle.update(&battery, &config);
                }
            }
        }
        Err(e) => {
            eprintln!("⚠ Could not read initial battery: {}", e);
        }
    }

    println!("👂 Listening for battery events (passive mode - minimal battery impact)...");
    println!("   Device will send events automatically when battery changes");
    println!("   Falling back to polling every {} seconds if no events received", config.update_interval);

    // ========================================
    // STEP 2: Hybrid approach: event-based + fallback polling
    // ========================================
    let mut event_timer = interval(Duration::from_millis(500)); // Check every 500ms for events
    let mut last_event_time = Instant::now();
    let mut polling_timer = interval(Duration::from_secs(config.update_interval)); // Fallback polling

    loop {
        tokio::select! {
            // Check for events
            _ = event_timer.tick() => {
                // Passive listening - does NOT wake up device!
                if let Ok(battery) = device.listen_for_battery_events() {
                    println!("📬 Battery event: {}% {:?}", battery.percentage, battery.status);
                    last_event_time = Instant::now();

                    // Update tray with new battery info
                    if let Some(ref t) = tray {
                        if let Ok(mut tray_handle) = t.lock() {
                            tray_handle.update(&battery, &config);
                        }
                    }
                }
            },
            // Fallback polling if no events received recently
            _ = polling_timer.tick() => {
                let time_since_last_event = last_event_time.elapsed();
                if time_since_last_event.as_secs() >= config.update_interval {
                    println!("🔄 No recent events, performing fallback poll...");
                    match device.get_battery() {
                        Ok(battery) => {
                            println!("📊 Fallback battery read: {}% {:?}", battery.percentage, battery.status);

                            // Update tray with new battery info
                            if let Some(ref t) = tray {
                                if let Ok(mut tray_handle) = t.lock() {
                                    tray_handle.update(&battery, &config);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠ Fallback battery read failed: {}", e);
                        }
                    }
                }
            }
        }
    }
}