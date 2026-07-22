//! Custom tick label formatting (matplotlib `FuncFormatter` / `StrMethodFormatter`).

use std::fmt;
use std::sync::Arc;

/// Formats a tick value as a label string.
///
/// Use [`TickFormatter::new`] for arbitrary closures (matplotlib
/// `FuncFormatter`), or the helpers [`fixed`](Self::fixed) /
/// [`percent`](Self::percent) / [`scientific`](Self::scientific).
///
/// ```
/// use plotine::prelude::*;
///
/// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
///     ax.line([0.0, 1.0, 2.0], [0.0, 0.5, 1.0]);
///     ax.y_tick_formatter(TickFormatter::percent(0));
///     ax.title("pct");
/// }).render_png().unwrap();
/// assert!(!png.is_empty());
/// ```
#[derive(Clone)]
pub struct TickFormatter {
    format: Arc<dyn Fn(f64) -> String + Send + Sync>,
}

impl fmt::Debug for TickFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TickFormatter")
    }
}

impl TickFormatter {
    /// Wrap a custom formatter closure.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(f64) -> String + Send + Sync + 'static,
    {
        Self {
            format: Arc::new(f),
        }
    }

    /// Fixed decimal places (`"{x:.{n}f}"` style).
    pub fn fixed(decimals: usize) -> Self {
        let n = decimals.min(12);
        Self::new(move |v| {
            if !v.is_finite() {
                return String::new();
            }
            let s = format!("{v:.n$}");
            s.replace('-', "\u{2212}")
        })
    }

    /// Percentage of the raw value (`v * 100` with `decimals` places + `%`).
    pub fn percent(decimals: usize) -> Self {
        let n = decimals.min(12);
        Self::new(move |v| {
            if !v.is_finite() {
                return String::new();
            }
            let s = format!("{:.n$}%", v * 100.0);
            s.replace('-', "\u{2212}")
        })
    }

    /// Scientific notation with `decimals` digits after the point.
    pub fn scientific(decimals: usize) -> Self {
        let n = decimals.min(12);
        Self::new(move |v| {
            if !v.is_finite() {
                return String::new();
            }
            let s = format!("{v:.n$e}");
            s.replace('-', "\u{2212}")
        })
    }

    /// Format one tick value.
    pub fn format(&self, value: f64) -> String {
        (self.format)(value)
    }
}
