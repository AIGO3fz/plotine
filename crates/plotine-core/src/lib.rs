//! Core primitives for plotine: geometry, color, transforms, scales, and ticks.

pub mod color;
pub mod colormap;
pub mod datetime;
pub mod error;
pub mod geom;
pub mod scale;
pub mod ticks;
pub mod transform;

pub use color::{Color, ColorParseError};
pub use colormap::{Cmap, Colormap, Norm, SegmentedColormap};
pub use datetime::{
    civil_from_unix, format_concise_datetime_ticks, format_unix_tick, unix_from_civil,
    DatetimeLocator,
};
pub use error::{PlotError, Result};
pub use geom::{Point, Rect, Size};
pub use scale::{LinearScale, LogScale, Scale, ScaleKind, ScaleType, SymlogScale};
pub use ticks::{format_aligned_ticks, ticks_from_values, Tick, TickLocator};
pub use transform::{Affine, DataToPixel};
