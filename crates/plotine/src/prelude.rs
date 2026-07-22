//! Common imports for plotine users and coding agents.

pub use crate::animation::Animation;
pub use crate::artist::{ArrowStyle, ErrBars};
pub use crate::axes::{category_indices, Axes, GridAxis};
pub use crate::axes3d::Axes3D;
pub use crate::figure::Figure;
pub use crate::geo::GeoProjection;
pub use crate::layout::GridSpec;
pub use crate::legend::Legend;
pub use crate::math;
pub use crate::nav::NavMode;
pub use crate::projection::Camera;
pub use crate::recipes::{HeatmapOrigin, StepMode, TableLoc};
pub use crate::secondary::{SecondaryAxis, SecondaryTransform};
pub use crate::series::{IntoSeries, Series};
pub use crate::style::{Hatch, LineStyle, MarkerStyle, Spines};
pub use crate::subplots::SubplotGrid;
pub use crate::theme::Theme;
pub use crate::tick_format::TickFormatter;
pub use crate::view::{Axes3DView, PanelView, ViewSnapshot};
pub use plotine_core::{
    Cmap, Color, ColorParseError, Colormap, Norm, PlotError, Result, ScaleType, SegmentedColormap,
};
pub use plotine_render::{TextAlign, TextBaseline};
