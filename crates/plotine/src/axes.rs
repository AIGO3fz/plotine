use plotine_core::{
    Cmap, Color, Colormap, DatetimeLocator, Norm, PlotError, Result, ScaleKind, ScaleType, Tick,
    TickLocator,
};
use plotine_render::{TextAlign, TextBaseline};

use crate::artist::{
    AnnotatePlot, AreaPlot, AxHLinePlot, AxHSpanPlot, AxLinePlot, AxVLinePlot, AxVSpanPlot,
    BarHPlot, BarPlot, BarbsPlot, BoxPlot, BrokenBarHPlot, CirclePlot, ColorbarSpec, ContourPlot,
    ContourfPlot, EllipsePlot, ErrorBarPlot, EventPlot, FillBetweenPlot, FillBetweenXPlot,
    HLinesPlot, HeatmapPlot, HexbinPlot, Hist2dPlot, HistPlot, LinePlot, PcolorMeshPlot, PiePlot,
    PlotElement, PolarFramePlot, PolygonPlot, QuiverPlot, RectanglePlot, ScatterPlot, SpyPlot,
    StackPlot, StairsPlot, StemPlot, StepPlot, StreamPlot, TablePlot, TextPlot, TricontourPlot,
    TricontourfPlot, TripcolorPlot, VLinesPlot, ViolinPlot,
};
use crate::geo::{project_lonlat, GeoProjection};
use crate::legend::Legend;
use crate::mpl_policy::{
    geo as geo_policy, margin as margin_policy, pie as pie_policy, ticks as ticks_policy,
};
use crate::recipes::TableLoc;
use crate::secondary::{SecondaryAxis, SecondaryTransform};
use crate::style::{LineStyle, MarkerStyle};
use crate::tick_format::TickFormatter;

/// Which axes draw major grid lines (matplotlib `ax.grid(..., axis=)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridAxis {
    /// Vertical and horizontal major grid lines.
    #[default]
    Both,
    /// Vertical lines only (`axis="x"`).
    X,
    /// Horizontal lines only (`axis="y"`).
    Y,
}

/// Nested inset axes (matplotlib `inset_axes`) in parent axes fractions.
#[derive(Debug, Clone)]
pub(crate) struct InsetAxes {
    /// `[x0, y0, width, height]` in parent axes fractions (origin bottom-left, y up).
    pub rect: [f64; 4],
    pub axes: Box<Axes>,
}
use crate::recipes::{
    boxplot_stats, hist2d_bins, infer_bar_width, infer_barh_height, polar_rings,
    polar_to_cartesian, stackplot_ymax, violin_geoms, StepMode,
};
use crate::series::{IntoSeries, Series};

/// A single axes panel containing one or more plot artists.
///
/// `Axes` is the main surface for adding data visualizations. You receive an
/// `&mut Axes` inside closures passed to [`Figure::axes()`](crate::Figure::axes) or
/// [`SubplotGrid::at()`](crate::SubplotGrid::at). Call methods like [`line()`](Self::line),
/// [`scatter()`](Self::scatter), [`bar()`](Self::bar) etc. to add artists,
/// then configure axis limits, scales, labels, and legends.
///
/// # Example
///
/// ```
/// use plotine::prelude::*;
///
/// let png = Figure::new()
///     .size(4.0, 3.0)
///     .dpi(72.0)
///     .axes(|ax| {
///         ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5])
///             .color(Color::CRIMSON)
///             .width(2.0)
///             .label("trend");
///         ax.title("My Plot").x_label("x").y_label("y");
///         ax.legend(Legend::TopRight);
///     })
///     .render_png()
///     .unwrap();
/// assert!(!png.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Axes {
    pub(crate) title: Option<String>,
    pub(crate) x_label: Option<String>,
    pub(crate) y_label: Option<String>,
    pub(crate) x_min: f64,
    pub(crate) x_max: f64,
    pub(crate) y_min: f64,
    pub(crate) y_max: f64,
    pub(crate) x_lim_manual: bool,
    pub(crate) y_lim_manual: bool,
    /// True after the first auto `expand_range_x` (not tied to `elements.len()`).
    pub(crate) x_lim_seeded: bool,
    /// True after the first auto `expand_range_y`.
    pub(crate) y_lim_seeded: bool,
    /// Unpadded data extents used to recompute margins (matplotlib-style).
    pub(crate) x_data_min: f64,
    pub(crate) x_data_max: f64,
    pub(crate) y_data_min: f64,
    pub(crate) y_data_max: f64,
    pub(crate) y_pad_include_zero: bool,
    /// Sticky x=0 edge (matplotlib barh / hist baseline): do not pad past zero.
    pub(crate) x_pad_include_zero: bool,
    pub(crate) show_grid: Option<bool>,
    pub(crate) grid_axis: GridAxis,
    /// Grid stroke dash (matplotlib `grid(..., linestyle=...)`). Default solid.
    pub(crate) grid_linestyle: crate::style::LineStyle,
    pub(crate) legend: Option<Legend>,
    pub(crate) legend_ncol: usize,
    /// Optional override for title fontsize in points (theme default otherwise).
    pub(crate) title_fontsize: Option<f32>,
    /// Optional override for x-axis label fontsize in points.
    pub(crate) x_label_fontsize: Option<f32>,
    /// Optional override for y-axis label fontsize in points.
    pub(crate) y_label_fontsize: Option<f32>,
    pub(crate) x_scale_type: ScaleType,
    pub(crate) y_scale_type: ScaleType,
    /// When true, x values are Unix timestamps (UTC seconds) and tick labels are dates.
    pub(crate) x_datetime: bool,
    /// When true, y values are Unix timestamps (UTC seconds) and tick labels are dates.
    pub(crate) y_datetime: bool,
    /// Optional explicit major tick values for the x-axis (e.g. categorical 1..n).
    pub(crate) x_ticks: Option<Vec<f64>>,
    /// Optional explicit major tick values for the y-axis.
    pub(crate) y_ticks: Option<Vec<f64>>,
    /// Category labels for x ticks at positions `0..n` (overrides numeric formatting).
    pub(crate) x_categories: Option<Vec<String>>,
    /// Category labels for y ticks at positions `0..n`.
    pub(crate) y_categories: Option<Vec<String>>,
    pub(crate) elements: Vec<PlotElement>,
    pub(crate) next_color_index: usize,
    /// Nested twin axes sharing this panel's x-axis (right-hand y).
    pub(crate) twin_y: Option<Box<Axes>>,
    /// Nested twin axes sharing this panel's y-axis (top x).
    pub(crate) twin_x: Option<Box<Axes>>,
    /// Nested inset axes (independent layout rects inside this panel).
    pub(crate) insets: Vec<InsetAxes>,
    /// Top secondary x-axis (transformed ticks; no artists).
    pub(crate) secondary_x: Option<SecondaryAxis>,
    /// Right secondary y-axis (transformed ticks; no artists).
    pub(crate) secondary_y: Option<SecondaryAxis>,
    /// True when this axes is a twin (not a top-level panel host).
    pub(crate) is_twin: bool,
    /// True when this axes is an inset (colorbar disallowed; nesting allowed).
    pub(crate) is_inset: bool,
    /// Polar projection chrome (circular spine, θ/r labels; no cartesian ticks).
    pub(crate) polar: bool,
    /// Geographic map projection (lon/lat → plane); mutually exclusive with polar.
    pub(crate) geo: Option<GeoProjection>,
    /// When false, skip cartesian grid/spines/ticks (pie charts).
    pub(crate) frame_on: bool,
    /// Invert y mapping (matplotlib `imshow(origin='upper')` default index box).
    pub(crate) y_inverted: bool,
    /// Draw major x tick labels above the axes (matplotlib `xaxis.tick_top`, spy).
    pub(crate) x_ticks_top: bool,
    /// Equal aspect ratio (1:1 data→pixel mapping).
    pub(crate) aspect_equal: bool,
    /// Optional colorbar label (matplotlib `cbar.set_label`).
    pub(crate) colorbar_label: Option<String>,
    /// Which box edges (spines) are drawn.
    pub(crate) spines: crate::style::Spines,
    /// Draw minor ticks on the x-axis.
    pub(crate) x_minor_ticks: bool,
    /// Draw minor ticks on the y-axis.
    pub(crate) y_minor_ticks: bool,
    /// Optional custom x-axis tick label formatter.
    pub(crate) x_tick_formatter: Option<TickFormatter>,
    /// Optional custom y-axis tick label formatter.
    pub(crate) y_tick_formatter: Option<TickFormatter>,
}

impl Default for Axes {
    fn default() -> Self {
        Self {
            title: None,
            x_label: None,
            y_label: None,
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
            x_lim_manual: false,
            y_lim_manual: false,
            x_lim_seeded: false,
            y_lim_seeded: false,
            x_data_min: 0.0,
            x_data_max: 1.0,
            y_data_min: 0.0,
            y_data_max: 1.0,
            y_pad_include_zero: false,
            x_pad_include_zero: false,
            show_grid: None,
            grid_axis: GridAxis::Both,
            grid_linestyle: crate::style::LineStyle::Solid,
            legend: None,
            legend_ncol: 1,
            title_fontsize: None,
            x_label_fontsize: None,
            y_label_fontsize: None,
            x_scale_type: ScaleType::Linear,
            y_scale_type: ScaleType::Linear,
            x_datetime: false,
            y_datetime: false,
            x_ticks: None,
            y_ticks: None,
            x_categories: None,
            y_categories: None,
            elements: Vec::new(),
            next_color_index: 0,
            twin_y: None,
            twin_x: None,
            insets: Vec::new(),
            secondary_x: None,
            secondary_y: None,
            is_twin: false,
            is_inset: false,
            polar: false,
            geo: None,
            frame_on: true,
            y_inverted: false,
            x_ticks_top: false,
            aspect_equal: false,
            colorbar_label: None,
            spines: crate::style::Spines::all(),
            x_minor_ticks: false,
            y_minor_ticks: false,
            x_tick_formatter: None,
            y_tick_formatter: None,
        }
    }
}

impl Axes {
    /// Create an empty axes panel with default limits `[0, 1] × [0, 1]`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the axes title (drawn above the plot area).
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = Some(title.into());
        self
    }

    /// Override title fontsize in points (matplotlib `set_title(..., fontsize=)`).
    pub fn title_fontsize(&mut self, pt: f32) -> &mut Self {
        self.title_fontsize = Some(pt.max(1.0));
        self
    }

    /// Set the x-axis label.
    pub fn x_label(&mut self, label: impl Into<String>) -> &mut Self {
        self.x_label = Some(label.into());
        self
    }

    /// Override x-axis label fontsize in points.
    pub fn x_label_fontsize(&mut self, pt: f32) -> &mut Self {
        self.x_label_fontsize = Some(pt.max(1.0));
        self
    }

    /// Set the y-axis label (drawn rotated).
    pub fn y_label(&mut self, label: impl Into<String>) -> &mut Self {
        self.y_label = Some(label.into());
        self
    }

    /// Override y-axis label fontsize in points.
    pub fn y_label_fontsize(&mut self, pt: f32) -> &mut Self {
        self.y_label_fontsize = Some(pt.max(1.0));
        self
    }

    /// Manually set the x-axis data range (`min` must be `< max`).
    pub fn x_range(&mut self, min: f64, max: f64) -> &mut Self {
        self.x_min = min;
        self.x_max = max;
        self.x_data_min = min;
        self.x_data_max = max;
        self.x_lim_manual = true;
        self.x_lim_seeded = true;
        self
    }

    /// Manually set the y-axis data range (`min` must be `< max`).
    pub fn y_range(&mut self, min: f64, max: f64) -> &mut Self {
        self.y_min = min;
        self.y_max = max;
        self.y_data_min = min;
        self.y_data_max = max;
        self.y_lim_manual = true;
        self.y_lim_seeded = true;
        self
    }

    /// Force equal scaling on both axes (matplotlib `ax.set_aspect('equal')`).
    ///
    /// When enabled, the axes box is adjusted so one unit in x equals one
    /// unit in y on screen.
    pub fn aspect_equal(&mut self, equal: bool) -> &mut Self {
        self.aspect_equal = equal;
        self
    }

    /// Set the colorbar label (drawn rotated 90° beside the colorbar strip).
    pub fn colorbar_label(&mut self, label: impl Into<String>) -> &mut Self {
        self.colorbar_label = Some(label.into());
        self
    }

    /// Set explicit major tick positions on the x-axis (replaces auto ticks).
    ///
    /// Clears any prior [`x_categories`](Self::x_categories).
    pub fn x_ticks<I>(&mut self, ticks: I) -> &mut Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.x_categories = None;
        self.x_ticks = Some(ticks.into_iter().collect());
        self
    }

    /// Set explicit major tick positions on the y-axis (replaces auto ticks).
    ///
    /// Clears any prior [`y_categories`](Self::y_categories).
    pub fn y_ticks<I>(&mut self, ticks: I) -> &mut Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.y_categories = None;
        self.y_ticks = Some(ticks.into_iter().collect());
        self
    }

    /// Custom x-axis tick labels (matplotlib FuncFormatter / StrMethodFormatter).
    ///
    /// Does not change tick *positions*; only the drawn label strings. Ignored
    /// when [x_categories](Self::x_categories) is set.
    pub fn x_tick_formatter(&mut self, fmt: TickFormatter) -> &mut Self {
        self.x_tick_formatter = Some(fmt);
        self
    }

    /// Custom y-axis tick labels (matplotlib FuncFormatter / StrMethodFormatter).
    ///
    /// Does not change tick *positions*; only the drawn label strings. Ignored
    /// when [y_categories](Self::y_categories) is set.
    pub fn y_tick_formatter(&mut self, fmt: TickFormatter) -> &mut Self {
        self.y_tick_formatter = Some(fmt);
        self
    }

    /// Label the x-axis with categories at positions `0..n` (matplotlib categorical).
    ///
    /// Feed bar/line data with those indices (see [`category_indices`]). Forces a
    /// linear scale and clears datetime mode on x. View limits are left to artists
    /// (e.g. `bar`) so margins match mpl sticky-edge behaviour.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let cats = ["A", "B", "C"];
    /// let x = category_indices(cats.len());
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
    ///     ax.x_categories(cats);
    ///     ax.bar(&x, [3.0, 5.0, 2.0]).color(Color::STEEL_BLUE);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn x_categories<I, S>(&mut self, labels: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        let n = labels.len();
        if n == 0 {
            self.x_categories = None;
            return self;
        }
        self.x_categories = Some(labels);
        self.x_ticks = Some(category_indices(n));
        self.x_datetime = false;
        self.x_scale_type = ScaleType::Linear;
        self
    }

    /// Label the y-axis with categories at positions `0..n` (matplotlib categorical).
    pub fn y_categories<I, S>(&mut self, labels: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        let n = labels.len();
        if n == 0 {
            self.y_categories = None;
            return self;
        }
        self.y_categories = Some(labels);
        self.y_ticks = Some(category_indices(n));
        self.y_datetime = false;
        self.y_scale_type = ScaleType::Linear;
        self
    }

    /// Configure which box edges (spines) are drawn.
    pub fn spines(&mut self, spines: crate::style::Spines) -> &mut Self {
        self.spines = spines;
        self
    }

    /// Hide top and right spines (seaborn-style `despine`).
    pub fn despine(&mut self) -> &mut Self {
        self.spines = crate::style::Spines::bottom_left();
        self
    }

    /// Enable/disable minor ticks on both axes.
    pub fn minor_ticks(&mut self, on: bool) -> &mut Self {
        self.x_minor_ticks = on;
        self.y_minor_ticks = on;
        self
    }

    /// Enable/disable minor ticks on the x-axis.
    pub fn x_minor_ticks(&mut self, on: bool) -> &mut Self {
        self.x_minor_ticks = on;
        self
    }

    /// Enable/disable minor ticks on the y-axis.
    pub fn y_minor_ticks(&mut self, on: bool) -> &mut Self {
        self.y_minor_ticks = on;
        self
    }

    /// Override whether grid lines are shown (defaults to the theme).
    pub fn grid(&mut self, show: bool) -> &mut Self {
        self.show_grid = Some(show);
        self
    }

    /// Restrict grid lines to one axis (matplotlib `grid(..., axis="x"|"y")`).
    ///
    /// Does not force the grid on by itself; combine with [`grid`](Self::grid)
    /// or rely on the theme's `show_grid`.
    pub fn grid_axis(&mut self, axis: GridAxis) -> &mut Self {
        self.grid_axis = axis;
        self
    }

    /// Grid stroke dash pattern (matplotlib `grid(..., linestyle='--')`).
    ///
    /// Does not force the grid on; combine with [`grid`](Self::grid).
    pub fn grid_linestyle(&mut self, style: crate::style::LineStyle) -> &mut Self {
        self.grid_linestyle = style;
        self
    }

    /// Overlay a second y-axis on the right that shares this axes' x domain.
    ///
    /// Corresponds to matplotlib `twinx()` (shared x, independent y). Only one
    /// twin is kept; calling again replaces it. Nested `twin_y` inside the
    /// closure is ignored. Twin artists enter the host legend when
    /// [`legend`](Self::legend) is set on the host. Cannot be combined with a
    /// colorbar on the same panel.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let x = [0.0, 1.0, 2.0, 3.0];
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
    ///     ax.line(&x, [1.0, 2.0, 1.5, 3.0]).label("left").color(Color::STEEL_BLUE);
    ///     ax.y_label("left");
    ///     ax.twin_y(|ax2| {
    ///         ax2.line(&x, [10.0, 40.0, 20.0, 55.0])
    ///             .label("right")
    ///             .color(Color::CRIMSON);
    ///         ax2.y_label("right");
    ///     });
    ///     ax.legend(Legend::TopLeft);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn twin_y<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Axes),
    {
        if self.is_twin {
            return self;
        }
        let mut twin = Axes::new();
        twin.is_twin = true;
        twin.next_color_index = self.next_color_index;
        twin.x_scale_type = self.x_scale_type;
        twin.x_datetime = self.x_datetime;
        twin.x_min = self.x_min;
        twin.x_max = self.x_max;
        twin.x_data_min = self.x_data_min;
        twin.x_data_max = self.x_data_max;
        twin.x_lim_manual = self.x_lim_manual;
        twin.x_lim_seeded = self.x_lim_seeded;

        f(&mut twin);
        twin.finalize_artist_limits();

        self.next_color_index = twin.next_color_index;
        self.absorb_twin_x(&twin);
        // Host owns chrome; twin only contributes y scale/ticks/label + artists.
        twin.title = None;
        twin.x_label = None;
        twin.legend = None;
        twin.show_grid = None;
        twin.grid_axis = GridAxis::Both;
        twin.twin_y = None;
        twin.twin_x = None;
        twin.insets.clear();
        twin.x_ticks = None;
        twin.x_categories = None;
        self.twin_y = Some(Box::new(twin));
        self
    }

    /// Overlay a second x-axis on the top that shares this axes' y domain.
    ///
    /// Corresponds to matplotlib `twiny()` (shared y, independent x). Only one
    /// twin is kept; calling again replaces it. Nested `twin_x` inside the
    /// closure is ignored. Twin artists enter the host legend.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let y = [0.0, 1.0, 2.0, 3.0];
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
    ///     ax.line([1.0, 2.0, 3.0, 4.0], &y).label("bottom").color(Color::STEEL_BLUE);
    ///     ax.x_label("bottom x");
    ///     ax.twin_x(|ax2| {
    ///         ax2.line([10.0, 20.0, 30.0, 40.0], &y)
    ///             .label("top")
    ///             .color(Color::CRIMSON);
    ///         ax2.x_label("top x");
    ///     });
    ///     ax.legend(Legend::TopLeft);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn twin_x<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Axes),
    {
        if self.is_twin {
            return self;
        }
        let mut twin = Axes::new();
        twin.is_twin = true;
        twin.next_color_index = self.next_color_index;
        twin.y_scale_type = self.y_scale_type;
        twin.y_datetime = self.y_datetime;
        twin.y_min = self.y_min;
        twin.y_max = self.y_max;
        twin.y_data_min = self.y_data_min;
        twin.y_data_max = self.y_data_max;
        twin.y_pad_include_zero = self.y_pad_include_zero;
        twin.y_lim_manual = self.y_lim_manual;
        twin.y_lim_seeded = self.y_lim_seeded;

        f(&mut twin);
        twin.finalize_artist_limits();

        self.next_color_index = twin.next_color_index;
        self.absorb_twin_y(&twin);
        twin.title = None;
        twin.y_label = None;
        twin.legend = None;
        twin.show_grid = None;
        twin.grid_axis = GridAxis::Both;
        twin.twin_y = None;
        twin.twin_x = None;
        twin.insets.clear();
        twin.y_ticks = None;
        twin.y_categories = None;
        self.twin_x = Some(Box::new(twin));
        self
    }

    /// Add an inset axes in parent **axes fractions** (matplotlib `inset_axes`).
    ///
    /// `rect` is `[x0, y0, width, height]` with origin at the parent's bottom-left
    /// and `y` increasing upward (same convention as matplotlib
    /// `transform=ax.transAxes`). Multiple insets and one level of nesting are
    /// allowed. Calls on twins are ignored. Colorbar on an inset is unsupported.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    /// let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    /// let png = Figure::new().size(5.0, 3.5).dpi(72.0).axes(|ax| {
    ///     ax.line(&x, &y).color(Color::STEEL_BLUE);
    ///     ax.inset_axes([0.55, 0.55, 0.4, 0.4], |inset| {
    ///         inset.line(&x[..15], &y[..15]).color(Color::CRIMSON).width(1.5);
    ///         inset.title("zoom");
    ///     });
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn inset_axes<F>(&mut self, rect: [f64; 4], f: F) -> &mut Self
    where
        F: FnOnce(&mut Axes),
    {
        if self.is_twin {
            return self;
        }
        let [x0, y0, w, h] = rect;
        if !(x0.is_finite()
            && y0.is_finite()
            && w.is_finite()
            && h.is_finite()
            && w > 1e-6
            && h > 1e-6)
        {
            return self;
        }
        let mut inset = Axes::new();
        inset.is_inset = true;
        inset.next_color_index = self.next_color_index;
        f(&mut inset);
        inset.finalize_artist_limits();
        self.next_color_index = inset.next_color_index;
        self.insets.push(InsetAxes {
            rect: [x0, y0, w, h],
            axes: Box::new(inset),
        });
        self
    }

    /// Top secondary x-axis with a function transform (matplotlib `secondary_xaxis`).
    ///
    /// Unlike [`twin_x`](Self::twin_x), this does **not** host artists — it only
    /// shows ticks/labels for `forward(primary_x)`. `forward` and `inverse` must
    /// be approximate inverses. Cannot be combined with `twin_x` on the same panel.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// use std::f64::consts::PI;
    /// let th = [0.0, PI / 2.0, PI];
    /// let y = [0.0, 1.0, 0.0];
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
    ///     ax.line(&th, &y).color(Color::STEEL_BLUE);
    ///     ax.x_label("radians");
    ///     ax.secondary_x(f64::to_degrees, f64::to_radians, |sec| {
    ///         sec.label("degrees");
    ///     });
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn secondary_x<F>(
        &mut self,
        forward: fn(f64) -> f64,
        inverse: fn(f64) -> f64,
        configure: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut SecondaryAxis),
    {
        if self.is_twin || self.is_inset {
            return self;
        }
        let mut sec = SecondaryAxis::new(SecondaryTransform::Fn { forward, inverse });
        configure(&mut sec);
        self.secondary_x = Some(sec);
        self
    }

    /// Right secondary y-axis with a function transform (matplotlib `secondary_yaxis`).
    ///
    /// Cannot be combined with `twin_y` on the same panel.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
    ///     ax.line([0.0, 50.0, 100.0], [0.0, 50.0, 100.0]).color(Color::CRIMSON);
    ///     ax.y_label("°C");
    ///     ax.secondary_y_linear(1.8, 32.0, |sec| {
    ///         sec.label("°F");
    ///     });
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn secondary_y<F>(
        &mut self,
        forward: fn(f64) -> f64,
        inverse: fn(f64) -> f64,
        configure: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut SecondaryAxis),
    {
        if self.is_twin || self.is_inset {
            return self;
        }
        let mut sec = SecondaryAxis::new(SecondaryTransform::Fn { forward, inverse });
        configure(&mut sec);
        self.secondary_y = Some(sec);
        self
    }

    /// Top secondary x-axis with an affine map `secondary = scale * x + offset`.
    pub fn secondary_x_linear<F>(&mut self, scale: f64, offset: f64, configure: F) -> &mut Self
    where
        F: FnOnce(&mut SecondaryAxis),
    {
        if self.is_twin || self.is_inset {
            return self;
        }
        let mut sec = SecondaryAxis::new(SecondaryTransform::Linear { scale, offset });
        configure(&mut sec);
        self.secondary_x = Some(sec);
        self
    }

    /// Right secondary y-axis with an affine map `secondary = scale * y + offset`.
    pub fn secondary_y_linear<F>(&mut self, scale: f64, offset: f64, configure: F) -> &mut Self
    where
        F: FnOnce(&mut SecondaryAxis),
    {
        if self.is_twin || self.is_inset {
            return self;
        }
        let mut sec = SecondaryAxis::new(SecondaryTransform::Linear { scale, offset });
        configure(&mut sec);
        self.secondary_y = Some(sec);
        self
    }

    /// Show a legend for labeled artists.
    pub fn legend(&mut self, loc: Legend) -> &mut Self {
        self.legend = Some(loc);
        self
    }

    /// Set the number of columns in the legend (default 1).
    pub fn legend_ncol(&mut self, ncol: usize) -> &mut Self {
        self.legend_ncol = ncol.max(1);
        self
    }

    /// Set the x-axis scale (`Linear` / `Log` / `Symlog`).
    ///
    /// Prefer setting the scale before adding artists so auto-limits use
    /// multiplicative padding on log axes.
    pub fn x_scale(&mut self, scale: ScaleType) -> &mut Self {
        self.x_scale_type = scale;
        self
    }

    /// Set the y-axis scale (`Linear` / `Log` / `Symlog`).
    pub fn y_scale(&mut self, scale: ScaleType) -> &mut Self {
        self.y_scale_type = scale;
        self
    }

    /// Treat x values as Unix timestamps (UTC seconds) for tick labeling.
    pub fn x_datetime(&mut self, enable: bool) -> &mut Self {
        self.x_datetime = enable;
        self
    }

    /// Treat y values as Unix timestamps (UTC seconds) for tick labeling.
    pub fn y_datetime(&mut self, enable: bool) -> &mut Self {
        self.y_datetime = enable;
        self
    }

    /// Enable a geographic map projection (cartopy-thin).
    ///
    /// After this call, [`line`](Self::line) / [`scatter`](Self::scatter) treat
    /// inputs as longitude/latitude in degrees and store projected coordinates.
    /// Sets a global default extent and `aspect_equal(true)`. Mutually exclusive
    /// with polar axes.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(4.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.projection(GeoProjection::PlateCarree);
    ///     ax.coastline().color(Color::rgb(0x59, 0x59, 0x59));
    ///     ax.scatter([0.0, 116.4], [51.5, 39.9]).size(4.0);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn projection(&mut self, proj: GeoProjection) -> &mut Self {
        self.polar = false;
        self.geo = Some(proj);
        let (x0, x1, y0, y1) = proj.default_extent();
        self.x_range(x0, x1).y_range(y0, y1);
        self.aspect_equal(true);
        let (xl, yl) = proj.default_labels();
        if self.x_label.is_none() {
            self.x_label = Some(xl.into());
        }
        if self.y_label.is_none() {
            self.y_label = Some(yl.into());
        }
        self
    }

    /// Draw embedded Natural Earth 110m coastlines in the current geo projection.
    ///
    /// If no projection is set yet, enables [`GeoProjection::PlateCarree`].
    /// Returns the coastline line artist for styling. Segment breaks use NaN.
    pub fn coastline(&mut self) -> &mut LinePlot {
        if self.geo.is_none() {
            self.projection(GeoProjection::PlateCarree);
        }
        let (lon, lat) = crate::geo::coastline_lonlat();
        self.line(lon, lat)
            .width(geo_policy::COASTLINE_WIDTH_PT)
            .color(Color::rgb(0x40, 0x40, 0x40))
    }

    /// Draw GeoJSON geometries (lon/lat degrees) in the current projection.
    ///
    /// Supports `Point` / `LineString` / `Polygon` / `MultiLineString` /
    /// `MultiPolygon` inside a FeatureCollection or bare geometry. Holes and
    /// CRS are ignored (PlateCarree / Mercator only). Returns the number of
    /// artists added.
    pub fn geojson(&mut self, data: impl AsRef<[u8]>) -> plotine_core::Result<usize> {
        if self.geo.is_none() {
            self.projection(GeoProjection::PlateCarree);
        }
        let geoms = crate::geo::parse_geojson(data.as_ref())?;
        let mut n = 0usize;
        for g in geoms {
            match g {
                crate::geo::GeoGeom::Point(lon, lat) => {
                    self.scatter([lon], [lat]).size(4.0);
                    n += 1;
                }
                crate::geo::GeoGeom::LineString(pts) => {
                    if pts.len() < 2 {
                        continue;
                    }
                    let (x, y): (Vec<_>, Vec<_>) = pts.into_iter().unzip();
                    self.line(&x, &y).width(1.2).color(Color::STEEL_BLUE);
                    n += 1;
                }
                crate::geo::GeoGeom::Polygon(pts) => {
                    if pts.len() < 3 {
                        continue;
                    }
                    let (x, y): (Vec<_>, Vec<_>) = pts.into_iter().unzip();
                    self.polygon(&x, &y).alpha(0.35).color(Color::STEEL_BLUE);
                    n += 1;
                }
            }
        }
        Ok(n)
    }

    /// Load GeoJSON from a file path and draw it ([`Self::geojson`]).
    pub fn geojson_path(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> plotine_core::Result<usize> {
        let bytes =
            std::fs::read(path.as_ref()).map_err(|e| plotine_core::PlotError::io(e.to_string()))?;
        self.geojson(bytes)
    }

    /// Mutable access to the `index`-th line artist (0-based among `line` calls).
    ///
    /// Used with [`LinePlot::set_y`](crate::artist::LinePlot::set_y) for offline animation.
    pub fn line_at_mut(&mut self, index: usize) -> Option<&mut LinePlot> {
        self.elements
            .iter_mut()
            .filter_map(|e| match e {
                PlotElement::Line(p) => Some(p),
                _ => None,
            })
            .nth(index)
    }

    /// Mutable access to the `index`-th scatter artist (0-based among `scatter` calls).
    pub fn scatter_at_mut(&mut self, index: usize) -> Option<&mut ScatterPlot> {
        self.elements
            .iter_mut()
            .filter_map(|e| match e {
                PlotElement::Scatter(p) => Some(p),
                _ => None,
            })
            .nth(index)
    }

    /// Plot a line through `(x[i], y[i])`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]).color(Color::CRIMSON);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn line<X, Y>(&mut self, x: X, y: Y) -> &mut LinePlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let (x, y) = self.prepare_xy(x.into_series(), y.into_series());
        self.expand_limits_xy(&x, &y, false);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Line(LinePlot {
            x,
            y,
            color: None,
            width: 1.75,
            linestyle: LineStyle::Solid,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Line(line)) => line,
            _ => unreachable!(),
        }
    }

    /// Plot markers at `(x[i], y[i])`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.scatter([0.0, 1.0, 2.0], [0.2, 0.8, 0.4]).size(4.0);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn scatter<X, Y>(&mut self, x: X, y: Y) -> &mut ScatterPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let (x, y) = self.prepare_xy(x.into_series(), y.into_series());
        self.expand_limits_xy(&x, &y, false);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Scatter(ScatterPlot {
            x,
            y,
            color: None,
            // Matplotlib default `s=36` → diameter ≈ 6.77 pt.
            size: crate::mpl_policy::scatter::DEFAULT_DIAMETER_PT,
            marker: MarkerStyle::Circle,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Scatter(scatter)) => scatter,
            _ => unreachable!(),
        }
    }

    /// Vertical bars centered on `x` with the given heights.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.bar([1.0, 2.0, 3.0], [3.0, 7.0, 2.0]).color(Color::STEEL_BLUE);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn bar<X, Y>(&mut self, x: X, heights: Y) -> &mut BarPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let x = x.into_series();
        let heights = heights.into_series();
        let width_rel = 0.8;
        let width = infer_bar_width(x.as_slice(), width_rel);
        self.expand_bar_limits(&x, &heights, width, 0.0);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Bar(BarPlot {
            x,
            heights,
            width: width_rel,
            baseline: 0.0,
            color: None,
            edgecolor: None,
            hatch: crate::style::Hatch::None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Bar(bar)) => bar,
            _ => unreachable!(),
        }
    }

    /// Histogram of a 1D sample (default 10 equal-width bins).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.hist([0.1, 0.2, 0.4, 0.5, 0.8, 1.0]).bins(4);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn hist<D>(&mut self, data: D) -> &mut HistPlot
    where
        D: IntoSeries,
    {
        let data = data.into_series();
        let bin_count = 10;
        // Limits are applied in [`Self::finalize_artist_limits`] from the final
        // `bin_count` so `.bins(n)` after `hist(...)` does not leave a stale ymax
        // from the default 10-bin preview.
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Hist(HistPlot {
            data,
            bin_count,
            color: None,
            edgecolor: None,
            hatch: crate::style::Hatch::None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Hist(hist)) => hist,
            _ => unreachable!(),
        }
    }

    /// Filled area under `(x, y)` down to baseline 0.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.area([0.0, 1.0, 2.0], [0.5, 1.0, 0.2]).alpha(0.4);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn area<X, Y>(&mut self, x: X, y: Y) -> &mut AreaPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        self.expand_limits_xy(&x, &y, true);
        let color_index = self.peek_color();
        self.elements.push(PlotElement::Area(AreaPlot {
            x,
            y,
            baseline: 0.0,
            color: None,
            alpha: 0.35,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Area(area)) => area,
            _ => unreachable!(),
        }
    }

    /// Fill the region between two curves `(x, y1)` and `(x, y2)`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.fill_between([0.0, 1.0, 2.0], [1.0, 2.0, 1.5], [0.0, 0.5, 0.2]).alpha(0.4);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn fill_between<X, Y1, Y2>(&mut self, x: X, y1: Y1, y2: Y2) -> &mut FillBetweenPlot
    where
        X: IntoSeries,
        Y1: IntoSeries,
        Y2: IntoSeries,
    {
        let x = x.into_series();
        let y1 = y1.into_series();
        let y2 = y2.into_series();
        self.expand_limits_xy(&x, &y1, false);
        self.expand_limits_xy(&x, &y2, false);
        let color_index = self.peek_color();
        self.elements
            .push(PlotElement::FillBetween(FillBetweenPlot {
                x,
                y1,
                y2,
                color: None,
                alpha: 0.35,
                label: None,
                color_index,
            }));
        match self.elements.last_mut() {
            Some(PlotElement::FillBetween(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Fill the region between two vertical curves `(x1, y)` and `(x2, y)`.
    pub fn fill_betweenx<Y, X1, X2>(&mut self, y: Y, x1: X1, x2: X2) -> &mut FillBetweenXPlot
    where
        Y: IntoSeries,
        X1: IntoSeries,
        X2: IntoSeries,
    {
        let y = y.into_series();
        let x1 = x1.into_series();
        let x2 = x2.into_series();
        if let Some((ymin, ymax)) = y.min_max() {
            self.expand_range_y(ymin, ymax, false);
        }
        if let Some((a, b)) = x1.min_max() {
            self.expand_range_x(a, b);
        }
        if let Some((a, b)) = x2.min_max() {
            self.expand_range_x(a, b);
        }
        let color_index = self.peek_color();
        self.elements
            .push(PlotElement::FillBetweenX(FillBetweenXPlot {
                y,
                x1,
                x2,
                color: None,
                alpha: 0.35,
                label: None,
                color_index,
            }));
        match self.elements.last_mut() {
            Some(PlotElement::FillBetweenX(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Step plot of `(x, y)` (default [`StepMode::Pre`]).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.step([0.0, 1.0, 2.0], [1.0, 2.0, 1.5]).mode(StepMode::Mid);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn step<X, Y>(&mut self, x: X, y: Y) -> &mut StepPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        self.expand_limits_xy(&x, &y, false);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Step(StepPlot {
            x,
            y,
            mode: StepMode::Pre,
            color: None,
            width: 1.75,
            linestyle: LineStyle::Solid,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Step(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Stairs: constant `values` between consecutive `edges`.
    ///
    /// Requires `edges.len() == values.len() + 1`. Default baseline is `0.0`
    /// (matplotlib `stairs(..., baseline=0)`).
    pub fn stairs<E, V>(&mut self, edges: E, values: V) -> &mut StairsPlot
    where
        E: IntoSeries,
        V: IntoSeries,
    {
        let edges = edges.into_series();
        let values = values.into_series();
        let baseline = 0.0;
        if let Some((xmin, xmax)) = edges.min_max() {
            self.expand_range_x(xmin, xmax);
        }
        if let Some((ymin, ymax)) = values.min_max() {
            // Sticky baseline (matplotlib): include y=0 when baseline is 0.
            self.expand_range_y(ymin.min(baseline), ymax.max(baseline), true);
        } else if baseline.is_finite() {
            self.expand_range_y(baseline, baseline, true);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Stairs(StairsPlot {
            edges,
            values,
            baseline,
            color: None,
            width: 1.75,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Stairs(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Stem plot: vertical lines from baseline to each `(x, y)` with a head marker.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.stem([0.0, 1.0, 2.0], [1.0, 2.0, 1.5]).baseline(0.0);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn stem<X, Y>(&mut self, x: X, y: Y) -> &mut StemPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        self.expand_limits_xy(&x, &y, true);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Stem(StemPlot {
            x,
            y,
            baseline: 0.0,
            color: None,
            width: 1.25,
            marker_size: 4.0,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Stem(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Horizontal segments at each `y[i]` from `xmin` to `xmax`.
    ///
    /// `xmin` / `xmax` may be length 1 (broadcast) or match `y`.
    pub fn hlines<Y, X0, X1>(&mut self, y: Y, xmin: X0, xmax: X1) -> &mut HLinesPlot
    where
        Y: IntoSeries,
        X0: IntoSeries,
        X1: IntoSeries,
    {
        let y = y.into_series();
        let xmin = xmin.into_series();
        let xmax = xmax.into_series();
        if let Some((ymin, ymax)) = y.min_max() {
            self.expand_range_y(ymin, ymax, false);
        }
        if let Some((a, b)) = xmin.min_max() {
            self.expand_range_x(a, b);
        }
        if let Some((a, b)) = xmax.min_max() {
            self.expand_range_x(a, b);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::HLines(HLinesPlot {
            y,
            xmin,
            xmax,
            color: None,
            width: 1.5,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::HLines(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Vertical segments at each `x[i]` from `ymin` to `ymax`.
    pub fn vlines<X, Y0, Y1>(&mut self, x: X, ymin: Y0, ymax: Y1) -> &mut VLinesPlot
    where
        X: IntoSeries,
        Y0: IntoSeries,
        Y1: IntoSeries,
    {
        let x = x.into_series();
        let ymin = ymin.into_series();
        let ymax = ymax.into_series();
        if let Some((xmin, xmax)) = x.min_max() {
            self.expand_range_x(xmin, xmax);
        }
        if let Some((a, b)) = ymin.min_max() {
            self.expand_range_y(a, b, false);
        }
        if let Some((a, b)) = ymax.min_max() {
            self.expand_range_y(a, b, false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::VLines(VLinesPlot {
            x,
            ymin,
            ymax,
            color: None,
            width: 1.5,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::VLines(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Horizontal reference line at `y` spanning the full x-domain.
    pub fn axhline(&mut self, y: f64) -> &mut AxHLinePlot {
        if y.is_finite() {
            self.expand_range_y(y, y, false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::AxHLine(AxHLinePlot {
            y,
            color: None,
            width: 1.25,
            linestyle: LineStyle::Solid,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::AxHLine(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Vertical reference line at `x` spanning the full y-domain.
    pub fn axvline(&mut self, x: f64) -> &mut AxVLinePlot {
        if x.is_finite() {
            self.expand_range_x(x, x);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::AxVLine(AxVLinePlot {
            x,
            color: None,
            width: 1.25,
            linestyle: LineStyle::Solid,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::AxVLine(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Draw an infinite line through two points (matplotlib `ax.axline`).
    ///
    /// The line extends to the axes edges. Useful for reference lines like
    /// y=x or regression fits.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
    ///     ax.axline((0.0, 0.0), (1.0, 1.0))
    ///         .color(Color::CRIMSON)
    ///         .width(1.5);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn axline(
        &mut self,
        xy1: impl Into<(f64, f64)>,
        xy2: impl Into<(f64, f64)>,
    ) -> &mut AxLinePlot {
        let xy1 = xy1.into();
        let xy2 = xy2.into();
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::AxLine(AxLinePlot {
            xy1,
            xy2,
            color: None,
            width: 1.25,
            linestyle: LineStyle::Solid,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::AxLine(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Place text at data coordinates `(x, y)` (does not consume the color cycle).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
    ///     ax.text(1.0, 1.0, "peak")
    ///         .color(Color::CRIMSON)
    ///         .ha(TextAlign::Center)
    ///         .va(TextBaseline::Bottom);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn text(&mut self, x: f64, y: f64, text: impl Into<String>) -> &mut TextPlot {
        if x.is_finite() {
            self.expand_range_x(x, x);
        }
        if y.is_finite() {
            self.expand_range_y(y, y, false);
        }
        self.elements.push(PlotElement::Text(TextPlot {
            x,
            y,
            text: text.into(),
            color: Some(Color::LABEL),
            size: 10.0,
            align: TextAlign::Left,
            baseline: TextBaseline::Alphabetic,
            rotation_deg: 0.0,
            label: None,
            color_index: 0,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Text(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Overlay a table on the axes (matplotlib `ax.table`).
    ///
    /// Cells are placed in axes fraction space (default upper-right). Does not
    /// expand data limits.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
    ///     ax.bar([1.0, 2.0], [3.0, 4.0]);
    ///     ax.table([["A", "3"], ["B", "4"]])
    ///         .col_labels(["Item", "Value"])
    ///         .loc(TableLoc::UpperRight);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn table<R, C, S>(&mut self, rows: R) -> &mut TablePlot
    where
        R: IntoIterator<Item = C>,
        C: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let cells: Vec<Vec<String>> = rows
            .into_iter()
            .map(|row| row.into_iter().map(Into::into).collect())
            .collect();
        self.elements.push(PlotElement::Table(TablePlot {
            cells,
            col_labels: Vec::new(),
            row_labels: Vec::new(),
            loc: TableLoc::UpperRight,
            fontsize: 9.0,
            cell_pad: 4.0,
            edgecolor: Color::SPINE.with_alpha(0.85),
            facecolor: Color::WHITE,
            header_facecolor: Color::rgb(0xee, 0xee, 0xee),
            color: Some(Color::SPINE.with_alpha(0.85)),
            label: None,
            color_index: 0,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Table(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Annotate a data point `xy` with text at `xytext`, optionally with an arrow.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.line([0.0, 1.0, 2.0], [0.2, 0.9, 0.4]);
    ///     ax.annotate("max", (1.0, 0.9), (1.4, 1.15))
    ///         .arrow(true)
    ///         .color(Color::LABEL);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn annotate(
        &mut self,
        text: impl Into<String>,
        xy: (f64, f64),
        xytext: (f64, f64),
    ) -> &mut AnnotatePlot {
        for &(x, y) in &[xy, xytext] {
            if x.is_finite() {
                self.expand_range_x(x, x);
            }
            if y.is_finite() {
                self.expand_range_y(y, y, false);
            }
        }
        self.elements.push(PlotElement::Annotate(AnnotatePlot {
            xy,
            xytext,
            text: text.into(),
            arrow: true,
            arrow_style: crate::artist::ArrowStyle::Triangle,
            color: Some(Color::LABEL),
            arrow_color: None,
            arrow_width: 1.0,
            size: 10.0,
            align: TextAlign::Left,
            baseline: TextBaseline::Alphabetic,
            rotation_deg: 0.0,
            label: None,
            color_index: 0,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Annotate(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Horizontal bars centered on `y` with the given widths (extent along x).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.barh([1.0, 2.0, 3.0], [3.0, 7.0, 2.0]).color(Color::STEEL_BLUE);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn barh<Y, W>(&mut self, y: Y, widths: W) -> &mut BarHPlot
    where
        Y: IntoSeries,
        W: IntoSeries,
    {
        let y = y.into_series();
        let widths = widths.into_series();
        let height_rel = 0.8;
        let height = infer_barh_height(y.as_slice(), height_rel);
        self.expand_barh_limits(&y, &widths, height, 0.0);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::BarH(BarHPlot {
            y,
            widths,
            height: height_rel,
            baseline: 0.0,
            color: None,
            edgecolor: None,
            hatch: crate::style::Hatch::None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::BarH(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Pie chart of non-negative slice `sizes`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.pie([30.0, 20.0, 50.0]).labels(["A", "B", "C"]);
    ///     ax.legend(Legend::TopRight);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn pie<S>(&mut self, sizes: S) -> &mut PiePlot
    where
        S: IntoSeries,
    {
        let sizes = sizes.into_series();
        let v = pie_policy::VIEW;
        self.expand_range_tight_x(-v, v);
        self.expand_range_tight_y(-v, v);
        self.show_grid = Some(false);
        self.frame_on = false;
        let n = sizes.len().max(1);
        let color_index = self.alloc_color();
        for _ in 1..n {
            self.alloc_color();
        }
        self.elements.push(PlotElement::Pie(PiePlot {
            sizes,
            labels: Vec::new(),
            start_angle: 90.0,
            counterclock: false,
            color: None,
            edgecolor: None,
            alpha: 1.0,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Pie(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Stacked area plot: series in `ys` are stacked bottom→top along shared `x`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.stackplot([0.0, 1.0, 2.0], [[1.0, 1.5, 1.0], [0.5, 0.5, 1.0]])
    ///         .labels(["a", "b"]);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn stackplot<X, I, S>(&mut self, x: X, ys: I) -> &mut StackPlot
    where
        X: IntoSeries,
        I: IntoIterator<Item = S>,
        S: IntoSeries,
    {
        let x = x.into_series();
        let ys: Vec<Series> = ys.into_iter().map(|s| s.into_series()).collect();
        if let Some((xmin, xmax)) = x.min_max() {
            self.expand_range_x(xmin, xmax);
        }
        let refs: Vec<&[f64]> = ys.iter().map(|s| s.as_slice()).collect();
        if let Some(ymax) = stackplot_ymax(&refs) {
            self.expand_range_y(0.0, ymax, true);
        }
        let n = ys.len().max(1);
        let color_index = self.alloc_color();
        for _ in 1..n {
            self.alloc_color();
        }
        self.elements.push(PlotElement::StackPlot(StackPlot {
            x,
            ys,
            labels: Vec::new(),
            alpha: 0.85,
            color: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::StackPlot(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Event plot: each series is a row of event positions along x.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.eventplot([[1.0, 2.0, 5.0], [0.5, 3.0, 4.0]]);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn eventplot<I, S>(&mut self, positions: I) -> &mut EventPlot
    where
        I: IntoIterator<Item = S>,
        S: IntoSeries,
    {
        let positions: Vec<Series> = positions.into_iter().map(|s| s.into_series()).collect();
        let n = positions.len().max(1);
        self.expand_range_tight_y(0.5, n as f64 + 0.5);
        self.y_ticks = Some((1..=positions.len()).map(|i| i as f64).collect());
        let mut xmin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        for s in &positions {
            if let Some((a, b)) = s.min_max() {
                xmin = xmin.min(a);
                xmax = xmax.max(b);
            }
        }
        if xmin.is_finite() && xmax.is_finite() {
            self.expand_range_x(xmin, xmax);
        }
        let color_index = self.alloc_color();
        for _ in 1..n {
            self.alloc_color();
        }
        self.elements.push(PlotElement::EventPlot(EventPlot {
            positions,
            labels: Vec::new(),
            lineoffset: 0.8,
            linewidth: 1.5,
            color: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::EventPlot(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Broken horizontal bars: each `(xmin, width)` at `yrange = (ymin, height)`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     // yrange = (ymin, height), matching matplotlib
    ///     ax.broken_barh([(10.0, 50.0), (100.0, 20.0)], (15.0, 9.0));
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn broken_barh<I>(&mut self, xranges: I, yrange: (f64, f64)) -> &mut BrokenBarHPlot
    where
        I: IntoIterator<Item = (f64, f64)>,
    {
        let xranges: Vec<(f64, f64)> = xranges.into_iter().collect();
        let (y, height) = yrange;
        // Fold all segments first so padded margins use the full x span
        // (matches matplotlib `broken_barh` autoscale).
        let mut x0 = f64::INFINITY;
        let mut x1 = f64::NEG_INFINITY;
        for &(xmin, width) in &xranges {
            if xmin.is_finite() && width.is_finite() {
                x0 = x0.min(xmin.min(xmin + width));
                x1 = x1.max(xmin.max(xmin + width));
            }
        }
        if x0.is_finite() && x1.is_finite() {
            self.expand_range_x(x0, x1);
        }
        // Matplotlib yrange = (ymin, height): bar covers [y, y+height].
        if y.is_finite() && height.is_finite() {
            let h = height.abs();
            self.expand_range_y(y, y + h, false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::BrokenBarH(BrokenBarHPlot {
            xranges,
            y,
            height: height.abs().max(1e-9),
            color: None,
            edgecolor: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::BrokenBarH(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Filled closed polygon through `(x, y)` vertices.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.polygon([0.0, 1.0, 0.5], [0.0, 0.0, 1.0]).alpha(0.5);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn polygon<X, Y>(&mut self, x: X, y: Y) -> &mut PolygonPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        self.expand_limits_xy(&x, &y, false);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Polygon(PolygonPlot {
            x,
            y,
            color: None,
            edgecolor: None,
            alpha: 0.45,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Polygon(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Axis-aligned rectangle patch (matplotlib `patches.Rectangle`).
    ///
    /// `(x, y)` is the lower-left corner in data coords; `width` / `height` may be negative.
    pub fn rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut RectanglePlot {
        if let Some((x0, y0, x1, y1)) = crate::recipes::rectangle_data_rect(x, y, width, height) {
            self.expand_range_x(x0, x1);
            self.expand_range_y(y0, y1, false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Rectangle(RectanglePlot {
            x,
            y,
            width,
            height,
            color: None,
            edgecolor: None,
            alpha: 0.45,
            hatch: crate::style::Hatch::None,
            linewidth: 1.0,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Rectangle(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Circle patch centered at `(x, y)` with data-space `radius` (matplotlib `Circle`).
    pub fn circle(&mut self, x: f64, y: f64, radius: f64) -> &mut CirclePlot {
        if x.is_finite() && y.is_finite() && radius.is_finite() {
            let r = radius.abs();
            self.expand_range_x(x - r, x + r);
            self.expand_range_y(y - r, y + r, false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Circle(CirclePlot {
            x,
            y,
            radius,
            color: None,
            edgecolor: None,
            alpha: 0.45,
            linewidth: 1.0,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Circle(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Ellipse patch centered at `(x, y)` with data-space diameters (matplotlib `Ellipse`).
    pub fn ellipse(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut EllipsePlot {
        if [x, y, width, height].iter().all(|v| v.is_finite()) {
            let rx = width.abs() * 0.5;
            let ry = height.abs() * 0.5;
            self.expand_range_x(x - rx, x + rx);
            self.expand_range_y(y - ry, y + ry, false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Ellipse(EllipsePlot {
            x,
            y,
            width,
            height,
            color: None,
            edgecolor: None,
            alpha: 0.45,
            linewidth: 1.0,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Ellipse(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Horizontal band from `ymin` to `ymax` across the full x-domain.
    pub fn axhspan(&mut self, ymin: f64, ymax: f64) -> &mut AxHSpanPlot {
        if ymin.is_finite() && ymax.is_finite() {
            self.expand_range_y(ymin.min(ymax), ymin.max(ymax), false);
        }
        let color_index = self.peek_color();
        self.elements.push(PlotElement::AxHSpan(AxHSpanPlot {
            ymin,
            ymax,
            color: None,
            alpha: 0.25,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::AxHSpan(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Vertical band from `xmin` to `xmax` across the full y-domain.
    pub fn axvspan(&mut self, xmin: f64, xmax: f64) -> &mut AxVSpanPlot {
        if xmin.is_finite() && xmax.is_finite() {
            self.expand_range_x(xmin.min(xmax), xmin.max(xmax));
        }
        let color_index = self.peek_color();
        self.elements.push(PlotElement::AxVSpan(AxVSpanPlot {
            xmin,
            xmax,
            color: None,
            alpha: 0.25,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::AxVSpan(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Points with vertical error bars (`y ± yerr`).
    ///
    /// Add horizontal errors with [`.xerr(...)`](ErrorBarPlot::xerr). For
    /// asymmetric arms use [`.yerr_asym`](ErrorBarPlot::yerr_asym) /
    /// [`.xerr_asym`](ErrorBarPlot::xerr_asym) (matplotlib `yerr`/`xerr` of
    /// shape `(2, N)`).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.errorbar([0.0, 1.0, 2.0], [1.0, 1.5, 1.2], [0.1, 0.2, 0.15])
    ///         .xerr([0.05, 0.05, 0.05]);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn errorbar<X, Y, E>(&mut self, x: X, y: Y, yerr: E) -> &mut ErrorBarPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
        E: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let yerr = crate::artist::ErrBars::Symmetric(yerr.into_series());
        self.expand_errorbar_limits(&x, &y, &yerr);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::ErrorBar(ErrorBarPlot {
            x,
            y,
            yerr,
            xerr: None,
            color: None,
            // Slightly heavier than stock `lines.linewidth` for raster compare.
            width: 1.75,
            capsize: 4.0,
            connect: true,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::ErrorBar(eb)) => eb,
            _ => unreachable!(),
        }
    }

    /// Heatmap of a row-major `nrows × ncols` grid.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.heatmap(2, 3, [0.0, 0.5, 1.0, 0.2, 0.8, 0.4])
    ///         .cmap(Colormap::Viridis)
    ///         .colorbar(true);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn heatmap<V>(&mut self, nrows: usize, ncols: usize, values: V) -> &mut HeatmapPlot
    where
        V: IntoSeries,
    {
        let nrows = nrows.max(1);
        let ncols = ncols.max(1);
        let values = values.into_series();
        // Limits applied in [`Self::finalize_artist_limits`] so `.extent([...])`
        // replaces the default index box instead of unioning with it (mpl imshow).
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        self.elements.push(PlotElement::Heatmap(HeatmapPlot {
            nrows,
            ncols,
            values,
            cmap: Colormap::Viridis.into(),
            vmin: None,
            vmax: None,
            norm: Norm::Linear,
            origin: crate::recipes::HeatmapOrigin::Upper,
            extent: None,
            alpha: 1.0,
            colorbar: true,
            label: None,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Heatmap(hm)) => hm,
            _ => unreachable!(),
        }
    }

    /// Heatmap from an `ndarray::Array2<f64>` (row-major; requires `ndarray` feature).
    #[cfg(feature = "ndarray")]
    pub fn heatmap_array(&mut self, values: &ndarray::Array2<f64>) -> &mut HeatmapPlot {
        let (nrows, ncols, series) = crate::series::array2_row_major(values);
        self.heatmap(nrows, ncols, series)
    }

    /// 2D histogram of `(x, y)` samples.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.hist2d([0.1, 0.2, 0.8, 0.9], [0.1, 0.9, 0.2, 0.8]).bins(4);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn hist2d<X, Y>(&mut self, x: X, y: Y) -> &mut Hist2dPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        // Match matplotlib: extent is the bin edges (no extra data margin).
        let bins = hist2d_bins(x.as_slice(), y.as_slice(), 10, 10);
        if let (Some(&x0), Some(&x1)) = (bins.x_edges.first(), bins.x_edges.last()) {
            self.expand_range_tight_x(x0, x1);
        }
        if let (Some(&y0), Some(&y1)) = (bins.y_edges.first(), bins.y_edges.last()) {
            self.expand_range_tight_y(y0, y1);
        }
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        self.elements.push(PlotElement::Hist2d(Hist2dPlot {
            x,
            y,
            bins_x: 10,
            bins_y: 10,
            cmap: plotine_core::Colormap::Viridis.into(),
            vmin: None,
            vmax: None,
            norm: Norm::Linear,
            colorbar: true,
            label: None,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Hist2d(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Hexagonal binning of `(x, y)` samples.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.hexbin([0.1, 0.2, 0.8, 0.9, 0.5], [0.1, 0.9, 0.2, 0.8, 0.5]).gridsize(8);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn hexbin<X, Y>(&mut self, x: X, y: Y) -> &mut HexbinPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        // Matplotlib hexbin: `update_datalim(binning corners)` then
        // `_request_autoscale_view(tight=True)`. Docs: `tight=True` still applies
        // `ax.margins` (0.05) — it only skips locator round-out. PolyCollection
        // has empty sticky_edges (unlike hist2d QuadMesh), so the 5% pad sticks.
        if let Some((xmin, xmax, ymin, ymax)) =
            crate::recipes::hexbin_extent(x.as_slice(), y.as_slice())
        {
            self.expand_range_x(xmin, xmax);
            self.expand_range_y(ymin, ymax, false);
        } else {
            self.expand_limits_xy(&x, &y, false);
        }
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        self.elements.push(PlotElement::Hexbin(HexbinPlot {
            x,
            y,
            gridsize: 20,
            cmap: plotine_core::Colormap::Viridis.into(),
            vmin: None,
            vmax: None,
            norm: Norm::Linear,
            colorbar: true,
            label: None,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Hexbin(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Contour lines of a row-major `nrows × ncols` field (coords at integer indices).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let z = [0.0, 0.5, 1.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.0];
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.contour(3, 3, &z).levels(5).color(Color::CRIMSON).clabel(true);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn contour<V>(&mut self, nrows: usize, ncols: usize, values: V) -> &mut ContourPlot
    where
        V: IntoSeries,
    {
        let values = values.into_series();
        let nrows = nrows.max(2);
        let ncols = ncols.max(2);
        // Matplotlib `contour` with index mesh: data coords span [0, n-1].
        self.expand_range_tight_x(0.0, (ncols - 1) as f64);
        self.expand_range_tight_y(0.0, (nrows - 1) as f64);
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Contour(ContourPlot {
            nrows,
            ncols,
            values,
            levels: None,
            level_count: 8,
            color: None,
            width: 1.25,
            clabel: false,
            clabel_size: 9.0,
            clabel_color: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Contour(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Filled contours of a row-major `nrows × ncols` field.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let z = [0.0, 0.5, 1.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.0];
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.contourf(3, 3, &z).levels(6).cmap(Colormap::Viridis);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn contourf<V>(&mut self, nrows: usize, ncols: usize, values: V) -> &mut ContourfPlot
    where
        V: IntoSeries,
    {
        let values = values.into_series();
        let nrows = nrows.max(2);
        let ncols = ncols.max(2);
        // Matplotlib `contourf` with index mesh: data coords span [0, n-1].
        self.expand_range_tight_x(0.0, (ncols - 1) as f64);
        self.expand_range_tight_y(0.0, (nrows - 1) as f64);
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        self.elements.push(PlotElement::Contourf(ContourfPlot {
            nrows,
            ncols,
            values,
            levels: None,
            level_count: 10,
            cmap: plotine_core::Colormap::Viridis.into(),
            norm: Norm::Linear,
            colorbar: true,
            label: None,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Contourf(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Pseudocolor plot on a triangular mesh (matplotlib `tripcolor`).
    ///
    /// Requires `.triangles([[i, j, k], …])` with indices into `(x, y, z)`.
    /// Automatic Delaunay triangulation is not included in this MVP.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.tripcolor([0.0, 1.0, 0.5], [0.0, 0.0, 1.0], [0.0, 1.0, 0.5])
    ///         .triangles([[0, 1, 2]])
    ///         .cmap(Colormap::Coolwarm)
    ///         .colorbar(true);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn tripcolor<X, Y, Z>(&mut self, x: X, y: Y, z: Z) -> &mut TripcolorPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
        Z: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let z = z.into_series();
        self.expand_limits_xy(&x, &y, false);
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        self.elements.push(PlotElement::Tripcolor(TripcolorPlot {
            x,
            y,
            z,
            triangles: Vec::new(),
            cmap: Colormap::Viridis.into(),
            vmin: None,
            vmax: None,
            norm: Norm::Linear,
            colorbar: true,
            label: None,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Tripcolor(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Contour lines on a triangular mesh (matplotlib `tricontour`).
    ///
    /// Requires `.triangles([[i, j, k], …])`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.tricontour([0.0, 2.0, 0.0], [0.0, 0.0, 2.0], [0.0, 1.0, 0.0])
    ///         .triangles([[0, 1, 2]])
    ///         .levels(5)
    ///         .color(Color::CRIMSON);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn tricontour<X, Y, Z>(&mut self, x: X, y: Y, z: Z) -> &mut TricontourPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
        Z: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let z = z.into_series();
        // Matplotlib `ContourSet` sets sticky_edges to the full data range, so
        // a following autoscale snaps limits tight (0 on the spine) — unlike
        // tripcolor's empty sticky_edges which keep the 5% margin.
        if let Some((xmin, xmax)) = x.min_max() {
            self.expand_range_tight_x(xmin, xmax);
        }
        if let Some((ymin, ymax)) = y.min_max() {
            self.expand_range_tight_y(ymin, ymax);
        }
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Tricontour(TricontourPlot {
            x,
            y,
            z,
            triangles: Vec::new(),
            levels: None,
            level_count: 8,
            color: None,
            width: 1.0,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Tricontour(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Filled contours on a triangular mesh (matplotlib `tricontourf`).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.tricontourf([0.0, 2.0, 0.0], [0.0, 0.0, 2.0], [0.0, 1.0, 0.0])
    ///         .triangles([[0, 1, 2]])
    ///         .levels(5)
    ///         .cmap(Colormap::Viridis);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn tricontourf<X, Y, Z>(&mut self, x: X, y: Y, z: Z) -> &mut TricontourfPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
        Z: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let z = z.into_series();
        // Same ContourSet sticky-edge behaviour as `tricontour`.
        if let Some((xmin, xmax)) = x.min_max() {
            self.expand_range_tight_x(xmin, xmax);
        }
        if let Some((ymin, ymax)) = y.min_max() {
            self.expand_range_tight_y(ymin, ymax);
        }
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        self.elements
            .push(PlotElement::Tricontourf(TricontourfPlot {
                x,
                y,
                z,
                triangles: Vec::new(),
                levels: None,
                level_count: 10,
                cmap: Colormap::Viridis.into(),
                norm: Norm::Linear,
                colorbar: true,
                label: None,
            }));
        match self.elements.last_mut() {
            Some(PlotElement::Tricontourf(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Pseudocolor mesh with explicit edge coordinates.
    ///
    /// Requires `x_edges.len() == ncols + 1`, `y_edges.len() == nrows + 1`,
    /// `values.len() == nrows * ncols`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.pcolormesh([0.0, 1.0, 3.0], [0.0, 2.0], [1.0, 2.0]);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn pcolormesh<X, Y, V>(&mut self, x_edges: X, y_edges: Y, values: V) -> &mut PcolorMeshPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
        V: IntoSeries,
    {
        let x_edges = x_edges.into_series();
        let y_edges = y_edges.into_series();
        let values = values.into_series();
        if let Some((xmin, xmax)) = x_edges.min_max() {
            self.expand_range_tight_x(xmin, xmax);
        }
        if let Some((ymin, ymax)) = y_edges.min_max() {
            self.expand_range_tight_y(ymin, ymax);
        }
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        self.elements.push(PlotElement::PcolorMesh(PcolorMeshPlot {
            x_edges,
            y_edges,
            values,
            cmap: plotine_core::Colormap::Viridis.into(),
            vmin: None,
            vmax: None,
            norm: Norm::Linear,
            colorbar: true,
            label: None,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::PcolorMesh(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Spy plot: markers where `|z| > precision` on a row-major matrix.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let z = [0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0];
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.spy(3, 3, &z);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn spy<V>(&mut self, nrows: usize, ncols: usize, values: V) -> &mut SpyPlot
    where
        V: IntoSeries,
    {
        let values = values.into_series();
        let nrows = nrows.max(1);
        let ncols = ncols.max(1);
        self.expand_range_tight_x(-0.5, (ncols - 1) as f64 + 0.5);
        self.expand_range_tight_y(-0.5, (nrows - 1) as f64 + 0.5);
        // Matplotlib spy: origin top-left, x ticks on top, squares, no grid.
        self.x_ticks_top = true;
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Spy(SpyPlot {
            nrows,
            ncols,
            values,
            precision: 1e-8,
            // Roughly matches matplotlib `spy(..., markersize=8)`.
            marker_size: 6.0,
            color: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Spy(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Quiver (arrow) plot of vector field samples.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.quiver([0.0, 1.0], [0.0, 1.0], [1.0, 0.0], [0.0, 1.0]);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn quiver<X, Y, U, V>(&mut self, x: X, y: Y, u: U, v: V) -> &mut QuiverPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
        U: IntoSeries,
        V: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let u = u.into_series();
        let v = v.into_series();
        // `expand_limits_xy` already applies matplotlib-style 5% margins once.
        self.expand_limits_xy(&x, &y, false);
        // Matplotlib quiver leaves cartesian grid off unless the user enables it.
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Quiver(QuiverPlot {
            x,
            y,
            u,
            v,
            scale: None,
            width: crate::mpl_policy::quiver::WIDTH_PT,
            key_length: None,
            key_label: None,
            color: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Quiver(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Wind barbs at `(x, y)` with components `(u, v)` (matplotlib `barbs`).
    ///
    /// Magnitude is encoded with flags (50), full barbs (10), and half barbs (5)
    /// by default; staff length is fixed in points (not scaled by magnitude).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.barbs([0.0, 1.0], [0.0, 1.0], [25.0, 55.0], [0.0, 10.0])
    ///         .color(Color::STEEL_BLUE);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn barbs<X, Y, U, V>(&mut self, x: X, y: Y, u: U, v: V) -> &mut BarbsPlot
    where
        X: IntoSeries,
        Y: IntoSeries,
        U: IntoSeries,
        V: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let u = u.into_series();
        let v = v.into_series();
        // `expand_limits_xy` already applies matplotlib-style 5% margins once.
        self.expand_limits_xy(&x, &y, false);
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Barbs(BarbsPlot {
            x,
            y,
            u,
            v,
            length: 6.0,
            width: 1.0,
            flip: false,
            half: 5.0,
            full: 10.0,
            flag: 50.0,
            color: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Barbs(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Streamlines of a regular `nrows × ncols` vector field (`u`/`v` row-major).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let u = [0.0, 1.0, 0.0, 1.0];
    /// let v = [1.0, 0.0, 1.0, 0.0];
    /// let png = Figure::new().size(3.0, 2.5).dpi(72.0).axes(|ax| {
    ///     ax.streamplot(2, 2, &u, &v).density(1.0);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn streamplot<U, V>(&mut self, nrows: usize, ncols: usize, u: U, v: V) -> &mut StreamPlot
    where
        U: IntoSeries,
        V: IntoSeries,
    {
        let u = u.into_series();
        let v = v.into_series();
        let nrows = nrows.max(2);
        let ncols = ncols.max(2);
        // Match matplotlib sticky edges: data grid spans [0, n-1].
        self.expand_range_tight_x(0.0, (ncols - 1) as f64);
        self.expand_range_tight_y(0.0, (nrows - 1) as f64);
        // Matplotlib streamplot leaves cartesian grid off unless the user enables it.
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::StreamPlot(StreamPlot {
            u,
            v,
            nrows,
            ncols,
            density: 1.0,
            width: 1.0,
            arrow_size: 1.0,
            color: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::StreamPlot(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Polar line: `(theta, r)` in radians → Cartesian polyline + polar frame.
    ///
    /// Uses matplotlib-like polar chrome (circular spine, θ° / r labels, equal
    /// aspect). θ = 0 at +x, increasing counter-clockwise.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// use std::f64::consts::PI;
    /// let th: Vec<f64> = (0..60).map(|i| i as f64 * PI / 30.0).collect();
    /// let r: Vec<f64> = th.iter().map(|t| 1.0 + 0.3 * t.cos()).collect();
    /// let png = Figure::new().size(3.0, 3.0).dpi(72.0).axes(|ax| {
    ///     ax.polar_line(&th, &r).color(Color::CRIMSON);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn polar_line<T, R>(&mut self, theta: T, r: R) -> &mut LinePlot
    where
        T: IntoSeries,
        R: IntoSeries,
    {
        let theta = theta.into_series();
        let r = r.into_series();
        let data_rmax = r
            .as_slice()
            .iter()
            .filter(|v| v.is_finite())
            .map(|v| v.abs())
            .fold(0.0_f64, f64::max)
            .max(1e-9);
        let (rmax, _) = polar_rings(data_rmax, crate::mpl_policy::polar::RING_N_HINT);
        self.ensure_polar_frame(rmax);
        let (x, y) = polar_to_cartesian(theta.as_slice(), r.as_slice());
        // Polar owns tight limits (`self.polar`); `line` will not re-pad.
        self.line(x, y)
    }

    /// Polar scatter: `(theta, r)` in radians → Cartesian markers + polar frame.
    pub fn polar_scatter<T, R>(&mut self, theta: T, r: R) -> &mut ScatterPlot
    where
        T: IntoSeries,
        R: IntoSeries,
    {
        let theta = theta.into_series();
        let r = r.into_series();
        let data_rmax = r
            .as_slice()
            .iter()
            .filter(|v| v.is_finite())
            .map(|v| v.abs())
            .fold(0.0_f64, f64::max)
            .max(1e-9);
        let (rmax, _) = polar_rings(data_rmax, crate::mpl_policy::polar::RING_N_HINT);
        self.ensure_polar_frame(rmax);
        let (x, y) = polar_to_cartesian(theta.as_slice(), r.as_slice());
        self.scatter(x, y)
    }

    /// Explicit polar frame (rings + spokes) at the origin.
    pub fn polar_frame(&mut self, rmax: f64) -> &mut PolarFramePlot {
        self.ensure_polar_frame(rmax.abs().max(1e-9));
        match self.elements.iter_mut().find_map(|el| match el {
            PlotElement::PolarFrame(p) => Some(p),
            _ => None,
        }) {
            Some(p) => p,
            None => unreachable!(),
        }
    }

    /// Force cartesian view limits to the polar disk (undo line/scatter padding).
    fn set_polar_view(&mut self, rmax: f64) {
        let rmax = rmax.abs().max(1e-9);
        if !self.x_lim_manual {
            self.x_data_min = -rmax;
            self.x_data_max = rmax;
            self.x_min = -rmax;
            self.x_max = rmax;
            self.x_lim_seeded = true;
        }
        if !self.y_lim_manual {
            self.y_data_min = -rmax;
            self.y_data_max = rmax;
            self.y_min = -rmax;
            self.y_max = rmax;
            self.y_lim_seeded = true;
        }
    }

    fn ensure_polar_frame(&mut self, rmax: f64) {
        let rmax = rmax.abs().max(1e-9);
        self.polar = true;
        self.geo = None;
        if self.show_grid.is_none() {
            self.show_grid = Some(false);
        }
        self.set_polar_view(rmax);
        if let Some(PlotElement::PolarFrame(p)) = self
            .elements
            .iter_mut()
            .find(|el| matches!(el, PlotElement::PolarFrame(_)))
        {
            p.rmax = p.rmax.max(rmax);
            return;
        }
        // Insert frame first so data draws on top. Do not consume the color cycle.
        self.elements.insert(
            0,
            PlotElement::PolarFrame(PolarFramePlot {
                rmax,
                rings: crate::mpl_policy::polar::RING_N_HINT,
                spokes: 8,
                // Sentinel: draw path maps GRID → matplotlib polar `#b0b0b0`.
                color: Some(Color::GRID),
                width: 0.8,
                label: None,
                color_index: 0,
            }),
        );
    }

    /// Box-and-whisker plot for one or more sample groups.
    ///
    /// Groups are placed at x = 1, 2, … with Tukey whiskers (1.5×IQR).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.boxplot([[1.0, 2.0, 2.5, 3.0, 4.0], [2.0, 3.0, 3.5, 4.0, 5.0]]);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn boxplot<I, S>(&mut self, groups: I) -> &mut BoxPlot
    where
        I: IntoIterator<Item = S>,
        S: IntoSeries,
    {
        let groups: Vec<Series> = groups.into_iter().map(|s| s.into_series()).collect();
        let refs: Vec<&[f64]> = groups.iter().map(|s| s.as_slice()).collect();
        let stats = boxplot_stats(&refs, 0.55);
        if !stats.is_empty() {
            let n = groups.len() as f64;
            // Match matplotlib: tight categorical window, integer tick centers.
            self.expand_range_tight_x(0.5, n + 0.5);
            self.x_ticks = Some((1..=groups.len()).map(|i| i as f64).collect());
            let mut ymin = f64::INFINITY;
            let mut ymax = f64::NEG_INFINITY;
            for (s, _) in &stats {
                ymin = ymin.min(s.whisker_lo);
                ymax = ymax.max(s.whisker_hi);
                for &f in &s.fliers {
                    ymin = ymin.min(f);
                    ymax = ymax.max(f);
                }
            }
            if ymin.is_finite() && ymax.is_finite() {
                self.expand_range_y(ymin, ymax, false);
            }
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::BoxPlot(BoxPlot {
            groups,
            widths: 0.55,
            show_fliers: true,
            color: None,
            edgecolor: None,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::BoxPlot(bp)) => bp,
            _ => unreachable!(),
        }
    }

    /// Violin plot for one or more sample groups (Gaussian KDE).
    ///
    /// Groups are placed at x = 1, 2, ….
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.violin([[1.0, 2.0, 2.5, 3.0, 4.0], [2.0, 3.0, 3.5, 4.0, 5.0]]);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn violin<I, S>(&mut self, groups: I) -> &mut ViolinPlot
    where
        I: IntoIterator<Item = S>,
        S: IntoSeries,
    {
        let groups: Vec<Series> = groups.into_iter().map(|s| s.into_series()).collect();
        let points = crate::mpl_policy::violin::POINTS;
        let widths = crate::mpl_policy::violin::WIDTHS;
        let refs: Vec<&[f64]> = groups.iter().map(|s| s.as_slice()).collect();
        // Use the same `points` as draw (not a lower probe count).
        let geoms = violin_geoms(&refs, points);
        if !geoms.is_empty() {
            let n = groups.len() as f64;
            self.expand_range_tight_x(0.5, n + 0.5);
            self.x_ticks = Some((1..=groups.len()).map(|i| i as f64).collect());
            let mut ymin = f64::INFINITY;
            let mut ymax = f64::NEG_INFINITY;
            for g in &geoms {
                ymin = ymin.min(g.ymin);
                ymax = ymax.max(g.ymax);
            }
            if ymin.is_finite() && ymax.is_finite() {
                self.expand_range_y(ymin, ymax, false);
            }
        }
        let color_index = self.alloc_color();
        self.elements.push(PlotElement::Violin(ViolinPlot {
            groups,
            widths,
            points,
            // Stock mpl `showmedians=False`; opt in with `.show_median(true)`.
            show_median: false,
            show_extrema: true,
            color: None,
            edgecolor: None,
            alpha: 0.55,
            label: None,
            color_index,
        }));
        match self.elements.last_mut() {
            Some(PlotElement::Violin(vp)) => vp,
            _ => unreachable!(),
        }
    }

    /// Whether any artist requested a colorbar.
    pub(crate) fn needs_colorbar(&self) -> bool {
        self.colorbar_spec().is_some()
    }

    /// Effective title size in points (override or theme).
    pub(crate) fn title_size_pt(&self, theme: &crate::theme::Theme) -> f32 {
        self.title_fontsize.unwrap_or(theme.title_size)
    }

    /// Effective x-label size in points (override or theme).
    pub(crate) fn x_label_size_pt(&self, theme: &crate::theme::Theme) -> f32 {
        self.x_label_fontsize.unwrap_or(theme.label_size)
    }

    /// Effective y-label size in points (override or theme).
    pub(crate) fn y_label_size_pt(&self, theme: &crate::theme::Theme) -> f32 {
        self.y_label_fontsize.unwrap_or(theme.label_size)
    }

    pub(crate) fn has_twin_y(&self) -> bool {
        self.twin_y.is_some()
    }

    /// Merge twin x-domain into the host (used by [`twin_y`](Self::twin_y)).
    fn absorb_twin_x(&mut self, twin: &Axes) {
        if self.x_lim_manual {
            return;
        }
        if twin.x_lim_manual {
            self.x_min = twin.x_min;
            self.x_max = twin.x_max;
            self.x_data_min = twin.x_data_min;
            self.x_data_max = twin.x_data_max;
            self.x_lim_manual = true;
            self.x_lim_seeded = true;
            return;
        }
        if twin.x_lim_seeded {
            // Merge data extents and re-apply margins (not tight).
            self.expand_range_x(twin.x_data_min, twin.x_data_max);
        }
    }

    /// Merge twin y-domain into the host (used by [`twin_x`](Self::twin_x)).
    fn absorb_twin_y(&mut self, twin: &Axes) {
        if self.y_lim_manual {
            return;
        }
        if twin.y_lim_manual {
            self.y_min = twin.y_min;
            self.y_max = twin.y_max;
            self.y_data_min = twin.y_data_min;
            self.y_data_max = twin.y_data_max;
            self.y_lim_manual = true;
            self.y_lim_seeded = true;
            return;
        }
        if twin.y_lim_seeded {
            self.y_pad_include_zero |= twin.y_pad_include_zero;
            self.expand_range_y(twin.y_data_min, twin.y_data_max, twin.y_pad_include_zero);
        }
    }

    /// First artist that wants a colorbar.
    pub(crate) fn colorbar_spec(&self) -> Option<ColorbarSpec> {
        self.elements.iter().find_map(|el| match el {
            PlotElement::Heatmap(hm) if hm.colorbar => {
                let (vmin, vmax) = hm.value_limits();
                Some(ColorbarSpec {
                    cmap: hm.cmap.clone(),
                    vmin,
                    vmax,
                    norm: hm.norm,
                    boundaries: listed_colorbar_boundaries(&hm.cmap, vmin, vmax),
                })
            }
            PlotElement::Hist2d(h) if h.colorbar => {
                let (vmin, vmax) = h.value_limits();
                Some(ColorbarSpec {
                    cmap: h.cmap.clone(),
                    vmin,
                    vmax,
                    norm: h.norm,
                    boundaries: listed_colorbar_boundaries(&h.cmap, vmin, vmax),
                })
            }
            PlotElement::Hexbin(h) if h.colorbar => {
                let (vmin, vmax) = h.value_limits();
                Some(ColorbarSpec {
                    cmap: h.cmap.clone(),
                    vmin,
                    vmax,
                    norm: h.norm,
                    boundaries: listed_colorbar_boundaries(&h.cmap, vmin, vmax),
                })
            }
            PlotElement::Contourf(h) if h.colorbar => {
                let levels = h.resolved_levels();
                let (vmin, vmax) = h.value_limits();
                Some(ColorbarSpec {
                    cmap: h.cmap.clone(),
                    vmin,
                    vmax,
                    norm: h.norm,
                    boundaries: Some(levels),
                })
            }
            PlotElement::PcolorMesh(h) if h.colorbar => {
                let (vmin, vmax) = h.value_limits();
                Some(ColorbarSpec {
                    cmap: h.cmap.clone(),
                    vmin,
                    vmax,
                    norm: h.norm,
                    boundaries: listed_colorbar_boundaries(&h.cmap, vmin, vmax),
                })
            }
            PlotElement::Tripcolor(h) if h.colorbar => {
                let (vmin, vmax) = h.value_limits();
                Some(ColorbarSpec {
                    cmap: h.cmap.clone(),
                    vmin,
                    vmax,
                    norm: h.norm,
                    boundaries: listed_colorbar_boundaries(&h.cmap, vmin, vmax),
                })
            }
            PlotElement::Tricontourf(h) if h.colorbar => {
                let levels = h.resolved_levels();
                let (vmin, vmax) = h.value_limits();
                Some(ColorbarSpec {
                    cmap: h.cmap.clone(),
                    vmin,
                    vmax,
                    norm: h.norm,
                    boundaries: Some(levels),
                })
            }
            _ => None,
        })
    }

    fn alloc_color(&mut self) -> usize {
        let idx = self.next_color_index;
        self.next_color_index += 1;
        idx
    }

    /// Peek the next cycle index without advancing (matplotlib `fill_between` /
    /// `axhspan` / `axvspan` reuse C0 so a following `plot` stays on the same
    /// color).
    fn peek_color(&self) -> usize {
        self.next_color_index
    }

    /// When a geo projection is active, treat `(x,y)` as lon/lat degrees.
    fn prepare_xy(&self, x: Series, y: Series) -> (Series, Series) {
        match self.geo {
            Some(proj) => project_lonlat(proj, &x, &y).unwrap_or((x, y)),
            None => (x, y),
        }
    }

    fn expand_limits_xy(&mut self, x: &Series, y: &Series, include_zero_y: bool) {
        // Polar (and other tight spaces) already set the view disk; do not apply
        // cartesian `ax.margins` padding on top of transformed samples.
        // Geo maps set an explicit global extent via `projection()`.
        if self.polar || self.geo.is_some() {
            return;
        }
        if let Some((xmin, xmax)) = x.min_max() {
            self.expand_range_x(xmin, xmax);
        }
        if let Some((ymin, ymax)) = y.min_max() {
            self.expand_range_y(ymin, ymax, include_zero_y);
        }
    }

    fn expand_bar_limits(&mut self, x: &Series, heights: &Series, width: f64, baseline: f64) {
        if let Some((xmin, xmax)) = x.min_max() {
            let half = width.abs() * 0.5;
            self.expand_range_x(xmin - half, xmax + half);
        }
        if let Some((hmin, hmax)) = heights.min_max() {
            let y0 = baseline + hmin.min(0.0);
            let y1 = baseline + hmax.max(0.0);
            self.expand_range_y(y0.min(baseline), y1.max(baseline), true);
        }
    }

    fn expand_barh_limits(&mut self, y: &Series, widths: &Series, height: f64, baseline: f64) {
        if let Some((ymin, ymax)) = y.min_max() {
            let half = height.abs() * 0.5;
            self.expand_range_y(ymin - half, ymax + half, false);
        }
        if let Some((wmin, wmax)) = widths.min_max() {
            let x0 = baseline + wmin.min(0.0);
            let x1 = baseline + wmax.max(0.0);
            // Sticky baseline (matplotlib): x=0 stays flush to the spine.
            self.expand_range_x_inner(x0.min(baseline), x1.max(baseline), true);
        }
    }

    fn expand_errorbar_limits(&mut self, x: &Series, y: &Series, yerr: &crate::artist::ErrBars) {
        if let Some((xmin, xmax)) = x.min_max() {
            self.expand_range_x(xmin, xmax);
        }
        let (lower, upper) = yerr.arms();
        let mut ymin = f64::INFINITY;
        let mut ymax = f64::NEG_INFINITY;
        for ((&yi, &lo), &hi) in y.as_slice().iter().zip(lower.iter()).zip(upper.iter()) {
            if yi.is_finite() && lo.is_finite() && hi.is_finite() {
                ymin = ymin.min(yi - lo.abs());
                ymax = ymax.max(yi + hi.abs());
            }
        }
        if ymin.is_finite() && ymax.is_finite() {
            self.expand_range_y(ymin, ymax, false);
        }
    }

    fn expand_errorbar_x_limits(&mut self, x: &Series, xerr: &crate::artist::ErrBars) {
        let (lower, upper) = xerr.arms();
        let mut xmin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        for ((&xi, &lo), &hi) in x.as_slice().iter().zip(lower.iter()).zip(upper.iter()) {
            if xi.is_finite() && lo.is_finite() && hi.is_finite() {
                xmin = xmin.min(xi - lo.abs());
                xmax = xmax.max(xi + hi.abs());
            }
        }
        if xmin.is_finite() && xmax.is_finite() {
            self.expand_range_x(xmin, xmax);
        }
    }

    /// Absorb limits from artists configured after the initial `expand_*` call
    /// (currently [`ErrorBarPlot::xerr`] / asym variants and [`HistPlot::bins`]).
    /// Called when an axes closure finishes.
    pub(crate) fn finalize_artist_limits(&mut self) {
        let expansions: Vec<(Series, crate::artist::ErrBars, crate::artist::ErrBars)> = self
            .elements
            .iter()
            .filter_map(|el| match el {
                PlotElement::ErrorBar(p) => Some((p.x.clone(), p.yerr.clone(), p.xerr.clone()?)),
                _ => None,
            })
            .collect();
        // Re-apply y arms too (`.yerr_asym` may run after the initial expand).
        let y_expansions: Vec<(Series, Series, crate::artist::ErrBars)> = self
            .elements
            .iter()
            .filter_map(|el| match el {
                PlotElement::ErrorBar(p) => Some((p.x.clone(), p.y.clone(), p.yerr.clone())),
                _ => None,
            })
            .collect();
        for (x, y, yerr) in y_expansions {
            self.expand_errorbar_limits(&x, &y, &yerr);
        }
        for (x, _, xerr) in expansions {
            self.expand_errorbar_x_limits(&x, &xerr);
        }
        let hist_extents: Vec<(f64, f64, f64)> = self
            .elements
            .iter()
            .filter_map(|el| match el {
                PlotElement::Hist(h) => {
                    let bins = h.compute_bins();
                    let x0 = *bins.edges.first()?;
                    let x1 = *bins.edges.last()?;
                    let ymax = bins.counts.iter().copied().fold(0.0_f64, f64::max);
                    Some((x0, x1, ymax))
                }
                _ => None,
            })
            .collect();
        for (x0, x1, ymax) in hist_extents {
            self.expand_range_x(x0, x1);
            self.expand_range_y(0.0, ymax, true);
        }
        // Heatmap / imshow: default index box or explicit extent (exact, no margins).
        let heatmap_boxes: Vec<[f64; 4]> = self
            .elements
            .iter()
            .filter_map(|el| match el {
                PlotElement::Heatmap(hm) => Some(match hm.extent {
                    Some(e) => e,
                    None => [-0.5, (hm.ncols as f64) - 0.5, -0.5, (hm.nrows as f64) - 0.5],
                }),
                _ => None,
            })
            .collect();
        for [l, r, b, t] in heatmap_boxes {
            self.expand_range_tight_x(l, r);
            self.expand_range_tight_y(b, t);
        }
        // mpl imshow(origin='upper') with default index box inverts the y-axis.
        // Explicit extent keeps upright y (row 0 still at the visual top via cell order).
        // mpl spy also places row 0 at the top.
        self.y_inverted = self.elements.iter().any(|el| {
            matches!(
                el,
                PlotElement::Heatmap(hm)
                    if hm.origin == crate::recipes::HeatmapOrigin::Upper && hm.extent.is_none()
            ) || matches!(el, PlotElement::Spy(_))
        });
        if let Some(twin) = self.twin_y.as_mut() {
            twin.finalize_artist_limits();
        }
        if let Some(twin) = self.twin_x.as_mut() {
            twin.finalize_artist_limits();
        }
        for inset in &mut self.insets {
            inset.axes.finalize_artist_limits();
        }
    }

    fn expand_range_x(&mut self, xmin: f64, xmax: f64) {
        self.expand_range_x_inner(xmin, xmax, false);
    }

    fn expand_range_x_inner(&mut self, xmin: f64, xmax: f64, include_zero: bool) {
        if self.x_lim_manual {
            return;
        }
        let lo = xmin.min(xmax);
        let hi = xmin.max(xmax);
        let include_zero = include_zero && !matches!(self.x_scale_type, ScaleType::Log);
        if !self.x_lim_seeded {
            self.x_data_min = lo;
            self.x_data_max = hi;
            self.x_pad_include_zero = include_zero;
            self.x_lim_seeded = true;
        } else {
            self.x_data_min = self.x_data_min.min(lo);
            self.x_data_max = self.x_data_max.max(hi);
            self.x_pad_include_zero |= include_zero;
        }
        let (plo, phi) = padded_limits(
            self.x_data_min,
            self.x_data_max,
            self.x_scale_type,
            self.x_pad_include_zero,
        );
        self.x_min = plo;
        self.x_max = phi;
    }

    fn expand_range_y(&mut self, ymin: f64, ymax: f64, include_zero: bool) {
        if self.y_lim_manual {
            return;
        }
        let lo = ymin.min(ymax);
        let hi = ymin.max(ymax);
        let include_zero = include_zero && !matches!(self.y_scale_type, ScaleType::Log);
        if !self.y_lim_seeded {
            self.y_data_min = lo;
            self.y_data_max = hi;
            self.y_pad_include_zero = include_zero;
            self.y_lim_seeded = true;
        } else {
            self.y_data_min = self.y_data_min.min(lo);
            self.y_data_max = self.y_data_max.max(hi);
            self.y_pad_include_zero |= include_zero;
        }
        let (plo, phi) = padded_limits(
            self.y_data_min,
            self.y_data_max,
            self.y_scale_type,
            self.y_pad_include_zero,
        );
        self.y_min = plo;
        self.y_max = phi;
    }

    fn expand_range_tight_x(&mut self, xmin: f64, xmax: f64) {
        if self.x_lim_manual {
            return;
        }
        let lo = xmin.min(xmax);
        let hi = xmin.max(xmax);
        if !self.x_lim_seeded {
            self.x_data_min = lo;
            self.x_data_max = hi;
            self.x_lim_seeded = true;
        } else {
            self.x_data_min = self.x_data_min.min(lo);
            self.x_data_max = self.x_data_max.max(hi);
        }
        // Tight: view limits == data extent (no margin).
        self.x_min = self.x_data_min;
        self.x_max = self.x_data_max;
    }

    fn expand_range_tight_y(&mut self, ymin: f64, ymax: f64) {
        if self.y_lim_manual {
            return;
        }
        let lo = ymin.min(ymax);
        let hi = ymin.max(ymax);
        if !self.y_lim_seeded {
            self.y_data_min = lo;
            self.y_data_max = hi;
            self.y_lim_seeded = true;
        } else {
            self.y_data_min = self.y_data_min.min(lo);
            self.y_data_max = self.y_data_max.max(hi);
        }
        self.y_min = self.y_data_min;
        self.y_max = self.y_data_max;
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.needs_colorbar() && self.has_twin_y() {
            return Err(PlotError::Render {
                message: "twin_y cannot be combined with a colorbar on the same axes".into(),
                suggestion:
                    "remove .colorbar(true), or drop twin_y and use a second subplot instead",
            });
        }
        if self.is_inset && self.needs_colorbar() {
            return Err(PlotError::Render {
                message: "colorbar is not supported on inset_axes".into(),
                suggestion:
                    "call .colorbar(false) on inset artists, or put the colorbar on the host",
            });
        }
        if self.secondary_y.is_some() && self.has_twin_y() {
            return Err(PlotError::Render {
                message: "secondary_y cannot be combined with twin_y on the same axes".into(),
                suggestion:
                    "use either secondary_y (transformed ticks) or twin_y (independent artists)",
            });
        }
        if self.secondary_x.is_some() && self.twin_x.is_some() {
            return Err(PlotError::Render {
                message: "secondary_x cannot be combined with twin_x on the same axes".into(),
                suggestion:
                    "use either secondary_x (transformed ticks) or twin_x (independent artists)",
            });
        }
        for el in &self.elements {
            if let Some((norm, vmin, vmax)) = artist_norm_limits(el) {
                if matches!(norm, Norm::Log) && (vmin <= 0.0 || vmax <= 0.0 || !vmin.is_finite()) {
                    return Err(PlotError::log_non_positive(vmin.min(vmax)));
                }
            }
        }
        for el in &self.elements {
            match el {
                PlotElement::Line(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                }
                PlotElement::Scatter(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                }
                PlotElement::Area(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                }
                PlotElement::Step(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                }
                PlotElement::Stem(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                }
                PlotElement::Bar(p) => {
                    if p.x.len() != p.heights.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.heights.len()));
                    }
                }
                PlotElement::BarH(p) => {
                    if p.y.len() != p.widths.len() {
                        return Err(PlotError::length_mismatch(p.y.len(), p.widths.len()));
                    }
                }
                PlotElement::FillBetween(p) => {
                    if p.x.len() != p.y1.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y1.len()));
                    }
                    if p.x.len() != p.y2.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y2.len()));
                    }
                }
                PlotElement::FillBetweenX(p) => {
                    if p.y.len() != p.x1.len() {
                        return Err(PlotError::length_mismatch(p.y.len(), p.x1.len()));
                    }
                    if p.y.len() != p.x2.len() {
                        return Err(PlotError::length_mismatch(p.y.len(), p.x2.len()));
                    }
                }
                PlotElement::Stairs(p) => {
                    if p.edges.len() != p.values.len() + 1 {
                        return Err(PlotError::length_mismatch(
                            p.edges.len(),
                            p.values.len() + 1,
                        ));
                    }
                }
                PlotElement::HLines(p) => {
                    let n = p.y.len();
                    if p.xmin.len() != 1 && p.xmin.len() != n {
                        return Err(PlotError::length_mismatch(p.xmin.len(), n));
                    }
                    if p.xmax.len() != 1 && p.xmax.len() != n {
                        return Err(PlotError::length_mismatch(p.xmax.len(), n));
                    }
                }
                PlotElement::VLines(p) => {
                    let n = p.x.len();
                    if p.ymin.len() != 1 && p.ymin.len() != n {
                        return Err(PlotError::length_mismatch(p.ymin.len(), n));
                    }
                    if p.ymax.len() != 1 && p.ymax.len() != n {
                        return Err(PlotError::length_mismatch(p.ymax.len(), n));
                    }
                }
                PlotElement::ErrorBar(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                    if p.x.len() != p.yerr.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.yerr.len()));
                    }
                    if let crate::artist::ErrBars::Asymmetric { lower, upper } = &p.yerr {
                        if lower.len() != upper.len() {
                            return Err(PlotError::length_mismatch(lower.len(), upper.len()));
                        }
                    }
                    if let Some(xerr) = &p.xerr {
                        if p.x.len() != xerr.len() {
                            return Err(PlotError::length_mismatch(p.x.len(), xerr.len()));
                        }
                        if let crate::artist::ErrBars::Asymmetric { lower, upper } = xerr {
                            if lower.len() != upper.len() {
                                return Err(PlotError::length_mismatch(lower.len(), upper.len()));
                            }
                        }
                    }
                }
                PlotElement::Polygon(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                    if p.x.len() < 3 {
                        return Err(PlotError::length_mismatch(p.x.len(), 3));
                    }
                }
                PlotElement::StackPlot(p) => {
                    for y in &p.ys {
                        if p.x.len() != y.len() {
                            return Err(PlotError::length_mismatch(p.x.len(), y.len()));
                        }
                    }
                }
                PlotElement::Hist2d(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                }
                PlotElement::Hexbin(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                }
                PlotElement::Contour(p) => {
                    let expected = p.nrows.saturating_mul(p.ncols);
                    if p.values.len() != expected {
                        return Err(PlotError::heatmap_size_mismatch(
                            p.nrows,
                            p.ncols,
                            p.values.len(),
                        ));
                    }
                }
                PlotElement::Contourf(p) => {
                    let expected = p.nrows.saturating_mul(p.ncols);
                    if p.values.len() != expected {
                        return Err(PlotError::heatmap_size_mismatch(
                            p.nrows,
                            p.ncols,
                            p.values.len(),
                        ));
                    }
                }
                PlotElement::Tripcolor(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                    if p.x.len() != p.z.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.z.len()));
                    }
                    if let Err(msg) = crate::recipes::validate_triangles(p.x.len(), &p.triangles) {
                        return Err(PlotError::Render {
                            message: msg,
                            suggestion: "call .triangles([[i, j, k], …]) with valid vertex indices",
                        });
                    }
                }
                PlotElement::Tricontour(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                    if p.x.len() != p.z.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.z.len()));
                    }
                    if let Err(msg) = crate::recipes::validate_triangles(p.x.len(), &p.triangles) {
                        return Err(PlotError::Render {
                            message: msg,
                            suggestion: "call .triangles([[i, j, k], …]) with valid vertex indices",
                        });
                    }
                }
                PlotElement::Pie(_)
                | PlotElement::EventPlot(_)
                | PlotElement::BrokenBarH(_)
                | PlotElement::Hist(_)
                | PlotElement::AxHLine(_)
                | PlotElement::AxVLine(_)
                | PlotElement::AxHSpan(_)
                | PlotElement::AxVSpan(_)
                | PlotElement::Rectangle(_)
                | PlotElement::Circle(_)
                | PlotElement::Ellipse(_)
                | PlotElement::BoxPlot(_)
                | PlotElement::Violin(_)
                | PlotElement::Text(_)
                | PlotElement::Annotate(_)
                | PlotElement::Table(_) => {}
                PlotElement::Heatmap(p) => {
                    let expected = p.nrows.saturating_mul(p.ncols);
                    if p.values.len() != expected {
                        return Err(PlotError::heatmap_size_mismatch(
                            p.nrows,
                            p.ncols,
                            p.values.len(),
                        ));
                    }
                }
                PlotElement::Spy(p) => {
                    let expected = p.nrows.saturating_mul(p.ncols);
                    if p.values.len() != expected {
                        return Err(PlotError::heatmap_size_mismatch(
                            p.nrows,
                            p.ncols,
                            p.values.len(),
                        ));
                    }
                }
                PlotElement::PcolorMesh(p) => {
                    let ncols = p.x_edges.len().saturating_sub(1);
                    let nrows = p.y_edges.len().saturating_sub(1);
                    let expected = nrows.saturating_mul(ncols);
                    if p.x_edges.len() < 2 || p.y_edges.len() < 2 {
                        return Err(PlotError::length_mismatch(p.x_edges.len(), 2));
                    }
                    if p.values.len() != expected {
                        return Err(PlotError::heatmap_size_mismatch(
                            nrows,
                            ncols,
                            p.values.len(),
                        ));
                    }
                }
                PlotElement::Quiver(p) => {
                    let n = p.x.len();
                    if p.y.len() != n {
                        return Err(PlotError::length_mismatch(p.y.len(), n));
                    }
                    if p.u.len() != n {
                        return Err(PlotError::length_mismatch(p.u.len(), n));
                    }
                    if p.v.len() != n {
                        return Err(PlotError::length_mismatch(p.v.len(), n));
                    }
                }
                PlotElement::Barbs(p) => {
                    let n = p.x.len();
                    if p.y.len() != n {
                        return Err(PlotError::length_mismatch(p.y.len(), n));
                    }
                    if p.u.len() != n {
                        return Err(PlotError::length_mismatch(p.u.len(), n));
                    }
                    if p.v.len() != n {
                        return Err(PlotError::length_mismatch(p.v.len(), n));
                    }
                }
                PlotElement::StreamPlot(p) => {
                    let expected = p.nrows.saturating_mul(p.ncols);
                    if p.u.len() != expected {
                        return Err(PlotError::heatmap_size_mismatch(
                            p.nrows,
                            p.ncols,
                            p.u.len(),
                        ));
                    }
                    if p.v.len() != expected {
                        return Err(PlotError::heatmap_size_mismatch(
                            p.nrows,
                            p.ncols,
                            p.v.len(),
                        ));
                    }
                }
                PlotElement::PolarFrame(_) => {}
                PlotElement::Tricontourf(p) => {
                    if p.x.len() != p.y.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.y.len()));
                    }
                    if p.x.len() != p.z.len() {
                        return Err(PlotError::length_mismatch(p.x.len(), p.z.len()));
                    }
                }
                PlotElement::AxLine(_) => {}
            }
        }
        if let Some(twin) = &self.twin_y {
            twin.validate()?;
        }
        if let Some(twin) = &self.twin_x {
            twin.validate()?;
        }
        for inset in &self.insets {
            inset.axes.validate()?;
        }
        Ok(())
    }

    pub(crate) fn x_scale_kind(&self) -> Result<ScaleKind> {
        ScaleKind::build(self.x_scale_type, self.x_min, self.x_max)
    }

    pub(crate) fn y_scale_kind(&self) -> Result<ScaleKind> {
        ScaleKind::build(self.y_scale_type, self.y_min, self.y_max)
    }

    /// Major ticks for the x-axis (explicit overrides, else auto / datetime).
    #[allow(dead_code)] // convenience when axis pixel size is unavailable
    pub(crate) fn major_ticks_x(&self) -> Vec<Tick> {
        self.major_ticks_x_targeted(ticks_policy::LINEAR_TARGETS)
    }

    /// Major ticks for the y-axis (explicit overrides, else auto / datetime).
    #[allow(dead_code)] // convenience when axis pixel size is unavailable
    pub(crate) fn major_ticks_y(&self) -> Vec<Tick> {
        self.major_ticks_y_targeted(ticks_policy::LINEAR_TARGETS)
    }

    /// X ticks with an explicit locator target (size-aware `AutoLocator`).
    pub(crate) fn major_ticks_x_targeted(&self, targets: usize) -> Vec<Tick> {
        if let Some(cats) = &self.x_categories {
            return category_ticks(cats);
        }
        let mut ticks = resolve_ticks(
            self.x_ticks.as_deref(),
            self.x_datetime,
            self.x_scale_kind().ok(),
            targets,
        );
        if let Some(fmt) = &self.x_tick_formatter {
            for tick in &mut ticks {
                tick.label = fmt.format(tick.value);
            }
        }
        ticks
    }

    /// Y ticks with an explicit locator target (size-aware `AutoLocator`).
    pub(crate) fn major_ticks_y_targeted(&self, targets: usize) -> Vec<Tick> {
        if let Some(cats) = &self.y_categories {
            return category_ticks(cats);
        }
        let mut ticks = resolve_ticks(
            self.y_ticks.as_deref(),
            self.y_datetime,
            self.y_scale_kind().ok(),
            targets,
        );
        if let Some(fmt) = &self.y_tick_formatter {
            for tick in &mut ticks {
                tick.label = fmt.format(tick.value);
            }
        }
        ticks
    }

    /// Minor tick values for the x-axis, derived from major ticks.
    pub(crate) fn minor_tick_values_x(&self, majors: &[Tick]) -> Vec<f64> {
        let vals: Vec<f64> = majors.iter().map(|t| t.value).collect();
        minor_from_majors(
            &vals,
            self.x_min,
            self.x_max,
            matches!(self.x_scale_type, ScaleType::Log),
        )
    }

    /// Minor tick values for the y-axis, derived from major ticks.
    pub(crate) fn minor_tick_values_y(&self, majors: &[Tick]) -> Vec<f64> {
        let vals: Vec<f64> = majors.iter().map(|t| t.value).collect();
        minor_from_majors(
            &vals,
            self.y_min,
            self.y_max,
            matches!(self.y_scale_type, ScaleType::Log),
        )
    }
}

/// Subdivide major intervals into minor tick positions (matplotlib AutoMinorLocator-ish).
fn minor_from_majors(majors: &[f64], vmin: f64, vmax: f64, log: bool) -> Vec<f64> {
    if majors.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let coincides = |v: f64| {
        majors
            .iter()
            .any(|&m| (v - m).abs() <= (v.abs().max(m.abs()).max(1.0)) * 1e-12)
    };
    let push = |out: &mut Vec<f64>, v: f64| {
        if v >= vmin && v <= vmax && !coincides(v) {
            out.push(v);
        }
    };

    if log {
        for w in majors.windows(2) {
            let lo = w[0];
            let hi = w[1];
            if !(lo.is_finite() && hi.is_finite()) || lo <= 0.0 || hi <= 0.0 || hi <= lo {
                continue;
            }
            let mut exp = lo.log10().floor() as i32;
            let end_exp = hi.log10().ceil() as i32;
            while exp < end_exp {
                let decade = 10f64.powi(exp);
                for mult in 2..=9 {
                    let v = mult as f64 * decade;
                    if v > lo && v < hi {
                        push(&mut out, v);
                    }
                }
                exp += 1;
            }
        }
    } else {
        // matplotlib AutoMinorLocator: n=5 subdivisions when the major step is "nice".
        const N: usize = 5;
        for w in majors.windows(2) {
            let a = w[0];
            let b = w[1];
            if !(a.is_finite() && b.is_finite()) || b <= a {
                continue;
            }
            let step = (b - a) / N as f64;
            for i in 1..N {
                push(&mut out, a + step * i as f64);
            }
        }
    }
    out
}

/// Category data positions `0..n` (matplotlib string-category / `UnitData` indexing).
///
/// Boxplot/violin keep their own `1..=n` positions; use this helper with
/// [`Axes::x_categories`] / [`Axes::y_categories`] + `bar` / `barh`.
pub fn category_indices(n: usize) -> Vec<f64> {
    (0..n).map(|i| i as f64).collect()
}

/// Discrete colorbar edges for listed colormaps (matplotlib `ListedColormap`).
fn listed_colorbar_boundaries(cmap: &Cmap, vmin: f64, vmax: f64) -> Option<Vec<f64>> {
    let n = cmap.listed_len()?.max(1);
    Some(
        (0..=n)
            .map(|i| vmin + (vmax - vmin) * (i as f64) / n as f64)
            .collect(),
    )
}

fn category_ticks(labels: &[String]) -> Vec<Tick> {
    labels
        .iter()
        .enumerate()
        .map(|(i, label)| Tick {
            value: i as f64,
            label: label.clone(),
        })
        .collect()
}

fn artist_norm_limits(el: &PlotElement) -> Option<(Norm, f64, f64)> {
    match el {
        PlotElement::Heatmap(hm) => {
            let (vmin, vmax) = hm.value_limits();
            Some((hm.norm, vmin, vmax))
        }
        PlotElement::Hist2d(h) => {
            let (vmin, vmax) = h.value_limits();
            Some((h.norm, vmin, vmax))
        }
        PlotElement::Hexbin(h) => {
            let (vmin, vmax) = h.value_limits();
            Some((h.norm, vmin, vmax))
        }
        PlotElement::Contourf(h) => {
            let (vmin, vmax) = h.value_limits();
            Some((h.norm, vmin, vmax))
        }
        PlotElement::PcolorMesh(h) => {
            let (vmin, vmax) = h.value_limits();
            Some((h.norm, vmin, vmax))
        }
        PlotElement::Tripcolor(h) => {
            let (vmin, vmax) = h.value_limits();
            Some((h.norm, vmin, vmax))
        }
        _ => None,
    }
}

fn resolve_ticks(
    custom: Option<&[f64]>,
    datetime: bool,
    scale: Option<ScaleKind>,
    targets: usize,
) -> Vec<Tick> {
    if let Some(values) = custom {
        return plotine_core::ticks_from_values(values);
    }
    let Some(scale) = scale else {
        return Vec::new();
    };
    if datetime {
        let (min, max) = scale.domain();
        DatetimeLocator::default().ticks(min, max)
    } else {
        TickLocator::new(targets).ticks(scale)
    }
}

fn padded_limits(min: f64, max: f64, scale: ScaleType, include_zero: bool) -> (f64, f64) {
    // Policy: [`crate::mpl_policy::margin::PAD`] (matplotlib `ax.margins`).
    let pad_frac = margin_policy::PAD;
    let mut min = min;
    let mut max = max;
    if include_zero {
        min = min.min(0.0);
        max = max.max(0.0);
    }
    match scale {
        ScaleType::Log => {
            if min <= 0.0 {
                min = max.max(1e-6) * 1e-3;
            }
            if max <= min {
                max = min * 10.0;
            }
            let log_span = (max.log10() - min.log10()).max(1e-12);
            let pad = log_span * pad_frac;
            (10f64.powf(min.log10() - pad), 10f64.powf(max.log10() + pad))
        }
        _ => {
            let span = (max - min).abs().max(1e-12);
            let pad = span * pad_frac;
            // Matplotlib sticky edges: if the limit sits on 0, do not pad past it.
            let lo = if include_zero && min == 0.0 {
                0.0
            } else {
                min - pad
            };
            let hi = if include_zero && max == 0.0 {
                0.0
            } else {
                max + pad
            };
            (lo, hi)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sticky_zero_does_not_pad_past_baseline() {
        let (lo, hi) = padded_limits(0.0, 7.0, ScaleType::Linear, true);
        assert_eq!(lo, 0.0);
        assert!((hi - 7.35).abs() < 1e-12, "hi={hi}");
    }

    #[test]
    fn barh_sticky_zero_survives_hlines() {
        // Regression: hlines expanded x with non-sticky padding and pushed
        // xlim left of 0, leaving a gap between barh and the y spine.
        let mut ax = Axes::new();
        ax.barh([1.0, 2.0, 3.0], [4.0, 7.0, 2.5]);
        ax.hlines([2.5], 0.0, 8.0);
        assert!(
            ax.x_min.abs() < 1e-12,
            "x_min={} (expected sticky 0)",
            ax.x_min
        );
        assert!(ax.x_pad_include_zero);
    }

    #[test]
    fn tricontour_tightens_limits_like_mpl_sticky_edges() {
        // tripcolor alone keeps 5% margins; tricontour ContourSet sticky
        // edges snap 0 onto the spine.
        let mut ax = Axes::new();
        let tx = [0.0, 1.0, 2.0, 0.5, 1.5, 1.0];
        let ty = [0.0, 0.0, 0.0, 0.9, 0.9, 1.6];
        let tz = [0.0, 0.4, 0.1, 0.8, 1.0, 0.6];
        ax.tripcolor(tx, ty, tz);
        assert!(
            ax.y_min < -1e-6,
            "expected margin below 0, got {}",
            ax.y_min
        );
        ax.tricontour(tx, ty, tz);
        assert!(
            ax.y_min.abs() < 1e-12,
            "tricontour should sticky-tighten y_min to 0, got {}",
            ax.y_min
        );
        assert!((ax.y_max - 1.6).abs() < 1e-12);
        assert!(ax.x_min.abs() < 1e-12);
        assert!((ax.x_max - 2.0).abs() < 1e-12);
    }

    #[test]
    fn hist_bins_finalize_uses_final_bin_count() {
        // Default hist() used 10 bins (peak 43); `.bins(12)` peaks at 35.
        // Limits must follow the final bin count, not the default preview.
        let data: Vec<f64> = (0..200)
            .map(|i| {
                let t = i as f64 / 40.0;
                (t * 0.7).sin() + 0.15 * ((i % 17) as f64 / 17.0)
            })
            .collect();
        let mut ax = Axes::new();
        ax.hist(&data).bins(12);
        ax.finalize_artist_limits();
        assert!(
            (ax.y_data_max - 35.0).abs() < 1e-9,
            "y_data_max={} (stale 10-bin peak?)",
            ax.y_data_max
        );
        assert!(
            (ax.y_max - 36.75).abs() < 1e-9,
            "y_max={} (expected 35 * 1.05 pad)",
            ax.y_max
        );
    }

    #[test]
    fn non_sticky_pads_both_sides() {
        let (lo, hi) = padded_limits(1.0, 3.0, ScaleType::Linear, false);
        assert!((lo - 0.9).abs() < 1e-12, "lo={lo}");
        assert!((hi - 3.1).abs() < 1e-12, "hi={hi}");
    }

    #[test]
    fn broken_barh_autoscales_full_x_span() {
        // Regression: per-segment expand while `elements` was empty used to keep
        // only the last xrange, clipping the first bar (compare Broken BarH).
        let mut ax = Axes::new();
        ax.broken_barh([(10.0, 50.0), (100.0, 20.0), (150.0, 40.0)], (20.0, 9.0));
        ax.broken_barh([(40.0, 30.0), (120.0, 50.0)], (35.0, 9.0));
        assert!((ax.x_min - 1.0).abs() < 1e-9, "x_min={}", ax.x_min);
        assert!((ax.x_max - 199.0).abs() < 1e-9, "x_max={}", ax.x_max);
        assert!((ax.y_min - 18.8).abs() < 1e-9, "y_min={}", ax.y_min);
        assert!((ax.y_max - 45.2).abs() < 1e-9, "y_max={}", ax.y_max);
    }

    #[test]
    fn heatmap_extent_replaces_index_box() {
        // mpl imshow(extent=[0,10,0,4]) → xlim/ylim exactly that box (no −0.5 union).
        let mut ax = Axes::new();
        ax.heatmap(4, 4, [0.0; 16]).extent([0.0, 10.0, 0.0, 4.0]);
        ax.finalize_artist_limits();
        assert!((ax.x_min - 0.0).abs() < 1e-12, "x_min={}", ax.x_min);
        assert!((ax.x_max - 10.0).abs() < 1e-12, "x_max={}", ax.x_max);
        assert!((ax.y_min - 0.0).abs() < 1e-12, "y_min={}", ax.y_min);
        assert!((ax.y_max - 4.0).abs() < 1e-12, "y_max={}", ax.y_max);
        assert!(!ax.y_inverted, "explicit extent keeps upright y");
    }

    #[test]
    fn heatmap_default_upper_inverts_y() {
        let mut ax = Axes::new();
        ax.heatmap(2, 5, [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        ax.finalize_artist_limits();
        assert!(ax.y_inverted);
        assert!((ax.y_min - (-0.5)).abs() < 1e-12);
        assert!((ax.y_max - 1.5).abs() < 1e-12);
    }

    #[test]
    fn minor_from_majors_linear_n5() {
        let minors = minor_from_majors(&[0.0, 1.0], 0.0, 1.0, false);
        assert_eq!(minors.len(), 4);
        for (got, want) in minors.iter().zip([0.2, 0.4, 0.6, 0.8]) {
            assert!((got - want).abs() < 1e-12, "got={got} want={want}");
        }
    }
}
