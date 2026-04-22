// ============================================================
//  settings.rs
//
//  Mirrors the four Python methods:
//    • load_font_settings()
//    • save_font_settings()
//    • load_color_settings()
//    • save_color_settings()
//  …and the helper `is_light_color()`.
//
//  Settings are stored as plain-text files in the same save
//  folder used by the notes themselves, exactly like the
//  Python original.
// ============================================================

use std::path::{Path, PathBuf};

// ── font settings file: "font_family,font_size" ──────────────────────────────

/// Mirrors `load_font_settings()`.
/// Returns (family, size) or falls back to ("Monospace", 12).
pub fn load_font_settings(save_folder: &Path) -> (String, f32) {
    let font_file: PathBuf = save_folder.join("font_settings.txt");
    if font_file.exists() {
        if let Ok(contents) = std::fs::read_to_string(&font_file) {
            let parts: Vec<&str> = contents.trim().splitn(2, ',').collect();
            if parts.len() == 2 {
                if let Ok(size) = parts[1].trim().parse::<f32>() {
                    return (parts[0].trim().to_string(), size);
                }
            }
        }
    }
    ("Monospace".to_string(), 14.0) // default_font = Font(family="Consolas", size=12)
}

/// Mirrors `save_font_settings()`.
pub fn save_font_settings(save_folder: &Path, family: &str, size: f32) {
    let font_file: PathBuf = save_folder.join("font_settings.txt");
    let _ = std::fs::write(&font_file, format!("{},{}", family, size as i32));
}

// ── colour settings file: "#rrggbb" ──────────────────────────────────────────

/// Mirrors `load_color_settings()`.
/// Returns (bg_hex, fg_hex).  Defaults to white bg, black fg.
pub fn load_color_settings(save_folder: &Path) -> (egui::Color32, egui::Color32) {
    let color_file: PathBuf = save_folder.join("color_settings.txt");
    if color_file.exists() {
        if let Ok(contents) = std::fs::read_to_string(&color_file) {
            let hex = contents.trim();
            if let Some(bg) = hex_to_color32(hex) {
                let fg = if is_light_color(bg) {
                    egui::Color32::BLACK
                } else {
                    egui::Color32::WHITE
                };
                return (bg, fg);
            }
        }
    }
    (egui::Color32::WHITE, egui::Color32::BLACK)
}

/// Mirrors `save_color_settings(bg_color)`.
pub fn save_color_settings(save_folder: &Path, bg: egui::Color32) {
    let color_file: PathBuf = save_folder.join("color_settings.txt");
    let hex = color32_to_hex(bg);
    let _ = std::fs::write(&color_file, hex);
}

// ── helper: is_light_color ────────────────────────────────────────────────────

/// Mirrors `is_light_color(hex_color)`.
/// Uses the same perceived-luminance formula: 0.299*R + 0.587*G + 0.114*B.
pub fn is_light_color(c: egui::Color32) -> bool {
    let r = c.r() as f32;
    let g = c.g() as f32;
    let b = c.b() as f32;
    let brightness = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
    brightness > 0.5
}

/// Derives the correct foreground colour for a given background.
pub fn fg_for_bg(bg: egui::Color32) -> egui::Color32 {
    if is_light_color(bg) {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    }
}

// ── colour conversion helpers ─────────────────────────────────────────────────

/// "#rrggbb"  →  Color32
pub fn hex_to_color32(hex: &str) -> Option<egui::Color32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(egui::Color32::from_rgb(r, g, b))
    } else {
        None
    }
}

/// Color32  →  "#rrggbb"
pub fn color32_to_hex(c: egui::Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}
