//! Backend-agnostic rendering surface for plotine.

mod primitives;
mod renderer;

pub use primitives::{
    polyline_from_points, FillStyle, LineCap, LineJoin, StrokeStyle, TextAlign, TextBaseline,
    TextStyle,
};
pub use renderer::Renderer;
