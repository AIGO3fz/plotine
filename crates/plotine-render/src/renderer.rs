use kurbo::BezPath;
use plotine_core::{Point, Rect, Result};

use crate::primitives::{FillStyle, StrokeStyle, TextStyle};

/// Backend-agnostic drawing surface.
///
/// Implement this trait to add a new output format. The built-in backends are:
/// - `plotine-backend-skia::SkiaRenderer` — raster PNG via tiny-skia
/// - `plotine-backend-svg::SvgRenderer` — deterministic SVG
///
/// Recipes produce [`kurbo::BezPath`] geometry; backends only need to fill/stroke
/// paths, draw text, and manage a clip stack.
pub trait Renderer {
    fn clear(&mut self, color: plotine_core::Color) -> Result<()>;

    fn fill_rect(&mut self, rect: Rect, style: &FillStyle) -> Result<()>;

    fn stroke_rect(&mut self, rect: Rect, style: &StrokeStyle) -> Result<()>;

    fn fill_path(&mut self, path: &BezPath, style: &FillStyle) -> Result<()>;

    fn stroke_path(&mut self, path: &BezPath, style: &StrokeStyle) -> Result<()>;

    fn draw_line(&mut self, p0: Point, p1: Point, style: &StrokeStyle) -> Result<()> {
        let mut path = BezPath::new();
        path.move_to(p0.to_kurbo());
        path.line_to(p1.to_kurbo());
        self.stroke_path(&path, style)
    }

    fn draw_text(&mut self, text: &str, position: Point, style: &TextStyle) -> Result<()>;

    /// Blit an RGBA8 image with top-left at `position` (figure pixels).
    ///
    /// `rgba.len()` must be `width * height * 4`. Used by optional external LaTeX
    /// (`feature = "latex"`) and any future raster overlays.
    fn draw_rgba_image(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        position: Point,
    ) -> Result<()>;

    /// Measure text width/height in pixels for layout (upright face).
    fn measure_text(&self, text: &str, size_px: f32) -> Result<(f64, f64)> {
        self.measure_text_styled(text, size_px, false)
    }

    /// Measure text with optional italic/oblique face (mathtext variables).
    fn measure_text_styled(&self, text: &str, size_px: f32, italic: bool) -> Result<(f64, f64)>;

    /// Restrict subsequent drawing to `rect` until [`pop_clip`](Self::pop_clip).
    fn push_clip_rect(&mut self, rect: Rect) -> Result<()>;

    fn pop_clip(&mut self) -> Result<()>;

    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
