use plotine_core::{Color, Point};

#[derive(Debug, Clone, PartialEq)]
pub struct StrokeStyle {
    pub color: Color,
    pub width: f64,
    pub cap: LineCap,
    pub join: LineJoin,
    /// Dash pattern in pixels: alternating on/off lengths (even length ≥ 2).
    /// `None` means a solid stroke.
    pub dash: Option<Vec<f64>>,
}

impl StrokeStyle {
    pub fn new(color: Color, width: f64) -> Self {
        Self {
            color,
            width,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            dash: None,
        }
    }

    /// Attach a dash pattern (pixel units). Empty / odd-length patterns are ignored.
    pub fn with_dash(mut self, pattern: impl Into<Vec<f64>>) -> Self {
        let p: Vec<f64> = pattern.into();
        if p.len() >= 2 && p.len() % 2 == 0 && p.iter().all(|v| v.is_finite() && *v >= 0.0) {
            self.dash = Some(p);
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillStyle {
    pub color: Color,
    /// When false, raster backends fill without anti-aliasing (mesh cells).
    pub anti_alias: bool,
}

impl FillStyle {
    pub fn solid(color: Color) -> Self {
        Self {
            color,
            anti_alias: true,
        }
    }

    /// Opaque mesh fill: no AA, so adjacent cells do not leak the face color.
    pub fn solid_crisp(color: Color) -> Self {
        Self {
            color,
            anti_alias: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBaseline {
    Top,
    Middle,
    Alphabetic,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub color: Color,
    pub size_px: f32,
    pub align: TextAlign,
    pub baseline: TextBaseline,
    /// Clockwise degrees in screen space (y-down). Use `-90.0` for classic y-axis labels.
    pub rotation_deg: f64,
    /// Heavier stem coverage (contour clabels). Default body text stays regular.
    pub bold: bool,
    /// Oblique / italic face (mathtext variables; matplotlib math default).
    pub italic: bool,
}

impl TextStyle {
    pub fn new(color: Color, size_px: f32) -> Self {
        Self {
            color,
            size_px,
            align: TextAlign::Left,
            baseline: TextBaseline::Alphabetic,
            rotation_deg: 0.0,
            bold: false,
            italic: false,
        }
    }

    pub fn centered(mut self) -> Self {
        self.align = TextAlign::Center;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn baseline(mut self, baseline: TextBaseline) -> Self {
        self.baseline = baseline;
        self
    }

    pub fn rotation(mut self, degrees: f64) -> Self {
        self.rotation_deg = degrees;
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }
}

/// Convenience polyline builder from points.
pub fn polyline_from_points(points: &[Point]) -> kurbo::BezPath {
    let mut path = kurbo::BezPath::new();
    let mut iter = points.iter();
    if let Some(first) = iter.next() {
        path.move_to(first.to_kurbo());
        for p in iter {
            path.line_to(p.to_kurbo());
        }
    }
    path
}
