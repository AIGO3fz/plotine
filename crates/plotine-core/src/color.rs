//! Strongly-typed sRGB colors with perceptually-minded defaults.

use std::fmt;
use std::str::FromStr;

/// Error returned by [`Color::from_str`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorParseError {
    /// The original input that failed to parse.
    pub input: String,
    /// How to fix the call (for humans and agents).
    pub suggestion: &'static str,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown color `{}`. suggestion: {}",
            self.input, self.suggestion
        )
    }
}

impl std::error::Error for ColorParseError {}

/// 8-bit sRGB color with optional alpha.
///
/// Use named constants like [`Color::CRIMSON`] or construct via
/// [`Color::rgb()`] / [`Color::from_hex()`]. The default categorical color
/// cycle is available in [`DEFAULT_CYCLE`].
///
/// String parsing is supported for convenience (`"crimson"`, `"#dc143c"`),
/// but typed constants are preferred in generated code.
///
/// ```
/// use plotine_core::Color;
/// use std::str::FromStr;
///
/// assert_eq!(Color::from_str("crimson").unwrap(), Color::CRIMSON);
/// assert_eq!(Color::from_str("#4682b4").unwrap(), Color::STEEL_BLUE);
/// assert_eq!(Color::from_str("dc143c").unwrap(), Color::CRIMSON);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const fn from_hex(hex: u32) -> Self {
        Self::rgb(
            ((hex >> 16) & 0xff) as u8,
            ((hex >> 8) & 0xff) as u8,
            (hex & 0xff) as u8,
        )
    }

    /// Premultiplied floats in 0..=1 for raster backends.
    pub fn to_f32_premultiplied(self) -> [f32; 4] {
        let a = self.a as f32 / 255.0;
        [
            self.r as f32 / 255.0 * a,
            self.g as f32 / 255.0 * a,
            self.b as f32 / 255.0 * a,
            a,
        ]
    }

    pub fn to_rgba_u8(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Return a copy with alpha in `0.0..=1.0`.
    pub fn with_alpha(self, alpha: f64) -> Self {
        let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
        Self::rgba(self.r, self.g, self.b, a)
    }

    // Named palette — ink matches matplotlib stock (`text.color` / labels = black).
    pub const BLACK: Self = Self::rgb(0x00, 0x00, 0x00);
    pub const WHITE: Self = Self::rgb(0xff, 0xff, 0xff);
    pub const GRID: Self = Self::rgb(0xde, 0xe2, 0xe6);
    pub const SPINE: Self = Self::rgb(0x49, 0x50, 0x57);
    pub const TICK: Self = Self::rgb(0x49, 0x50, 0x57);
    pub const LABEL: Self = Self::rgb(0x00, 0x00, 0x00);
    pub const TITLE: Self = Self::rgb(0x00, 0x00, 0x00);
    pub const BACKGROUND: Self = Self::rgb(0xff, 0xff, 0xff);
    pub const AXES_FACE: Self = Self::rgb(0xfa, 0xfb, 0xfc);

    pub const CRIMSON: Self = Self::rgb(0xdc, 0x14, 0x3c);
    pub const STEEL_BLUE: Self = Self::rgb(0x46, 0x82, 0xb4);
    pub const FOREST_GREEN: Self = Self::rgb(0x22, 0x8b, 0x22);
    pub const DARK_ORANGE: Self = Self::rgb(0xff, 0x8c, 0x00);
    /// Matplotlib tableau "C1" — default boxplot median color.
    pub const TAB_ORANGE: Self = Self::rgb(0xff, 0x7f, 0x0e);
    pub const MEDIUM_PURPLE: Self = Self::rgb(0x93, 0x70, 0xdb);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl FromStr for Color {
    type Err = ColorParseError;

    /// Parse a CSS-like color name or hex string.
    ///
    /// Accepted forms:
    /// - Named: `crimson`, `steelblue` / `steel_blue`, `tab_orange`, …
    /// - Hex: `#rgb`, `#rrggbb`, `#rrggbbaa`, or the same without `#`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s.trim();
        if raw.is_empty() {
            return Err(ColorParseError {
                input: s.to_string(),
                suggestion: "pass a color name like \"crimson\" or hex like \"#dc143c\"",
            });
        }

        let lower = raw.to_ascii_lowercase();
        if let Some(c) = named_color(&lower) {
            return Ok(c);
        }

        if let Some(c) = parse_hex_color(&lower) {
            return Ok(c);
        }

        Err(ColorParseError {
            input: s.to_string(),
            suggestion: "use Color::CRIMSON / Color::rgb(r,g,b), or a known name / #rrggbb hex",
        })
    }
}

fn named_color(name: &str) -> Option<Color> {
    Some(match name {
        "black" => Color::BLACK,
        "white" => Color::WHITE,
        "grid" => Color::GRID,
        "spine" | "tick" => Color::SPINE,
        "label" => Color::LABEL,
        "title" => Color::TITLE,
        "background" | "bg" => Color::BACKGROUND,
        "axes_face" | "axesface" | "axes-face" => Color::AXES_FACE,
        "crimson" | "red" => Color::CRIMSON,
        "steelblue" | "steel_blue" | "steel-blue" | "blue" => Color::STEEL_BLUE,
        "forestgreen" | "forest_green" | "forest-green" | "green" => Color::FOREST_GREEN,
        "darkorange" | "dark_orange" | "dark-orange" | "orange" => Color::DARK_ORANGE,
        "taborange" | "tab_orange" | "tab-orange" | "c1" => Color::TAB_ORANGE,
        "mediumpurple" | "medium_purple" | "medium-purple" | "purple" => Color::MEDIUM_PURPLE,
        "cyan" | "teal" => Color::rgb(0x17, 0xa2, 0xb8),
        _ => return None,
    })
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 0x11;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 0x11;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 0x11;
            Some(Color::rgb(r, g, b))
        }
        6 => {
            let n = u32::from_str_radix(hex, 16).ok()?;
            Some(Color::from_hex(n))
        }
        8 => {
            let n = u32::from_str_radix(hex, 16).ok()?;
            Some(Color::rgba(
                ((n >> 24) & 0xff) as u8,
                ((n >> 16) & 0xff) as u8,
                ((n >> 8) & 0xff) as u8,
                (n & 0xff) as u8,
            ))
        }
        _ => None,
    }
}

/// Default categorical cycle — matplotlib `tab10` / `axes.prop_cycle`.
pub const DEFAULT_CYCLE: [Color; 10] = [
    Color::rgb(0x1f, 0x77, 0xb4), // C0
    Color::TAB_ORANGE,            // C1 #ff7f0e
    Color::rgb(0x2c, 0xa0, 0x2c), // C2
    Color::rgb(0xd6, 0x27, 0x28), // C3
    Color::rgb(0x94, 0x67, 0xbd), // C4
    Color::rgb(0x8c, 0x56, 0x4b), // C5
    Color::rgb(0xe3, 0x77, 0xc2), // C6
    Color::rgb(0x7f, 0x7f, 0x7f), // C7
    Color::rgb(0xbc, 0xbd, 0x22), // C8
    Color::rgb(0x17, 0xbe, 0xcf), // C9
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_and_hex() {
        assert_eq!(Color::from_str("Crimson").unwrap(), Color::CRIMSON);
        assert_eq!(Color::from_str("steel_blue").unwrap(), Color::STEEL_BLUE);
        assert_eq!(Color::from_str("#dc143c").unwrap(), Color::CRIMSON);
        assert_eq!(Color::from_str("dc143c").unwrap(), Color::CRIMSON);
        assert_eq!(Color::from_str("#f00").unwrap(), Color::rgb(255, 0, 0));
        assert_eq!(
            Color::from_str("#4682b480").unwrap(),
            Color::rgba(0x46, 0x82, 0xb4, 0x80)
        );
    }

    #[test]
    fn unknown_name_has_suggestion() {
        let err = Color::from_str("chartreuse").unwrap_err();
        assert!(err.suggestion.contains("Color::"));
        assert!(err.to_string().contains("suggestion:"));
    }
}
