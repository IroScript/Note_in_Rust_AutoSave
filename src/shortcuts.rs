// ============================================================
//  shortcuts.rs
//
//  Mirrors the Python `ShortcutsHelp(tk.Toplevel)` class.
//
//  Displayed when the user presses F1.
//  The shortcut categories and entries are identical to the
//  Python original.
// ============================================================

use egui::{Color32, RichText, ScrollArea};

// ── data ──────────────────────────────────────────────────────────────────────

/// All shortcut categories, each with a list of (key, description) pairs.
/// Mirrors the `shortcuts` dict inside `ShortcutsHelp.__init__`.
pub fn shortcut_data() -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
    vec![
        (
            "Navigation",
            vec![
                ("Ctrl+Down",  "Go to last line"),
                ("Ctrl+Up",    "Go to first line"),
                ("Ctrl+Right", "Go to end of line"),
                ("Ctrl+Left",  "Go to start of line"),
            ],
        ),
        (
            "Editing",
            vec![
                ("Ctrl+F",                    "Find text"),
                ("Right-click on selection",  "Copy selected text"),
                ("Right-click without sel.",  "Paste text"),
            ],
        ),
        (
            "File Operations",
            vec![
                ("Ctrl+N",      "New file"),
                ("Alt+Delete",  "Delete current file"),
                ("Ctrl+R",      "Rename file"),
                ("Ctrl+O",      "Open save folder"),
                ("Alt+Q",       "Close application"),
            ],
        ),
        (
            "Formatting",
            vec![
                ("Alt+F",         "Change font"),
                ("Alt+S",         "Change font size"),
                ("Ctrl+Scroll",   "Zoom in / out"),
                ("Ctrl+Shift+B",  "Change background color"),
            ],
        ),
    ]
}

// ── ShortcutsWindow ───────────────────────────────────────────────────────────

/// Mirrors the `ShortcutsHelp` Toplevel.
/// `open` is toggled by the caller (F1 key binding).
pub struct ShortcutsWindow {
    pub open: bool,
}

impl Default for ShortcutsWindow {
    fn default() -> Self {
        Self { open: false }
    }
}

impl ShortcutsWindow {
    /// Draw the modal window.  Call this every frame; it is a no-op when
    /// `self.open == false`.
    ///
    /// Mirrors the layout built in `ShortcutsHelp.__init__`:
    ///   • header "Keyboard Shortcuts"
    ///   • scrollable area with category headers + separator + key/desc rows
    ///   • "Close" button
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        let mut should_close = false;
        
        // Modal window – mirrors `self.transient(parent)` + `self.grab_set()`
        egui::Window::new("Keyboard Shortcuts")
            .open(&mut self.open)
            .resizable(true)
            .default_size([600.0, 450.0])
            .collapsible(false)
            .show(ctx, |ui| {
                // Header (mirrors `title_label` with font("Arial", 18, "bold"))
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Keyboard Shortcuts")
                            .size(20.0)
                            .strong()
                            .color(Color32::from_rgb(0x2c, 0x3e, 0x50)),
                    );
                    ui.add_space(12.0);
                });

                // Scrollable shortcut list
                ScrollArea::vertical().show(ui, |ui| {
                    for (category, items) in shortcut_data() {
                        // Category header (mirrors `fg="#16a085"` label)
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(category)
                                .size(13.0)
                                .strong()
                                .color(Color32::from_rgb(0x16, 0xa0, 0x85)),
                        );
                        ui.separator(); // mirrors `ttk.Separator`

                        // Each shortcut row
                        for (key, description) in items {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0); // left indent

                                // Key badge (mirrors the grey `key_frame`)
                                egui::Frame::none()
                                    .fill(Color32::from_rgb(0xe0, 0xe0, 0xe0))
                                    .rounding(egui::Rounding::same(3.0))
                                    .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(key)
                                                .monospace()
                                                .size(10.0)
                                                .strong()
                                                .color(Color32::from_rgb(0x2c, 0x3e, 0x50)),
                                        );
                                    });

                                ui.add_space(8.0);

                                // Description
                                ui.label(
                                    RichText::new(description)
                                        .size(11.0),
                                );
                            });
                            ui.add_space(3.0);
                        }
                        ui.add_space(4.0);
                    }
                });

                ui.add_space(12.0);

                // Close button (mirrors the blue "Close" button)
                ui.vertical_centered(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("  Close  ")
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(0x34, 0x98, 0xdb)),
                        )
                        .clicked()
                    {
                        should_close = true;
                    }
                });

                ui.add_space(8.0);
            });
        
        if should_close {
            self.open = false;
        }
    }
}
