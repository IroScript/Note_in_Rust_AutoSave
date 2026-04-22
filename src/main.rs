// ============================================================
//  irak_notes – main.rs
//  Entry point.  Mirrors the `main()` function in the Python
//  original and the `root = tk.Tk()` / `root.mainloop()` call.
// ============================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console on Windows release

mod app;
mod color_picker;   // ← Part 2: real colour-picker dialog
mod dialogs;
mod file_ops;
mod line_numbers;
mod settings;
mod shortcuts;

use app::IrakNotesApp;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    // ── window setup (mirrors root.geometry("900x600") + theme) ──────────
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Irak Notes")
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    // Try to run with glow (OpenGL), if it fails, show error message
    match eframe::run_native(
        "Irak Notes",
        native_options,
        Box::new(|cc| {
            // Apply the "clam"-like visuals (mirrors style.theme_use("clam"))
            let mut visuals = egui::Visuals::light();
            visuals.override_text_color = None;
            cc.egui_ctx.set_visuals(visuals);
            Box::new(IrakNotesApp::new(cc))
        }),
    ) {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            eprintln!("\nYour system doesn't support OpenGL 2.0+.");
            eprintln!("Please use the Python version instead:");
            eprintln!("  python Irak_Note.py");
            eprintln!("\nThe Python version has all the same features and works on any system.");
            Err(e)
        }
    }
}
