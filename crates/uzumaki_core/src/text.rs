use std::collections::HashMap;
use std::sync::Arc;

use cosmic_text::fontdb;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};
use vello::peniko::{Blob, Brush, Color, Fill, FontData};
use vello::{Glyph, Scene};

type FontId = fontdb::ID;

pub struct TextRenderer {
    pub font_system: FontSystem,
    // Maps cosmic-text font IDs to vello Fonts.
    // cosmic-text identifies loaded fonts by fontdb::ID; vello needs its own Font
    // handle (built from the same raw bytes) to render glyph outlines on the GPU.
    font_cache: HashMap<FontId, FontData>,
}

impl TextRenderer {
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();

        // Load bundled Roboto so we always have a known font available,
        // even on systems with limited installed fonts.
        let roboto = include_bytes!("../assets/Roboto-Regular.ttf");
        font_system.db_mut().load_font_data(roboto.to_vec());

        Self {
            font_system,
            font_cache: HashMap::new(),
        }
    }

    /// Extracts raw font file bytes from cosmic-text's fontdb and constructs
    /// a vello Font. This is the key bridge between the two libraries:
    /// cosmic-text uses the bytes for shaping/layout (via rustybuzz),
    /// vello uses the same bytes to read glyph outlines for GPU rendering (via skrifa).
    fn ensure_font_cached(&mut self, font_id: FontId) {
        if self.font_cache.contains_key(&font_id) {
            return;
        }
        // fontdb::Database::with_face_data gives us the raw font file bytes
        // and the face index within that file (relevant for .ttc collections).
        let font_data = self
            .font_system
            .db()
            .with_face_data(font_id, |data, index| (data.to_vec(), index));
        if let Some((data, index)) = font_data {
            let font = FontData::new(
                Blob::new(Arc::new(data) as Arc<dyn AsRef<[u8]> + Send + Sync>),
                index,
            );
            self.font_cache.insert(font_id, font);
        }
    }

    fn layout_buffer(
        &mut self,
        text: &str,
        attrs: Attrs<'_>,
        font_size: f32,
        width: Option<f32>,
        height: Option<f32>,
    ) -> Buffer {
        let metrics = Metrics::new(font_size, (font_size * 1.2).round());
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_text(&mut self.font_system, text, &attrs, Shaping::Advanced, None);
        buffer.set_size(&mut self.font_system, width, height);
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }

    fn cache_fonts_from_buffer(&mut self, buffer: &Buffer) {
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                self.ensure_font_cached(glyph.font_id);
            }
        }
    }

    pub fn draw_text(
        &mut self,
        scene: &mut Scene,
        text: &str,
        attrs: Attrs<'_>,
        font_size: f32,
        width: f32,
        height: f32,
        position: (f32, f32),
        color: Color,
    ) {
        let buffer = self.layout_buffer(text, attrs, font_size, Some(width), Some(height));
        self.cache_fonts_from_buffer(&buffer);

        // Second pass: draw glyphs, grouping consecutive runs by font_id
        let (px, py) = position;
        for run in buffer.layout_runs() {
            // Group consecutive glyphs by font_id so each draw_glyphs call
            // uses a single font (required by the vello API).
            let mut by_font: Vec<(FontId, Vec<Glyph>)> = Vec::new();

            for glyph in run.glyphs.iter() {
                let vello_glyph = Glyph {
                    id: glyph.glyph_id as u32,
                    x: px + glyph.x,
                    y: py + run.line_y,
                };

                if let Some(last) = by_font.last_mut() {
                    if last.0 == glyph.font_id {
                        last.1.push(vello_glyph);
                        continue;
                    }
                }
                by_font.push((glyph.font_id, vec![vello_glyph]));
            }

            for (font_id, glyphs) in by_font {
                if let Some(vello_font) = self.font_cache.get(&font_id) {
                    scene
                        .draw_glyphs(vello_font)
                        .font_size(font_size)
                        .brush(&Brush::Solid(color))
                        .draw(Fill::NonZero, glyphs.into_iter());
                }
            }
        }
    }

    pub fn measure_text(
        &mut self,
        text: &str,
        attrs: Attrs<'_>,
        font_size: f32,
        max_width: Option<f32>,
        max_height: Option<f32>,
    ) -> (f32, f32) {
        let buffer = self.layout_buffer(text, attrs, font_size, max_width, max_height);
        self.cache_fonts_from_buffer(&buffer);

        let mut measured_width: f32 = 0.0;
        let mut measured_height: f32 = 0.0;

        for run in buffer.layout_runs() {
            // Use glyph extents to avoid relying on line_w when wrap width is tiny.
            for glyph in run.glyphs.iter() {
                measured_width = measured_width.max(glyph.x + glyph.w);
            }
            measured_height = measured_height.max(run.line_top + run.line_height);
        }

        if measured_height == 0.0 {
            measured_height = buffer.metrics().line_height;
        }
        if measured_width == 0.0 {
            measured_width = (text.len() as f32) * (font_size * 0.6);
        }

        (measured_width, measured_height)
    }
}
