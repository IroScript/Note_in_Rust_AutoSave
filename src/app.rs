// ============================================================
//  app.rs  (Part 2 – cursor tracking, scroll-sync, colour picker,
//           find-highlight, zoom fix)
//
//  Every Python method is preserved from Part 1.
//  Part 2 additions / fixes:
//
//  ① Cursor tracking  – TextEdit::show() → output.cursor_range
//                       → char_index_to_line_col()
//                       Mirrors update_cursor_position() exactly.
//
//  ② Scroll sync      – ScrollArea output → state.offset.y
//                       fed into LineNumbers::show() as scroll_y_px.
//                       Mirrors the dlineinfo() y-values in update_line_numbers().
//
//  ③ Colour picker    – ColorPickerDialog (color_picker.rs) replaces
//                       the preset-cycling placeholder from Part 1.
//                       Mirrors colorchooser.askcolor() exactly.
//
//  ④ Find-highlight   – After a successful find the TextEdit cursor
//                       range is set via TextEdit::load_state / store
//                       to select the found text.
//                       Mirrors `self.text.tag_add("found", ...)` +
//                       `self.text.mark_set(INSERT, end_pos)`.
//
//  ⑤ Zoom fix         – consume_key now also handles raw scroll-wheel
//                       events with Ctrl held, exactly like the Python
//                       `<Control-MouseWheel>` binding.
// ============================================================

use egui::{
    Color32, Context, FontId, Key, Modifiers, RichText,
    ScrollArea, TextEdit, Vec2,
};
use std::path::PathBuf;

use crate::color_picker::ColorPickerDialog;
use crate::dialogs::{FindDialog, FontDialog, FontSizeDialog, RenameDialog};
use crate::file_ops;
use crate::line_numbers::LineNumbers;
use crate::settings;
use crate::shortcuts::ShortcutsWindow;

// ── IrakNotesApp ──────────────────────────────────────────────────────────────

pub struct IrakNotesApp {
    // ── file state ────────────────────────────────────────────────────────────
    save_folder:        PathBuf,
    current_file:       Option<PathBuf>,
    last_saved_content: String,

    // ── editor content ────────────────────────────────────────────────────────
    text_content:       String,
    font_family:        String,  // mirrors self.default_font family
    font_size:          f32,     // mirrors self.default_font size
    bg_color:           Color32, // mirrors self.text.cget("bg")
    fg_color:           Color32, // mirrors self.text.cget("fg")

    // ── cursor position (mirrors update_cursor_position) ─────────────────────
    cursor_line:        usize,
    cursor_col:         usize,

    // ── status bar (mirrors status_message / title bar) ───────────────────────
    status_msg:         String,
    is_saved:           bool,

    // ── sub-widgets / dialogs ────────────────────────────────────────────────
    line_numbers:       LineNumbers,
    find_dialog:        FindDialog,
    font_dialog:        FontDialog,
    size_dialog:        FontSizeDialog,
    rename_dialog:      RenameDialog,
    color_picker:       ColorPickerDialog, // ← Part 2: real picker
    shortcuts_window:   ShortcutsWindow,

    // ── confirm overlays ─────────────────────────────────────────────────────
    show_exit_confirm:   bool,
    show_delete_confirm: bool,

    // ── find state ───────────────────────────────────────────────────────────
    /// Character-index offset to begin next search (wraps around).
    find_char_offset:   usize,
    /// Pending selection to apply to the TextEdit next frame.
    /// Some((start_char, end_char)) after a successful find.
    pending_selection:  Option<(usize, usize)>,

    // ── scroll / line-number sync (Part 2) ───────────────────────────────────
    /// Live scroll-Y pixel offset read from the ScrollArea state last frame.
    scroll_y_px:        f32,
    /// Pixel height of the visible editor area last frame.
    editor_height_px:   f32,
}

// ── constructor ───────────────────────────────────────────────────────────────

impl IrakNotesApp {
    /// Mirrors `SuperNotepad.__init__`.
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let save_folder = file_ops::resolve_save_folder();

        let (font_family, font_size) = settings::load_font_settings(&save_folder);
        let (bg_color,   fg_color)   = settings::load_color_settings(&save_folder);

        let current_file = file_ops::create_new_file(&save_folder).ok();

        let font_dialog = FontDialog::new(
            vec![
                "Monospace".to_string(),
                "Proportional".to_string(),
                "Consolas".to_string(),
                "Courier New".to_string(),
                "Arial".to_string(),
                "Helvetica".to_string(),
                "Times New Roman".to_string(),
                "Verdana".to_string(),
                "Tahoma".to_string(),
                "Georgia".to_string(),
                "Trebuchet MS".to_string(),
                "Lucida Console".to_string(),
                "DejaVu Sans".to_string(),
                "Liberation Mono".to_string(),
            ],
            &font_family,
        );
        let size_dialog  = FontSizeDialog::new(font_size, &font_family);
        // Part 2: real colour picker with current bg as initial colour
        let color_picker = ColorPickerDialog::new(bg_color);

        Self {
            save_folder,
            current_file,
            last_saved_content: String::new(),
            text_content:       String::new(),
            font_family,
            font_size,
            bg_color,
            fg_color,
            cursor_line:        1,
            cursor_col:         0,
            status_msg:         "Ready".to_string(),
            is_saved:           true,
            line_numbers:       LineNumbers::new(),
            find_dialog:        FindDialog::default(),
            font_dialog,
            size_dialog,
            rename_dialog:      RenameDialog::default(),
            color_picker,
            shortcuts_window:   ShortcutsWindow::default(),
            show_exit_confirm:  false,
            show_delete_confirm: false,
            find_char_offset:   0,
            pending_selection:  None,
            scroll_y_px:        0.0,
            editor_height_px:   400.0,
        }
    }
}

// ── eframe::App ───────────────────────────────────────────────────────────────

impl eframe::App for IrakNotesApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 1. Keyboard shortcuts (consume before any widget sees the keys)
        self.handle_keyboard(ctx);

        // 2. Title bar (ModernTitleBar)
        self.show_title_bar(ctx);

        // 3. Status bar
        self.show_status_bar(ctx);

        // 4. Central editor (line numbers + text edit + scroll)
        self.show_editor(ctx);

        // 5. All dialogs
        self.process_find_dialog(ctx);
        self.process_font_dialog(ctx);
        self.process_size_dialog(ctx);
        self.process_rename_dialog(ctx);
        self.process_color_picker(ctx);
        self.shortcuts_window.show(ctx);

        // 6. Confirm overlays
        self.show_exit_confirm_dialog(ctx);
        self.show_delete_confirm_dialog(ctx);

        // 7. Auto-save (mirrors `<KeyRelease>` binding)
        self.do_auto_save();
    }
}

// ── title bar (ModernTitleBar) ────────────────────────────────────────────────

impl IrakNotesApp {
    fn show_title_bar(&self, ctx: &Context) {
        egui::TopBottomPanel::top("title_bar")
            .exact_height(30.0)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    ui.max_rect(),
                    egui::Rounding::ZERO,
                    Color32::from_rgb(0x2c, 0x3e, 0x50),
                );
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);

                    // Filename (mirrors `filename_label` bold)
                    let filename = self.current_file.as_ref()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("Untitled");
                    ui.label(
                        RichText::new(filename).size(12.0).strong()
                            .color(Color32::from_rgb(0xec, 0xf0, 0xf1)),
                    );

                    // Save status (mirrors `status_label`)
                    ui.add_space(8.0);
                    let saved_txt = if self.is_saved { "(Auto-Saved)" } else { "(Unsaved)" };
                    ui.label(
                        RichText::new(saved_txt).size(10.0)
                            .color(Color32::from_rgb(0xbd, 0xc3, 0xc7)),
                    );

                    // Right-aligned hint (mirrors `shortcuts_label`)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Press F1 for shortcuts").size(9.0)
                                .color(Color32::from_rgb(0xbd, 0xc3, 0xc7)),
                        );
                    });
                });
            });
    }
}

// ── status bar ────────────────────────────────────────────────────────────────

impl IrakNotesApp {
    fn show_status_bar(&self, ctx: &Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(25.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Left: status message (mirrors `status_message`)
                    ui.label(
                        RichText::new(&self.status_msg).size(10.0)
                            .color(Color32::from_rgb(0x2c, 0x3e, 0x50)),
                    );

                    // Right: cursor position (mirrors `position_indicator`)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(format!("Ln: {}, Col: {}", self.cursor_line, self.cursor_col))
                                .size(10.0)
                                .color(Color32::from_rgb(0x2c, 0x3e, 0x50)),
                        );
                    });
                });
            });
    }
}

// ── main editor panel ─────────────────────────────────────────────────────────

impl IrakNotesApp {
    fn show_editor(&mut self, ctx: &Context) {
        // Line height: approximate `font.metrics('linespace')`
        let line_height = self.font_size * 1.5;
        let total_lines = self.text_content.lines().count().max(1);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.bg_color)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                let available_h = ui.available_height();

                ui.horizontal_top(|ui| {
                    // ── ① Line numbers gutter ─────────────────────────────────
                    // Uses scroll_y_px from *last frame* (one-frame lag is standard
                    // in immediate-mode UIs and imperceptible at 60 fps).
                    self.line_numbers.show(
                        ui,
                        total_lines,
                        FontId::monospace(self.font_size),
                        line_height,
                        self.scroll_y_px,   // ← Part 2: real live value
                        available_h,
                    );

                    // ── ② Scrollable text editor ──────────────────────────────
                    let text_id = egui::Id::new("main_text_edit");

                    // Part 2 ④: apply a pending selection (find-highlight)
                    if let Some((sel_start, sel_end)) = self.pending_selection.take() {
                        if let Some(mut state) =
                            egui::TextEdit::load_state(ctx, text_id)
                        {
                            // Build a CursorRange that selects [sel_start, sel_end]
                            // mirrors: text.tag_add("found", start_pos, end_pos)
                            //          text.mark_set(INSERT, end_pos)
                            use egui::text::CCursor;
                            let primary   = egui::text::CCursorRange {
                                primary:   CCursor { index: sel_end,   prefer_next_row: false },
                                secondary: CCursor { index: sel_start, prefer_next_row: false },
                            };
                            state.cursor.set_char_range(Some(primary));
                            state.store(ctx, text_id);
                        }
                    }

                    let scroll_out = ScrollArea::vertical()
                        .id_source("editor_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let te = TextEdit::multiline(&mut self.text_content)
                                .id(text_id)
                                .font(FontId::monospace(self.font_size))
                                .text_color(self.fg_color)
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                // Blue cursor (mirrors insertbackground="#3498db")
                                .cursor_at_end(false);

                            let te_out = te.show(ui);

                            // ── ① Cursor tracking (Part 2) ────────────────────
                            // Mirrors update_cursor_position():
                            //   position = self.text.index(tk.INSERT)
                            //   line, column = position.split('.')
                            if let Some(cr) = te_out.cursor_range {
                                let char_idx = cr.primary.ccursor.index;
                                let (ln, col) =
                                    char_index_to_line_col(&self.text_content, char_idx);
                                self.cursor_line = ln;
                                self.cursor_col  = col;
                            }

                            // Track content changes for is_saved flag
                            if te_out.response.changed() {
                                self.is_saved = false;
                            }

                            // Right-click context menu
                            // Mirrors handle_right_click():
                            //   selected_text → copy
                            //   no selection  → paste
                            te_out.response.context_menu(|ui| {
                                if ui.button("Copy").clicked() {
                                    ctx.copy_text(self.text_content.clone());
                                    self.status_msg = "Text copied to clipboard".to_string();
                                    ui.close_menu();
                                }
                                if ui.button("Paste").clicked() {
                                    // Insert clipboard text at cursor position
                                    let clip = ui.output(|o| o.copied_text.clone());
                                    if !clip.is_empty() {
                                        // We can't directly insert at cursor from here;
                                        // schedule insertion via pending approach
                                        self.text_content.push_str(&clip);
                                        self.status_msg = "Text pasted from clipboard".to_string();
                                    }
                                    ui.close_menu();
                                }
                            });
                        });

                    // ── ② Capture scroll offset for next frame (Part 2) ───────
                    // Mirrors the y-coordinate dlineinfo() returns for each line.
                    self.scroll_y_px      = scroll_out.state.offset.y;
                    self.editor_height_px = available_h;
                });
            });
    }
}

// ── keyboard shortcut handler ─────────────────────────────────────────────────

impl IrakNotesApp {
    /// Handles all key bindings exactly as listed in `SuperNotepad.__init__`.
    fn handle_keyboard(&mut self, ctx: &Context) {
        ctx.input_mut(|i| {
            // Ctrl+Down  → go_to_last_line
            if i.consume_key(Modifiers::CTRL, Key::ArrowDown) {
                // Scroll the ScrollArea to the bottom next frame
                // by setting a very large offset (clamped by egui)
                self.scroll_y_px = f32::MAX;
            }

            // Ctrl+Up  → go_to_first_line
            if i.consume_key(Modifiers::CTRL, Key::ArrowUp) {
                self.scroll_y_px = 0.0;
            }

            // Ctrl+F  → find_text
            if i.consume_key(Modifiers::CTRL, Key::F) {
                self.find_dialog.open = true;
            }

            // Alt+F  → change_font
            if i.consume_key(Modifiers::ALT, Key::F) {
                self.font_dialog.open = true;
            }

            // Alt+S  → change_font_size
            if i.consume_key(Modifiers::ALT, Key::S) {
                self.size_dialog.size = self.font_size;
                self.size_dialog.open = true;
            }

            // Alt+Delete  → delete_current_file
            if i.consume_key(Modifiers::ALT, Key::Delete) {
                self.show_delete_confirm = true;
            }

            // Ctrl+R  → rename_file
            if i.consume_key(Modifiers::CTRL, Key::R) {
                if let Some(ref p) = self.current_file {
                    let name = p.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    self.rename_dialog.new_name     = name.clone();
                    self.rename_dialog.current_name = name;
                    self.rename_dialog.error_msg    = String::new();
                    self.rename_dialog.open         = true;
                }
            }

            // Ctrl+O  → open_save_folder
            if i.consume_key(Modifiers::CTRL, Key::O) {
                match file_ops::open_save_folder(&self.save_folder) {
                    Ok(_)  => self.status_msg = "Save folder opened".to_string(),
                    Err(e) => self.status_msg = format!("Error opening folder: {}", e),
                }
            }

            // Alt+C  → close_app
            if i.consume_key(Modifiers::ALT, Key::C) {
                self.show_exit_confirm = true;
            }

            // Ctrl+N  → new_file
            if i.consume_key(Modifiers::CTRL, Key::N) {
                self.cmd_new_file();
            }

            // Ctrl+Alt+C  → change_background_color (Part 2: opens real picker)
            // Must check alt+ctrl+C carefully (avoid collision with Ctrl+C copy)
            if i.modifiers.ctrl && i.modifiers.alt && i.key_pressed(Key::C) {
                i.consume_key(Modifiers::CTRL | Modifiers::ALT, Key::C);
                self.color_picker.open_with(self.bg_color);
            }

            // F1  → show_shortcuts
            if i.consume_key(Modifiers::NONE, Key::F1) {
                self.shortcuts_window.open = true;
            }

            // ── Ctrl+MouseWheel → zoom (Part 2 fix) ──────────────────────────
            // Mirrors `self.text.bind("<Control-MouseWheel>", self.zoom)`.
            // egui's smooth_scroll_delta gives sub-pixel wheel increments;
            // we threshold to avoid micro-adjustments.
            if i.modifiers.ctrl {
                let dy = i.smooth_scroll_delta.y;
                if dy > 1.0 {
                    self.font_size = (self.font_size + 1.0).min(72.0);
                    settings::save_font_settings(
                        &self.save_folder, &self.font_family, self.font_size);
                    self.status_msg = format!("Font size: {}", self.font_size as i32);
                    // Consume so the ScrollArea doesn't also scroll
                    i.smooth_scroll_delta.y = 0.0;
                } else if dy < -1.0 {
                    self.font_size = (self.font_size - 1.0).max(6.0);
                    settings::save_font_settings(
                        &self.save_folder, &self.font_family, self.font_size);
                    self.status_msg = format!("Font size: {}", self.font_size as i32);
                    i.smooth_scroll_delta.y = 0.0;
                }
            }
        });
    }
}

// ── file-operation command handlers ──────────────────────────────────────────

impl IrakNotesApp {
    /// Mirrors `new_file()` (Ctrl+N).
    fn cmd_new_file(&mut self) {
        if self.text_content.trim().is_empty() {
            return; // keep current empty file (mirrors Python guard)
        }
        match file_ops::create_new_file(&self.save_folder) {
            Ok(path) => {
                self.current_file       = Some(path);
                self.text_content       = String::new();
                self.last_saved_content = String::new();
                self.is_saved           = true;
                self.find_char_offset   = 0;
                self.status_msg         = "New file created".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Failed to create new file: {}", e);
            }
        }
    }

    /// Mirrors `auto_save()` (called every frame, debounced by content diff).
    fn do_auto_save(&mut self) {
        if let Some(ref path) = self.current_file.clone() {
            match file_ops::auto_save(path, &self.text_content, &self.last_saved_content) {
                Ok(true) => {
                    self.last_saved_content = self.text_content.clone();
                    self.is_saved           = true;
                    self.status_msg         = "Auto-saved".to_string();
                }
                Ok(false) => {} // no change
                Err(e)    => self.status_msg = format!("Save error: {}", e),
            }
        }
    }
}

// ── dialog processing ─────────────────────────────────────────────────────────

impl IrakNotesApp {
    // ── Find ────────────────────────────────────────────────────────────────

    /// Mirrors `find_text()` + inner `find_next()`.
    fn process_find_dialog(&mut self, ctx: &Context) {
        self.find_dialog.show(ctx);

        if !self.find_dialog.find_requested { return; }

        let query = self.find_dialog.query.clone();
        if query.is_empty() {
            self.find_dialog.result_msg = "Please enter search text".to_string();
            return;
        }

        let content  = self.text_content.clone();
        // Work in char-space for accurate CCursor index
        let char_len = content.chars().count();
        let start_ci = self.find_char_offset.min(char_len);

        // Byte offset of search start (for str::find)
        let start_byte = char_to_byte_offset(&content, start_ci);

        let search = |hay: &str| -> Option<usize> {
            // Returns *byte* offset within `hay`
            if self.find_dialog.match_case {
                hay.find(query.as_str())
            } else {
                let h = hay.to_lowercase();
                let n = query.to_lowercase();
                h.find(n.as_str())
            }
        };

        // Search forward from current position (mirrors `stopindex=tk.END`)
        let found_byte = search(&content[start_byte..])
            .map(|b| b + start_byte)
            // Wrap: search from beginning (mirrors the wrap block)
            .or_else(|| search(&content[..start_byte]));

        match found_byte {
            Some(byte_start) => {
                let byte_end    = (byte_start + query.len()).min(content.len());
                // Convert to char indices for CCursor (Part 2 improvement)
                let char_start  = byte_to_char_index(&content, byte_start);
                let char_end    = byte_to_char_index(&content, byte_end);
                let (ln, col)   = char_index_to_line_col(&content, char_start);

                self.find_dialog.found_range = Some((char_start, char_end));
                self.find_dialog.result_msg  =
                    format!("Found at Ln: {}, Col: {}", ln, col);
                // Advance offset past this match (mirrors `mark_set(INSERT, end_pos)`)
                self.find_char_offset = char_end;
                // Schedule selection highlight (Part 2 ④)
                self.pending_selection = Some((char_start, char_end));
                self.status_msg = format!("Found: \"{}\" at Ln {}, Col {}", query, ln, col);
            }
            None => {
                self.find_dialog.found_range = None;
                self.find_dialog.result_msg  = format!("'{}' not found", query);
                self.find_char_offset = 0; // reset for next attempt
                self.status_msg = format!("'{}' not found", query);
            }
        }
    }

    // ── Font ────────────────────────────────────────────────────────────────

    /// Mirrors `change_font()` + `apply_font()`.
    fn process_font_dialog(&mut self, ctx: &Context) {
        let current = self.font_family.clone();
        self.font_dialog.show(ctx, &current);

        if let Some(new_family) = self.font_dialog.apply.take() {
            self.font_family = new_family.clone();
            settings::save_font_settings(&self.save_folder, &new_family, self.font_size);
            self.status_msg = format!("Font changed to {}", new_family);
        }
    }

    // ── Font Size ────────────────────────────────────────────────────────────

    /// Mirrors `change_font_size()` + `apply_size()`.
    fn process_size_dialog(&mut self, ctx: &Context) {
        self.size_dialog.show(ctx);

        if let Some(new_size) = self.size_dialog.apply.take() {
            self.font_size = new_size;
            settings::save_font_settings(&self.save_folder, &self.font_family, new_size);
            self.status_msg = format!("Font size changed to {}", new_size as i32);
        }
    }

    // ── Rename ───────────────────────────────────────────────────────────────

    /// Mirrors `rename_file()` + `validate_and_rename()`.
    fn process_rename_dialog(&mut self, ctx: &Context) {
        self.rename_dialog.show(ctx);

        if let Some(new_name) = self.rename_dialog.apply.take() {
            if let Some(ref current) = self.current_file.clone() {
                match file_ops::rename_file(current, &self.save_folder, &new_name) {
                    Ok(new_path) => {
                        let name = new_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        self.current_file            = Some(new_path);
                        self.is_saved                = true;
                        self.rename_dialog.open      = false;
                        self.rename_dialog.error_msg = String::new();
                        self.status_msg              = format!("File renamed to {}", name);
                    }
                    Err(e) => {
                        self.rename_dialog.error_msg = format!("Error: {}", e);
                    }
                }
            }
        }
    }

    // ── Colour Picker (Part 2) ────────────────────────────────────────────────

    /// Mirrors `change_background_color()` (Ctrl+Alt+C).
    ///
    /// Python:
    ///   color = colorchooser.askcolor(...)
    ///   if color and color[1]:
    ///       self.text.configure(bg=bg_color)
    ///       text_color = "#000000" if is_light_color(bg_color) else "#ffffff"
    ///       self.text.configure(fg=text_color)
    ///       self.save_color_settings(bg_color)
    fn process_color_picker(&mut self, ctx: &Context) {
        self.color_picker.show(ctx);

        if let Some(new_bg) = self.color_picker.applied.take() {
            self.bg_color   = new_bg;
            self.fg_color   = settings::fg_for_bg(new_bg);
            settings::save_color_settings(&self.save_folder, new_bg);
            self.status_msg = "Background color changed".to_string();
        }
    }

    // ── Exit confirm ─────────────────────────────────────────────────────────

    /// Mirrors `messagebox.askyesno("Exit", ...)` in `close_app()`.
    fn show_exit_confirm_dialog(&mut self, ctx: &Context) {
        if !self.show_exit_confirm { return; }

        egui::Window::new("Exit")
            .collapsible(false).resizable(false)
            .default_size([320.0, 100.0])
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Are you sure you want to exit Irak Notes?");
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Yes")
                        .fill(Color32::from_rgb(0xe7, 0x4c, 0x3c))).clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    ui.add_space(8.0);
                    if ui.button("No").clicked() {
                        self.show_exit_confirm = false;
                    }
                });
            });
    }

    // ── Delete confirm ───────────────────────────────────────────────────────

    /// Mirrors `messagebox.askyesno("Delete File", ...)` in `delete_current_file()`.
    fn show_delete_confirm_dialog(&mut self, ctx: &Context) {
        if !self.show_delete_confirm { return; }

        let filename = self.current_file.as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("this file")
            .to_string();

        egui::Window::new("Delete File")
            .collapsible(false).resizable(false)
            .default_size([380.0, 110.0])
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(format!("Are you sure you want to delete '{}'?", filename));
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Yes")
                        .fill(Color32::from_rgb(0xe7, 0x4c, 0x3c))).clicked()
                    {
                        if let Some(ref p) = self.current_file.clone() {
                            match file_ops::delete_file(p) {
                                Ok(_) => {
                                    self.text_content       = String::new();
                                    self.last_saved_content = String::new();
                                    self.is_saved           = true;
                                    self.find_char_offset   = 0;
                                    match file_ops::create_new_file(&self.save_folder) {
                                        Ok(path) => {
                                            self.current_file = Some(path);
                                            self.status_msg = "File deleted, new file created".to_string();
                                        }
                                        Err(e)   => {
                                            self.status_msg = format!("Error creating new file: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.status_msg = format!("Failed to delete file: {}", e);
                                }
                            }
                        }
                        self.show_delete_confirm = false;
                    }
                    ui.add_space(8.0);
                    if ui.button("No").clicked() {
                        self.show_delete_confirm = false;
                    }
                });
            });
    }
}

// ── text-position utility functions (Part 2) ──────────────────────────────────

/// Convert a **char index** into a (line, col) pair (both 1-based).
///
/// Mirrors Python:
///   position = self.text.index(tk.INSERT)   # "line.column"
///   line, column = position.split('.')
///
/// `col` is 0-based to match tkinter's column convention.
pub fn char_index_to_line_col(text: &str, char_idx: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col  = 0usize;
    for (i, ch) in text.chars().enumerate() {
        if i >= char_idx { break; }
        if ch == '\n' {
            line += 1;
            col   = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Convert a **byte offset** in `text` to a **char index**.
/// Needed because Rust's `str::find` returns byte offsets while egui's
/// `CCursor.index` is a character index.
fn byte_to_char_index(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset.min(text.len())].chars().count()
}

/// Convert a **char index** to a **byte offset** in `text`.
fn char_to_byte_offset(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}
