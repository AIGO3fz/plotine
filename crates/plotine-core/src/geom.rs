//! Lightweight geometry helpers shared across layers.

use kurbo::{Point as KurboPoint, Rect as KurboRect, Size as KurboSize};

/// 2D point in pixel or data space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn to_kurbo(self) -> KurboPoint {
        KurboPoint::new(self.x, self.y)
    }
}

impl From<(f64, f64)> for Point {
    fn from(value: (f64, f64)) -> Self {
        Self::new(value.0, value.1)
    }
}

/// Width / height pair.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn to_kurbo(self) -> KurboSize {
        KurboSize::new(self.width, self.height)
    }
}

/// Axis-aligned rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Rect {
    pub const fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self { x0, y0, x1, y1 }
    }

    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Self::new(
            origin.x,
            origin.y,
            origin.x + size.width,
            origin.y + size.height,
        )
    }

    pub fn width(self) -> f64 {
        self.x1 - self.x0
    }

    pub fn height(self) -> f64 {
        self.y1 - self.y0
    }

    pub fn center(self) -> Point {
        Point::new((self.x0 + self.x1) * 0.5, (self.y0 + self.y1) * 0.5)
    }

    pub fn contains(self, p: Point) -> bool {
        p.x >= self.x0 && p.x <= self.x1 && p.y >= self.y0 && p.y <= self.y1
    }

    pub fn inset(self, left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self::new(
            self.x0 + left,
            self.y0 + top,
            self.x1 - right,
            self.y1 - bottom,
        )
    }

    pub fn to_kurbo(self) -> KurboRect {
        KurboRect::new(self.x0, self.y0, self.x1, self.y1)
    }
}
