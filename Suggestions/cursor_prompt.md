# Cursor AI Prompt — Irak Notes Rust (egui) Bug Fixes

## Project Location
`C:\Users\Irak\Desktop\NoteApp\Irak_Note_5.1\irak_notes_rust`

## Overview
Fix the following 10 bugs in the Rust/egui desktop note-taking app. Work only with the Rust source files. Do NOT reference or follow the Python file at `C:\Users\Irak\Desktop\NoteApp\Irak_Note_5.1\Irak_Note.py`. All changes go inside the `irak_notes_rust` folder.

---

## Bug 1 — Find Text shows raw byte position instead of "Line N, Col N/N"

**File:** `src/app.rs`

**Problem:** The result message shows something like "Found at position 854" instead of "Found at Line 15, Column 45/500". Two find-processing methods exist (`process_find_dialog` and the find block inside `process_dialogs`). They conflict. The `process_find_dialog` method is never called from `update()` but still exists, causing confusion. The active path inside `process_dialogs` uses `search_in.find()` which returns a raw byte offset and the conversion to char index may be miscalculated. Also `col` from `char_index_to_line_col` is 0-based, so column display starts at 0 instead of 1.

**Fix:**

1. Delete the entire `process_find_dialog` method from the `impl IrakNotesApp` block (it is dead code — never called from `update()`).

2. Replace the find block inside `process_dialogs` with this correct implementation that uses wrap-around, proper char-index conversion, and 1-based column display:

```rust
// Inside process_dialogs, replace the find block:
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

        // Search forward from cursor, then wrap from beginning
        let found_byte = search(&content[start_byte..])
            .map(|b| b + start_byte)
            .or_else(|| {
                if start_byte > 0 { search(&content[..start_byte]) } else { None }
            });

        match found_byte {
            Some(byte_start) => {
                let byte_end   = (byte_start + query.len()).min(content.len());
                let char_start = byte_to_char_index(&content, byte_start);
                let char_end   = byte_to_char_index(&content, byte_end);
                let (ln, col)  = char_index_to_line_col(&content, char_start);
                let total_col  = line_total_cols(&content, ln);
                let col_1based = col + 1; // convert 0-based to 1-based

                self.find_dialog.result_msg = format!(
                    "Found at Line {}, Column {}/{}",
                    ln, col_1based, total_col
                );
                self.status_msg = format!(
                    "Found: \"{}\" at Line {}, Col {}",
                    query, ln, col_1based
                );
                self.find_char_offset  = char_end;
                self.pending_selection = Some((char_start, char_end));
            }
            None => {
                self.find_dialog.result_msg = format!("\"{}\" not found", query);
                self.status_msg = format!("\"{}\" not found", query);
                self.find_char_offset = 0;
            }
        }
    }
}
```

3. Also update `show_status_bar` to display cursor position as 1-based column:
```rust
// In show_status_bar, change the format string:
format!("Ln: {}, Col: {}", self.cursor_line, self.cursor_col + 1)
```

---

## Bug 2 — App crashes on Ctrl+Right or Ctrl+Left

**File:** `src/app.rs`

**Root cause:** Two problems combined:
- `ctx.input(|i| ...)` reads events WITHOUT consuming them. So egui's `TextEdit` ALSO sees `Ctrl+Left` / `Ctrl+Right` and performs its own word-navigation at the same time as the app tries to set `pending_selection`. Both fire in the same frame, corrupting TextEdit cursor state → crash/panic.
- `line_last_non_whitespace_char_index` and `line_first_non_whitespace_char_index` can create invalid byte-slice ranges on certain edge cases (empty lines, cursor at very end).

**Fix — Part A:** In `handle_keyboard`, change `ctx.input(|i| ...)` to `ctx.input_mut(|i| ...)` and consume the four arrow keys with Ctrl so the TextEdit never sees them:

```rust
// Replace the entire ctx.input block in handle_keyboard with:
ctx.input_mut(|i| {
    if i.consume_key(egui::Modifiers::CTRL, Key::F) { open_find = true; }
    if i.consume_key(egui::Modifiers::ALT,  Key::F) { open_font = true; }
    if i.consume_key(egui::Modifiers::ALT,  Key::S) { open_size = true; }
    if i.consume_key(egui::Modifiers::CTRL, Key::R) { open_rename = true; }
    if i.consume_key(egui::Modifiers::CTRL, Key::N) { new_file = true; }
    if i.consume_key(egui::Modifiers::CTRL, Key::O) { open_folder = true; }
    if i.consume_key(egui::Modifiers::ALT,  Key::Delete) { delete_file = true; }
    if i.consume_key(egui::Modifiers::ALT,  Key::Q) { close_app = true; }
    if i.consume_key(egui::Modifiers::CTRL, Key::ArrowUp)    { go_top = true; }
    if i.consume_key(egui::Modifiers::CTRL, Key::ArrowDown)  { go_bottom = true; }
    if i.consume_key(egui::Modifiers::CTRL, Key::ArrowLeft)  { go_line_start = true; }
    if i.consume_key(egui::Modifiers::CTRL, Key::ArrowRight) { go_line_end = true; }
    if i.consume_key(egui::Modifiers::NONE, Key::F1) { open_shortcuts = true; }
    // Color picker: use Ctrl+Shift+B instead of Ctrl+Alt+C (see Bug 10)
    if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, Key::B) { open_color = true; }

    // Mouse wheel zoom with Ctrl
    if i.modifiers.ctrl {
        let dy = i.smooth_scroll_delta.y;
        if dy > 0.0 { zoom_in = true; }
        else if dy < 0.0 { zoom_out = true; }
    }
});
```

**Fix — Part B:** Make `line_last_non_whitespace_char_index` and `line_first_non_whitespace_char_index` panic-safe:

```rust
fn line_first_non_whitespace_char_index(text: &str, line_1_based: usize) -> usize {
    let start = line_start_char_index(text, line_1_based);
    let end   = line_end_char_index(text, line_1_based);
    if end <= start { return start; }

    let mut offset = 0usize;
    for ch in text.chars().skip(start).take(end - start) {
        if !ch.is_whitespace() { return start + offset; }
        offset += 1;
    }
    start
}

fn line_last_non_whitespace_char_index(text: &str, line_1_based: usize) -> usize {
    let start = line_start_char_index(text, line_1_based);
    let end   = line_end_char_index(text, line_1_based);
    if end <= start { return end; }

    let chars: Vec<char> = text.chars().skip(start).take(end - start).collect();
    for idx in (0..chars.len()).rev() {
        if !chars[idx].is_whitespace() { return start + idx + 1; }
    }
    end
}
```

This avoids any `char_to_byte_offset` + string-slice combination entirely.

---

## Bug 3 & 4 — Right-click copy (selected) and paste (no selection) not working

**File:** `src/app.rs`

**Problem:** `te_out.response.secondary_clicked()` and `te_out.response.context_menu(...)` both try to handle right-click. In egui, `context_menu` absorbs the right-click event so `secondary_clicked()` always returns false. The two handlers fight each other and neither works reliably.

**Fix:** Remove the `secondary_clicked()` block entirely. Keep only the `context_menu` block, but fix it to read the cursor range correctly (it must be captured before the closure):

```rust
// Remove this entire block (delete it):
// if te_out.response.secondary_clicked() { ... }

// Replace the context_menu block with:
let captured_cursor_range = te_out.cursor_range;
te_out.response.context_menu(|ui| {
    let has_selection = captured_cursor_range.map_or(false, |cr| {
        cr.primary.ccursor.index != cr.secondary.ccursor.index
    });

    if has_selection {
        if ui.button("📋  Copy").clicked() {
            if let Some(cr) = captured_cursor_range {
                let a = cr.primary.ccursor.index;
                let b = cr.secondary.ccursor.index;
                let (s, e) = (a.min(b), a.max(b));
                let selected: String = self.text_content.chars().skip(s).take(e - s).collect();
                if !selected.is_empty() {
                    ctx.copy_text(selected.clone());
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(selected);
                    }
                    self.status_msg = "Text copied".to_string();
                }
            }
            ui.close_menu();
        }
    }

    if ui.button("📋  Paste").clicked() {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Ok(clip) = cb.get_text() {
                self.insert_text_at_cursor(captured_cursor_range, &clip);
                self.status_msg = "Text pasted".to_string();
                self.is_saved = false;
            }
        }
        ui.close_menu();
    }

    if ui.button("Select All").clicked() {
        let total = self.text_content.chars().count();
        self.pending_selection = Some((0, total));
        ui.close_menu();
    }
});
```

---

## Bug 5 — Ctrl+Up / Ctrl+Down not moving cursor (just shaking UI)

**File:** `src/app.rs`

**Problem:** After Bug 2 fix (using `consume_key`), these should work. But additionally, `pending_selection = Some((0,0))` for Ctrl+Up is correct. The "shaking" happens because the TextEdit ALSO handles Ctrl+Up/Down before the key is consumed. After applying Bug 2 fix this should be resolved.

**Additional fix:** After setting `pending_selection` for go_top/go_bottom, also request a scroll to show the cursor. Add this to the go_top and go_bottom handlers:

```rust
if go_top {
    self.pending_selection = Some((0, 0));
    // Scroll to top by resetting scroll offset next frame
    self.scroll_y_px = 0.0;
}
if go_bottom {
    let len = self.text_content.chars().count();
    self.pending_selection = Some((len, len));
    // Scroll to bottom will happen automatically as TextEdit follows cursor
}
```

---

## Bug 6 — Ctrl+Left / Ctrl+Right not moving to line start/end

This is the same root cause as Bug 2 (not consuming the keys). After applying Bug 2 fix with `consume_key`, the line-start and line-end navigation will work. The logic in `go_line_start` and `go_line_end` (smart home: first non-whitespace, then absolute start) is correct.

No additional change needed beyond Bug 2 fix.

---

## Bug 7 — Ctrl+F: cursor should immediately blink in search field

**File:** `src/app.rs`

**Problem:** When Ctrl+F opens the find dialog, `focus_editor_next_frame` might get set to true elsewhere and steal focus back from the find dialog's text field.

**Fix:** When opening find dialog, explicitly prevent editor from stealing focus:

```rust
if open_find {
    self.find_dialog.open = true;
    self.find_dialog.focus_query_next_frame = true;
    self.focus_editor_next_frame = false; // ← ADD THIS LINE
}
```

Also in `dialogs.rs` `FindDialog::show`, make the focus request happen on the frame AFTER the dialog opens (not the same frame), because the window might not exist yet on the first frame:

```rust
// In FindDialog::show, change focus logic:
let resp = ui.add_sized(
    [270.0, 22.0],
    TextEdit::singleline(&mut self.query).font(egui::TextStyle::Body),
);
if self.focus_query_next_frame {
    resp.request_focus();
    self.focus_query_next_frame = false;
}
// Also trigger find on Enter key press (not just lost_focus):
if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
    self.find_requested = true;
}
```

---

## Bug 8 — App startup: cursor should immediately blink in editor

**File:** `src/app.rs`

**Problem:** `focus_editor_next_frame: true` is set in the constructor, but `request_focus()` is called INSIDE a ScrollArea closure which may not execute on frame 0. The focus request might be applied before the widget exists.

**Fix:** Keep `focus_editor_next_frame: true` in the constructor. In `show_editor`, move the focus request to trigger for TWO frames (frame 0 and frame 1) to guarantee it lands:

```rust
// Change the focus_editor_next_frame field type to u8 (frame countdown):
// In struct: focus_editor_frames_remaining: u8
// In constructor: focus_editor_frames_remaining: 3  (3 frames = guaranteed)

// In show_editor, replace:
// if self.focus_editor_next_frame {
//     te_out.response.request_focus();
//     self.focus_editor_next_frame = false;
// }
// With:
if self.focus_editor_frames_remaining > 0 {
    te_out.response.request_focus();
    self.focus_editor_frames_remaining -= 1;
}
```

Update all places that set `focus_editor_next_frame = true` to set `focus_editor_frames_remaining = 2` instead. Update all places that set it to `false` to set it to `0`.

---

## Bug 9 — Ctrl+R: cursor should immediately blink in rename field

**File:** `src/app.rs` and `src/dialogs.rs`

Same root cause as Bug 7. Apply same fix pattern:

```rust
// In handle_keyboard, open_rename block:
if open_rename {
    // ... existing name setup code ...
    self.rename_dialog.open = true;
    self.rename_dialog.focus_name_next_frame = true;
    self.focus_editor_frames_remaining = 0; // prevent editor stealing focus
}
```

In `dialogs.rs` `RenameDialog::show`, also trigger rename on Enter:
```rust
// Change the focus + Enter handling:
let resp = ui.add_sized([280.0, 22.0], TextEdit::singleline(&mut self.new_name));
if self.focus_name_next_frame {
    resp.request_focus();
    self.focus_name_next_frame = false;
}
if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
    if !self.new_name.trim().is_empty() {
        self.apply = Some(self.new_name.clone());
    }
}
```

---

## Bug 10 — Font change: should show live preview in background

**File:** `src/app.rs`

**Problem:** The font only changes AFTER the user clicks Apply and the dialog closes. The user wants to see the font changing live in the background editor while browsing the list.

**Fix:** In `show_editor`, use `selected_font` from the dialog as the live preview font while the dialog is open:

```rust
// In show_editor, replace:
// let editor_font = self.editor_font_id();
// With:
let live_font_family = if self.font_dialog.open {
    self.font_dialog.selected_font.clone()
} else {
    self.font_family.clone()
};

// Then replace editor_font_id() call with inline:
let lower = live_font_family.to_lowercase();
let is_mono = lower.contains("mono") || lower.contains("consolas")
    || lower.contains("courier") || lower.contains("lucida console")
    || lower.contains("liberation");
let editor_font = if is_mono {
    FontId::monospace(self.font_size)
} else {
    FontId::proportional(self.font_size)
};
```

Also in `FontDialog::show` in `dialogs.rs`, update the preview swatch to use the actually selected font:

```rust
// Replace the preview swatch label with:
ui.label(
    RichText::new("AaBbCcDdEe 123456789")
        .font(egui::FontId::proportional(16.0))
        // Note: egui only supports Monospace/Proportional built-in fonts.
        // The preview shows size correctly; family is visual approximation.
);
```

---

## Bug 11 — Ctrl+Scroll: zoom not working

**File:** `src/app.rs`

**Problem:** `i.raw_scroll_delta.y` may be 0.0 because egui routes scroll events to the hovered ScrollArea widget first. By the time `handle_keyboard` reads them, they are consumed. Also `raw_scroll_delta` vs `smooth_scroll_delta` differs by OS.

**Fix:** Use `i.events` to detect scroll events directly, checking for the Ctrl modifier:

```rust
// Replace the zoom detection in handle_keyboard ctx.input_mut block:
for event in &i.events {
    if let egui::Event::Scroll(scroll_vec) = event {
        if i.modifiers.ctrl {
            if scroll_vec.y > 0.0 { zoom_in = true; }
            else if scroll_vec.y < 0.0 { zoom_out = true; }
        }
    }
    // Also handle MouseWheel events as fallback:
    if let egui::Event::MouseWheel { delta, modifiers, .. } = event {
        if modifiers.ctrl {
            let dy = match delta {
                egui::MouseWheelUnit::Line(v) => v.y,
                egui::MouseWheelUnit::Point(v) => v.y,
                egui::MouseWheelUnit::Page(v) => v.y,
            };
            if dy > 0.0 { zoom_in = true; }
            else if dy < 0.0 { zoom_out = true; }
        }
    }
}
```

Note: egui's `Event` enum variants change between versions. Check the actual egui version in `Cargo.toml` and use the correct variant names. For egui 0.27+, use `Event::MouseWheel`. For older versions, use `Event::Scroll`. Use whichever compiles.

---

## Bug 12 — Ctrl+Alt+C opens Windows Copilot instead of color picker

**File:** `src/app.rs` and `src/shortcuts.rs`

**Problem:** `Ctrl+Alt+C` is a Windows 10/11 global hotkey registered by Windows Copilot. It is intercepted by Windows before the app ever sees it. This cannot be overridden without low-level OS hook code — not worth doing.

**Fix:** Change the color picker shortcut to `Ctrl+Shift+B` (B for Background). This does not conflict with any Windows system hotkey.

In `src/app.rs` — change the key binding (already done in Bug 2 fix above via `consume_key`):
```rust
// Already handled in Bug 2 fix — using Ctrl+Shift+B
if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, Key::B) { open_color = true; }
```

In `src/shortcuts.rs` — update the shortcuts table:
```rust
// In shortcut_data(), change:
// ("Ctrl+Alt+C", "Change background color"),
// To:
("Ctrl+Shift+B", "Change background color"),
```

---

## Summary of all files to modify

| File | Bugs Fixed |
|------|-----------|
| `src/app.rs` | 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 |
| `src/dialogs.rs` | 7, 9, 10 |
| `src/shortcuts.rs` | 12 |

## Important notes for Cursor AI

- Do NOT modify `src/file_ops.rs`, `src/line_numbers.rs`, `src/settings.rs`, `src/color_picker.rs`, or `src/main.rs` — they are correct and do not need changes.
- After all edits, run `cargo build` to verify zero compile errors before finishing.
- If `egui::Modifiers::CTRL | egui::Modifiers::SHIFT` syntax does not compile for the installed egui version, use `egui::Modifiers { ctrl: true, shift: true, ..Default::default() }` instead.
- If `Event::MouseWheel` does not exist in the installed egui version, use only the `Event::Scroll` approach or check the egui changelog for the correct variant name.
- The `byte_to_char_index` and `char_to_byte_offset` private functions already exist at the bottom of `app.rs` — do NOT add duplicate definitions.
- `line_total_cols` already exists in `app.rs` — do NOT add a duplicate.
- When replacing `focus_editor_next_frame: bool` with `focus_editor_frames_remaining: u8`, update ALL references in the file (constructor, `show_editor`, `handle_keyboard` open_find block, open_rename block).
