use tray_icon::{TrayIcon, TrayIconBuilder, Icon};
use image::{Rgba, RgbaImage};
use rusttype::{Font, Scale};
use crate::hidpp::battery::BatteryInfo;
use crate::tray::TrayUpdater;
use crate::config::AppConfig;

pub struct WindowsTray {
    tray: TrayIcon,
    last_percentage: u8,
    last_charging: bool,
}

pub fn create_tray() -> Result<WindowsTray, Box<dyn std::error::Error>> {
    // Create initial icon with "--"
    let (icon_rgba, icon_width, icon_height) = create_text_icon("--", Rgba([255, 255, 255, 255]))?;
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)?;

    let tray = TrayIconBuilder::new()
        .with_tooltip("Logitech Battery Monitor")
        .with_icon(icon)
        .build()?;

    Ok(WindowsTray {
        tray,
        last_percentage: 255,
        last_charging: false,
    })
}

impl TrayUpdater for WindowsTray {
    fn update(&mut self, battery: &BatteryInfo, config: &AppConfig) {
        // Update if percentage or charging status changed
        if battery.percentage == self.last_percentage && battery.charging == self.last_charging {
            return;
        }

        self.last_percentage = battery.percentage;
        self.last_charging = battery.charging;

        // Determine color based on config
        let color = if battery.percentage <= config.red_threshold && !config.disable_red {
            Rgba([255, 0, 0, 255])      // Red
        } else if battery.percentage <= config.yellow_threshold && !config.disable_yellow {
            Rgba([255, 255, 0, 255])    // Yellow
        } else {
            Rgba([255, 255, 255, 255])  // White
        };

        // Generate text
        let text = format!("{}", battery.percentage);

        // Create new icon with colored text
        if let Ok((icon_rgba, icon_width, icon_height)) = create_text_icon(&text, color) {
            if let Ok(icon) = Icon::from_rgba(icon_rgba, icon_width, icon_height) {
                let _ = self.tray.set_icon(Some(icon));
            }
        }

        // Update tooltip with detailed info
        let status_emoji = if battery.charging { "⚡" } else { "🔌" };
        let tooltip = format!(
            "Logitech Battery: {}% {}\nStatus: {:?}",
            battery.percentage,
            status_emoji,
            battery.status
        );
        let _ = self.tray.set_tooltip(Some(&tooltip));
    }
}

/// Generates a 16x16 icon with text in the specified color
fn create_text_icon(text: &str, color: Rgba<u8>) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    const WIDTH: u32 = 16;
    const HEIGHT: u32 = 16;

    // Create a transparent 16x16 bitmap
    let mut image = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([0, 0, 0, 0]));

    // Try to load a system font (Consolas or Arial)
    let font_data = std::fs::read("C:\\Windows\\Fonts\\consola.ttf")
        .or_else(|_| std::fs::read("C:\\Windows\\Fonts\\arial.ttf"))
        .or_else(|_| std::fs::read("C:\\Windows\\Fonts\\cour.ttf"))?;

    let font = Font::try_from_vec(font_data)
        .ok_or("Failed to load font")?;

    // Determine font size and position based on text length
    let (scale, x_offset, y_offset) = if text.len() == 1 {
        // Single digit: 0-9
        (Scale::uniform(13.0), 4.0, 1.0)
    } else if text.len() == 2 {
        // Two digits: 10-99
        (Scale::uniform(11.0), 1.0, 1.0)
    } else {
        // Three digits: 100
        (Scale::uniform(9.0), 0.0, 2.0)
    };

    // Use rusttype to layout the text and draw it pixel by pixel
    let v_metrics = font.v_metrics(scale);
    let start = rusttype::Point { x: x_offset, y: y_offset + v_metrics.ascent };

    let glyphs: Vec<rusttype::PositionedGlyph> = font.layout(text, scale, start).collect();

    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let px = (bounding_box.min.x + x as i32) as u32;
                let py = (bounding_box.min.y + y as i32) as u32;

                if px < WIDTH && py < HEIGHT {
                    // Apply color with alpha blending based on the glyph's coverage value 'v'
                    let alpha = (v * 255.0) as u8;
                    let current_pixel = image.get_pixel(px, py);
                    let blended_color = blend_colors(*current_pixel, color, alpha);
                    image.put_pixel(px, py, blended_color);
                }
            });
        }
    }

    // Convert to vector of bytes for Icon
    Ok((image.into_raw(), WIDTH, HEIGHT))
}

// Helper function to blend colors with alpha
fn blend_colors(background: image::Rgba<u8>, foreground: Rgba<u8>, alpha: u8) -> image::Rgba<u8> {
    let alpha_f = alpha as f32 / 255.0;
    let bg = background.0;
    let fg = foreground.0;

    let r = (fg[0] as f32 * alpha_f + bg[0] as f32 * (1.0 - alpha_f)) as u8;
    let g = (fg[1] as f32 * alpha_f + bg[1] as f32 * (1.0 - alpha_f)) as u8;
    let b = (fg[2] as f32 * alpha_f + bg[2] as f32 * (1.0 - alpha_f)) as u8;
    let a = ((fg[3] as f32 * alpha_f + bg[3] as f32 * (1.0 - alpha_f)) as u8).max(bg[3]); // Preserve background alpha if foreground is transparent

    image::Rgba([r, g, b, a])
}