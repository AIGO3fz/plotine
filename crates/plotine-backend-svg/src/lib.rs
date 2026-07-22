//! Deterministic SVG emitter for plotine.

use std::fs;
use std::path::Path;

use kurbo::{BezPath, PathEl};
use plotine_core::{Color, PlotError, Point, Rect, Result};
use plotine_render::{FillStyle, Renderer, StrokeStyle, TextAlign, TextBaseline, TextStyle};
use plotine_text::TextEngine;

/// Accumulates drawing commands into a reproducible SVG document.
pub struct SvgRenderer {
    width: u32,
    height: u32,
    nodes: Vec<String>,
    clip_stack: Vec<usize>,
    clip_defs: Vec<String>,
    text: TextEngine,
    background: Option<Color>,
}

impl SvgRenderer {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            width: width.max(1),
            height: height.max(1),
            nodes: Vec::new(),
            clip_stack: Vec::new(),
            clip_defs: Vec::new(),
            text: TextEngine::new(),
            background: None,
        })
    }

    /// Render the accumulated drawing commands as an SVG string.
    pub fn to_svg_string(&self) -> String {
        let mut out = String::new();
        out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        out.push('\n');
        out.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
            w = self.width,
            h = self.height
        ));
        out.push('\n');
        if !self.clip_defs.is_empty() {
            out.push_str("<defs>\n");
            for d in &self.clip_defs {
                out.push_str(d);
                out.push('\n');
            }
            out.push_str("</defs>\n");
        }
        if let Some(bg) = self.background {
            out.push_str(&format!(
                r#"<rect x="0" y="0" width="{}" height="{}" fill="{}" />"#,
                self.width,
                self.height,
                css_color(bg)
            ));
            out.push('\n');
        }
        for n in &self.nodes {
            out.push_str(n);
            out.push('\n');
        }
        out.push_str("</svg>\n");
        out
    }

    /// Consume and return the SVG string (convenience alias).
    pub fn into_svg(self) -> String {
        self.to_svg_string()
    }

    pub fn save_svg(&self, path: impl AsRef<Path>) -> Result<()> {
        let svg = self.to_svg_string();
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| PlotError::io(e.to_string()))?;
            }
        }
        fs::write(path, svg).map_err(|e| PlotError::io(e.to_string()))
    }

    fn clip_attr(&self) -> String {
        match self.clip_stack.last() {
            Some(id) => format!(r#" clip-path="url(#c{id})""#),
            None => String::new(),
        }
    }

    fn push_node(&mut self, node: String) {
        self.nodes.push(node);
    }
}

impl Renderer for SvgRenderer {
    fn clear(&mut self, color: Color) -> Result<()> {
        self.background = Some(color);
        self.nodes.clear();
        Ok(())
    }

    fn fill_rect(&mut self, rect: Rect, style: &FillStyle) -> Result<()> {
        self.push_node(format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"{} />"#,
            f(rect.x0),
            f(rect.y0),
            f(rect.width()),
            f(rect.height()),
            css_color(style.color),
            self.clip_attr()
        ));
        Ok(())
    }

    fn stroke_rect(&mut self, rect: Rect, style: &StrokeStyle) -> Result<()> {
        self.push_node(format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="{}"{}{} />"#,
            f(rect.x0),
            f(rect.y0),
            f(rect.width()),
            f(rect.height()),
            css_color(style.color),
            f(style.width),
            dash_attr(style),
            self.clip_attr()
        ));
        Ok(())
    }

    fn fill_path(&mut self, path: &BezPath, style: &FillStyle) -> Result<()> {
        self.push_node(format!(
            r#"<path d="{}" fill="{}"{} />"#,
            path_d(path),
            css_color(style.color),
            self.clip_attr()
        ));
        Ok(())
    }

    fn stroke_path(&mut self, path: &BezPath, style: &StrokeStyle) -> Result<()> {
        self.push_node(format!(
            r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}"{}{} />"#,
            path_d(path),
            css_color(style.color),
            f(style.width),
            match style.cap {
                plotine_render::LineCap::Butt => "butt",
                plotine_render::LineCap::Round => "round",
                plotine_render::LineCap::Square => "square",
            },
            match style.join {
                plotine_render::LineJoin::Miter => "miter",
                plotine_render::LineJoin::Round => "round",
                plotine_render::LineJoin::Bevel => "bevel",
            },
            dash_attr(style),
            self.clip_attr()
        ));
        Ok(())
    }

    fn draw_text(&mut self, text: &str, position: Point, style: &TextStyle) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        let anchor = match style.align {
            TextAlign::Left => "start",
            TextAlign::Center => "middle",
            TextAlign::Right => "end",
        };
        // Approximate baseline with dominant-baseline.
        let baseline = match style.baseline {
            TextBaseline::Top => "text-before-edge",
            TextBaseline::Middle => "middle",
            TextBaseline::Alphabetic => "alphabetic",
            TextBaseline::Bottom => "text-after-edge",
        };
        let transform = if style.rotation_deg.abs() > 1e-6 {
            format!(
                r#" transform="rotate({} {} {})""#,
                f(style.rotation_deg),
                f(position.x),
                f(position.y)
            )
        } else {
            String::new()
        };
        let escaped = xml_escape(text);
        let font_style = if style.italic {
            r#" font-style="italic""#
        } else {
            ""
        };
        self.push_node(format!(
            r#"<text x="{}" y="{}" fill="{}" font-family="{}" font-size="{}" text-anchor="{}" dominant-baseline="{}"{}{}{}>{}</text>"#,
            f(position.x),
            f(position.y),
            css_color(style.color),
            plotine_text::svg_font_family_list(),
            f(style.size_px as f64),
            anchor,
            baseline,
            font_style,
            transform,
            self.clip_attr(),
            escaped
        ));
        Ok(())
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
        let png = encode_rgba_png(rgba, width, height)?;
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png);
        self.push_node(format!(
            r#"<image x="{}" y="{}" width="{}" height="{}" preserveAspectRatio="none" href="data:image/png;base64,{}"{} />"#,
            f(position.x),
            f(position.y),
            f(width as f64),
            f(height as f64),
            b64,
            self.clip_attr()
        ));
        Ok(())
    }

    fn measure_text_styled(&self, text: &str, size_px: f32, italic: bool) -> Result<(f64, f64)> {
        self.text.measure_styled(text, size_px, italic)
    }

    fn push_clip_rect(&mut self, rect: Rect) -> Result<()> {
        let id = self.clip_defs.len();
        self.clip_defs.push(format!(
            r#"<clipPath id="c{id}"><rect x="{}" y="{}" width="{}" height="{}" /></clipPath>"#,
            f(rect.x0),
            f(rect.y0),
            f(rect.width().max(0.0)),
            f(rect.height().max(0.0))
        ));
        self.clip_stack.push(id);
        Ok(())
    }

    fn pop_clip(&mut self) -> Result<()> {
        self.clip_stack.pop();
        Ok(())
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

fn dash_attr(style: &StrokeStyle) -> String {
    match &style.dash {
        Some(pattern) if !pattern.is_empty() => {
            let parts: Vec<String> = pattern.iter().map(|v| f(*v)).collect();
            format!(r#" stroke-dasharray="{}""#, parts.join(","))
        }
        _ => String::new(),
    }
}

fn f(v: f64) -> String {
    // Fixed decimals keep SVG byte-stable across platforms.
    if !v.is_finite() {
        return "0".into();
    }
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn css_color(c: Color) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        format!("rgba({},{},{},{:.3})", c.r, c.g, c.b, c.a as f64 / 255.0)
    }
}

fn path_d(path: &BezPath) -> String {
    let mut d = String::new();
    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => d.push_str(&format!("M{} {} ", f(p.x), f(p.y))),
            PathEl::LineTo(p) => d.push_str(&format!("L{} {} ", f(p.x), f(p.y))),
            PathEl::QuadTo(p1, p2) => d.push_str(&format!(
                "Q{} {} {} {} ",
                f(p1.x),
                f(p1.y),
                f(p2.x),
                f(p2.y)
            )),
            PathEl::CurveTo(p1, p2, p3) => d.push_str(&format!(
                "C{} {} {} {} {} {} ",
                f(p1.x),
                f(p1.y),
                f(p2.x),
                f(p2.y),
                f(p3.x),
                f(p3.y)
            )),
            PathEl::ClosePath => d.push_str("Z "),
        }
    }
    d.trim_end().to_string()
}

fn encode_rgba_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| PlotError::render(format!("PNG header: {e}")))?;
        writer
            .write_image_data(rgba)
            .map_err(|e| PlotError::render(format!("PNG encode: {e}")))?;
    }
    Ok(buf)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
