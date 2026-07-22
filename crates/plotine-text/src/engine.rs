use std::sync::{Mutex, OnceLock};

use cosmic_text::{
    Attrs, Buffer, CacheKey, Family, FontSystem, Metrics, Shaping, Style, SwashCache, SwashContent,
    Weight,
};
use plotine_core::{PlotError, Point, Result};

/// Quantize font size to whole pixels so swash hinting lands on a stable grid.
#[inline]
fn quantize_size_px(size_px: f32) -> f32 {
    size_px.round().max(1.0)
}

/// Embedded DejaVu Sans bytes (shared with PDF/SVG font databases).
pub const EMBEDDED_FONT: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
/// Embedded DejaVu Sans Oblique for mathtext italic (matplotlib math default).
pub const EMBEDDED_FONT_OBLIQUE: &[u8] = include_bytes!("../fonts/DejaVuSans-Oblique.ttf");
/// Family name registered for [`EMBEDDED_FONT`].
pub const FONT_FAMILY: &str = "DejaVu Sans";

fn font_system() -> &'static Mutex<FontSystem> {
    static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();
    FONT_SYSTEM.get_or_init(|| {
        let mut fs = FontSystem::new();
        fs.db_mut().load_font_data(EMBEDDED_FONT.to_vec());
        fs.db_mut().load_font_data(EMBEDDED_FONT_OBLIQUE.to_vec());
        Mutex::new(fs)
    })
}

/// Run `f` with exclusive access to the shared [`FontSystem`].
pub(crate) fn with_font_system<T>(f: impl FnOnce(&mut FontSystem) -> Result<T>) -> Result<T> {
    let mut fs = font_system()
        .lock()
        .map_err(|_| PlotError::text("font system lock poisoned"))?;
    f(&mut fs)
}

/// Rasterized glyph mask ready for backends to blit.
#[derive(Debug, Clone)]
pub struct GlyphImage {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub content: GlyphContent,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphContent {
    Mask,
    SubpixelMask,
    Color,
}

/// Text measurement + shaping facade over cosmic-text.
pub struct TextEngine {
    swash: SwashCache,
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEngine {
    pub fn new() -> Self {
        let _ = font_system();
        Self {
            swash: SwashCache::new(),
        }
    }

    fn attrs(italic: bool) -> Attrs<'static> {
        let style = if italic { Style::Italic } else { Style::Normal };
        Attrs::new()
            .family(Family::Name(FONT_FAMILY))
            .weight(Weight::NORMAL)
            .style(style)
    }

    pub fn measure(&self, text: &str, size_px: f32) -> Result<(f64, f64)> {
        self.measure_styled(text, size_px, false)
    }

    pub fn measure_styled(&self, text: &str, size_px: f32, italic: bool) -> Result<(f64, f64)> {
        let size_px = quantize_size_px(size_px);
        let mut fs = font_system()
            .lock()
            .map_err(|_| PlotError::text("font system lock poisoned"))?;
        let metrics = Metrics::new(size_px, size_px * 1.2);
        let mut buffer = Buffer::new(&mut fs, metrics);
        buffer.set_text(&mut fs, text, &Self::attrs(italic), Shaping::Advanced);
        buffer.shape_until_scroll(&mut fs, false);

        let width = buffer
            .layout_runs()
            .map(|run| run.line_w as f64)
            .fold(0.0_f64, f64::max);
        let height = buffer
            .layout_runs()
            .map(|run| run.line_height as f64)
            .sum::<f64>()
            .max(size_px as f64);
        Ok((width, height))
    }

    /// Shape `text` and return positioned glyph images relative to baseline-left `origin`.
    pub fn glyphs_at(
        &mut self,
        text: &str,
        origin: Point,
        size_px: f32,
    ) -> Result<Vec<GlyphImage>> {
        self.glyphs_at_styled(text, origin, size_px, false)
    }

    pub fn glyphs_at_styled(
        &mut self,
        text: &str,
        origin: Point,
        size_px: f32,
        italic: bool,
    ) -> Result<Vec<GlyphImage>> {
        let size_px = quantize_size_px(size_px);
        let mut fs = font_system()
            .lock()
            .map_err(|_| PlotError::text("font system lock poisoned"))?;
        let metrics = Metrics::new(size_px, size_px * 1.2);
        let mut buffer = Buffer::new(&mut fs, metrics);
        buffer.set_size(&mut fs, Some(10_000.0), Some(size_px * 2.0));
        buffer.set_text(&mut fs, text, &Self::attrs(italic), Shaping::Advanced);
        buffer.shape_until_scroll(&mut fs, false);

        // Snap glyph origins to the pixel grid (zero subpixel bin). Fractional
        // x-bins make swash AA softer and prevent stem cores from reaching
        // full coverage — the main "blurry text" feel vs FreeType/Agg.
        let ox = origin.x as f32;
        let oy = origin.y as f32;

        let mut out = Vec::new();
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let x_off = glyph.font_size * glyph.x_offset;
                let y_off = glyph.font_size * glyph.y_offset;
                let x = (glyph.x + x_off + ox).round();
                let y = (glyph.y - y_off + oy).round();
                let (cache_key, px, py) = CacheKey::new(
                    glyph.font_id,
                    glyph.glyph_id,
                    glyph.font_size,
                    (x, y),
                    glyph.cache_key_flags,
                );
                let image_ref = self.swash.get_image(&mut fs, cache_key);
                if let Some(image) = image_ref.as_ref() {
                    if image.placement.width == 0 || image.placement.height == 0 {
                        continue;
                    }
                    let content = match image.content {
                        SwashContent::Mask => GlyphContent::Mask,
                        SwashContent::SubpixelMask => GlyphContent::SubpixelMask,
                        SwashContent::Color => GlyphContent::Color,
                    };
                    out.push(GlyphImage {
                        x: px + image.placement.left,
                        y: py - image.placement.top,
                        width: image.placement.width,
                        height: image.placement.height,
                        content,
                        data: image.data.clone(),
                    });
                }
            }
        }
        Ok(out)
    }
}
