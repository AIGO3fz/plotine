//! Mutable plot artists returned by Axes plotting methods.

use plotine_core::{Cmap, Color, Norm};
use plotine_render::{TextAlign, TextBaseline};

use crate::recipes::{HistogramBins, StepMode, TableLoc};
use crate::series::{IntoSeries, Series};
use crate::style::{Hatch, LineStyle, MarkerStyle};

/// Common properties shared by all artist types.
pub(crate) trait ArtistProps {
    fn explicit_color(&self) -> Option<Color>;
    fn color_index(&self) -> usize;
    fn label(&self) -> Option<&str>;
    fn legend_kind(&self) -> LegendKind;

    fn resolved_color(&self, cycle: &[Color]) -> Color {
        self.explicit_color()
            .unwrap_or_else(|| cycle[self.color_index() % cycle.len()])
    }
}

/// Implement ArtistProps for a struct with the standard fields.
macro_rules! impl_artist_props {
    ($ty:ty, $kind:expr) => {
        impl ArtistProps for $ty {
            fn explicit_color(&self) -> Option<Color> {
                self.color
            }
            fn color_index(&self) -> usize {
                self.color_index
            }
            fn label(&self) -> Option<&str> {
                self.label.as_deref()
            }
            fn legend_kind(&self) -> LegendKind {
                $kind
            }
        }
    };
}

#[derive(Debug, Clone)]
pub(crate) enum PlotElement {
    Line(LinePlot),
    Scatter(ScatterPlot),
    Bar(BarPlot),
    BarH(BarHPlot),
    Hist(HistPlot),
    Area(AreaPlot),
    FillBetween(FillBetweenPlot),
    FillBetweenX(FillBetweenXPlot),
    Step(StepPlot),
    Stairs(StairsPlot),
    Stem(StemPlot),
    HLines(HLinesPlot),
    VLines(VLinesPlot),
    AxHLine(AxHLinePlot),
    AxVLine(AxVLinePlot),
    AxHSpan(AxHSpanPlot),
    AxVSpan(AxVSpanPlot),
    Polygon(PolygonPlot),
    Rectangle(RectanglePlot),
    Circle(CirclePlot),
    Ellipse(EllipsePlot),
    Pie(PiePlot),
    StackPlot(StackPlot),
    EventPlot(EventPlot),
    BrokenBarH(BrokenBarHPlot),
    ErrorBar(ErrorBarPlot),
    Heatmap(HeatmapPlot),
    Hist2d(Hist2dPlot),
    Hexbin(HexbinPlot),
    Contour(ContourPlot),
    Contourf(ContourfPlot),
    Tripcolor(TripcolorPlot),
    Tricontour(TricontourPlot),
    Tricontourf(TricontourfPlot),
    PcolorMesh(PcolorMeshPlot),
    Spy(SpyPlot),
    Quiver(QuiverPlot),
    Barbs(BarbsPlot),
    StreamPlot(StreamPlot),
    PolarFrame(PolarFramePlot),
    BoxPlot(BoxPlot),
    Violin(ViolinPlot),
    Text(TextPlot),
    Annotate(AnnotatePlot),
    Table(TablePlot),
    AxLine(AxLinePlot),
}

/// Shared colorbar parameters for heatmap-like artists.
#[derive(Debug, Clone)]
pub(crate) struct ColorbarSpec {
    pub cmap: Cmap,
    pub vmin: f64,
    pub vmax: f64,
    pub norm: Norm,
    /// Contourf / BoundaryNorm level edges (matplotlib discrete colorbar).
    pub boundaries: Option<Vec<f64>>,
}

impl PlotElement {
    pub(crate) fn resolved_color(&self, cycle: &[Color]) -> Color {
        self.as_props().resolved_color(cycle)
    }

    pub(crate) fn label(&self) -> Option<&str> {
        self.as_props().label()
    }

    pub(crate) fn legend_kind(&self) -> LegendKind {
        self.as_props().legend_kind()
    }

    /// Legend rows for this artist (pie/stack/event may emit multiple).
    pub(crate) fn legend_items(&self, cycle: &[Color]) -> Vec<(String, Color, LegendKind)> {
        match self {
            Self::Pie(p) => {
                if p.labels.is_empty() {
                    return self.single_legend(cycle);
                }
                p.labels
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        let color = cycle[(p.color_index + i) % cycle.len()];
                        (label.clone(), color, LegendKind::Patch)
                    })
                    .collect()
            }
            Self::StackPlot(p) => {
                if p.labels.is_empty() {
                    return self.single_legend(cycle);
                }
                p.labels
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        let color = cycle[(p.color_index + i) % cycle.len()].with_alpha(p.alpha);
                        (label.clone(), color, LegendKind::Patch)
                    })
                    .collect()
            }
            Self::EventPlot(p) => {
                if p.labels.is_empty() {
                    return self.single_legend(cycle);
                }
                p.labels
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        let color = cycle[(p.color_index + i) % cycle.len()];
                        (label.clone(), color, LegendKind::Line(LineStyle::Solid))
                    })
                    .collect()
            }
            _ => self.single_legend(cycle),
        }
    }

    fn single_legend(&self, cycle: &[Color]) -> Vec<(String, Color, LegendKind)> {
        match self.label() {
            Some(label) => {
                let kind = match self {
                    Self::Line(p) => LegendKind::Line(p.linestyle),
                    Self::Step(p) => LegendKind::Line(p.linestyle),
                    Self::AxHLine(p) => LegendKind::Line(p.linestyle),
                    Self::AxVLine(p) => LegendKind::Line(p.linestyle),
                    Self::AxLine(p) => LegendKind::Line(p.linestyle),
                    _ => self.legend_kind(),
                };
                // Patch fills draw with artist alpha; legend must match the ink
                // (matplotlib PolyCollection legend handle), not opaque RGB.
                let color = match self {
                    Self::Area(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::FillBetween(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::FillBetweenX(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::AxHSpan(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::AxVSpan(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::StackPlot(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::Polygon(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::Rectangle(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::Circle(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    Self::Ellipse(p) => self.resolved_color(cycle).with_alpha(p.alpha),
                    _ => self.resolved_color(cycle),
                };
                vec![(label.to_string(), color, kind)]
            }
            None => Vec::new(),
        }
    }

    fn as_props(&self) -> &dyn ArtistProps {
        match self {
            Self::Line(p) => p,
            Self::Scatter(p) => p,
            Self::Bar(p) => p,
            Self::BarH(p) => p,
            Self::Hist(p) => p,
            Self::Area(p) => p,
            Self::FillBetween(p) => p,
            Self::FillBetweenX(p) => p,
            Self::Step(p) => p,
            Self::Stairs(p) => p,
            Self::Stem(p) => p,
            Self::HLines(p) => p,
            Self::VLines(p) => p,
            Self::AxHLine(p) => p,
            Self::AxVLine(p) => p,
            Self::AxHSpan(p) => p,
            Self::AxVSpan(p) => p,
            Self::Polygon(p) => p,
            Self::Rectangle(p) => p,
            Self::Circle(p) => p,
            Self::Ellipse(p) => p,
            Self::Pie(p) => p,
            Self::StackPlot(p) => p,
            Self::EventPlot(p) => p,
            Self::BrokenBarH(p) => p,
            Self::ErrorBar(p) => p,
            Self::Heatmap(p) => p,
            Self::Hist2d(p) => p,
            Self::Hexbin(p) => p,
            Self::Contour(p) => p,
            Self::Contourf(p) => p,
            Self::Tripcolor(p) => p,
            Self::Tricontour(p) => p,
            Self::Tricontourf(p) => p,
            Self::PcolorMesh(p) => p,
            Self::Spy(p) => p,
            Self::Quiver(p) => p,
            Self::Barbs(p) => p,
            Self::StreamPlot(p) => p,
            Self::PolarFrame(p) => p,
            Self::BoxPlot(p) => p,
            Self::Violin(p) => p,
            Self::Text(p) => p,
            Self::Annotate(p) => p,
            Self::Table(p) => p,
            Self::AxLine(p) => p,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegendKind {
    Line(LineStyle),
    Marker,
    /// Line + marker + error-bar cross (matplotlib `ErrorbarContainer` handle).
    ErrorBar,
    Patch,
}

impl_artist_props!(LinePlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(ScatterPlot, LegendKind::Marker);
impl_artist_props!(BarPlot, LegendKind::Patch);
impl_artist_props!(BarHPlot, LegendKind::Patch);
impl_artist_props!(HistPlot, LegendKind::Patch);
impl_artist_props!(AreaPlot, LegendKind::Patch);
impl_artist_props!(FillBetweenPlot, LegendKind::Patch);
impl_artist_props!(FillBetweenXPlot, LegendKind::Patch);
impl_artist_props!(StepPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(StairsPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(StemPlot, LegendKind::Marker);
impl_artist_props!(HLinesPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(VLinesPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(AxHLinePlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(AxVLinePlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(AxHSpanPlot, LegendKind::Patch);
impl_artist_props!(AxVSpanPlot, LegendKind::Patch);
impl_artist_props!(PolygonPlot, LegendKind::Patch);
impl_artist_props!(RectanglePlot, LegendKind::Patch);
impl_artist_props!(CirclePlot, LegendKind::Patch);
impl_artist_props!(EllipsePlot, LegendKind::Patch);
impl_artist_props!(PiePlot, LegendKind::Patch);
impl_artist_props!(StackPlot, LegendKind::Patch);
impl_artist_props!(EventPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(BrokenBarHPlot, LegendKind::Patch);
impl_artist_props!(ErrorBarPlot, LegendKind::ErrorBar);
impl_artist_props!(BoxPlot, LegendKind::Patch);
impl_artist_props!(ViolinPlot, LegendKind::Patch);

impl ArtistProps for HeatmapPlot {
    fn explicit_color(&self) -> Option<Color> {
        None
    }
    fn color_index(&self) -> usize {
        0
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn legend_kind(&self) -> LegendKind {
        LegendKind::Patch
    }
}

impl ArtistProps for Hist2dPlot {
    fn explicit_color(&self) -> Option<Color> {
        None
    }
    fn color_index(&self) -> usize {
        0
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn legend_kind(&self) -> LegendKind {
        LegendKind::Patch
    }
}

impl ArtistProps for HexbinPlot {
    fn explicit_color(&self) -> Option<Color> {
        None
    }
    fn color_index(&self) -> usize {
        0
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn legend_kind(&self) -> LegendKind {
        LegendKind::Patch
    }
}

impl ArtistProps for ContourfPlot {
    fn explicit_color(&self) -> Option<Color> {
        None
    }
    fn color_index(&self) -> usize {
        0
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn legend_kind(&self) -> LegendKind {
        LegendKind::Patch
    }
}

impl ArtistProps for TripcolorPlot {
    fn explicit_color(&self) -> Option<Color> {
        None
    }
    fn color_index(&self) -> usize {
        0
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn legend_kind(&self) -> LegendKind {
        LegendKind::Patch
    }
}

impl_artist_props!(TricontourPlot, LegendKind::Line(LineStyle::Solid));

impl ArtistProps for TricontourfPlot {
    fn explicit_color(&self) -> Option<Color> {
        None
    }
    fn color_index(&self) -> usize {
        0
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn legend_kind(&self) -> LegendKind {
        LegendKind::Patch
    }
}

impl ArtistProps for PcolorMeshPlot {
    fn explicit_color(&self) -> Option<Color> {
        None
    }
    fn color_index(&self) -> usize {
        0
    }
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    fn legend_kind(&self) -> LegendKind {
        LegendKind::Patch
    }
}

impl_artist_props!(ContourPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(SpyPlot, LegendKind::Marker);
impl_artist_props!(QuiverPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(BarbsPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(StreamPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(PolarFramePlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(TextPlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(AnnotatePlot, LegendKind::Line(LineStyle::Solid));
impl_artist_props!(TablePlot, LegendKind::Patch);

/// A line series with builder-style styling.
#[derive(Debug, Clone)]
pub struct LinePlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) linestyle: LineStyle,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl LinePlot {
    /// Set the stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the stroke width in points (scaled by figure DPI).
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Set the dash pattern (`'-'`, `'--'`, `':'`, `'-.'`).
    pub fn linestyle(&mut self, style: LineStyle) -> &mut Self {
        self.linestyle = style;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    /// Replace y values (matplotlib `line.set_ydata`). Length must match `x`.
    ///
    /// Does **not** auto-rescale axes — set `y_range` before animating if needed.
    pub fn set_y<Y: crate::series::IntoSeries>(&mut self, y: Y) -> plotine_core::Result<&mut Self> {
        let y = y.into_series();
        if y.len() != self.x.len() {
            return Err(plotine_core::PlotError::length_mismatch(
                self.x.len(),
                y.len(),
            ));
        }
        self.y = y;
        Ok(self)
    }

    /// Replace x and y (matplotlib `line.set_data`). Lengths must match.
    pub fn set_xy<X, Y>(&mut self, x: X, y: Y) -> plotine_core::Result<&mut Self>
    where
        X: crate::series::IntoSeries,
        Y: crate::series::IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        if x.len() != y.len() {
            return Err(plotine_core::PlotError::length_mismatch(x.len(), y.len()));
        }
        self.x = x;
        self.y = y;
        Ok(self)
    }
}

/// A scatter series with builder-style styling.
#[derive(Debug, Clone)]
pub struct ScatterPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) color: Option<Color>,
    pub(crate) size: f64,
    pub(crate) marker: MarkerStyle,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl ScatterPlot {
    /// Set the marker fill color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Marker diameter in points (scaled by figure DPI).
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = size.max(0.5);
        self
    }

    /// Marker shape (`'o'`, `'s'`, `'^'`, `'+'`, …).
    pub fn marker(&mut self, style: MarkerStyle) -> &mut Self {
        self.marker = style;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    /// Replace y values. Length must match `x`. Does not auto-rescale axes.
    pub fn set_y<Y: crate::series::IntoSeries>(&mut self, y: Y) -> plotine_core::Result<&mut Self> {
        let y = y.into_series();
        if y.len() != self.x.len() {
            return Err(plotine_core::PlotError::length_mismatch(
                self.x.len(),
                y.len(),
            ));
        }
        self.y = y;
        Ok(self)
    }

    /// Replace x and y. Lengths must match. Does not auto-rescale axes.
    pub fn set_xy<X, Y>(&mut self, x: X, y: Y) -> plotine_core::Result<&mut Self>
    where
        X: crate::series::IntoSeries,
        Y: crate::series::IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        if x.len() != y.len() {
            return Err(plotine_core::PlotError::length_mismatch(x.len(), y.len()));
        }
        self.x = x;
        self.y = y;
        Ok(self)
    }
}

/// Vertical bars centered on categorical/numeric x positions.
#[derive(Debug, Clone)]
pub struct BarPlot {
    pub(crate) x: Series,
    pub(crate) heights: Series,
    /// Relative width in `(0, 1]` of the median x-spacing (default 0.8).
    pub(crate) width: f64,
    pub(crate) baseline: f64,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) hatch: Hatch,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl BarPlot {
    /// Set the bar fill color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the bar edge (stroke) color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Fill hatch pattern (matplotlib `hatch=`).
    pub fn hatch(&mut self, hatch: Hatch) -> &mut Self {
        self.hatch = hatch;
        self
    }

    /// Relative bar width in `(0, 1]` of the median x-spacing.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.clamp(0.05, 1.0);
        self
    }

    /// Y-value from which bars grow (default `0.0`).
    pub fn baseline(&mut self, baseline: f64) -> &mut Self {
        self.baseline = baseline;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Histogram of a 1D sample.
#[derive(Debug, Clone)]
pub struct HistPlot {
    pub(crate) data: Series,
    pub(crate) bin_count: usize,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) hatch: Hatch,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl HistPlot {
    /// Set the bin fill color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the bin edge (stroke) color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Fill hatch pattern (matplotlib `hatch=`).
    pub fn hatch(&mut self, hatch: Hatch) -> &mut Self {
        self.hatch = hatch;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    /// Number of equal-width bins (default 10).
    pub fn bins(&mut self, n: usize) -> &mut Self {
        self.bin_count = n.max(1);
        self
    }

    pub(crate) fn compute_bins(&self) -> HistogramBins {
        crate::recipes::histogram(self.data.as_slice(), self.bin_count)
    }
}

/// Filled area under a curve.
#[derive(Debug, Clone)]
pub struct AreaPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) baseline: f64,
    pub(crate) color: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl AreaPlot {
    /// Set the fill/stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.35`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Y-value the area is filled down to (default `0.0`).
    pub fn baseline(&mut self, baseline: f64) -> &mut Self {
        self.baseline = baseline;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Fill between two curves sharing the same `x`.
#[derive(Debug, Clone)]
pub struct FillBetweenPlot {
    pub(crate) x: Series,
    pub(crate) y1: Series,
    pub(crate) y2: Series,
    pub(crate) color: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl FillBetweenPlot {
    /// Set the fill color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.35`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Fill between two vertical curves sharing the same `y`.
#[derive(Debug, Clone)]
pub struct FillBetweenXPlot {
    pub(crate) y: Series,
    pub(crate) x1: Series,
    pub(crate) x2: Series,
    pub(crate) color: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl FillBetweenXPlot {
    /// Set the fill color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.35`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Step plot of `(x, y)`.
#[derive(Debug, Clone)]
pub struct StepPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) mode: StepMode,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) linestyle: LineStyle,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl StepPlot {
    /// Set the stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Set the dash pattern (`'-'`, `'--'`, `':'`, `'-.'`).
    pub fn linestyle(&mut self, style: LineStyle) -> &mut Self {
        self.linestyle = style;
        self
    }

    /// Step placement (`Pre` / `Mid` / `Post`).
    pub fn mode(&mut self, mode: StepMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Stairs plot: constant values between bin `edges`.
#[derive(Debug, Clone)]
pub struct StairsPlot {
    pub(crate) edges: Series,
    pub(crate) values: Series,
    pub(crate) baseline: f64,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl StairsPlot {
    /// Set the stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Y-value of the stairs baseline (matplotlib default `0.0`).
    ///
    /// The path drops vertically to this value at the first and last edge.
    /// Pass `f64::NAN` to omit the baseline drops (open staircase).
    pub fn baseline(&mut self, baseline: f64) -> &mut Self {
        self.baseline = baseline;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Stem plot: vertical lines from baseline to markers.
#[derive(Debug, Clone)]
pub struct StemPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) baseline: f64,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) marker_size: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl StemPlot {
    /// Set the stem/marker color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stem stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Head marker radius in points.
    pub fn marker_size(&mut self, size: f64) -> &mut Self {
        self.marker_size = size.max(0.5);
        self
    }

    /// Y-value of the stem baseline (default `0.0`).
    pub fn baseline(&mut self, baseline: f64) -> &mut Self {
        self.baseline = baseline;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Horizontal line segments at given y positions.
#[derive(Debug, Clone)]
pub struct HLinesPlot {
    pub(crate) y: Series,
    pub(crate) xmin: Series,
    pub(crate) xmax: Series,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl HLinesPlot {
    /// Set the stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Vertical line segments at given x positions.
#[derive(Debug, Clone)]
pub struct VLinesPlot {
    pub(crate) x: Series,
    pub(crate) ymin: Series,
    pub(crate) ymax: Series,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl VLinesPlot {
    /// Set the stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Horizontal line spanning the full x-domain at a fixed y.
#[derive(Debug, Clone)]
pub struct AxHLinePlot {
    pub(crate) y: f64,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) linestyle: LineStyle,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl AxHLinePlot {
    /// Set the stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Set the dash pattern (`'-'`, `'--'`, `':'`, `'-.'`).
    pub fn linestyle(&mut self, style: LineStyle) -> &mut Self {
        self.linestyle = style;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Vertical line spanning the full y-domain at a fixed x.
#[derive(Debug, Clone)]
pub struct AxVLinePlot {
    pub(crate) x: f64,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) linestyle: LineStyle,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl AxVLinePlot {
    /// Set the stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Set the dash pattern (`'-'`, `'--'`, `':'`, `'-.'`).
    pub fn linestyle(&mut self, style: LineStyle) -> &mut Self {
        self.linestyle = style;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Horizontal bars centered on `y` with given widths along x.
#[derive(Debug, Clone)]
pub struct BarHPlot {
    pub(crate) y: Series,
    pub(crate) widths: Series,
    pub(crate) height: f64,
    pub(crate) baseline: f64,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) hatch: Hatch,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl BarHPlot {
    /// Set the bar fill color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the bar edge (stroke) color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Fill hatch pattern (matplotlib `hatch=`).
    pub fn hatch(&mut self, hatch: Hatch) -> &mut Self {
        self.hatch = hatch;
        self
    }

    /// Relative bar height in `(0, 1]` of the median y-spacing.
    pub fn height(&mut self, height: f64) -> &mut Self {
        self.height = height.clamp(0.05, 1.0);
        self
    }

    /// X-value from which bars grow (default `0.0`).
    pub fn baseline(&mut self, baseline: f64) -> &mut Self {
        self.baseline = baseline;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Symmetric or asymmetric error magnitudes (matplotlib `yerr` / `xerr`).
///
/// Asymmetric form matches `yerr` of shape `(2, N)`: `lower` is the downward /
/// leftward arm, `upper` the upward / rightward arm (both non-negative).
#[derive(Debug, Clone)]
pub enum ErrBars {
    /// `value ± err` (1-D matplotlib `yerr` / `xerr`).
    Symmetric(Series),
    /// Independent lower/upper arms (2×N matplotlib `yerr` / `xerr`).
    Asymmetric {
        /// Downward / leftward error magnitudes (non-negative).
        lower: Series,
        /// Upward / rightward error magnitudes (non-negative).
        upper: Series,
    },
}

impl ErrBars {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Symmetric(s) => s.len(),
            Self::Asymmetric { lower, .. } => lower.len(),
        }
    }

    pub(crate) fn arms(&self) -> (&[f64], &[f64]) {
        match self {
            Self::Symmetric(s) => (s.as_slice(), s.as_slice()),
            Self::Asymmetric { lower, upper } => (lower.as_slice(), upper.as_slice()),
        }
    }
}

/// Points with error bars (`y ± yerr`, optional `x ± xerr`).
#[derive(Debug, Clone)]
pub struct ErrorBarPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) yerr: ErrBars,
    /// Optional horizontal errors (matplotlib `xerr=`). Absorbed into x-limits at
    /// the end of the [`crate::Axes`] configuration closure.
    pub(crate) xerr: Option<ErrBars>,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) capsize: f64,
    /// Draw a polyline through the central points (matplotlib `fmt='o-'` style).
    pub(crate) connect: bool,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl ErrorBarPlot {
    /// Replace vertical errors with asymmetric arms (matplotlib `yerr=(lower, upper)`).
    ///
    /// `lower` / `upper` are non-negative magnitudes: bar spans
    /// `[y - lower, y + upper]`.
    pub fn yerr_asym<L, U>(&mut self, lower: L, upper: U) -> &mut Self
    where
        L: IntoSeries,
        U: IntoSeries,
    {
        self.yerr = ErrBars::Asymmetric {
            lower: lower.into_series(),
            upper: upper.into_series(),
        };
        self
    }

    /// Horizontal error bars (`x ± xerr`), matching matplotlib `errorbar(..., xerr=...)`.
    ///
    /// Auto x-limits are updated when the enclosing `axes` / subplot closure finishes.
    pub fn xerr<E: IntoSeries>(&mut self, xerr: E) -> &mut Self {
        self.xerr = Some(ErrBars::Symmetric(xerr.into_series()));
        self
    }

    /// Asymmetric horizontal errors (matplotlib `xerr=(lower, upper)`).
    pub fn xerr_asym<L, U>(&mut self, lower: L, upper: U) -> &mut Self
    where
        L: IntoSeries,
        U: IntoSeries,
    {
        self.xerr = Some(ErrBars::Asymmetric {
            lower: lower.into_series(),
            upper: upper.into_series(),
        });
        self
    }

    /// Set the marker/cap color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width of the error bars in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Cap half-width in points (scaled by figure DPI).
    pub fn capsize(&mut self, capsize: f64) -> &mut Self {
        self.capsize = capsize.max(0.0);
        self
    }

    /// Connect central markers with a line (default `true`, like matplotlib `fmt='o-'`).
    pub fn connect(&mut self, connect: bool) -> &mut Self {
        self.connect = connect;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// 2D grid heatmap (`values` are row-major, length `nrows * ncols`).
#[derive(Debug, Clone)]
pub struct HeatmapPlot {
    pub(crate) nrows: usize,
    pub(crate) ncols: usize,
    pub(crate) values: Series,
    pub(crate) cmap: Cmap,
    pub(crate) vmin: Option<f64>,
    pub(crate) vmax: Option<f64>,
    pub(crate) norm: Norm,
    pub(crate) origin: crate::recipes::HeatmapOrigin,
    /// Optional data-space extent `[left, right, bottom, top]` (matplotlib `imshow(extent=…)`).
    pub(crate) extent: Option<[f64; 4]>,
    pub(crate) alpha: f64,
    pub(crate) colorbar: bool,
    pub(crate) label: Option<String>,
}

impl HeatmapPlot {
    /// Colormap used to map values to colors (default Viridis).
    pub fn cmap(&mut self, cmap: impl Into<Cmap>) -> &mut Self {
        self.cmap = cmap.into();
        self
    }

    /// Row-0 placement: [`HeatmapOrigin::Upper`](crate::HeatmapOrigin::Upper)
    /// (default, like `imshow`) or `Lower`.
    pub fn origin(&mut self, origin: crate::recipes::HeatmapOrigin) -> &mut Self {
        self.origin = origin;
        self
    }

    /// Data-coordinate bounding box `[left, right, bottom, top]` (matplotlib `extent=`).
    ///
    /// When set, cells are mapped into this rectangle instead of the default
    /// integer-index centers `(-0.5 … ncols-0.5, …)`.
    pub fn extent(&mut self, extent: [f64; 4]) -> &mut Self {
        self.extent = Some(extent);
        self
    }

    /// Cell fill opacity in `0.0..=1.0` (default `1.0`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Value normalization before colormap sampling (default [`Norm::Linear`]).
    pub fn norm(&mut self, norm: Norm) -> &mut Self {
        self.norm = norm;
        self
    }

    /// Lower end of the colormap scale (default: data minimum).
    pub fn vmin(&mut self, vmin: f64) -> &mut Self {
        self.vmin = Some(vmin);
        self
    }

    /// Upper end of the colormap scale (default: data maximum).
    pub fn vmax(&mut self, vmax: f64) -> &mut Self {
        self.vmax = Some(vmax);
        self
    }

    /// Show a colorbar strip to the right of the axes (default `true`).
    pub fn colorbar(&mut self, show: bool) -> &mut Self {
        self.colorbar = show;
        self
    }

    /// Optional legend label (rarely used for heatmaps).
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn value_limits(&self) -> (f64, f64) {
        crate::recipes::heatmap_limits(self.values.as_slice(), self.vmin, self.vmax)
    }
}

/// 2D histogram of `(x, y)` samples.
#[derive(Debug, Clone)]
pub struct Hist2dPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) bins_x: usize,
    pub(crate) bins_y: usize,
    pub(crate) cmap: Cmap,
    pub(crate) vmin: Option<f64>,
    pub(crate) vmax: Option<f64>,
    pub(crate) norm: Norm,
    pub(crate) colorbar: bool,
    pub(crate) label: Option<String>,
}

impl Hist2dPlot {
    /// Number of bins along both axes.
    pub fn bins(&mut self, n: usize) -> &mut Self {
        let n = n.max(1);
        self.bins_x = n;
        self.bins_y = n;
        self
    }

    /// Number of bins along x and y separately.
    pub fn bins_xy(&mut self, nx: usize, ny: usize) -> &mut Self {
        self.bins_x = nx.max(1);
        self.bins_y = ny.max(1);
        self
    }

    /// Colormap for counts (default Viridis).
    pub fn cmap(&mut self, cmap: impl Into<Cmap>) -> &mut Self {
        self.cmap = cmap.into();
        self
    }

    /// Value normalization (default [`Norm::Linear`]).
    pub fn norm(&mut self, norm: Norm) -> &mut Self {
        self.norm = norm;
        self
    }

    /// Lower end of the colormap scale.
    pub fn vmin(&mut self, vmin: f64) -> &mut Self {
        self.vmin = Some(vmin);
        self
    }

    /// Upper end of the colormap scale.
    pub fn vmax(&mut self, vmax: f64) -> &mut Self {
        self.vmax = Some(vmax);
        self
    }

    /// Show a colorbar strip (default `true`).
    pub fn colorbar(&mut self, show: bool) -> &mut Self {
        self.colorbar = show;
        self
    }

    /// Optional legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn value_limits(&self) -> (f64, f64) {
        let bins = crate::recipes::hist2d_bins(
            self.x.as_slice(),
            self.y.as_slice(),
            self.bins_x,
            self.bins_y,
        );
        crate::recipes::hist2d_limits(&bins.counts, self.vmin, self.vmax)
    }
}

/// Hexagonal binning of `(x, y)` samples.
#[derive(Debug, Clone)]
pub struct HexbinPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) gridsize: usize,
    pub(crate) cmap: Cmap,
    pub(crate) vmin: Option<f64>,
    pub(crate) vmax: Option<f64>,
    pub(crate) norm: Norm,
    pub(crate) colorbar: bool,
    pub(crate) label: Option<String>,
}

impl HexbinPlot {
    /// Approximate number of hexes across the x-span (default 20).
    pub fn gridsize(&mut self, n: usize) -> &mut Self {
        self.gridsize = n.max(2);
        self
    }

    /// Colormap for counts (default Viridis).
    pub fn cmap(&mut self, cmap: impl Into<Cmap>) -> &mut Self {
        self.cmap = cmap.into();
        self
    }

    /// Value normalization (default [`Norm::Linear`]).
    pub fn norm(&mut self, norm: Norm) -> &mut Self {
        self.norm = norm;
        self
    }

    /// Lower end of the colormap scale.
    pub fn vmin(&mut self, vmin: f64) -> &mut Self {
        self.vmin = Some(vmin);
        self
    }

    /// Upper end of the colormap scale.
    pub fn vmax(&mut self, vmax: f64) -> &mut Self {
        self.vmax = Some(vmax);
        self
    }

    /// Show a colorbar strip (default `true`).
    pub fn colorbar(&mut self, show: bool) -> &mut Self {
        self.colorbar = show;
        self
    }

    /// Optional legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn value_limits(&self) -> (f64, f64) {
        use plotine_core::{DataToPixel, LinearScale, Rect, ScaleKind};
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 1.0).expect("unit")),
            ScaleKind::Linear(LinearScale::new(0.0, 1.0).expect("unit")),
            Rect::new(0.0, 0.0, 1.0, 1.0),
        );
        let (_, lo, hi) = crate::recipes::hexbin_cells(
            self.x.as_slice(),
            self.y.as_slice(),
            self.gridsize,
            &self.cmap,
            self.vmin,
            self.vmax,
            self.norm,
            &t,
        );
        (lo, hi)
    }
}

/// Contour lines of a row-major `nrows × ncols` scalar field.
#[derive(Debug, Clone)]
pub struct ContourPlot {
    pub(crate) nrows: usize,
    pub(crate) ncols: usize,
    pub(crate) values: Series,
    pub(crate) levels: Option<Vec<f64>>,
    pub(crate) level_count: usize,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) clabel: bool,
    pub(crate) clabel_size: f64,
    pub(crate) clabel_color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl ContourPlot {
    /// Explicit contour levels (overrides `.levels(n)` count).
    pub fn level_values<I>(&mut self, levels: I) -> &mut Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.levels = Some(levels.into_iter().filter(|v| v.is_finite()).collect());
        self
    }

    /// Number of automatic levels (default 8).
    pub fn levels(&mut self, n: usize) -> &mut Self {
        self.level_count = n.max(1);
        self
    }

    /// Stroke color (default: categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Label each contour level with its value (matplotlib `clabel`, inline gap).
    pub fn clabel(&mut self, on: bool) -> &mut Self {
        self.clabel = on;
        self
    }

    /// Contour label font size in points (default `9`).
    pub fn clabel_size(&mut self, size: f64) -> &mut Self {
        self.clabel_size = size.max(1.0);
        self
    }

    /// Contour label color (default [`Color::LABEL`]).
    pub fn clabel_color(&mut self, color: Color) -> &mut Self {
        self.clabel_color = Some(color);
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn resolved_levels(&self) -> Vec<f64> {
        if let Some(ref levels) = self.levels {
            levels.clone()
        } else {
            crate::recipes::auto_levels(self.values.as_slice(), self.level_count)
        }
    }
}

/// Filled contours of a row-major scalar field.
#[derive(Debug, Clone)]
pub struct ContourfPlot {
    pub(crate) nrows: usize,
    pub(crate) ncols: usize,
    pub(crate) values: Series,
    pub(crate) levels: Option<Vec<f64>>,
    pub(crate) level_count: usize,
    pub(crate) cmap: Cmap,
    pub(crate) norm: Norm,
    pub(crate) colorbar: bool,
    pub(crate) label: Option<String>,
}

impl ContourfPlot {
    /// Explicit fill levels.
    pub fn level_values<I>(&mut self, levels: I) -> &mut Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.levels = Some(levels.into_iter().filter(|v| v.is_finite()).collect());
        self
    }

    /// Number of automatic levels (default 10).
    pub fn levels(&mut self, n: usize) -> &mut Self {
        self.level_count = n.max(2);
        self
    }

    /// Colormap (default Viridis).
    pub fn cmap(&mut self, cmap: impl Into<Cmap>) -> &mut Self {
        self.cmap = cmap.into();
        self
    }

    /// Value normalization (default [`Norm::Linear`]).
    pub fn norm(&mut self, norm: Norm) -> &mut Self {
        self.norm = norm;
        self
    }

    /// Show a colorbar (default `true`).
    pub fn colorbar(&mut self, show: bool) -> &mut Self {
        self.colorbar = show;
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn resolved_levels(&self) -> Vec<f64> {
        if let Some(ref levels) = self.levels {
            levels.clone()
        } else {
            crate::recipes::auto_levels(self.values.as_slice(), self.level_count)
        }
    }

    pub(crate) fn value_limits(&self) -> (f64, f64) {
        let levels = self.resolved_levels();
        if levels.len() >= 2 {
            (
                levels.iter().copied().fold(f64::INFINITY, f64::min),
                levels.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        } else {
            crate::recipes::heatmap_limits(self.values.as_slice(), None, None)
        }
    }
}

/// Pseudocolor mesh with explicit edge coordinates.
#[derive(Debug, Clone)]
pub struct PcolorMeshPlot {
    pub(crate) x_edges: Series,
    pub(crate) y_edges: Series,
    pub(crate) values: Series,
    pub(crate) cmap: Cmap,
    pub(crate) vmin: Option<f64>,
    pub(crate) vmax: Option<f64>,
    pub(crate) norm: Norm,
    pub(crate) colorbar: bool,
    pub(crate) label: Option<String>,
}

impl PcolorMeshPlot {
    /// Colormap (default Viridis).
    pub fn cmap(&mut self, cmap: impl Into<Cmap>) -> &mut Self {
        self.cmap = cmap.into();
        self
    }

    /// Value normalization (default [`Norm::Linear`]).
    pub fn norm(&mut self, norm: Norm) -> &mut Self {
        self.norm = norm;
        self
    }

    /// Lower colormap bound.
    pub fn vmin(&mut self, vmin: f64) -> &mut Self {
        self.vmin = Some(vmin);
        self
    }

    /// Upper colormap bound.
    pub fn vmax(&mut self, vmax: f64) -> &mut Self {
        self.vmax = Some(vmax);
        self
    }

    /// Show a colorbar (default `true`).
    pub fn colorbar(&mut self, show: bool) -> &mut Self {
        self.colorbar = show;
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn value_limits(&self) -> (f64, f64) {
        crate::recipes::pcolormesh_limits(self.values.as_slice(), self.vmin, self.vmax)
    }
}

/// Spy plot: markers at non-zero (above precision) matrix entries.
#[derive(Debug, Clone)]
pub struct SpyPlot {
    pub(crate) nrows: usize,
    pub(crate) ncols: usize,
    pub(crate) values: Series,
    pub(crate) precision: f64,
    pub(crate) marker_size: f64,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl SpyPlot {
    /// Absolute threshold for "nonzero" (default `1e-8`).
    pub fn precision(&mut self, precision: f64) -> &mut Self {
        self.precision = precision.abs();
        self
    }

    /// Marker radius in points.
    pub fn marker_size(&mut self, size: f64) -> &mut Self {
        self.marker_size = size.max(0.5);
        self
    }

    /// Marker color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Vector field of arrows at `(x, y)` with components `(u, v)`.
#[derive(Debug, Clone)]
pub struct QuiverPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) u: Series,
    pub(crate) v: Series,
    pub(crate) scale: Option<f64>,
    pub(crate) width: f64,
    pub(crate) key_length: Option<f64>,
    pub(crate) key_label: Option<String>,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl QuiverPlot {
    /// Manual scale (data units per arrow unit); larger → shorter arrows.
    pub fn scale(&mut self, scale: f64) -> &mut Self {
        self.scale = Some(scale.max(1e-12));
        self
    }

    /// Shaft stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Draw a reference key arrow of data-length `length` with `label`.
    pub fn quiverkey(&mut self, length: f64, label: impl Into<String>) -> &mut Self {
        self.key_length = Some(length.abs().max(1e-12));
        self.key_label = Some(label.into());
        self
    }

    /// Arrow color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Wind barbs at `(x, y)` with components `(u, v)` (matplotlib `barbs`).
#[derive(Debug, Clone)]
pub struct BarbsPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) u: Series,
    pub(crate) v: Series,
    /// Staff length parameter in points (default 6; matplotlib-compatible scale).
    pub(crate) length: f64,
    pub(crate) width: f64,
    pub(crate) flip: bool,
    pub(crate) half: f64,
    pub(crate) full: f64,
    pub(crate) flag: f64,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl BarbsPlot {
    /// Staff length parameter in points (default `6`, matplotlib `barbs` default scale).
    ///
    /// Matches matplotlib's effective on-screen size: verts are scaled by
    /// `length/2`, so the drawn staff is `length²/2` points tall.
    pub fn length(&mut self, length: f64) -> &mut Self {
        self.length = length.max(2.0);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Flip feathers to the opposite side of the staff (Southern Hemisphere).
    pub fn flip(&mut self, on: bool) -> &mut Self {
        self.flip = on;
        self
    }

    /// Magnitude increments for half / full / flag (defaults 5 / 10 / 50).
    pub fn increments(&mut self, half: f64, full: f64, flag: f64) -> &mut Self {
        self.half = half.max(1e-12);
        self.full = full.max(self.half);
        self.flag = flag.max(self.full);
        self
    }

    /// Barb color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Streamlines of a regular-grid vector field (`u`/`v` row-major).
#[derive(Debug, Clone)]
pub struct StreamPlot {
    pub(crate) u: Series,
    pub(crate) v: Series,
    pub(crate) nrows: usize,
    pub(crate) ncols: usize,
    /// Matplotlib-compatible density (`1.0` → 30×30 occupancy mask).
    pub(crate) density: f64,
    pub(crate) width: f64,
    /// Matplotlib `arrowsize` (`1.0` default → mutation_scale 10 pt; `0.0` disables).
    pub(crate) arrow_size: f64,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl StreamPlot {
    /// Streamline closeness (matplotlib `density`; default `1.0` → 30×30 mask).
    pub fn density(&mut self, density: f64) -> &mut Self {
        self.density = density.clamp(0.1, 8.0);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Matplotlib `arrowsize` (default `1.0` → 10 pt mutation scale; `0.0` hides).
    pub fn arrow_size(&mut self, size: f64) -> &mut Self {
        self.arrow_size = size.max(0.0);
        self
    }

    /// Line color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Polar grid frame (rings + spokes) centered at the origin.
#[derive(Debug, Clone)]
pub struct PolarFramePlot {
    pub(crate) rmax: f64,
    /// Hint for automatic ring count (MaxNLocator-style), default [`mpl_policy::polar::RING_N_HINT`](crate::mpl_policy::polar::RING_N_HINT).
    pub(crate) rings: usize,
    pub(crate) spokes: usize,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl PolarFramePlot {
    /// Outer radius in data units.
    pub fn rmax(&mut self, rmax: f64) -> &mut Self {
        self.rmax = rmax.abs().max(1e-9);
        self
    }

    /// Preferred number of radial rings / tick levels (default 5, mpl-like).
    pub fn rings(&mut self, rings: usize) -> &mut Self {
        self.rings = rings.clamp(3, 12);
        self
    }

    /// Number of angular spokes (default 8 → 45° steps).
    pub fn spokes(&mut self, spokes: usize) -> &mut Self {
        self.spokes = spokes.clamp(4, 24);
        self
    }

    /// Grid stroke color (default theme grid).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Grouped box-and-whisker plot.
#[derive(Debug, Clone)]
pub struct BoxPlot {
    pub(crate) groups: Vec<Series>,
    /// Box width as a fraction of the unit category spacing (default 0.55).
    pub(crate) widths: f64,
    pub(crate) show_fliers: bool,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl BoxPlot {
    /// Set the box fill color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the box/whisker stroke color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Box width as a fraction of unit category spacing.
    pub fn widths(&mut self, widths: f64) -> &mut Self {
        self.widths = widths.clamp(0.1, 0.95);
        self
    }

    /// Draw outlier points beyond the Tukey whiskers (default `true`).
    pub fn show_fliers(&mut self, show: bool) -> &mut Self {
        self.show_fliers = show;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Grouped violin plot (Gaussian KDE).
#[derive(Debug, Clone)]
pub struct ViolinPlot {
    pub(crate) groups: Vec<Series>,
    /// Max body width as a fraction of unit category spacing (default 0.5, matplotlib).
    pub(crate) widths: f64,
    pub(crate) points: usize,
    pub(crate) show_median: bool,
    /// Vertical min–max stem with end caps (matplotlib `showextrema=True`).
    pub(crate) show_extrema: bool,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl ViolinPlot {
    /// Set the body fill color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the outline stroke color (no outline by default).
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Max body width as a fraction of unit category spacing.
    pub fn widths(&mut self, widths: f64) -> &mut Self {
        self.widths = widths.clamp(0.1, 0.95);
        self
    }

    /// Number of KDE sample points along the density (16–256).
    pub fn points(&mut self, points: usize) -> &mut Self {
        self.points = points.clamp(16, 256);
        self
    }

    /// Draw a horizontal median marker (default `false`, matplotlib `showmedians`).
    pub fn show_median(&mut self, show: bool) -> &mut Self {
        self.show_median = show;
        self
    }

    /// Draw the vertical extrema stem and end caps (default `true`).
    pub fn show_extrema(&mut self, show: bool) -> &mut Self {
        self.show_extrema = show;
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.55`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Pie chart of non-negative slice sizes.
#[derive(Debug, Clone)]
pub struct PiePlot {
    pub(crate) sizes: Series,
    pub(crate) labels: Vec<String>,
    pub(crate) start_angle: f64,
    /// When true, slices advance counter-clockwise; matplotlib default is `false`.
    pub(crate) counterclock: bool,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl PiePlot {
    /// Per-slice labels (drawn beside wedges; also used in the legend).
    pub fn labels<I, S>(&mut self, labels: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Start angle in degrees from +x (default `90`).
    pub fn start_angle(&mut self, degrees: f64) -> &mut Self {
        self.start_angle = degrees;
        self
    }

    /// Slice direction (default `false` = clockwise, matching matplotlib).
    pub fn counterclock(&mut self, counterclock: bool) -> &mut Self {
        self.counterclock = counterclock;
        self
    }

    /// Override all wedges with a single fill color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Wedge edge stroke color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.9`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Single legend label when per-slice labels are not set.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Stacked area plot of multiple series sharing `x`.
#[derive(Debug, Clone)]
pub struct StackPlot {
    pub(crate) x: Series,
    pub(crate) ys: Vec<Series>,
    pub(crate) labels: Vec<String>,
    pub(crate) alpha: f64,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl StackPlot {
    /// Per-series legend labels (bottom→top order).
    pub fn labels<I, S>(&mut self, labels: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.85`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Force every layer to the same color (rarely useful).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Single legend label when per-series labels are not set.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Event / raster plot: parallel ticks at given positions per row.
#[derive(Debug, Clone)]
pub struct EventPlot {
    pub(crate) positions: Vec<Series>,
    pub(crate) labels: Vec<String>,
    pub(crate) lineoffset: f64,
    pub(crate) linewidth: f64,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl EventPlot {
    /// Per-row legend labels.
    pub fn labels<I, S>(&mut self, labels: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Vertical span of each tick in data units (default `0.8`).
    pub fn lineoffset(&mut self, lineoffset: f64) -> &mut Self {
        self.lineoffset = lineoffset.abs().max(0.05);
        self
    }

    /// Stroke width in points.
    pub fn linewidth(&mut self, width: f64) -> &mut Self {
        self.linewidth = width.max(0.1);
        self
    }

    /// Override all rows with a single color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Single legend label when per-row labels are not set.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Broken horizontal bars: `(xmin, width)` ranges at a fixed y band.
#[derive(Debug, Clone)]
pub struct BrokenBarHPlot {
    pub(crate) xranges: Vec<(f64, f64)>,
    pub(crate) y: f64,
    pub(crate) height: f64,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl BrokenBarHPlot {
    /// Set the bar fill color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the bar edge stroke color (no outline by default).
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Axis-aligned rectangle patch (matplotlib `Rectangle`).
#[derive(Debug, Clone)]
pub struct RectanglePlot {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) hatch: Hatch,
    pub(crate) linewidth: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl RectanglePlot {
    /// Set the fill color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the outline stroke color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.45`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Fill hatch pattern.
    pub fn hatch(&mut self, hatch: Hatch) -> &mut Self {
        self.hatch = hatch;
        self
    }

    /// Edge stroke width in points (default `1.0`).
    pub fn linewidth(&mut self, width: f64) -> &mut Self {
        self.linewidth = width.max(0.0);
        self
    }

    /// Legend label for this patch.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Circle patch in data coordinates (matplotlib `Circle`).
#[derive(Debug, Clone)]
pub struct CirclePlot {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) radius: f64,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) linewidth: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl CirclePlot {
    /// Set the fill color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the outline stroke color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.45`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Edge stroke width in points (default `1.0`).
    pub fn linewidth(&mut self, width: f64) -> &mut Self {
        self.linewidth = width.max(0.0);
        self
    }

    /// Legend label for this patch.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Ellipse patch in data coordinates (matplotlib `Ellipse`).
#[derive(Debug, Clone)]
pub struct EllipsePlot {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) linewidth: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl EllipsePlot {
    /// Set the fill color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the outline stroke color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.45`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Edge stroke width in points (default `1.0`).
    pub fn linewidth(&mut self, width: f64) -> &mut Self {
        self.linewidth = width.max(0.0);
        self
    }

    /// Legend label for this patch.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Filled closed polygon.
#[derive(Debug, Clone)]
pub struct PolygonPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) color: Option<Color>,
    pub(crate) edgecolor: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl PolygonPlot {
    /// Set the fill color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Set the outline stroke color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.45`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Horizontal band spanning the full x-domain.
#[derive(Debug, Clone)]
pub struct AxHSpanPlot {
    pub(crate) ymin: f64,
    pub(crate) ymax: f64,
    pub(crate) color: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl AxHSpanPlot {
    /// Set the fill color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.25`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Vertical band spanning the full y-domain.
#[derive(Debug, Clone)]
pub struct AxVSpanPlot {
    pub(crate) xmin: f64,
    pub(crate) xmax: f64,
    pub(crate) color: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl AxVSpanPlot {
    /// Set the fill color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Fill opacity in `0.0..=1.0` (default `0.25`).
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Matplotlib-style table overlaid on the axes (does not enter the legend).
#[derive(Debug, Clone)]
pub struct TablePlot {
    pub(crate) cells: Vec<Vec<String>>,
    pub(crate) col_labels: Vec<String>,
    pub(crate) row_labels: Vec<String>,
    pub(crate) loc: TableLoc,
    pub(crate) fontsize: f64,
    pub(crate) cell_pad: f64,
    pub(crate) edgecolor: Color,
    pub(crate) facecolor: Color,
    pub(crate) header_facecolor: Color,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl TablePlot {
    /// Column header labels (inserted as the first row).
    pub fn col_labels<I, S>(&mut self, labels: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.col_labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Row header labels (inserted as the first column).
    pub fn row_labels<I, S>(&mut self, labels: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.row_labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Placement relative to the axes box.
    pub fn loc(&mut self, loc: TableLoc) -> &mut Self {
        self.loc = loc;
        self
    }

    /// Cell font size in points (default `9`).
    pub fn fontsize(&mut self, pt: f64) -> &mut Self {
        self.fontsize = pt.max(5.0);
        self
    }

    /// Padding inside each cell in points (default `4`).
    pub fn cell_pad(&mut self, pt: f64) -> &mut Self {
        self.cell_pad = pt.max(0.0);
        self
    }

    /// Grid / border color.
    pub fn edgecolor(&mut self, color: Color) -> &mut Self {
        self.edgecolor = color;
        self.color = Some(color);
        self
    }

    /// Body cell fill color.
    pub fn facecolor(&mut self, color: Color) -> &mut Self {
        self.facecolor = color;
        self
    }

    /// Header cell fill color.
    pub fn header_facecolor(&mut self, color: Color) -> &mut Self {
        self.header_facecolor = color;
        self
    }
}

/// Text annotation at a data-coordinate anchor (does not enter the legend).
#[derive(Debug, Clone)]
pub struct TextPlot {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) text: String,
    pub(crate) color: Option<Color>,
    pub(crate) size: f64,
    pub(crate) align: TextAlign,
    pub(crate) baseline: TextBaseline,
    pub(crate) rotation_deg: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl TextPlot {
    /// Text / ink color (default [`Color::LABEL`]).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Font size in points (default `10`).
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = size.max(1.0);
        self
    }

    /// Horizontal alignment relative to the anchor.
    pub fn ha(&mut self, align: TextAlign) -> &mut Self {
        self.align = align;
        self
    }

    /// Vertical alignment relative to the anchor.
    pub fn va(&mut self, baseline: TextBaseline) -> &mut Self {
        self.baseline = baseline;
        self
    }

    /// Clockwise rotation in screen degrees (y-down).
    pub fn rotation(&mut self, degrees: f64) -> &mut Self {
        self.rotation_deg = degrees;
        self
    }
}

/// Matplotlib-style `arrowstyle` for [`AnnotatePlot`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowStyle {
    /// Filled triangular head at `xy` (default; matplotlib `-|>`).
    #[default]
    Triangle,
    /// Open V head at `xy` (matplotlib `->`).
    Simple,
    /// Outward square bracket at `xy` (matplotlib `-\[` / `BracketB`).
    Bracket,
    /// Open V heads at both ends (matplotlib `<->`).
    BothEnds,
}

/// Text plus optional arrow from `xytext` toward `xy` (data coordinates).
#[derive(Debug, Clone)]
pub struct AnnotatePlot {
    pub(crate) xy: (f64, f64),
    pub(crate) xytext: (f64, f64),
    pub(crate) text: String,
    pub(crate) arrow: bool,
    pub(crate) arrow_style: ArrowStyle,
    pub(crate) color: Option<Color>,
    pub(crate) arrow_color: Option<Color>,
    pub(crate) arrow_width: f64,
    pub(crate) size: f64,
    pub(crate) align: TextAlign,
    pub(crate) baseline: TextBaseline,
    pub(crate) rotation_deg: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl AnnotatePlot {
    /// Draw an arrow from the text toward `xy` (default `true`).
    pub fn arrow(&mut self, on: bool) -> &mut Self {
        self.arrow = on;
        self
    }

    /// Arrow head geometry (matplotlib `arrowprops.arrowstyle`).
    pub fn arrow_style(&mut self, style: ArrowStyle) -> &mut Self {
        self.arrow_style = style;
        self.arrow = true;
        self
    }

    /// Annotation text color (default [`Color::LABEL`]).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Arrow stroke/fill color (defaults to the text color).
    pub fn arrow_color(&mut self, color: Color) -> &mut Self {
        self.arrow_color = Some(color);
        self
    }

    /// Arrow shaft width in points (default `1.0`).
    pub fn arrow_width(&mut self, width: f64) -> &mut Self {
        self.arrow_width = width.max(0.1);
        self
    }

    /// Font size in points (default `10`).
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = size.max(1.0);
        self
    }

    /// Horizontal alignment of the text at `xytext`.
    pub fn ha(&mut self, align: TextAlign) -> &mut Self {
        self.align = align;
        self
    }

    /// Vertical alignment of the text at `xytext`.
    pub fn va(&mut self, baseline: TextBaseline) -> &mut Self {
        self.baseline = baseline;
        self
    }

    /// Clockwise rotation in screen degrees (y-down).
    pub fn rotation(&mut self, degrees: f64) -> &mut Self {
        self.rotation_deg = degrees;
        self
    }
}

/// Pseudocolor plot on an unstructured triangular mesh (`tripcolor`).
#[derive(Debug, Clone)]
pub struct TripcolorPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) z: Series,
    pub(crate) triangles: Vec<[usize; 3]>,
    pub(crate) cmap: Cmap,
    pub(crate) vmin: Option<f64>,
    pub(crate) vmax: Option<f64>,
    pub(crate) norm: Norm,
    pub(crate) colorbar: bool,
    pub(crate) label: Option<String>,
}

impl TripcolorPlot {
    /// Triangle vertex indices into `(x, y, z)`.
    ///
    /// Optional — when omitted, Delaunay triangulation is auto-computed at draw time.
    pub fn triangles<I>(&mut self, tris: I) -> &mut Self
    where
        I: IntoIterator<Item = [usize; 3]>,
    {
        self.triangles = tris.into_iter().collect();
        self
    }

    /// Colormap (default Viridis).
    pub fn cmap(&mut self, cmap: impl Into<Cmap>) -> &mut Self {
        self.cmap = cmap.into();
        self
    }

    /// Value normalization (default [`Norm::Linear`]).
    pub fn norm(&mut self, norm: Norm) -> &mut Self {
        self.norm = norm;
        self
    }

    /// Lower colormap bound.
    pub fn vmin(&mut self, vmin: f64) -> &mut Self {
        self.vmin = Some(vmin);
        self
    }

    /// Upper colormap bound.
    pub fn vmax(&mut self, vmax: f64) -> &mut Self {
        self.vmax = Some(vmax);
        self
    }

    /// Show a colorbar (default `true`).
    pub fn colorbar(&mut self, show: bool) -> &mut Self {
        self.colorbar = show;
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn value_limits(&self) -> (f64, f64) {
        if let (Some(lo), Some(hi)) = (self.vmin, self.vmax) {
            return (lo.min(hi), lo.max(hi));
        }
        // Flat shading (mpl default): clim from face means, not vertex extrema.
        let (zmin, zmax) =
            crate::recipes::tripcolor_face_limits(self.z.as_slice(), &self.triangles)
                .unwrap_or((0.0, 1.0));
        let lo = self.vmin.unwrap_or(zmin);
        let hi = self.vmax.unwrap_or(zmax);
        (lo.min(hi), lo.max(hi))
    }
}

/// Contour lines on an unstructured triangular mesh (`tricontour`).
#[derive(Debug, Clone)]
pub struct TricontourPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) z: Series,
    pub(crate) triangles: Vec<[usize; 3]>,
    pub(crate) levels: Option<Vec<f64>>,
    pub(crate) level_count: usize,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl TricontourPlot {
    /// Triangle vertex indices into `(x, y, z)`.
    ///
    /// Optional — when omitted, Delaunay triangulation is auto-computed at draw time.
    pub fn triangles<I>(&mut self, tris: I) -> &mut Self
    where
        I: IntoIterator<Item = [usize; 3]>,
    {
        self.triangles = tris.into_iter().collect();
        self
    }

    /// Explicit contour levels.
    pub fn level_values<I>(&mut self, levels: I) -> &mut Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.levels = Some(levels.into_iter().filter(|v| v.is_finite()).collect());
        self
    }

    /// Number of automatic levels (default 8).
    pub fn levels(&mut self, n: usize) -> &mut Self {
        self.level_count = n.max(1);
        self
    }

    /// Stroke color (default: categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}

/// Filled contours on an unstructured triangular mesh (`tricontourf`).
#[derive(Debug, Clone)]
pub struct TricontourfPlot {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) z: Series,
    pub(crate) triangles: Vec<[usize; 3]>,
    pub(crate) levels: Option<Vec<f64>>,
    pub(crate) level_count: usize,
    pub(crate) cmap: Cmap,
    pub(crate) norm: Norm,
    pub(crate) colorbar: bool,
    pub(crate) label: Option<String>,
}

impl TricontourfPlot {
    /// Triangle vertex indices into `(x, y, z)`.
    ///
    /// Optional — when omitted, Delaunay triangulation is auto-computed at draw time.
    pub fn triangles<I>(&mut self, tris: I) -> &mut Self
    where
        I: IntoIterator<Item = [usize; 3]>,
    {
        self.triangles = tris.into_iter().collect();
        self
    }

    /// Explicit fill levels.
    pub fn level_values<I>(&mut self, levels: I) -> &mut Self
    where
        I: IntoIterator<Item = f64>,
    {
        self.levels = Some(levels.into_iter().filter(|v| v.is_finite()).collect());
        self
    }

    /// Number of automatic levels (default 10).
    pub fn levels(&mut self, n: usize) -> &mut Self {
        self.level_count = n.max(2);
        self
    }

    /// Colormap (default Viridis).
    pub fn cmap(&mut self, cmap: impl Into<Cmap>) -> &mut Self {
        self.cmap = cmap.into();
        self
    }

    /// Value normalization (default [`Norm::Linear`]).
    pub fn norm(&mut self, norm: Norm) -> &mut Self {
        self.norm = norm;
        self
    }

    /// Show a colorbar (default `true`).
    pub fn colorbar(&mut self, show: bool) -> &mut Self {
        self.colorbar = show;
        self
    }

    /// Legend label.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn resolved_levels(&self) -> Vec<f64> {
        if let Some(ref levels) = self.levels {
            levels.clone()
        } else {
            crate::recipes::resolve_tri_levels(
                self.z.as_slice(),
                &self.triangles,
                None,
                self.level_count,
            )
        }
    }

    pub(crate) fn value_limits(&self) -> (f64, f64) {
        let levels = self.resolved_levels();
        if levels.len() >= 2 {
            (
                levels.iter().copied().fold(f64::INFINITY, f64::min),
                levels.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        } else {
            crate::recipes::tripcolor_face_limits(self.z.as_slice(), &self.triangles)
                .unwrap_or((0.0, 1.0))
        }
    }
}

/// Infinite line through two data points, clipped to axes bounds (`axline`).
#[derive(Debug, Clone)]
pub struct AxLinePlot {
    pub(crate) xy1: (f64, f64),
    pub(crate) xy2: (f64, f64),
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) linestyle: LineStyle,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl_artist_props!(AxLinePlot, LegendKind::Line(LineStyle::Solid));

impl AxLinePlot {
    /// Set the stroke color (overrides the categorical cycle).
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Stroke width in points.
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.1);
        self
    }

    /// Set the dash pattern (`'-'`, `'--'`, `':'`, `'-.'`).
    pub fn linestyle(&mut self, style: LineStyle) -> &mut Self {
        self.linestyle = style;
        self
    }

    /// Legend label for this series.
    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.label = Some(label.into());
        self
    }
}
