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

    // Initial read (might fail if mouse is asleep - that's OK!)
    println!("Reading initial battery level...");
    match device.get_battery() {
        Ok(battery) => {
            println!("Initial battery: {}% {:?}", battery.percentage, battery.status);
            if let Some(ref t) = tray {
                if let Ok(mut tray_handle) = t.lock() {
                    tray_handle.update(&battery, &config);
                }
            }
        }
        Err(e) => {
            eprintln!("Could not read initial battery: {}", e);
            eprintln!("Mouse might be sleeping or disconnected - will retry...");
        }
    }
    
    println!("Listening for battery events (passive mode)...");
    println!("Fallback polling every {} seconds if no events received", config.polling_interval);
    
    let mut event_timer = interval(Duration::from_millis(500));
    let mut last_event_time = Instant::now();
    let mut polling_timer = interval(Duration::from_secs(config.polling_interval));
    
    // Track if we've received any events - for auto-disabling events if unsupported
    let mut events_received = false;
    let mut event_listening_enabled = true;
    let max_attempts_without_events = 10; // after 10 seconds without events, disable event listening
    let mut no_event_attempts = 0;
    
    loop {
        tokio::select! {
            // Check for events every 500ms (only if enabled)
            _ = event_timer.tick() => {
                if event_listening_enabled {
                    // ✅ Passive listening - does NOT wake device
                    if let Ok(battery) = device.listen_for_battery_events() {
                        println!("Battery event: {}% {:?}", battery.percentage, battery.status);
                        last_event_time = Instant::now();
                        events_received = true;
                        no_event_attempts = 0; // reset counter
                        
                        if let Some(ref t) = tray {
                            if let Ok(mut tray_handle) = t.lock() {
                                tray_handle.update(&battery, &config);
                            }
                        }
                    } else {
                        // No event received - increment counter
                        no_event_attempts += 1;
                        
                        // If we've tried for a while without receiving events, consider disabling
                        if !events_received && no_event_attempts >= max_attempts_without_events {
                            println!("⚠ No events received after {} attempts, disabling event listening", max_attempts_without_events);
                            event_listening_enabled = false;
                            println!("ℹ️  Switching to polling-only mode");
                        }
                    }
                }
            }
            
            // Fallback polling - always active
            _ = polling_timer.tick() => {
                let time_since_last_event = last_event_time.elapsed();
                
                if time_since_last_event.as_secs() > config.polling_interval || !event_listening_enabled {
                    println!("No recent events, trying graceful poll...");
                    
                    // ✅ Graceful poll - returns cached value if sleeping
                    match device.get_battery() {
                        Ok(battery) => {
                            println!("Polled battery: {}% {:?}", battery.percentage, battery.status);
                            last_event_time = Instant::now();
                            
                            if let Some(ref t) = tray {
                                if let Ok(mut tray_handle) = t.lock() {
                                    tray_handle.update(&battery, &config);
                                }
                            }
                        }
                        Err(e) => {
                            // ✅ Don't panic - just log and continue
                            eprintln!("Graceful poll failed: {} - using cached value", e);
                        }
                    }
                }
            }
        }
    }
}