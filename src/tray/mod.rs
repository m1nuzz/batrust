pub mod windows;
pub mod linux;
pub mod macos;

use crate::hidpp::battery::BatteryInfo;
use crate::config::AppConfig;

pub trait TrayUpdater {
    fn update(&mut self, battery: &BatteryInfo, config: &AppConfig);
}

#[cfg(target_os = "windows")]
pub fn create_tray_for_platform() -> Result<Box<dyn TrayUpdater>, Box<dyn std::error::Error>> {
    windows::create_tray().map(|tray| Box::new(tray) as Box<dyn TrayUpdater>)
}

#[cfg(target_os = "linux")]
pub fn create_tray_for_platform() -> Result<Box<dyn TrayUpdater>, Box<dyn std::error::Error>> {
    linux::create_tray().map(|tray| Box::new(tray) as Box<dyn TrayUpdater>)
}

#[cfg(target_os = "macos")]
pub fn create_tray_for_platform() -> Result<Box<dyn TrayUpdater>, Box<dyn std::error::Error>> {
    macos::create_tray().map(|tray| Box::new(tray) as Box<dyn TrayUpdater>)
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn create_tray_for_platform() -> Result<Box<dyn TrayUpdater>, Box<dyn std::error::Error>> {
    Err("Unsupported platform".into())
}