//! Text measurement and rasterization backed by cosmic-text.
//!
//! The concrete engine is hidden behind [`TextEngine`] so M0 can swap cosmic-text
//! for parley later without touching chart layout code.

mod engine;
mod fonts;

pub use engine::{
    GlyphContent, GlyphImage, TextEngine, EMBEDDED_FONT, EMBEDDED_FONT_OBLIQUE, FONT_FAMILY,
};
pub use fonts::{
    register_font_data, register_font_file, registered_families, registered_font_data,
    svg_font_family_list,
};
