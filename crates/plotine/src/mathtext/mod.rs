//! Matplotlib-style mathtext layout (TeX-like `$...$`, no external LaTeX).
//!
//! Supports a practical subset: Greek/symbols, `^`/`_` scripts, `\frac{a}{b}`,
//! `\sqrt{…}` / `\sqrt[n]{…}`, `\begin{matrix|pmatrix|bmatrix}`, roman
//! function names (`\sin`, `\cos`, …), and `\displaystyle` / `\limits` for
//! large-operator placement. Mixed plain + math strings are fine:
//! `Amplitude $e^{-t}$`.

use plotine_core::{Point, Result};
use plotine_render::{Renderer, TextAlign, TextBaseline, TextStyle};

mod layout;
mod parse;

pub use layout::{draw_mathtext, measure_mathtext};
pub use parse::needs_mathtext;

/// Draw text, routing through mathtext when `$...$` is present.
///
/// With `feature = "latex"` and [`Figure::usetex`](crate::figure::Figure::usetex),
/// math strings go through the system LaTeX pipeline instead.
pub fn draw_text(
    renderer: &mut dyn Renderer,
    text: &str,
    pos: Point,
    style: &TextStyle,
) -> Result<()> {
    #[cfg(feature = "latex")]
    if crate::latex::usetex_active() && needs_mathtext(text) {
        return crate::latex::draw_text(renderer, text, pos, style);
    }
    if needs_mathtext(text) {
        draw_mathtext(renderer, text, pos, style)
    } else {
        renderer.draw_text(text, pos, style)
    }
}

/// Measure text, routing through mathtext when `$...$` is present.
pub fn measure_text(renderer: &dyn Renderer, text: &str, size_px: f32) -> Result<(f64, f64)> {
    #[cfg(feature = "latex")]
    if crate::latex::usetex_active() && needs_mathtext(text) {
        let _ = renderer; // layout still needs a renderer for the mathtext path
        return crate::latex::measure_text(text, size_px, plotine_core::Color::BLACK);
    }
    if needs_mathtext(text) {
        measure_mathtext(renderer, text, size_px)
    } else {
        renderer.measure_text(text, size_px)
    }
}

/// Convenience: left-aligned alphabetic draw (titles/labels use full [`TextStyle`]).
pub fn draw_simple(
    renderer: &mut dyn Renderer,
    text: &str,
    pos: Point,
    color: plotine_core::Color,
    size_px: f32,
) -> Result<()> {
    let style = TextStyle::new(color, size_px)
        .align(TextAlign::Left)
        .baseline(TextBaseline::Alphabetic);
    draw_text(renderer, text, pos, &style)
}
