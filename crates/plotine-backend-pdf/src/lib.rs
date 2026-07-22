//! Vector PDF backend for plotine.
//!
//! Renders through the deterministic SVG backend, then converts with `svg2pdf`
//! using the embedded DejaVu Sans font (no system font dependency).

use std::fs;
use std::path::Path;

use kurbo::BezPath;
use plotine_backend_svg::SvgRenderer;
use plotine_core::{Color, PlotError, Point, Rect, Result};
use plotine_render::{FillStyle, Renderer, StrokeStyle, TextStyle};
use plotine_text::{registered_font_data, EMBEDDED_FONT, FONT_FAMILY};
use svg2pdf::{ConversionOptions, PageOptions};

/// Accumulates drawing commands and emits a PDF document.
pub struct PdfRenderer {
    inner: SvgRenderer,
}

impl PdfRenderer {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            inner: SvgRenderer::new(width, height)?,
        })
    }

    /// Convert the accumulated drawing commands into PDF bytes.
    pub fn into_pdf(self) -> Result<Vec<u8>> {
        let svg = self.inner.into_svg();
        svg_to_pdf(&svg)
    }

    pub fn save_pdf(&self, path: impl AsRef<Path>) -> Result<()> {
        // Clone via re-render path: SvgRenderer has no Clone; rebuild PDF from current SVG.
        let svg = self.inner.to_svg_string();
        let pdf = svg_to_pdf(&svg)?;
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| PlotError::io(e.to_string()))?;
            }
        }
        fs::write(path, pdf).map_err(|e| PlotError::io(e.to_string()))
    }
}

/// Convert an SVG document (from [`SvgRenderer`]) into PDF bytes.
pub fn svg_to_pdf(svg: &str) -> Result<Vec<u8>> {
    let mut options = svg2pdf::usvg::Options::default();
    options.fontdb_mut().load_font_data(EMBEDDED_FONT.to_vec());
    // Extra faces registered via `plotine_text::register_font_*` (e.g. CJK).
    for data in registered_font_data() {
        options.fontdb_mut().load_font_data(data.to_vec());
    }
    // SVG emitter lists `{FONT_FAMILY}, …`; embedded DejaVu face must match.
    debug_assert_eq!(FONT_FAMILY, "DejaVu Sans");
    let tree = svg2pdf::usvg::Tree::from_str(svg, &options)
        .map_err(|e| PlotError::render(format!("SVG→PDF parse failed: {e}")))?;
    svg2pdf::to_pdf(&tree, ConversionOptions::default(), PageOptions::default())
        .map_err(|e| PlotError::render(format!("SVG→PDF conversion failed: {e}")))
}

impl Renderer for PdfRenderer {
    fn clear(&mut self, color: Color) -> Result<()> {
        self.inner.clear(color)
    }

    fn fill_rect(&mut self, rect: Rect, style: &FillStyle) -> Result<()> {
        self.inner.fill_rect(rect, style)
    }

    fn stroke_rect(&mut self, rect: Rect, style: &StrokeStyle) -> Result<()> {
        self.inner.stroke_rect(rect, style)
    }

    fn fill_path(&mut self, path: &BezPath, style: &FillStyle) -> Result<()> {
        self.inner.fill_path(path, style)
    }

    fn stroke_path(&mut self, path: &BezPath, style: &StrokeStyle) -> Result<()> {
        self.inner.stroke_path(path, style)
    }

    fn draw_text(&mut self, text: &str, position: Point, style: &TextStyle) -> Result<()> {
        self.inner.draw_text(text, position, style)
    }

    fn draw_rgba_image(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        position: Point,
    ) -> Result<()> {
        self.inner.draw_rgba_image(rgba, width, height, position)
    }

    fn measure_text_styled(&self, text: &str, size_px: f32, italic: bool) -> Result<(f64, f64)> {
        self.inner.measure_text_styled(text, size_px, italic)
    }

    fn push_clip_rect(&mut self, rect: Rect) -> Result<()> {
        self.inner.push_clip_rect(rect)
    }

    fn pop_clip(&mut self) -> Result<()> {
        self.inner.pop_clip()
    }

    fn width(&self) -> u32 {
        self.inner.width()
    }

    fn height(&self) -> u32 {
        self.inner.height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_render::FillStyle;

    #[test]
    fn pdf_magic_and_stable() {
        let mut r = PdfRenderer::new(120, 80).unwrap();
        r.clear(Color::WHITE).unwrap();
        r.fill_rect(
            Rect::new(10.0, 10.0, 50.0, 40.0),
            &FillStyle::solid(Color::CRIMSON),
        )
        .unwrap();
        let a = r.into_pdf().unwrap();
        assert!(a.starts_with(b"%PDF"));

        let mut r2 = PdfRenderer::new(120, 80).unwrap();
        r2.clear(Color::WHITE).unwrap();
        r2.fill_rect(
            Rect::new(10.0, 10.0, 50.0, 40.0),
            &FillStyle::solid(Color::CRIMSON),
        )
        .unwrap();
        let b = r2.into_pdf().unwrap();
        assert_eq!(a, b, "PDF should be byte-stable across runs");
    }
}
