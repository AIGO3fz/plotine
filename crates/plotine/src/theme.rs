//! Visual themes and point→pixel conversion helpers.

use plotine_core::Color;

use crate::mpl_policy::font as font_policy;

/// Convert typographic points to device pixels (matplotlib convention: 72 pt = 1 in).
#[inline]
pub fn points_to_px(points: f64, dpi: f64) -> f64 {
    points * (dpi / 72.0).max(0.05)
}

/// `f32` variant of [`points_to_px`].
#[inline]
pub fn points_to_px_f32(points: f32, dpi: f64) -> f32 {
    points_to_px(points as f64, dpi) as f32
}

/// Visual theme controlling colors, font sizes, and geometry of figure chrome.
///
/// Font sizes and stroke widths are in **points** (1/72 inch) and are converted
/// to pixels at render time via [`points_to_px`] using the figure DPI.
///
/// Three built-in themes are provided: [`Theme::light()`] (default),
/// [`Theme::dark()`], and [`Theme::paper()`]. Create a custom theme by
/// starting from one and overriding fields.
///
/// # Example
///
/// ```
/// use plotine::prelude::*;
///
/// let custom = Theme { show_grid: false, ..Theme::dark() };
/// let png = Figure::new()
///     .size(3.0, 2.0)
///     .dpi(72.0)
///     .theme(custom)
///     .axes(|ax| {
///         ax.line([0.0, 1.0], [0.0, 1.0]);
///     })
///     .render_png()
///     .unwrap();
/// assert!(!png.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Figure background fill.
    pub background: Color,
    /// Axes panel face fill.
    pub axes_face: Color,
    /// Axes spine (border) color.
    pub spine: Color,
    /// Tick mark color.
    pub tick: Color,
    /// Grid line color.
    pub grid: Color,
    /// Axis / tick label text color.
    pub label: Color,
    /// Title text color.
    pub title: Color,
    /// Title font size in points.
    pub title_size: f32,
    /// Axis label font size in points.
    pub label_size: f32,
    /// Tick label font size in points.
    pub tick_label_size: f32,
    /// Spine stroke width in points.
    pub spine_width: f64,
    /// Tick stroke width in points.
    pub tick_width: f64,
    /// Tick length in points.
    pub tick_length: f64,
    /// Grid stroke width in points.
    pub grid_width: f64,
    /// Whether grid lines are drawn by default.
    pub show_grid: bool,
}

impl Theme {
    /// Light publication-oriented defaults.
    ///
    /// Font sizes match matplotlib stock rcParams (`font.size=10`,
    /// title=`large`→12, labels/ticks=`medium`→10).
    pub fn light() -> Self {
        Self {
            background: Color::BACKGROUND,
            axes_face: Color::AXES_FACE,
            spine: Color::SPINE,
            tick: Color::TICK,
            grid: Color::GRID,
            label: Color::LABEL,
            title: Color::TITLE,
            title_size: font_policy::TITLE_PT,
            label_size: font_policy::LABEL_PT,
            tick_label_size: font_policy::TICK_PT,
            spine_width: 0.95,
            tick_width: 0.9,
            tick_length: 5.0,
            grid_width: 0.75,
            show_grid: true,
        }
    }

    /// Dark theme for slides / OLED-friendly reports.
    pub fn dark() -> Self {
        Self {
            background: Color::rgb(0x1a, 0x1d, 0x21),
            axes_face: Color::rgb(0x21, 0x25, 0x29),
            spine: Color::rgb(0xad, 0xb5, 0xbd),
            tick: Color::rgb(0xad, 0xb5, 0xbd),
            grid: Color::rgb(0x49, 0x50, 0x57),
            label: Color::rgb(0xde, 0xe2, 0xe6),
            title: Color::rgb(0xf8, 0xf9, 0xfa),
            title_size: font_policy::TITLE_PT,
            label_size: font_policy::LABEL_PT,
            tick_label_size: font_policy::TICK_PT,
            spine_width: 0.95,
            tick_width: 0.9,
            tick_length: 5.0,
            grid_width: 0.75,
            show_grid: true,
        }
    }

    /// Paper theme: warmer face, slightly heavier spines for print.
    pub fn paper() -> Self {
        Self {
            background: Color::rgb(0xff, 0xff, 0xff),
            axes_face: Color::rgb(0xff, 0xfc, 0xf5),
            spine: Color::rgb(0x33, 0x37, 0x3d),
            tick: Color::rgb(0x33, 0x37, 0x3d),
            grid: Color::rgb(0xe9, 0xec, 0xef),
            label: Color::rgb(0x21, 0x25, 0x29),
            title: Color::rgb(0x12, 0x12, 0x12),
            title_size: font_policy::TITLE_PT,
            label_size: font_policy::LABEL_PT,
            tick_label_size: font_policy::TICK_PT,
            spine_width: 1.1,
            tick_width: 1.0,
            tick_length: 5.5,
            grid_width: 0.7,
            show_grid: true,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
