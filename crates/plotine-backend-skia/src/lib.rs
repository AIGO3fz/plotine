//! Raster backend powered by tiny-skia + plotine-text.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use kurbo::{BezPath, PathEl};
use plotine_core::{Color, PlotError, Point, Rect, Result};
use plotine_render::{
    FillStyle, LineCap, LineJoin, Renderer, StrokeStyle, TextAlign, TextBaseline, TextStyle,
};
use plotine_text::TextEngine;
use tiny_skia::{
    FillRule, FilterQuality, LineCap as SkiaCap, LineJoin as SkiaJoin, Mask, Paint,
    Path as SkiaPath, PathBuilder, Pixmap, PixmapPaint, PremultipliedColorU8, Stroke, StrokeDash,
    Transform,
};

/// In-memory RGBA pixmap renderer.
pub struct SkiaRenderer {
    pixmap: Pixmap,
    text: TextEngine,
    clip_stack: Vec<Option<Mask>>,
}

impl SkiaRenderer {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let pixmap = Pixmap::new(width.max(1), height.max(1)).ok_or_else(|| {
            PlotError::render(format!("failed to allocate pixmap {width}x{height}"))
        })?;
        Ok(Self {
            pixmap,
            text: TextEngine::new(),
            clip_stack: Vec::new(),
        })
    }

    pub fn pixmap(&self) -> &Pixmap {
        &self.pixmap
    }

    pub fn into_pixmap(self) -> Pixmap {
        self.pixmap
    }

    pub fn encode_png(&self) -> Result<Vec<u8>> {
        let rgba = self.pixmap.data();
        let width = self.pixmap.width();
        let height = self.pixmap.height();

        // Drop the alpha channel when every pixel is opaque — typical for
        // publication figures and much closer to matplotlib's PNG sizes.
        let fully_opaque = rgba.chunks_exact(4).all(|px| px[3] == 255);
        let (color, payload) = if fully_opaque {
            let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
            for px in rgba.chunks_exact(4) {
                rgb.extend_from_slice(&px[..3]);
            }
            (png::ColorType::Rgb, rgb)
        } else {
            (png::ColorType::Rgba, rgba.to_vec())
        };

        let mut bytes = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut bytes);
            let mut encoder = png::Encoder::new(&mut cursor, width, height);
            encoder.set_color(color);
            encoder.set_depth(png::BitDepth::Eight);
            // Deterministic and competitive with matplotlib/Pillow zlib defaults.
            encoder.set_compression(png::Compression::Best);
            encoder.set_filter(png::FilterType::Paeth);
            encoder.set_adaptive_filter(png::AdaptiveFilterType::Adaptive);
            let mut writer = encoder
                .write_header()
                .map_err(|e| PlotError::io(e.to_string()))?;
            writer
                .write_image_data(&payload)
                .map_err(|e| PlotError::io(e.to_string()))?;
        }
        Ok(bytes)
    }

    pub fn save_png(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| PlotError::io(e.to_string()))?;
            }
        }
        let bytes = self.encode_png()?;
        let file = File::create(path).map_err(|e| PlotError::io(e.to_string()))?;
        let mut writer = BufWriter::new(file);
        use std::io::Write;
        writer
            .write_all(&bytes)
            .map_err(|e| PlotError::io(e.to_string()))?;
        Ok(())
    }

    fn paint_from(color: Color, anti_alias: bool) -> Paint<'static> {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = anti_alias;
        paint
    }

    fn to_skia_path(path: &BezPath) -> Result<SkiaPath> {
        let mut pb = PathBuilder::new();
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => pb.move_to(p.x as f32, p.y as f32),
                PathEl::LineTo(p) => pb.line_to(p.x as f32, p.y as f32),
                PathEl::QuadTo(p1, p2) => {
                    pb.quad_to(p1.x as f32, p1.y as f32, p2.x as f32, p2.y as f32)
                }
                PathEl::CurveTo(p1, p2, p3) => pb.cubic_to(
                    p1.x as f32,
                    p1.y as f32,
                    p2.x as f32,
                    p2.y as f32,
                    p3.x as f32,
                    p3.y as f32,
                ),
                PathEl::ClosePath => pb.close(),
            }
        }
        pb.finish()
            .ok_or_else(|| PlotError::render("failed to build skia path"))
    }

    fn stroke_from(style: &StrokeStyle) -> Stroke {
        let dash = style.dash.as_ref().and_then(|pattern| {
            let arr: Vec<f32> = pattern.iter().map(|v| *v as f32).collect();
            StrokeDash::new(arr, 0.0)
        });
        Stroke {
            width: style.width as f32,
            miter_limit: 4.0,
            line_cap: match style.cap {
                LineCap::Butt => SkiaCap::Butt,
                LineCap::Round => SkiaCap::Round,
                LineCap::Square => SkiaCap::Square,
            },
            line_join: match style.join {
                LineJoin::Miter => SkiaJoin::Miter,
                LineJoin::Round => SkiaJoin::Round,
                LineJoin::Bevel => SkiaJoin::Bevel,
            },
            dash,
        }
    }

    fn blit_glyph(&mut self, glyph: &plotine_text::GlyphImage, color: Color, bold: bool) {
        use plotine_text::GlyphContent;
        let w = glyph.width as i32;
        let h = glyph.height as i32;
        let pm_w = self.pixmap.width() as i32;
        let pm_h = self.pixmap.height() as i32;
        let data = self.pixmap.data_mut();

        for gy in 0..h {
            for gx in 0..w {
                let px = glyph.x + gx;
                let py = glyph.y + gy;
                if px < 0 || py < 0 || px >= pm_w || py >= pm_h {
                    continue;
                }
                let idx = (gy as u32 * glyph.width + gx as u32) as usize;
                let coverage = match glyph.content {
                    GlyphContent::Mask => glyph.data[idx] as f32 / 255.0,
                    // Average RGB coverage (do not drop G/B — that softens stems).
                    GlyphContent::SubpixelMask => {
                        let i = idx * 3;
                        let r = glyph.data[i] as f32;
                        let g = glyph.data[i + 1] as f32;
                        let b = glyph.data[i + 2] as f32;
                        (r + g + b) / (3.0 * 255.0)
                    }
                    GlyphContent::Color => glyph.data[idx * 4 + 3] as f32 / 255.0,
                };
                // Body text: mild fringe cleanup. Contour clabels: denser stems.
                let coverage = if bold {
                    sharpen_glyph_coverage_bold(coverage)
                } else {
                    sharpen_glyph_coverage(coverage)
                };
                if coverage <= 0.0 {
                    continue;
                }
                let di = ((py * pm_w + px) * 4) as usize;
                let src_a = (color.a as f32 / 255.0) * coverage;
                let dst_a = data[di + 3] as f32 / 255.0;
                let out_a = src_a + dst_a * (1.0 - src_a);
                if out_a <= 0.0 {
                    continue;
                }
                let blend = |s: u8, d: u8| -> u8 {
                    let s = s as f32 / 255.0;
                    let d = d as f32 / 255.0;
                    (((s * src_a + d * dst_a * (1.0 - src_a)) / out_a) * 255.0).round() as u8
                };
                data[di] = blend(color.r, data[di]);
                data[di + 1] = blend(color.g, data[di + 1]);
                data[di + 2] = blend(color.b, data[di + 2]);
                data[di + 3] = (out_a * 255.0).round() as u8;
            }
        }
    }

    fn draw_text_upright(&mut self, text: &str, position: Point, style: &TextStyle) -> Result<()> {
        let (w, h) = self
            .text
            .measure_styled(text, style.size_px, style.italic)?;
        let mut origin = position;
        match style.align {
            TextAlign::Left => {}
            TextAlign::Center => origin.x -= w * 0.5,
            TextAlign::Right => origin.x -= w,
        }
        match style.baseline {
            TextBaseline::Alphabetic => {}
            TextBaseline::Top => origin.y += style.size_px as f64 * 0.8,
            TextBaseline::Middle => origin.y += h * 0.25,
            TextBaseline::Bottom => origin.y -= style.size_px as f64 * 0.2,
        }
        // Snap to the pixel grid so swash hinting lands on integer advances.
        origin.x = origin.x.round();
        origin.y = origin.y.round();

        let glyphs = self
            .text
            .glyphs_at_styled(text, origin, style.size_px, style.italic)?;
        for glyph in glyphs {
            self.blit_glyph(&glyph, style.color, style.bold);
        }
        Ok(())
    }

    fn draw_text_rotated(&mut self, text: &str, position: Point, style: &TextStyle) -> Result<()> {
        let (w, h) = self
            .text
            .measure_styled(text, style.size_px, style.italic)?;
        let pad = 4u32;
        let tw = (w.ceil() as u32 + pad * 2).max(1);
        let th = (h.ceil() as u32 + pad * 2).max(1);
        let mut tmp = SkiaRenderer::new(tw, th)?;
        tmp.pixmap.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 0));

        let local = TextStyle {
            rotation_deg: 0.0,
            align: TextAlign::Left,
            baseline: TextBaseline::Alphabetic,
            ..style.clone()
        };
        let ox = pad as f64;
        let oy = pad as f64 + style.size_px as f64 * 0.85;
        tmp.draw_text_upright(text, Point::new(ox, oy), &local)?;

        let mut ax = ox;
        let mut ay = oy;
        match style.align {
            TextAlign::Left => {}
            TextAlign::Center => ax += w * 0.5,
            TextAlign::Right => ax += w,
        }
        match style.baseline {
            TextBaseline::Alphabetic => {}
            TextBaseline::Top => ay -= style.size_px as f64 * 0.8,
            TextBaseline::Middle => ay -= h * 0.25,
            TextBaseline::Bottom => ay += style.size_px as f64 * 0.2,
        }

        let angle = style.rotation_deg;
        let norm = ((angle % 360.0) + 360.0) % 360.0;

        // ±90° / 180°: integer pixel remaps — no filtered resampling.
        if (norm - 90.0).abs() < 0.5 {
            let (rotated, nax, nay) = rotate_pixmap_cw90(&tmp.pixmap, ax, ay);
            self.blit_pixmap_integer(&rotated, position.x - nax, position.y - nay);
            return Ok(());
        }
        if (norm - 270.0).abs() < 0.5 || (norm + 90.0).abs() < 0.5 {
            let (rotated, nax, nay) = rotate_pixmap_ccw90(&tmp.pixmap, ax, ay);
            self.blit_pixmap_integer(&rotated, position.x - nax, position.y - nay);
            return Ok(());
        }
        if (norm - 180.0).abs() < 0.5 {
            let (rotated, nax, nay) = rotate_pixmap_180(&tmp.pixmap, ax, ay);
            self.blit_pixmap_integer(&rotated, position.x - nax, position.y - nay);
            return Ok(());
        }

        // Arbitrary angles (clabel / tilted ticks): 2× supersample + nearest rotate.
        let ss = 2.0_f32;
        let style_ss = TextStyle {
            size_px: style.size_px * ss,
            rotation_deg: 0.0,
            align: TextAlign::Left,
            baseline: TextBaseline::Alphabetic,
            ..style.clone()
        };
        let (w2, h2) = self
            .text
            .measure_styled(text, style_ss.size_px, style_ss.italic)?;
        let pad2 = pad * 2;
        let tw2 = (w2.ceil() as u32 + pad2 * 2).max(1);
        let th2 = (h2.ceil() as u32 + pad2 * 2).max(1);
        let mut tmp2 = SkiaRenderer::new(tw2, th2)?;
        tmp2.pixmap.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 0));
        let ox2 = pad2 as f64;
        let oy2 = pad2 as f64 + style_ss.size_px as f64 * 0.85;
        tmp2.draw_text_upright(text, Point::new(ox2, oy2), &style_ss)?;
        let mut ax2 = ox2;
        let mut ay2 = oy2;
        match style.align {
            TextAlign::Left => {}
            TextAlign::Center => ax2 += w2 * 0.5,
            TextAlign::Right => ax2 += w2,
        }
        match style.baseline {
            TextBaseline::Alphabetic => {}
            TextBaseline::Top => ay2 -= style_ss.size_px as f64 * 0.8,
            TextBaseline::Middle => ay2 -= h2 * 0.25,
            TextBaseline::Bottom => ay2 += style_ss.size_px as f64 * 0.2,
        }

        // Map 2× buffer → destination with rotate around anchor, then /2 scale.
        let transform = Transform::from_translate(position.x as f32, position.y as f32)
            .pre_concat(Transform::from_scale(1.0 / ss, 1.0 / ss))
            .pre_concat(Transform::from_rotate(style.rotation_deg as f32))
            .pre_concat(Transform::from_translate(-ax2 as f32, -ay2 as f32));
        let paint = PixmapPaint {
            quality: FilterQuality::Nearest,
            ..PixmapPaint::default()
        };
        let src = tmp2.pixmap;
        self.clipped_draw(|pixmap, clip| {
            pixmap.draw_pixmap(0, 0, src.as_ref(), &paint, transform, clip);
        });
        Ok(())
    }

    fn blit_pixmap_integer(&mut self, src: &Pixmap, dx: f64, dy: f64) {
        let x = dx.round() as i32;
        let y = dy.round() as i32;
        let paint = PixmapPaint {
            quality: FilterQuality::Nearest,
            ..PixmapPaint::default()
        };
        self.clipped_draw(|pixmap, clip| {
            pixmap.draw_pixmap(x, y, src.as_ref(), &paint, Transform::identity(), clip);
        });
    }

    /// Draw with the active clip mask. Uses raw pointer to break the
    /// `&mut pixmap` / `&clip_stack` borrow conflict safely: the callback
    /// never mutates the clip_stack and we hold `&mut self` for the duration.
    fn clipped_draw<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Pixmap, Option<&Mask>),
    {
        let clip_ptr = self.clip_stack.as_ptr();
        let clip_len = self.clip_stack.len();
        // SAFETY: we hold &mut self exclusively; the closure only writes to
        // pixmap, never touches clip_stack. The slice is valid for the call.
        let clip = unsafe {
            std::slice::from_raw_parts(clip_ptr, clip_len)
                .iter()
                .rev()
                .flatten()
                .next()
        };
        f(&mut self.pixmap, clip);
    }
}

impl Renderer for SkiaRenderer {
    fn clear(&mut self, color: Color) -> Result<()> {
        self.pixmap.fill(tiny_skia::Color::from_rgba8(
            color.r, color.g, color.b, color.a,
        ));
        Ok(())
    }

    fn fill_rect(&mut self, rect: Rect, style: &FillStyle) -> Result<()> {
        let mut pb = PathBuilder::new();
        pb.push_rect(
            tiny_skia::Rect::from_xywh(
                rect.x0 as f32,
                rect.y0 as f32,
                rect.width() as f32,
                rect.height() as f32,
            )
            .ok_or_else(|| PlotError::render("invalid fill rect"))?,
        );
        let path = pb
            .finish()
            .ok_or_else(|| PlotError::render("failed to build rect path"))?;
        let paint = Self::paint_from(style.color, style.anti_alias);
        self.clipped_draw(|pixmap, clip| {
            pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                clip,
            );
        });
        Ok(())
    }

    fn stroke_rect(&mut self, rect: Rect, style: &StrokeStyle) -> Result<()> {
        let mut path = BezPath::new();
        path.move_to(kurbo::Point::new(rect.x0, rect.y0));
        path.line_to(kurbo::Point::new(rect.x1, rect.y0));
        path.line_to(kurbo::Point::new(rect.x1, rect.y1));
        path.line_to(kurbo::Point::new(rect.x0, rect.y1));
        path.close_path();
        self.stroke_path(&path, style)
    }

    fn fill_path(&mut self, path: &BezPath, style: &FillStyle) -> Result<()> {
        let sk = Self::to_skia_path(path)?;
        let paint = Self::paint_from(style.color, style.anti_alias);
        self.clipped_draw(|pixmap, clip| {
            pixmap.fill_path(&sk, &paint, FillRule::Winding, Transform::identity(), clip);
        });
        Ok(())
    }

    fn stroke_path(&mut self, path: &BezPath, style: &StrokeStyle) -> Result<()> {
        let sk = Self::to_skia_path(path)?;
        let paint = Self::paint_from(style.color, true);
        let stroke = Self::stroke_from(style);
        self.clipped_draw(|pixmap, clip| {
            pixmap.stroke_path(&sk, &paint, &stroke, Transform::identity(), clip);
        });
        Ok(())
    }

    fn draw_text(&mut self, text: &str, position: Point, style: &TextStyle) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        if style.rotation_deg.abs() < 1e-6 {
            self.draw_text_upright(text, position, style)
        } else {
            self.draw_text_rotated(text, position, style)
        }
    }

    fn draw_rgba_image(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        position: Point,
    ) -> Result<()> {
        let expected = (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(4);
        if rgba.len() != expected {
            return Err(PlotError::render(format!(
                "RGBA buffer length {} != {width}×{height}×4",
                rgba.len()
            )));
        }
        if width == 0 || height == 0 {
            return Ok(());
        }
        let mut src = Pixmap::new(width, height)
            .ok_or_else(|| PlotError::render("failed to allocate RGBA pixmap"))?;
        for (i, px) in src.pixels_mut().iter_mut().enumerate() {
            let o = i * 4;
            *px = PremultipliedColorU8::from_rgba(rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3])
                .unwrap_or_else(|| PremultipliedColorU8::from_rgba(0, 0, 0, 0).unwrap());
        }
        self.blit_pixmap_integer(&src, position.x, position.y);
        Ok(())
    }

    fn measure_text_styled(&self, text: &str, size_px: f32, italic: bool) -> Result<(f64, f64)> {
        self.text.measure_styled(text, size_px, italic)
    }

    fn push_clip_rect(&mut self, rect: Rect) -> Result<()> {
        let mut pb = PathBuilder::new();
        pb.push_rect(
            tiny_skia::Rect::from_xywh(
                rect.x0 as f32,
                rect.y0 as f32,
                rect.width().max(0.0) as f32,
                rect.height().max(0.0) as f32,
            )
            .ok_or_else(|| PlotError::render("invalid clip rect"))?,
        );
        let path = pb
            .finish()
            .ok_or_else(|| PlotError::render("failed to build clip path"))?;
        let mut mask = Mask::new(self.pixmap.width(), self.pixmap.height())
            .ok_or_else(|| PlotError::render("failed to allocate clip mask"))?;
        mask.fill_path(&path, FillRule::Winding, true, Transform::identity());
        self.clip_stack.push(Some(mask));
        Ok(())
    }

    fn pop_clip(&mut self) -> Result<()> {
        self.clip_stack.pop();
        Ok(())
    }

    fn width(&self) -> u32 {
        self.pixmap.width()
    }

    fn height(&self) -> u32 {
        self.pixmap.height()
    }
}

/// Mild AA cleanup for regular body text (titles / ticks / axis labels).
///
/// Former curve fattened stems vs FreeType/Agg; keep only light fringe dust
/// cut so glyphs stay regular weight (contour clabels use the bold path).
#[inline]
fn sharpen_glyph_coverage(c: f32) -> f32 {
    if c < 0.08 {
        return 0.0;
    }
    // Near-identity: drop soft halo only.
    (c * 1.02).clamp(0.0, 1.0)
}

/// Denser stems for contour clabels (`TextStyle::bold`).
#[inline]
fn sharpen_glyph_coverage_bold(c: f32) -> f32 {
    if c < 0.05 {
        return 0.0;
    }
    let c = c.powf(0.55).clamp(0.0, 1.0);
    (1.0 - (1.0 - c).powf(2.15)).clamp(0.0, 1.0)
}

/// Rotate pixmap 90° clockwise (y-down). Returns `(rotated, new_ax, new_ay)`.
fn rotate_pixmap_cw90(src: &Pixmap, ax: f64, ay: f64) -> (Pixmap, f64, f64) {
    let w = src.width();
    let h = src.height();
    let mut dst = Pixmap::new(h, w).expect("rotate cw pixmap");
    let s = src.data();
    let d = dst.data_mut();
    for y in 0..h {
        for x in 0..w {
            let si = ((y * w + x) * 4) as usize;
            let nx = h - 1 - y;
            let ny = x;
            let di = ((ny * h + nx) * 4) as usize;
            d[di..di + 4].copy_from_slice(&s[si..si + 4]);
        }
    }
    // (x,y) → (h-y, x)
    (dst, h as f64 - ay, ax)
}

/// Rotate pixmap 90° counter-clockwise (y-down).
fn rotate_pixmap_ccw90(src: &Pixmap, ax: f64, ay: f64) -> (Pixmap, f64, f64) {
    let w = src.width();
    let h = src.height();
    let mut dst = Pixmap::new(h, w).expect("rotate ccw pixmap");
    let s = src.data();
    let d = dst.data_mut();
    for y in 0..h {
        for x in 0..w {
            let si = ((y * w + x) * 4) as usize;
            let nx = y;
            let ny = w - 1 - x;
            let di = ((ny * h + nx) * 4) as usize;
            d[di..di + 4].copy_from_slice(&s[si..si + 4]);
        }
    }
    // (x,y) → (y, w-x)
    (dst, ay, w as f64 - ax)
}

/// Rotate pixmap 180°.
fn rotate_pixmap_180(src: &Pixmap, ax: f64, ay: f64) -> (Pixmap, f64, f64) {
    let w = src.width();
    let h = src.height();
    let mut dst = Pixmap::new(w, h).expect("rotate 180 pixmap");
    let s = src.data();
    let d = dst.data_mut();
    for y in 0..h {
        for x in 0..w {
            let si = ((y * w + x) * 4) as usize;
            let nx = w - 1 - x;
            let ny = h - 1 - y;
            let di = ((ny * w + nx) * 4) as usize;
            d[di..di + 4].copy_from_slice(&s[si..si + 4]);
        }
    }
    (dst, w as f64 - ax, h as f64 - ay)
}
