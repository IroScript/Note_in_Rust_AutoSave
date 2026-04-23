// ============================================================
//  dialogs.rs  (Part 2 – FontDialog listbox bug fixed)
//
//  All modal dialog windows.  Mirrors the Python methods:
//    • find_text()          – Ctrl+F
//    • change_font()        – Alt+F      ← listbox bug fixed
//    • change_font_size()   – Alt+S
//    • rename_file()        – Ctrl+R
//
//  The colour-picker dialog lives in color_picker.rs.
// ============================================================

use egui::{Color32, RichText, ScrollArea, TextEdit};

// ── FindDialog ────────────────────────────────────────────────────────────────

/// Mirrors the `find_text()` Toplevel.
pub struct FindDialog {
    pub open:           bool,
    pub query:          String,
    pub match_case:     bool,
    pub result_msg:     String,
    pub find_requested: bool,
    pub focus_query_next_frame: bool,
}

impl Default for FindDialog {
    fn default() -> Self {
        Self {
            open:           false,
            query:          String::new(),
            match_case:     false,
            result_msg:     String::new(),
            find_requested: false,
            focus_query_next_frame: false,
        }
    }
}

impl FindDialog {
    /// Draw the find dialog.  No-op when `self.open == false`.
    ///
    /// Mirrors `find_text()` Toplevel layout:
    ///   "Search for:" label + entry
    ///   "Match case" checkbox
    ///   result feedback label
    ///   [Find Next]  [Close]
    pub fn show(&mut self, ctx: &egui::Context) {
        self.find_requested = false;

        if !self.open { return; }

        let mut open = self.open;
        egui::Window::new("Find Text")
            .open(&mut open)
            .resizable(false)
            .default_size([420.0, 150.0])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.add_space(10.0);

                // "Search for:" + entry
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Search for:").size(11.0));
                    ui.add_space(4.0);
                    let resp = ui.add_sized(
                        [270.0, 22.0],
                        TextEdit::singleline(&mut self.query).font(egui::TextStyle::Body),
                    );
                    if self.focus_query_next_frame {
                        resp.request_focus();
                        self.focus_query_next_frame = false;
                    }
                    if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.find_requested = true;
                    }
                });

                ui.add_space(4.0);

                // "Match case" checkbox
                ui.horizontal(|ui| {
                    ui.add_space(82.0);
                    ui.checkbox(&mut self.match_case, "Match case");
                });

                ui.add_space(4.0);

                // Feedback label (mirrors `result_var`)
                if !self.result_msg.is_empty() {
                    ui.label(
                        RichText::new(&self.result_msg)
                            .size(10.0)
                            .italics()
                            .color(Color32::from_rgb(0x7f, 0x8c, 0x8d)),
                    );
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                // [Find Next]  [Close]
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Close").size(10.0).color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x95, 0xa5, 0xa6)),
                        ).clicked() {
                            self.open = false;
                        }

                        ui.add_space(4.0);

                        if ui.add(
                            egui::Button::new(
                                RichText::new("Find Next").size(10.0).strong().color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x34, 0x98, 0xdb)),
                        ).clicked() {
                            self.find_requested = true;
                        }
                    });
                });

                ui.add_space(6.0);
            });

        self.open = open;
    }
}

// ── FontDialog ────────────────────────────────────────────────────────────────

/// Mirrors `change_font()` Toplevel.
///
/// Part 2 fix: Part 1 called `ui.selectable_label` twice per font entry
/// (once for the label, once for an empty string), which caused duplicate
/// click regions.  The fix is a single `selectable_value` call per row.
pub struct FontDialog {
    pub open:          bool,
    pub search_filter: String,
    pub selected_font: String,
    /// Set to Some(family) for one frame when the user confirms.
    pub apply:         Option<String>,

    filtered:          Vec<String>, // currently displayed list
    all_fonts:         Vec<String>, // unfiltered master list
}

impl FontDialog {
    /// Mirrors the Toplevel init: put current font at top, show all families.
    pub fn new(all_fonts: Vec<String>, current_family: &str) -> Self {
        let mut fonts = all_fonts;
        // Put current font at the top (mirrors Python logic)
        if let Some(pos) = fonts.iter().position(|f| f == current_family) {
            let cur = fonts.remove(pos);
            fonts.insert(0, cur);
        }
        let filtered = fonts.clone();
        Self {
            open:          false,
            search_filter: String::new(),
            selected_font: current_family.to_string(),
            apply:         None,
            filtered,
            all_fonts:     fonts,
        }
    }

    /// Re-filter `self.filtered` after the search box changes.
    fn refresh_filter(&mut self) {
        let q = self.search_filter.to_lowercase();
        self.filtered = self.all_fonts
            .iter()
            .filter(|f| f.to_lowercase().contains(&q))
            .cloned()
            .collect();
        // Keep a valid selection
        if !self.filtered.contains(&self.selected_font) {
            if let Some(f) = self.filtered.first() {
                self.selected_font = f.clone();
            }
        }
    }

    /// Draw the font-selection dialog.
    ///
    /// Mirrors `change_font()` Toplevel layout:
    ///   header
    ///   search box
    ///   scrollable listbox
    ///   preview swatch
    ///   current-font label
    ///   [Apply]  [Cancel]
    pub fn show(&mut self, ctx: &egui::Context, current_family: &str) {
        self.apply = None;
        if !self.open { return; }

        let mut open = self.open;
        egui::Window::new("Select Font")
            .open(&mut open)
            .resizable(true)
            .default_size([460.0, 460.0])
            .collapsible(false)
            .show(ctx, |ui| {
                // Header
                ui.label(
                    RichText::new("Select Font Family")
                        .size(15.0)
                        .strong()
                        .color(Color32::from_rgb(0x2c, 0x3e, 0x50)),
                );
                ui.add_space(8.0);

                // Search box (mirrors `search_entry`)
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Search:").size(11.0));
                    ui.add_space(4.0);
                    let prev = self.search_filter.clone();
                    ui.add_sized([300.0, 22.0], TextEdit::singleline(&mut self.search_filter));
                    if self.search_filter != prev {
                        self.refresh_filter();
                    }
                });
                ui.add_space(6.0);

                // ── Listbox (mirrors `font_list` Listbox) ─────────────────────
                // Part 2 fix: a single `selectable_value` per row replaces the
                // broken double `selectable_label` pattern from Part 1.
                egui::Frame::none()
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xcc, 0xcc, 0xcc)))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .id_source("font_list")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                // Clone to satisfy borrow checker
                                let fonts: Vec<String> = self.filtered.clone();
                                for font in &fonts {
                                    // `selectable_value` is the idiomatic
                                    // single-call equivalent of Listbox row.
                                    let resp = ui.selectable_value(
                                        &mut self.selected_font,
                                        font.clone(),
                                        font.as_str(),
                                    );
                                    // Double-click → immediate apply
                                    // (mirrors `font_list.bind("<Double-Button-1>", ...)`)
                                    if resp.double_clicked() {
                                        self.apply = Some(font.clone());
                                        self.open  = false;
                                    }
                                }
                            });
                    });

                ui.add_space(8.0);

                // Preview swatch (mirrors `preview_label`)
                egui::Frame::none()
                    .fill(Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xcc, 0xcc, 0xcc)))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("AaBbCcDdEe 123456789")
                                .font(egui::FontId::proportional(16.0)),
                        );
                    });

                ui.add_space(4.0);

                // Current-font info (mirrors `current_font_label`)
                ui.label(
                    RichText::new(format!("Current Font: {}", current_family))
                        .size(9.0)
                        .color(Color32::from_rgb(0x7f, 0x8c, 0x8d)),
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(4.0);

                // [Apply]  [Cancel]
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Apply").size(11.0).strong().color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x34, 0x98, 0xdb)),
                        ).clicked() {
                            self.apply = Some(self.selected_font.clone());
                            self.open  = false;
                        }
                        ui.add_space(8.0);
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Cancel").size(11.0).color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x95, 0xa5, 0xa6)),
                        ).clicked() {
                            self.open = false;
                        }
                    });
                });
                ui.add_space(6.0);
            });

        self.open = open;
    }
}

// ── FontSizeDialog ────────────────────────────────────────────────────────────

/// Mirrors `change_font_size()` Toplevel with slider.
pub struct FontSizeDialog {
    pub open:  bool,
    pub size:  f32,            // mirrors `size_var`
    pub apply: Option<f32>,    // Some(size) when confirmed
}

impl FontSizeDialog {
    pub fn new(current_size: f32, _font_family: &str) -> Self {
        Self { open: false, size: current_size, apply: None }
    }

    /// Draw the font-size dialog.
    ///
    /// Mirrors `change_font_size()` Toplevel layout:
    ///   header
    ///   "Size: N" label  +  ttk.Scale (from_=6, to=72)
    ///   preview swatch
    ///   [Apply]  [Cancel]
    pub fn show(&mut self, ctx: &egui::Context) {
        self.apply = None;
        if !self.open { return; }

        let mut open = self.open;
        egui::Window::new("Font Size")
            .open(&mut open)
            .resizable(false)
            .default_size([360.0, 210.0])
            .collapsible(false)
            .show(ctx, |ui| {
                // Header
                ui.label(
                    RichText::new("Select Font Size")
                        .size(15.0)
                        .strong()
                        .color(Color32::from_rgb(0x2c, 0x3e, 0x50)),
                );
                ui.add_space(12.0);

                // "Size: N" label (mirrors `size_label` that updates with slider)
                ui.label(RichText::new(format!("Size: {}", self.size as i32)).size(12.0));
                ui.add_space(4.0);

                // Slider (mirrors `ttk.Scale from_=6 to=72 orient="horizontal"`)
                ui.add(
                    egui::Slider::new(&mut self.size, 6.0..=72.0)
                        .show_value(false)
                        .clamp_to_range(true),
                );

                ui.add_space(12.0);

                // Preview swatch (mirrors `preview_text` label)
                egui::Frame::none()
                    .fill(Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xcc, 0xcc, 0xcc)))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("AaBbCc 123")
                                .font(egui::FontId::proportional(self.size)),
                        );
                    });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(6.0);

                // [Apply]  [Cancel]
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Apply").size(11.0).strong().color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x34, 0x98, 0xdb)),
                        ).clicked() {
                            // Mirrors `if 6 <= new_size <= 72` validation
                            if self.size >= 6.0 && self.size <= 72.0 {
                                self.apply = Some(self.size);
                                self.open  = false;
                            }
                        }
                        ui.add_space(8.0);
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Cancel").size(11.0).color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x95, 0xa5, 0xa6)),
                        ).clicked() {
                            self.open = false;
                        }
                    });
                });
                ui.add_space(6.0);
            });

        self.open = open;
    }
}

// ── RenameDialog ──────────────────────────────────────────────────────────────

/// Mirrors the rename Toplevel inside `rename_file()`.
pub struct RenameDialog {
    pub open:         bool,
    pub new_name:     String,
    pub error_msg:    String,       // mirrors `status_var` (red)
    pub current_name: String,       // mirrors `current_name` label
    pub apply:        Option<String>,
    pub focus_name_next_frame: bool,
}

impl Default for RenameDialog {
    fn default() -> Self {
        Self {
            open:         false,
            new_name:     String::new(),
            error_msg:    String::new(),
            current_name: String::new(),
            apply:        None,
            focus_name_next_frame: false,
        }
    }
}

impl RenameDialog {
    /// Draw the rename dialog.
    ///
    /// Mirrors `rename_file()` Toplevel layout:
    ///   header
    ///   "New name:" label + entry
    ///   current-file info label
    ///   error / status label
    ///   [Rename]  [Cancel]
    pub fn show(&mut self, ctx: &egui::Context) {
        self.apply = None;
        if !self.open { return; }

        let mut open = self.open;
        egui::Window::new("Rename File")
            .open(&mut open)
            .resizable(false)
            .default_size([420.0, 190.0])
            .collapsible(false)
            .show(ctx, |ui| {
                // Header
                ui.label(
                    RichText::new("Rename File")
                        .size(15.0)
                        .strong()
                        .color(Color32::from_rgb(0x2c, 0x3e, 0x50)),
                );
                ui.add_space(10.0);

                // "New name:" + entry (mirrors `name_entry`)
                ui.horizontal(|ui| {
                    ui.label(RichText::new("New name:").size(11.0));
                    ui.add_space(4.0);
                    let resp = ui.add_sized(
                        [280.0, 22.0],
                        TextEdit::singleline(&mut self.new_name),
                    );
                    if self.focus_name_next_frame {
                        resp.request_focus();
                        self.focus_name_next_frame = false;
                    }
                    if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.new_name.trim().is_empty() {
                            self.apply = Some(self.new_name.clone());
                        }
                    }
                });

                ui.add_space(6.0);

                // Info label (mirrors `info_label` italic grey text)
                ui.label(
                    RichText::new(format!("Current file: {}", self.current_name))
                        .size(10.0)
                        .italics()
                        .color(Color32::from_rgb(0x7f, 0x8c, 0x8d)),
                );

                ui.add_space(4.0);

                // Error label (mirrors `status_label` in red #e74c3c)
                if !self.error_msg.is_empty() {
                    ui.label(
                        RichText::new(&self.error_msg)
                            .size(10.0)
                            .color(Color32::from_rgb(0xe7, 0x4c, 0x3c)),
                    );
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                // [Rename]  [Cancel]
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Cancel").size(11.0).color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x95, 0xa5, 0xa6)),
                        ).clicked() {
                            self.open = false;
                        }
                        ui.add_space(8.0);
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Rename").size(11.0).strong().color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x34, 0x98, 0xdb)),
                        ).clicked() {
                            self.apply = Some(self.new_name.clone());
                        }
                    });
                });
                ui.add_space(6.0);
            });

        self.open = open;
    }
}
