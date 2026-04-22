// ============================================================
//  color_picker.rs
//
//  Mirrors `change_background_color()` in the Python original:
//      color = colorchooser.askcolor(
//          title="Choose Background Color",
//          initialcolor=self.text.cget("bg")
//      )
//
//  Since `rfd` does not expose a native OS colour picker,
//  we use egui's built-in `egui::color_picker` widget inside
//  a window.  The UX is equivalent: the user picks a colour,
//  presses "Apply", and the editor background + foreground
//  are updated and persisted – exactly as in the Python version.
// ============================================================

use egui::{Color32, Context, RichText};

// ── ColorPickerDialog ─────────────────────────────────────────────────────────

/// State for the background-colour-picker window.
pub struct ColorPickerDialog {
    /// Whether the window is visible.
    pub open: bool,

    /// The colour currently shown in the picker wheel.
    /// Initialised to the editor's current background each time the dialog
    /// opens (mirrors `initialcolor=self.text.cget("bg")`).
    pub color: Color32,

    /// Set to `Some(color)` for one frame when the user clicks "Apply".
    /// The app reads this and then sets it back to `None`.
    pub applied: Option<Color32>,
}

impl ColorPickerDialog {
    /// Create with the editor's current background as the initial colour.
    pub fn new(initial_bg: Color32) -> Self {
        Self {
            open:    false,
            color:   initial_bg,
            applied: None,
        }
    }

    /// Re-open the dialog and reset its working colour to the current bg.
    /// Call this from the Ctrl+Alt+C handler.
    pub fn open_with(&mut self, current_bg: Color32) {
        self.color   = current_bg;
        self.applied = None;
        self.open    = true;
    }

    /// Draw the picker window every frame.  No-op when `self.open == false`.
    ///
    /// Layout mirrors the Python dialog:
    ///   title  "Choose Background Color"
    ///   colour wheel / sliders
    ///   preview swatch
    ///   [Apply]  [Cancel]
    pub fn show(&mut self, ctx: &Context) {
        self.applied = None; // reset each frame

        if !self.open {
            return;
        }

        let mut open = self.open;
        egui::Window::new("Choose Background Color")
            .open(&mut open)
            .resizable(true)
            .collapsible(false)
            .default_size([320.0, 380.0])
            .show(ctx, |ui| {
                ui.add_space(6.0);

                // ── colour picker widget ──────────────────────────────────────
                // egui::color_picker::color_picker_color32 renders the full HSV
                // wheel + hex input that tkinter's colorchooser provides.
                egui::color_picker::color_picker_color32(
                    ui,
                    &mut self.color,
                    egui::color_picker::Alpha::Opaque, // background is always opaque
                );

                ui.add_space(10.0);

                // ── preview swatch ────────────────────────────────────────────
                // Shows the selected colour as a wide bar so the user can judge
                // readability before applying (analogous to the preview section
                // in the Python colorchooser dialog).
                let preview_rect = ui.allocate_exact_size(
                    egui::Vec2::new(ui.available_width(), 32.0),
                    egui::Sense::hover(),
                ).0;
                ui.painter().rect_filled(preview_rect, egui::Rounding::same(4.0), self.color);

                // Show sample text in contrasting colour over the swatch
                let contrast = if is_light(self.color) {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                };
                ui.painter().text(
                    preview_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Sample Text  AaBbCc 123",
                    egui::FontId::proportional(13.0),
                    contrast,
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                // ── Apply / Cancel buttons ────────────────────────────────────
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Cancel (mirrors closing without returning a colour)
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Cancel")
                                        .size(11.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(0x95, 0xa5, 0xa6)),
                            )
                            .clicked()
                        {
                            self.open = false;
                        }

                        ui.add_space(8.0);

                        // Apply (mirrors `if color and color[1]:` branch)
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Apply")
                                        .size(11.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(0x34, 0x98, 0xdb)),
                            )
                            .clicked()
                        {
                            self.applied = Some(self.color);
                            self.open    = false;
                        }
                    });
                });

                ui.add_space(6.0);
            });

        self.open = open;
    }
}

// ── internal helper ───────────────────────────────────────────────────────────

/// Same perceived-luminance formula as `settings::is_light_color`.
fn is_light(c: Color32) -> bool {
    let brightness = (0.299 * c.r() as f32
        + 0.587 * c.g() as f32
        + 0.114 * c.b() as f32)
        / 255.0;
    brightness > 0.5
}
