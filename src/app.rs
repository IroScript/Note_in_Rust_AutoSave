// ============================================================
//  app.rs  –  Fixed version
//
//  Fixes applied:
//    1. Right-click: selection → copy, no selection → paste (no context menu)
//    2. Ctrl+Up → go to first line (top), Ctrl+Down → go to last line (bottom)
//    3. Ctrl+Left → go to start of CURRENT line, Ctrl+Right → go to end of CURRENT line
//       (no crash, no smart navigation)
//    4. Find text shows "Line N, Column C/Total"
//    5. Opening the app auto-focuses editor with blinking cursor
//    6. Ctrl+F focuses find input with blinking cursor
//    7. Ctrl+R focuses rename input with blinking cursor
//    8. Font change applies visually to the editor
//    9. Ctrl+mouse wheel zoom works
//   10. Background color shortcut changed to Ctrl+Shift+B
//       (Ctrl+Alt+C conflicts with Windows Copilot global hotkey)
// ============================================================

use egui::{
    Color32, Context, FontId, Key, RichText,
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
    cursor_char_index:  usize,

    // ── status bar (mirrors status_message / title bar) ───────────────────────
    status_msg:         String,
    is_saved:           bool,

    // ── sub-widgets / dialogs ────────────────────────────────────────────────
    line_numbers:       LineNumbers,
    find_dialog:        FindDialog,
    font_dialog:        FontDialog,
    size_dialog:        FontSizeDialog,
    rename_dialog:      RenameDialog,
    color_picker:       ColorPickerDialog,
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
    /// Give keyboard focus to the main editor for N upcoming frames.
    focus_editor_frames_remaining: u8,

    // ── scroll / line-number sync ────────────────────────────────────────────
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
            cursor_char_index:  0,
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
            // FIX #5: Auto-focus the editor on startup (give it enough frames)
            focus_editor_frames_remaining: 10,
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

        // 5. All dialogs - show them AFTER editor so they appear on top
        self.shortcuts_window.show(ctx);
        self.find_dialog.show(ctx);
        self.font_dialog.show(ctx, &self.font_family);
        self.size_dialog.show(ctx);
        self.rename_dialog.show(ctx);
        self.color_picker.show(ctx);

        // 6. Confirm overlays
        self.show_exit_confirm_dialog(ctx);
        self.show_delete_confirm_dialog(ctx);

        // 7. Auto-save (mirrors `<KeyRelease>` binding)
        self.do_auto_save();
        
        // 8. Process dialog results
        self.process_dialogs(ctx);
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
                            RichText::new(format!("Ln: {}, Col: {}", self.cursor_line, self.cursor_col + 1))
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
    fn current_selection_range(
        &self,
        cursor_range: Option<egui::text::CursorRange>,
    ) -> Option<(usize, usize)> {
        cursor_range.and_then(|cr| {
            let a = cr.primary.ccursor.index;
            let b = cr.secondary.ccursor.index;
            if a == b {
                None
            } else {
                Some((a.min(b), a.max(b)))
            }
        })
    }

    fn insert_text_at_cursor(
        &mut self,
        cursor_range: Option<egui::text::CursorRange>,
        text: &str,
    ) {
        if text.is_empty() {
            return;
        }
        let char_len = self.text_content.chars().count();
        let insert_at = cursor_range
            .map(|cr| cr.primary.ccursor.index.min(char_len))
            .unwrap_or(char_len);
        let byte_pos = char_to_byte_offset(&self.text_content, insert_at);
        self.text_content.insert_str(byte_pos, text);
        let new_pos = insert_at + text.chars().count();
        self.pending_selection = Some((new_pos, new_pos));
    }

    fn resolve_editor_font(&self) -> FontId {
        let live_font_family = if self.font_dialog.open {
            self.font_dialog.selected_font.clone()
        } else {
            self.font_family.clone()
        };
        let lower = live_font_family.to_lowercase();
        let is_monospace = lower.contains("mono")
            || lower.contains("consolas")
            || lower.contains("courier")
            || lower.contains("lucida console")
            || lower.contains("liberation");
        if is_monospace {
            FontId::monospace(self.font_size)
        } else {
            FontId::proportional(self.font_size)
        }
    }

    fn show_editor(&mut self, ctx: &Context) {
        // Line height: approximate `font.metrics('linespace')`
        let line_height = self.font_size * 1.5;
        let total_lines = self.text_content.lines().count().max(1);
        let editor_font = self.resolve_editor_font();

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
                    self.line_numbers.show(
                        ui,
                        total_lines,
                        editor_font.clone(),
                        line_height,
                        self.scroll_y_px,
                        available_h,
                    );

                    // ── ② Scrollable text editor ──────────────────────────────
                    let text_id = egui::Id::new("main_text_edit");

                    // Apply a pending selection (find-highlight, navigation)
                    if let Some((sel_start, sel_end)) = self.pending_selection.take() {
                        if let Some(mut state) =
                            egui::TextEdit::load_state(ctx, text_id)
                        {
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
                                .font(editor_font.clone())
                                .text_color(self.fg_color)
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                .cursor_at_end(false);

                            let te_out = te.show(ui);
                            
                            // FIX #5, #6, #7: Auto-focus editor with blinking cursor
                            if self.focus_editor_frames_remaining > 0 {
                                te_out.response.request_focus();
                                self.focus_editor_frames_remaining -= 1;
                            }

                            // ── Cursor tracking ────────────────────────────────
                            if let Some(cr) = te_out.cursor_range {
                                let char_idx = cr.primary.ccursor.index;
                                let (ln, col) =
                                    char_index_to_line_col(&self.text_content, char_idx);
                                self.cursor_line = ln;
                                self.cursor_col  = col;
                                self.cursor_char_index = char_idx;
                            }

                            // Track content changes for is_saved flag
                            if te_out.response.changed() {
                                self.is_saved = false;
                            }

                            // FIX #1: Right-click handler
                            // Right-click with selection → copy to clipboard
                            // Right-click without selection → paste from clipboard
                            // No context menu, exactly like Python's handle_right_click
                            let captured_cursor_range = te_out.cursor_range;
                            if te_out.response.secondary_clicked() {
                                let has_selection = captured_cursor_range.map_or(false, |cr| {
                                    cr.primary.ccursor.index != cr.secondary.ccursor.index
                                });

                                if has_selection {
                                    // Copy selected text
                                    if let Some((start, end)) = self.current_selection_range(captured_cursor_range) {
                                        let selected: String = self.text_content.chars().skip(start).take(end - start).collect();
                                        ctx.copy_text(selected.clone());
                                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                            let _ = clipboard.set_text(selected);
                                        }
                                        self.status_msg = "Text copied to clipboard".to_string();
                                    }
                                } else {
                                    // Paste from clipboard
                                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                        if let Ok(clip) = clipboard.get_text() {
                                            if !clip.is_empty() {
                                                self.insert_text_at_cursor(captured_cursor_range, &clip);
                                                self.status_msg = "Text pasted from clipboard".to_string();
                                                self.is_saved = false;
                                            } else {
                                                self.status_msg = "Clipboard is empty".to_string();
                                            }
                                        } else {
                                            self.status_msg = "Clipboard is empty".to_string();
                                        }
                                    }
                                }
                            }
                        });

                    // Capture scroll offset for next frame
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
        let mut open_find = false;
        let mut open_font = false;
        let mut open_size = false;
        let mut open_rename = false;
        let mut open_color = false;
        let mut open_shortcuts = false;
        let mut new_file = false;
        let mut open_folder = false;
        let mut delete_file = false;
        let mut close_app = false;
        let mut go_top = false;
        let mut go_bottom = false;
        let mut go_line_start = false;
        let mut go_line_end = false;
        let mut zoom_in = false;
        let mut zoom_out = false;
        
        ctx.input_mut(|i| {
            // FIX #6: Ctrl+F → open find dialog and focus entry
            if i.consume_key(egui::Modifiers::CTRL, Key::F) { open_find = true; }
            // FIX #8: Alt+F → open font dialog
            if i.consume_key(egui::Modifiers::ALT, Key::F) { open_font = true; }
            if i.consume_key(egui::Modifiers::ALT, Key::S) { open_size = true; }
            // FIX #7: Ctrl+R → open rename dialog and focus entry
            if i.consume_key(egui::Modifiers::CTRL, Key::R) { open_rename = true; }
            if i.consume_key(egui::Modifiers::CTRL, Key::N) { new_file = true; }
            if i.consume_key(egui::Modifiers::CTRL, Key::O) { open_folder = true; }
            if i.consume_key(egui::Modifiers::ALT, Key::Delete) { delete_file = true; }
            if i.consume_key(egui::Modifiers::ALT, Key::Q) { close_app = true; }
            
            // FIX #2: Ctrl+Up → first line, Ctrl+Down → last line
            if i.consume_key(egui::Modifiers::CTRL, Key::ArrowUp) { go_top = true; }
            if i.consume_key(egui::Modifiers::CTRL, Key::ArrowDown) { go_bottom = true; }
            
            // FIX #3: Ctrl+Left → start of current line, Ctrl+Right → end of current line
            if i.consume_key(egui::Modifiers::CTRL, Key::ArrowLeft) { go_line_start = true; }
            if i.consume_key(egui::Modifiers::CTRL, Key::ArrowRight) { go_line_end = true; }
            
            if i.consume_key(egui::Modifiers::NONE, Key::F1) { open_shortcuts = true; }
            
            // FIX #10: Ctrl+Shift+B for background color
            // (Ctrl+Alt+C conflicts with Windows 10 Copilot global hotkey)
            if i.consume_key(
                egui::Modifiers { ctrl: true, shift: true, ..Default::default() },
                Key::B
            ) { open_color = true; }

            // FIX #9: Ctrl+MouseWheel zoom
            // We must remove the scroll events from the list to prevent them
            // from also scrolling the editor
            let mut consumed_indices = Vec::new();
            for (idx, event) in i.events.iter().enumerate() {
                if let egui::Event::MouseWheel { delta, modifiers, .. } = event {
                    if modifiers.ctrl {
                        if delta.y > 0.0 { zoom_in = true; }
                        else if delta.y < 0.0 { zoom_out = true; }
                        consumed_indices.push(idx);
                    }
                }
            }
            // Remove consumed scroll events in reverse order to preserve indices
            for idx in consumed_indices.into_iter().rev() {
                i.events.remove(idx);
            }
        });
        
        // Apply actions
        if open_find {
            self.find_dialog.open = true;
            self.find_dialog.focus_query_next_frame = true;
            // FIX #6: Stop focusing the editor so the find dialog entry gets focus
            self.focus_editor_frames_remaining = 0;
        }
        if open_font {
            self.font_dialog.open = true;
        }
        if open_size {
            self.size_dialog.size = self.font_size;
            self.size_dialog.open = true;
        }
        if open_rename {
            if let Some(ref p) = self.current_file {
                let name = p.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                self.rename_dialog.new_name = name.clone();
                self.rename_dialog.current_name = name;
                self.rename_dialog.error_msg = String::new();
                self.rename_dialog.open = true;
                // FIX #7: Focus the rename entry with blinking cursor
                self.rename_dialog.focus_name_next_frame = true;
                self.focus_editor_frames_remaining = 0;
            }
        }
        if open_color {
            self.color_picker.open_with(self.bg_color);
        }
        if open_shortcuts {
            self.shortcuts_window.open = true;
        }
        if new_file {
            self.cmd_new_file();
        }
        if open_folder {
            match file_ops::open_save_folder(&self.save_folder) {
                Ok(_) => self.status_msg = "Save folder opened".to_string(),
                Err(e) => self.status_msg = format!("Error: {}", e),
            }
        }
        if delete_file {
            self.show_delete_confirm = true;
        }
        if close_app {
            self.show_exit_confirm = true;
        }
        
        // FIX #2: Ctrl+Up → go to first line (position 0)
        if go_top {
            self.pending_selection = Some((0, 0));
            self.scroll_y_px = 0.0;
            self.status_msg = "Go to first line".to_string();
        }
        
        // FIX #2: Ctrl+Down → go to last line (end of text)
        if go_bottom {
            let len = self.text_content.chars().count();
            self.pending_selection = Some((len, len));
            self.status_msg = "Go to last line".to_string();
        }
        
        // FIX #3: Ctrl+Left → go to start of current line
        // Mirrors Python: go_to_line_start
        //   current_pos = self.text.index(tk.INSERT)
        //   line_start = current_pos.split('.')[0] + ".0"
        //   self.text.mark_set(tk.INSERT, line_start)
        if go_line_start {
            let (line, _col) = char_index_to_line_col(&self.text_content, self.cursor_char_index);
            let target = line_start_char_index(&self.text_content, line);
            self.pending_selection = Some((target, target));
        }
        
        // FIX #3: Ctrl+Right → go to end of current line
        // Mirrors Python: go_to_line_end
        //   current_pos = self.text.index(tk.INSERT)
        //   line_end = current_pos.split('.')[0] + ".end"
        //   self.text.mark_set(tk.INSERT, line_end)
        if go_line_end {
            let (line, _col) = char_index_to_line_col(&self.text_content, self.cursor_char_index);
            let target = line_end_char_index(&self.text_content, line);
            self.pending_selection = Some((target, target));
        }
        
        // FIX #9: Zoom in/out with Ctrl+MouseWheel
        if zoom_in {
            self.font_size = (self.font_size + 1.0).min(72.0);
            settings::save_font_settings(&self.save_folder, &self.font_family, self.font_size);
            self.status_msg = format!("Font size: {}", self.font_size as i32);
        }
        if zoom_out {
            self.font_size = (self.font_size - 1.0).max(6.0);
            settings::save_font_settings(&self.save_folder, &self.font_family, self.font_size);
            self.status_msg = format!("Font size: {}", self.font_size as i32);
        }
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
                // Focus editor after creating new file
                self.focus_editor_frames_remaining = 5;
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
    
    /// Process all dialog results
    fn process_dialogs(&mut self, _ctx: &Context) {
        // Process find dialog
        // FIX #4: Find text shows "Line N, Column C/Total"
        if self.find_dialog.find_requested {
            let query = self.find_dialog.query.clone();
            self.find_dialog.find_requested = false;
            if query.is_empty() {
                self.find_dialog.result_msg = "Please enter search text".to_string();
            } else {
                let content = self.text_content.clone();
                let char_len = content.chars().count();
                let start_ci = self.find_char_offset.min(char_len);
                let start_byte = char_to_byte_offset(&content, start_ci);

                let search = |hay: &str| -> Option<usize> {
                    if self.find_dialog.match_case {
                        hay.find(query.as_str())
                    } else {
                        hay.to_lowercase().find(query.to_lowercase().as_str())
                    }
                };

                let found_byte = search(&content[start_byte..])
                    .map(|b| b + start_byte)
                    .or_else(|| if start_byte > 0 { search(&content[..start_byte]) } else { None });

                match found_byte {
                    Some(byte_start) => {
                        let byte_end   = (byte_start + query.len()).min(content.len());
                        let char_start = byte_to_char_index(&content, byte_start);
                        let char_end   = byte_to_char_index(&content, byte_end);
                        let (ln, col)  = char_index_to_line_col(&content, char_start);
                        let total_col  = line_total_cols(&content, ln);
                        let col_1based = col + 1;

                        // FIX #4: Show "Line N, Column C/Total" instead of character position
                        self.find_dialog.result_msg = format!(
                            "Found at Line {}, Column {}/{}",
                            ln, col_1based, total_col
                        );
                        self.status_msg = format!(
                            "Found: \"{}\" at Line {}, Col {}",
                            query, ln, col_1based
                        );
                        self.find_char_offset = char_end;
                        self.pending_selection = Some((char_start, char_end));
                        // Focus the editor to show the selection
                        self.focus_editor_frames_remaining = 3;
                    }
                    None => {
                        self.find_dialog.result_msg = format!("\"{}\" not found", query);
                        self.status_msg = format!("\"{}\" not found", query);
                        self.find_char_offset = 0;
                    }
                }
            }
        }
        
        // FIX #8: Process font dialog — apply the font change visually
        if let Some(new_font) = self.font_dialog.apply.take() {
            self.font_family = new_font.clone();
            settings::save_font_settings(&self.save_folder, &new_font, self.font_size);
            self.status_msg = format!("Font changed to: {}", new_font);
            // Give focus back to editor after font change
            self.focus_editor_frames_remaining = 3;
        }
        
        // Process size dialog
        if let Some(new_size) = self.size_dialog.apply.take() {
            self.font_size = new_size;
            settings::save_font_settings(&self.save_folder, &self.font_family, new_size);
            self.status_msg = format!("Font size: {}", new_size as i32);
            self.focus_editor_frames_remaining = 3;
        }
        
        // Process rename dialog
        if let Some(new_name) = self.rename_dialog.apply.take() {
            if let Some(ref current) = self.current_file.clone() {
                match file_ops::rename_file(current, &self.save_folder, &new_name) {
                    Ok(new_path) => {
                        self.current_file = Some(new_path);
                        self.rename_dialog.open = false;
                        self.status_msg = format!("Renamed to: {}", new_name);
                        // Focus editor after rename
                        self.focus_editor_frames_remaining = 3;
                    }
                    Err(e) => {
                        self.rename_dialog.error_msg = e;
                    }
                }
            }
        }
        
        // Process color picker
        if let Some(new_bg) = self.color_picker.applied.take() {
            self.bg_color = new_bg;
            self.fg_color = settings::fg_for_bg(new_bg);
            settings::save_color_settings(&self.save_folder, new_bg);
            self.status_msg = "Background color changed".to_string();
        }
    }
}

impl IrakNotesApp {
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

// ── text-position utility functions ───────────────────────────────────────────

/// Convert a **char index** into a (line, col) pair.
/// `line` is 1-based, `col` is 0-based (to match tkinter's column convention).
///
/// Mirrors Python:
///   position = self.text.index(tk.INSERT)   # "line.column"
///   line, column = position.split('.')
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

/// Get the char index of the start of a 1-based line.
fn line_start_char_index(text: &str, line_1_based: usize) -> usize {
    if line_1_based <= 1 {
        return 0;
    }
    let mut current_line = 1usize;
    for (i, ch) in text.chars().enumerate() {
        if ch == '\n' {
            current_line += 1;
            if current_line == line_1_based {
                return i + 1;
            }
        }
    }
    // If the requested line doesn't exist, return end of text
    text.chars().count()
}

/// Get the char index of the end of a 1-based line (before the \n).
fn line_end_char_index(text: &str, line_1_based: usize) -> usize {
    let mut current_line = 1usize;
    for (i, ch) in text.chars().enumerate() {
        if ch == '\n' {
            if current_line == line_1_based {
                return i;
            }
            current_line += 1;
        }
    }
    // Last line (no trailing \n)
    text.chars().count()
}

/// Get the total number of columns (characters) in a 1-based line.
fn line_total_cols(text: &str, line_1_based: usize) -> usize {
    let start = line_start_char_index(text, line_1_based);
    let end = line_end_char_index(text, line_1_based);
    end.saturating_sub(start)
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
