//! Line dash patterns, marker shapes, hatch fills, and spine visibility.

/// Stroke dash pattern for lines (matplotlib `linestyle`).
///
/// Patterns are scaled by stroke width at draw time so they stay readable
/// across DPI settings.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineStyle {
    /// Solid line (`'-'`).
    #[default]
    Solid,
    /// Dashed line (`'--'`).
    Dashed,
    /// Dotted line (`':'`).
    Dotted,
    /// Dash–dot line (`'-.'`).
    DashDot,
}

impl LineStyle {
    /// Pixel dash array for a given stroke width in pixels.
    ///
    /// Returns `None` for [`LineStyle::Solid`].
    pub fn dash_pattern(self, width_px: f64) -> Option<Vec<f64>> {
        let w = width_px.max(0.5);
        match self {
            Self::Solid => None,
            Self::Dashed => Some(vec![6.0 * w, 4.0 * w]),
            Self::Dotted => Some(vec![1.0 * w, 2.5 * w]),
            Self::DashDot => Some(vec![6.0 * w, 3.0 * w, 1.0 * w, 3.0 * w]),
        }
    }
}

/// Marker shape for scatter / stem / spy (matplotlib `marker`).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MarkerStyle {
    /// Filled circle (`'o'`).
    #[default]
    Circle,
    /// Filled square (`'s'`).
    Square,
    /// Filled upward triangle (`'^'`).
    Triangle,
    /// Filled downward triangle (`'v'`).
    TriangleDown,
    /// Filled diamond (`'D'`).
    Diamond,
    /// Plus / crosshair (`'+'`, stroked).
    Plus,
    /// X mark (`'x'`, stroked).
    Cross,
    /// Five-point star (`'*'`, filled).
    Star,
    /// Tiny filled point (`'.'`).
    Point,
}

impl MarkerStyle {
    /// `true` when the marker is drawn as a stroke (not a filled path).
    pub fn is_stroke(self) -> bool {
        matches!(self, Self::Plus | Self::Cross)
    }
}

/// Fill hatch pattern (matplotlib `hatch`) for bars / patches.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Hatch {
    /// Solid fill only (no hatch).
    #[default]
    None,
    /// Diagonal lines `/`.
    Diagonal,
    /// Diagonal lines `\`.
    DiagonalBack,
    /// Crossed diagonals `x`.
    Cross,
    /// Horizontal lines `-`.
    Horizontal,
    /// Vertical lines `|`.
    Vertical,
    /// Grid `+` (horizontal + vertical).
    Grid,
    /// Sparse dots `.`.
    Dots,
}

impl Hatch {
    /// `true` when hatch strokes should be drawn over the fill.
    pub fn is_drawn(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Which box spines (axes borders) are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spines {
    /// Left y-spine.
    pub left: bool,
    /// Bottom x-spine.
    pub bottom: bool,
    /// Right y-spine.
    pub right: bool,
    /// Top x-spine.
    pub top: bool,
}

impl Default for Spines {
    fn default() -> Self {
        Self::all()
    }
}

impl Spines {
    /// All four spines visible (matplotlib default).
    pub fn all() -> Self {
        Self {
            left: true,
            bottom: true,
            right: true,
            top: true,
        }
    }

    /// Only left + bottom (common paper / seaborn `despine` style).
    pub fn bottom_left() -> Self {
        Self {
            left: true,
            bottom: true,
            right: false,
            top: false,
        }
    }

    /// Show/hide the top spine.
    pub fn top(mut self, on: bool) -> Self {
        self.top = on;
        self
    }

    /// Show/hide the right spine.
    pub fn right(mut self, on: bool) -> Self {
        self.right = on;
        self
    }

    /// Show/hide the bottom spine.
    pub fn bottom(mut self, on: bool) -> Self {
        self.bottom = on;
        self
    }

    /// Show/hide the left spine.
    pub fn left(mut self, on: bool) -> Self {
        self.left = on;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_has_no_dash() {
        assert!(LineStyle::Solid.dash_pattern(2.0).is_none());
        assert_eq!(LineStyle::Dashed.dash_pattern(2.0).unwrap().len(), 2);
        assert_eq!(LineStyle::DashDot.dash_pattern(1.0).unwrap().len(), 4);
    }

    #[test]
    fn spines_bottom_left() {
        let s = Spines::bottom_left();
        assert!(s.left && s.bottom && !s.top && !s.right);
    }
}
