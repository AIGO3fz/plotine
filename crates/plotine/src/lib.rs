//! plotine — a high-level, LLM-friendly Rust native scientific plotting library
//! (static 2D + basic 3D).
//!
//! ```
//! use plotine::prelude::*;
//!
//! let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.1).collect();
//! let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
//! let png = Figure::new()
//!     .size(4.0, 3.0)
//!     .dpi(72.0)
//!     .subplots(2, 1, |g| {
//!         g.at(0, 0, |ax| {
//!             ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
//!             ax.title("Top");
//!         });
//!         g.at(1, 0, |ax| {
//!             ax.scatter(&x, &y).size(3.0);
//!             ax.title("Bottom");
//!         });
//!     })
//!     .render_png()
//!     .expect("png");
//! assert!(!png.is_empty());
//! ```
//!
//! See the [`prelude`] module and the repository's `AGENTS.md` / `llms.txt`
//! for agent-oriented API guidance.

#![warn(missing_docs)]

/// Offline multi-frame animation (PNG sequence / GIF / MP4).
pub mod animation;
/// Mutable plot artists returned by [`Axes`] methods.
pub mod artist;
/// Axes panel API (limits, scales, chart methods).
pub mod axes;
/// 3D axes panel (line3d, scatter3d, surface, wireframe, bar3d).
pub mod axes3d;
mod draw;
mod draw3d;
/// Optional system tools (`ffmpeg`, Ghostscript) for EPS / MP4 export.
#[cfg(any(feature = "eps", feature = "mp4"))]
mod ext_tools;
/// Top-level figure builder and rendering entry points.
pub mod figure;
/// Geographic projections + coastline (cartopy-thin).
pub mod geo;
/// Interactive window (`feature = "gui"`): pan/zoom, 3D rotate, export.
#[cfg(feature = "gui")]
pub mod gui;
/// Re-export egui for [`Figure::show_with`](figure::Figure::show_with) widget UIs.
#[cfg(feature = "gui")]
pub use egui;
/// Optional CJK / custom font loading (`feature = "cjk"`).
#[cfg(feature = "cjk")]
pub mod fonts;
/// External LaTeX via system `latex`/`dvipng` (`feature = "latex"` + [`Figure::usetex`](figure::Figure::usetex)).
#[cfg(feature = "latex")]
pub mod latex;
/// Subplot grid geometry and tight-layout insets.
pub mod layout;
/// Legend placement.
pub mod legend;
/// Unicode helpers for Greek letters and super/subscripts in labels.
pub mod math;
/// Matplotlib-style mathtext layout (`$...$`, fractions, scripts; no LaTeX binary).
pub mod mathtext;
/// Matplotlib-aligned geometry / style policy (shared by layout, polar, pie, …).
pub mod mpl_policy;
/// Matplotlib-aligned navigation math and view history.
pub mod nav;
/// Convenient re-exports for application crates and agents.
pub mod prelude;
/// 3D→2D projection utilities (camera, rotation, painter's algorithm).
pub mod projection;
/// Pure geometry recipes (advanced / testing).
pub mod recipes;
/// Secondary axis transforms (matplotlib `secondary_xaxis` / `secondary_yaxis`).
pub mod secondary;
/// Numeric series adapters (`IntoSeries`).
pub mod series;
/// Seaborn-style statistical helpers (`corr_heatmap` / `pair_scatter` / `regline`).
pub mod stats;
/// Line dash patterns, marker shapes, and spine visibility.
pub mod style;
/// Multi-panel subplot grid builder.
pub mod subplots;
/// Built-in visual themes.
pub mod theme;
pub mod tick_format;
/// Interactive view snapshots (limits / 3D camera).
pub mod view;

#[cfg(feature = "polars")]
#[path = "polars_support.rs"]
pub mod polars;

pub use animation::{AnimFrame, Animation};
pub use artist::{
    AnnotatePlot, AreaPlot, ArrowStyle, AxHLinePlot, AxHSpanPlot, AxLinePlot, AxVLinePlot,
    AxVSpanPlot, BarHPlot, BarPlot, BarbsPlot, BoxPlot, BrokenBarHPlot, ContourPlot, ContourfPlot,
    ErrBars, ErrorBarPlot, EventPlot, FillBetweenPlot, FillBetweenXPlot, HLinesPlot, HeatmapPlot,
    HexbinPlot, Hist2dPlot, HistPlot, LinePlot, PcolorMeshPlot, PiePlot, PolarFramePlot,
    PolygonPlot, QuiverPlot, ScatterPlot, SpyPlot, StackPlot, StairsPlot, StemPlot, StepPlot,
    StreamPlot, TablePlot, TextPlot, TricontourPlot, TricontourfPlot, TripcolorPlot, VLinesPlot,
    ViolinPlot,
};
pub use axes::{category_indices, Axes, GridAxis};
pub use axes3d::Axes3D;
pub use figure::Figure;
pub use geo::GeoProjection;
#[cfg(feature = "gui")]
pub use gui::ShowHandle;
pub use layout::GridSpec;
pub use legend::Legend;
pub use nav::{NavMode, ViewHistory};
pub use plotine_core::{
    Cmap, Color, ColorParseError, Colormap, Norm, PlotError, Result, ScaleType, SegmentedColormap,
};
pub use plotine_render::{TextAlign, TextBaseline};
pub use projection::Camera;
pub use recipes::{HeatmapOrigin, StepMode, TableLoc};
pub use secondary::{SecondaryAxis, SecondaryTransform};
pub use series::{IntoSeries, Series};
pub use style::{Hatch, LineStyle, MarkerStyle, Spines};
pub use subplots::SubplotGrid;
pub use theme::Theme;
pub use tick_format::TickFormatter;
pub use view::{Axes3DView, PanelView, ViewSnapshot};
