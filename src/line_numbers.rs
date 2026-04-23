// ============================================================
//  line_numbers.rs  (Part 2 – scroll-sync rewrite)
//
//  Mirrors the Python `LineNumbers(tk.Canvas)` class.
//
//  Part 1 estimated first/last visible line from stored state.
//  Part 2 receives the real `scroll_y_px` (pixel offset) from
//  the `ScrollArea`'s own state output, giving exact sync.
//
//  Key Python methods mirrored:
//    • __init__              → LineNumbers::new()
//    • attach()              → collapsed into show() (immediate mode)
//    • update_line_numbers() → show()  called every frame
//
//  Python canvas attributes reproduced:
//    • bg='#f0f0f0'          → self.bg_color
//    • width=30 (min)        → MIN_GUTTER_W
//    • fill="#606060"        → self.fg_color
//    • anchor="ne" at width-5 → right-aligned digit painting
//    • y_pos = 2             → TOP_OFFSET
// ============================================================

use egui::{Color32, FontId, Pos2, Rounding, Stroke, Ui, Vec2};

// ── layout constants ──────────────────────────────────────────────────────────

/// Minimum gutter width in logical pixels (mirrors `width=30`).
const MIN_GUTTER_W: f32 = 30.0;
/// Right-side padding inside the gutter (mirrors `width - 5` offset).
const RIGHT_PAD: f32 = 6.0;
/// Small top offset so the first digit aligns with the first text row.
/// Mirrors `y_pos = 2` in the Python loop.
const TOP_OFFSET: f32 = 2.0;

// ── LineNumbers ───────────────────────────────────────────────────────────────

/// Persistent state.  Rendering happens entirely in `show()`.
pub struct LineNumbers {
    /// Gutter background colour  (mirrors `bg='#f0f0f0'`).
    pub bg_color:  Color32,
    /// Digit text colour          (mirrors `fill="#606060"`).
    pub fg_color:  Color32,
    /// Separator line colour.
    pub sep_color: Color32,
}

impl Default for LineNumbers {
    fn default() -> Self {
        Self {
            bg_color:  Color32::from_rgb(0xf0, 0xf0, 0xf0),
            fg_color:  Color32::from_rgb(0x60, 0x60, 0x60),
            sep_color: Color32::from_rgb(0xcc, 0xcc, 0xcc),
        }
    }
}

impl LineNumbers {
    /// Constructor — mirrors `LineNumbers.__init__`.
    pub fn new() -> Self {
        Self::default()
    }

    // ── show ─────────────────────────────────────────────────────────────────

    /// Render the gutter.  Must be called *before* the `ScrollArea` each frame
    /// so it occupies the left side of the horizontal strip.
    ///
    /// Parameters:
    ///
    /// | Rust param       | Python equivalent                              |
    /// |------------------|------------------------------------------------|
    /// | `ui`             | the tkinter Canvas widget's parent             |
    /// | `total_lines`    | `int(text.index('end-1c').split('.')[0])`      |
    /// | `font_id`        | `self.font` (matched to editor font)           |
    /// | `line_height_px` | `self.font.metrics('linespace')`               |
    /// | `scroll_y_px`    | real scroll offset from `ScrollArea` state     |
    /// | `visible_height` | gutter visible height in pixels                |
    ///
    /// Returns the gutter width allocated so the caller can offset the editor.
    pub fn show(
        &self,
        ui:             &mut Ui,
        total_lines:    usize,
        font_id:        FontId,
        line_height_px: f32,
        scroll_y_px:    f32,
        visible_height: f32,
    ) -> f32 {
        // ── compute gutter width from digit count ─────────────────────────────
        // Mirrors: `digits = len(str(total_lines))`
        //          `width  = digits * font.measure('0') + 10`
        let digits        = digit_count(total_lines.max(1));
        let char_w_approx = font_id.size * 0.62; // approximate monospace char width
        let gutter_w      = (digits as f32 * char_w_approx + RIGHT_PAD + 8.0)
                                .max(MIN_GUTTER_W);

        let (gutter_rect, _) = ui.allocate_exact_size(
            Vec2::new(gutter_w, visible_height),
            egui::Sense::hover(),
        );

        let painter = ui.painter();

        // ── background (mirrors `bg='#f0f0f0'`) ──────────────────────────────
        painter.rect_filled(gutter_rect, Rounding::ZERO, self.bg_color);

        // ── right-edge separator line ─────────────────────────────────────────
        painter.line_segment(
            [
                Pos2::new(gutter_rect.right(), gutter_rect.top()),
                Pos2::new(gutter_rect.right(), gutter_rect.bottom()),
            ],
            Stroke::new(1.0, self.sep_color),
        );

        if line_height_px <= 0.0 || visible_height <= 0.0 || total_lines == 0 {
            return gutter_w;
        }

        // ── draw line numbers for actual text lines only ──────────────────────
        //
        // Unlike the previous version that calculated visible lines from scroll position,
        // we now simply draw all line numbers at their logical positions.
        // This ensures each actual line (separated by \n) gets exactly one line number,
        // regardless of text wrapping.
        //
        // Python equivalent:
        //   for line in range(1, total_lines + 1):
        //       dline = self.text_widget.dlineinfo(f"{line}.0")
        //       if dline:
        //           y = dline[1]
        //           self.create_text(width-5, y, anchor="ne", text=str(line), ...)
        
        for line_num in 1..=total_lines {
            // Calculate Y position for this logical line
            // Each logical line starts at (line_num - 1) * line_height_px
            let logical_y = (line_num - 1) as f32 * line_height_px;
            
            // Adjust for scroll offset
            let y = gutter_rect.top() + TOP_OFFSET + logical_y - scroll_y_px;

            // Skip if outside visible area (with some margin for smooth scrolling)
            if y > gutter_rect.bottom() + line_height_px {
                continue;
            }
            if y + line_height_px < gutter_rect.top() - line_height_px {
                continue;
            }

            let label  = format!("{}", line_num);
            let galley = ui.fonts(|fonts| {
                fonts.layout_no_wrap(label, font_id.clone(), self.fg_color)
            });
            // Right-align (mirrors `anchor="ne"`)
            let x = gutter_rect.right() - RIGHT_PAD - galley.rect.width();
            painter.galley(Pos2::new(x, y), galley, self.fg_color);
        }

        gutter_w
    }
}

// ── helper ────────────────────────────────────────────────────────────────────

/// Count decimal digits in `n`  (mirrors `len(str(total_lines))`).
fn digit_count(n: usize) -> usize {
    if n == 0 { return 1; }
    let mut c = 0usize;
    let mut v = n;
    while v > 0 { c += 1; v /= 10; }
    c
}
