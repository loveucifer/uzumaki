use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color as VelloColor, Fill};
use vello::Scene;

use crate::style::{Bounds, Color, Corners, Edges, Style};
use crate::text::{GlyphPos2D, TextRenderer};

/// Snapshot of input state collected during the render-tree walk.
/// Decouples painting from the live InputState/Node.
pub struct InputRenderInfo {
    pub display_text: String,
    pub placeholder: String,
    pub font_size: f32,
    pub text_color: Color,
    pub focused: bool,
    pub sel_start: usize,
    pub sel_end: usize,
    pub cursor_pos: usize,
    pub scroll_offset: f32,
    pub scroll_offset_y: f32,
    pub blink_visible: bool,
    pub multiline: bool,
}

/// Paint an input element with its text, selection highlight, and cursor.
pub fn paint_input(
    scene: &mut Scene,
    text_renderer: &mut TextRenderer,
    bounds: Bounds,
    style: &Style,
    input: &InputRenderInfo,
    scale: f64,
) {
    let padding: f64 = 8.0;
    let text_x = bounds.x + padding;
    let text_y = bounds.y;
    let text_w = (bounds.width - padding * 2.0).max(0.0);
    let text_h = bounds.height;

    // Paint background with focus-aware border
    let mut paint_style = style.clone();
    if input.focused {
        paint_style.border_widths = Edges::all(2.0);
        paint_style.border_color = Some(Color::rgba(86, 156, 214, 255));
    } else {
        if !paint_style.border_widths.any_nonzero() {
            paint_style.border_widths = Edges::all(1.0);
        }
        if paint_style.border_color.is_none() {
            paint_style.border_color = Some(Color::rgba(60, 60, 60, 255));
        }
    }
    if paint_style.background.is_none() {
        paint_style.background = Some(Color::rgba(30, 30, 30, 255));
    }
    if !paint_style.corner_radii.any_nonzero() {
        paint_style.corner_radii = Corners::uniform(4.0);
    }

    paint_style.paint(bounds, scene, scale, |_| {});

    // Clip to text area
    let clip_rect = Rect::new(text_x, text_y, text_x + text_w, text_y + text_h);
    scene.push_clip_layer(Fill::NonZero, Affine::scale(scale), &clip_rect);

    let is_empty = input.display_text.is_empty();
    let line_height = (input.font_size * 1.2).round();

    if input.multiline {
        paint_multiline(scene, text_renderer, input, text_x, text_y, text_w, text_h, line_height, is_empty, style, scale);
    } else {
        paint_singleline(scene, text_renderer, input, text_x, text_y, text_w, text_h, line_height, is_empty, scale);
    }

    scene.pop_layer();
}

// ── Multiline input rendering ────────────────────────────────────────

fn paint_multiline(
    scene: &mut Scene,
    text_renderer: &mut TextRenderer,
    input: &InputRenderInfo,
    text_x: f64,
    text_y: f64,
    text_w: f64,
    text_h: f64,
    line_height: f32,
    is_empty: bool,
    style: &Style,
    scale: f64,
) {
    let top_pad: f32 = if style.padding.top > 0.0 {
        style.padding.top
    } else {
        4.0
    };
    let wrap_width = Some(text_w as f32);

    let positions = if !is_empty {
        text_renderer.grapheme_positions_2d(&input.display_text, input.font_size, wrap_width)
    } else {
        vec![GlyphPos2D { x: 0.0, y: 0.0 }]
    };

    let scroll_y = input.scroll_offset_y;

    if is_empty && !input.placeholder.is_empty() {
        text_renderer.draw_text(
            scene,
            &input.placeholder,
            cosmic_text::Attrs::new(),
            input.font_size,
            text_w as f32,
            text_h as f32,
            (text_x as f32, text_y as f32 + top_pad - scroll_y),
            VelloColor::from_rgba8(128, 128, 128, 255),
            scale,
        );
    } else if !is_empty {
        // Draw selection highlight
        if input.focused
            && input.sel_start != input.sel_end
            && input.sel_start < positions.len()
            && input.sel_end <= positions.len()
        {
            paint_multiline_selection(
                scene, &positions, input, text_x, text_y, text_w,
                line_height, top_pad, scroll_y, scale,
            );
        }

        // Draw text with wrapping
        text_renderer.draw_text(
            scene,
            &input.display_text,
            cosmic_text::Attrs::new(),
            input.font_size,
            text_w as f32,
            text_h as f32 + scroll_y + 10000.0,
            (text_x as f32, text_y as f32 + top_pad - scroll_y),
            input.text_color.to_vello(),
            scale,
        );
    }

    // Draw cursor
    if input.focused && input.blink_visible && !positions.is_empty() {
        let cp = if input.cursor_pos < positions.len() {
            positions[input.cursor_pos]
        } else {
            *positions.last().unwrap()
        };
        let cursor_x = cp.x as f64;
        let cursor_y = cp.y as f64 + top_pad as f64 - scroll_y as f64;
        paint_cursor(scene, text_x + cursor_x, text_y + cursor_y, line_height, scale);
    }
}

fn paint_multiline_selection(
    scene: &mut Scene,
    positions: &[GlyphPos2D],
    input: &InputRenderInfo,
    text_x: f64,
    text_y: f64,
    text_w: f64,
    line_height: f32,
    top_pad: f32,
    scroll_y: f32,
    scale: f64,
) {
    let sel_color = VelloColor::from_rgba8(56, 121, 185, 128);
    let sel_end_clamped = input.sel_end.min(positions.len() - 1);
    let start_pos = positions[input.sel_start];
    let end_pos = positions[sel_end_clamped];

    // Collect unique visual line y-values within the selection range
    let mut line_ys: Vec<f32> = Vec::new();
    for i in input.sel_start..=sel_end_clamped {
        let y = positions[i].y;
        if line_ys.last().map_or(true, |&ly| (y - ly).abs() > 1.0) {
            line_ys.push(y);
        }
    }

    let line_has_content = |ly: f32| -> bool {
        (input.sel_start..=sel_end_clamped)
            .any(|i| (positions[i].y - ly).abs() < 1.0 && positions[i].x > 0.5)
    };

    // Find the max x on a given line across ALL positions (not just selected range),
    // so the selection extends to the true end of line content.
    let line_max_x = |ly: f32| -> f32 {
        let mut max_x: f32 = 0.0;
        for pos in positions {
            if (pos.y - ly).abs() < 1.0 {
                max_x = max_x.max(pos.x);
            }
        }
        max_x
    };

    let num_lines = line_ys.len();
    for (idx, &ly) in line_ys.iter().enumerate() {
        let sy = ly as f64 + top_pad as f64 - scroll_y as f64;
        let has_content = line_has_content(ly);

        let (rx1, rx2) = if num_lines == 1 {
            (text_x + start_pos.x as f64, text_x + end_pos.x as f64)
        } else if idx == 0 {
            // First line: from selection start to right edge
            if has_content {
                let max_x = line_max_x(ly);
                (text_x + start_pos.x as f64, text_x + (max_x as f64).max(text_w))
            } else {
                (text_x + start_pos.x as f64, text_x + start_pos.x as f64 + 8.0)
            }
        } else if idx == num_lines - 1 {
            // Last line: from left edge to selection end
            let ex = end_pos.x as f64;
            if ex < 0.5 {
                (text_x, text_x + 8.0)
            } else {
                (text_x, text_x + ex)
            }
        } else {
            // Middle line: full width
            if has_content {
                (text_x, text_x + text_w)
            } else {
                (text_x, text_x + 8.0)
            }
        };

        if rx2 > rx1 {
            scene.fill(
                Fill::NonZero,
                Affine::scale(scale),
                sel_color,
                None,
                &Rect::new(rx1, text_y + sy, rx2, text_y + sy + line_height as f64),
            );
        }
    }
}

// ── Single-line input rendering ──────────────────────────────────────

fn paint_singleline(
    scene: &mut Scene,
    text_renderer: &mut TextRenderer,
    input: &InputRenderInfo,
    text_x: f64,
    text_y: f64,
    text_w: f64,
    text_h: f64,
    line_height: f32,
    is_empty: bool,
    scale: f64,
) {
    let text_offset_y = ((text_h as f32 - line_height) / 2.0).max(0.0);

    let positions = if !is_empty {
        text_renderer.grapheme_x_positions(&input.display_text, input.font_size)
    } else {
        vec![0.0]
    };

    if is_empty && !input.placeholder.is_empty() {
        text_renderer.draw_text(
            scene,
            &input.placeholder,
            cosmic_text::Attrs::new(),
            input.font_size,
            text_w as f32,
            text_h as f32,
            (text_x as f32, text_y as f32 + text_offset_y),
            VelloColor::from_rgba8(128, 128, 128, 255),
            scale,
        );
    } else if !is_empty {
        // Draw selection highlight
        if input.focused
            && input.sel_start != input.sel_end
            && input.sel_start < positions.len()
            && input.sel_end <= positions.len()
        {
            let x1 = (positions[input.sel_start] - input.scroll_offset) as f64;
            let x2 = (positions[input.sel_end] - input.scroll_offset) as f64;
            let sel_rect = Rect::new(
                text_x + x1,
                text_y + text_offset_y as f64,
                text_x + x2,
                text_y + text_offset_y as f64 + line_height as f64,
            );
            scene.fill(
                Fill::NonZero,
                Affine::scale(scale),
                VelloColor::from_rgba8(56, 121, 185, 128),
                None,
                &sel_rect,
            );
        }

        text_renderer.draw_text(
            scene,
            &input.display_text,
            cosmic_text::Attrs::new(),
            input.font_size,
            text_w as f32 + input.scroll_offset + 10000.0,
            text_h as f32,
            (text_x as f32 - input.scroll_offset, text_y as f32 + text_offset_y),
            input.text_color.to_vello(),
            scale,
        );
    }

    // Draw cursor
    if input.focused && input.blink_visible {
        let cursor_x = if input.cursor_pos < positions.len() {
            (positions[input.cursor_pos] - input.scroll_offset) as f64
        } else {
            0.0
        };
        paint_cursor(scene, text_x + cursor_x, text_y + text_offset_y as f64, line_height, scale);
    }
}

// ── Shared helpers ───────────────────────────────────────────────────

fn paint_cursor(
    scene: &mut Scene,
    x: f64,
    y: f64,
    line_height: f32,
    scale: f64,
) {
    let cursor_rect = Rect::new(x, y + 2.0, x + 1.5, y + line_height as f64 - 2.0);
    scene.fill(
        Fill::NonZero,
        Affine::scale(scale),
        VelloColor::from_rgba8(212, 212, 212, 255),
        None,
        &cursor_rect,
    );
}
