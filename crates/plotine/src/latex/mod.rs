//! External LaTeX rendering (`feature = "latex"`).
//!
//! Pipeline: system `latex` → DVI → `dvipng` (tight PNG) → RGBA blit via
//! [`Renderer::draw_rgba_image`](plotine_render::Renderer::draw_rgba_image).
//! Active only when [`Figure::usetex`](crate::figure::Figure::usetex) is set
//! for the duration of [`Figure::draw`](crate::figure::Figure::draw).

use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use plotine_core::{Color, PlotError, Point, Result};
use plotine_render::{Renderer, TextAlign, TextBaseline, TextStyle};

#[derive(Clone, Copy)]
struct LatexCtx {
    usetex: bool,
    dpi: f64,
}

impl Default for LatexCtx {
    fn default() -> Self {
        Self {
            usetex: false,
            dpi: 150.0,
        }
    }
}

thread_local! {
    static CTX: Cell<LatexCtx> = const { Cell::new(LatexCtx {
        usetex: false,
        dpi: 150.0,
    }) };
}

/// RAII scope for usetex + figure DPI during a draw pass.
pub struct UsetexGuard {
    prev: LatexCtx,
}

impl UsetexGuard {
    /// Enable/disable external LaTeX for the current thread until drop.
    pub fn enter(usetex: bool, dpi: f64) -> Self {
        CTX.with(|c| {
            let prev = c.get();
            c.set(LatexCtx {
                usetex,
                dpi: dpi.max(1.0),
            });
            Self { prev }
        })
    }
}

impl Drop for UsetexGuard {
    fn drop(&mut self) {
        CTX.with(|c| c.set(self.prev));
    }
}

/// Whether the current draw pass requested external LaTeX.
pub fn usetex_active() -> bool {
    CTX.with(|c| c.get().usetex)
}

fn current_dpi() -> f64 {
    CTX.with(|c| c.get().dpi)
}

#[derive(Clone)]
struct Glyph {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    /// Distance from image top to alphabetic baseline (pixels).
    ascent: f64,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    text: String,
    size_milli: u32,
    dpi_milli: u32,
    color: u32,
}

fn cache() -> &'static Mutex<HashMap<CacheKey, Glyph>> {
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, Glyph>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn tool_on_path(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// True when both `latex` and `dvipng` appear to be runnable.
pub fn tools_available() -> bool {
    tool_on_path("latex") && tool_on_path("dvipng")
}

fn ensure_tools() -> Result<()> {
    if !tool_on_path("latex") {
        return Err(PlotError::latex_unavailable(
            "`latex` executable not found on PATH",
        ));
    }
    if !tool_on_path("dvipng") {
        return Err(PlotError::latex_unavailable(
            "`dvipng` executable not found on PATH",
        ));
    }
    Ok(())
}

fn render_cached(text: &str, size_px: f32, color: Color) -> Result<Glyph> {
    let dpi = current_dpi();
    let key = CacheKey {
        text: text.to_string(),
        size_milli: (size_px * 1000.0).round() as u32,
        dpi_milli: (dpi * 1000.0).round() as u32,
        color: u32::from_be_bytes([color.r, color.g, color.b, color.a]),
    };
    if let Ok(guard) = cache().lock() {
        if let Some(g) = guard.get(&key) {
            return Ok(g.clone());
        }
    }
    let glyph = render_once(text, size_px, dpi, color)?;
    if let Ok(mut guard) = cache().lock() {
        guard.insert(key, glyph.clone());
    }
    Ok(glyph)
}

fn render_once(text: &str, size_px: f32, dpi: f64, color: Color) -> Result<Glyph> {
    ensure_tools()?;
    let dir =
        tempfile::tempdir().map_err(|e| PlotError::latex_failed(format!("temp directory: {e}")))?;
    let work = dir.path();
    let tex_path = work.join("formula.tex");
    let pt = (size_px as f64) * 72.0 / dpi;
    let pt = pt.clamp(4.0, 72.0);
    let leading = pt * 1.2;
    let body = text.trim();
    let tex = format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath,amssymb}}
\pagestyle{{empty}}
\begin{{document}}
\fontsize{{{pt:.4}}}{{{leading:.4}}}\selectfont
{body}
\end{{document}}
"#
    );
    fs::write(&tex_path, tex).map_err(|e| PlotError::latex_failed(format!("write tex: {e}")))?;

    let latex_out = Command::new("latex")
        .current_dir(work)
        .args(["-interaction=nonstopmode", "-halt-on-error", "formula.tex"])
        .output()
        .map_err(|e| PlotError::latex_failed(format!("spawn latex: {e}")))?;
    if !latex_out.status.success() {
        let log = read_sidecar(work, "formula.log");
        let stderr = String::from_utf8_lossy(&latex_out.stderr);
        return Err(PlotError::latex_failed(format!(
            "latex failed (exit {:?})\n{stderr}\n{log}",
            latex_out.status.code()
        )));
    }
    let dvi = work.join("formula.dvi");
    if !dvi.is_file() {
        return Err(PlotError::latex_failed("latex did not produce formula.dvi"));
    }

    let png_path = work.join("formula.png");
    let fg = format!(
        "rgb {:.6} {:.6} {:.6}",
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0
    );
    let dpi_arg = format!("{}", dpi.round().max(72.0) as u32);
    let dvipng_out = Command::new("dvipng")
        .current_dir(work)
        .args([
            "-D",
            &dpi_arg,
            "-T",
            "tight",
            "-bg",
            "Transparent",
            "-fg",
            &fg,
            "--depth",
            "-o",
            "formula.png",
            "formula.dvi",
        ])
        .output()
        .map_err(|e| PlotError::latex_failed(format!("spawn dvipng: {e}")))?;
    if !dvipng_out.status.success() || !png_path.is_file() {
        let stderr = String::from_utf8_lossy(&dvipng_out.stderr);
        let stdout = String::from_utf8_lossy(&dvipng_out.stdout);
        return Err(PlotError::latex_failed(format!(
            "dvipng failed (exit {:?})\n{stdout}\n{stderr}",
            dvipng_out.status.code()
        )));
    }

    let depth = parse_depth(&String::from_utf8_lossy(&dvipng_out.stdout));
    let (rgba, width, height) = decode_png(&png_path)?;
    let ascent = if let Some(d) = depth {
        (height as f64 - d).max(0.0)
    } else {
        height as f64 * 0.8
    };
    Ok(Glyph {
        rgba,
        width,
        height,
        ascent,
    })
}

fn read_sidecar(work: &Path, name: &str) -> String {
    fs::read_to_string(work.join(name)).unwrap_or_default()
}

fn parse_depth(stdout: &str) -> Option<f64> {
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("depth=") {
            if let Ok(v) = rest.trim().parse::<f64>() {
                return Some(v);
            }
        }
        // Some builds: "depth = 12"
        if let Some(idx) = line.find("depth") {
            let rest = &line[idx..];
            if let Some(eq) = rest.find('=') {
                let num = rest[eq + 1..].trim().split_whitespace().next()?;
                if let Ok(v) = num.parse::<f64>() {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn decode_png(path: &PathBuf) -> Result<(Vec<u8>, u32, u32)> {
    let data = fs::read(path).map_err(|e| PlotError::latex_failed(format!("read png: {e}")))?;
    let decoder = png::Decoder::new(Cursor::new(data));
    let mut reader = decoder
        .read_info()
        .map_err(|e| PlotError::latex_failed(format!("png info: {e}")))?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| PlotError::latex_failed(format!("png frame: {e}")))?;
    let width = info.width;
    let height = info.height;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            let rgb = &buf[..info.buffer_size()];
            let mut out = Vec::with_capacity(width as usize * height as usize * 4);
            for chunk in rgb.chunks_exact(3) {
                out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            out
        }
        png::ColorType::GrayscaleAlpha => {
            let ga = &buf[..info.buffer_size()];
            let mut out = Vec::with_capacity(width as usize * height as usize * 4);
            for chunk in ga.chunks_exact(2) {
                out.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
            out
        }
        png::ColorType::Grayscale => {
            let g = &buf[..info.buffer_size()];
            let mut out = Vec::with_capacity(width as usize * height as usize * 4);
            for &v in g {
                out.extend_from_slice(&[v, v, v, 255]);
            }
            out
        }
        other => {
            return Err(PlotError::latex_failed(format!(
                "unsupported PNG color type from dvipng: {other:?}"
            )));
        }
    };
    Ok((rgba, width, height))
}

/// Measure external-LaTeX text (width × height in figure pixels).
pub fn measure_text(text: &str, size_px: f32, color: Color) -> Result<(f64, f64)> {
    let g = render_cached(text, size_px, color)?;
    Ok((g.width as f64, g.height as f64))
}

/// Draw external-LaTeX text at `pos` honoring align / baseline (rotation ignored).
pub fn draw_text(
    renderer: &mut dyn Renderer,
    text: &str,
    pos: Point,
    style: &TextStyle,
) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    // Rotated usetex is uncommon; fall back would need a transformed blit.
    // Keep upright placement for the first cut.
    let g = render_cached(text, style.size_px, style.color)?;
    let descent = (g.height as f64 - g.ascent).max(0.0);
    let ax = match style.align {
        TextAlign::Left => 0.0,
        TextAlign::Center => g.width as f64 * 0.5,
        TextAlign::Right => g.width as f64,
    };
    let ay = match style.baseline {
        TextBaseline::Alphabetic => 0.0,
        TextBaseline::Top => -g.ascent,
        TextBaseline::Middle => (descent - g.ascent) * 0.5,
        TextBaseline::Bottom => descent,
    };
    // Image top-left: baseline at pos → top = pos.y - ascent + ay offset from anchor.
    let top_left = Point::new(pos.x - ax, pos.y - g.ascent - ay);
    renderer.draw_rgba_image(&g.rgba, g.width, g.height, top_left)
}
