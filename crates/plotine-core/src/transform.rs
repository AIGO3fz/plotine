//! Coordinate transform pipeline: data → unit → pixel.

use crate::geom::{Point, Rect};
use crate::scale::ScaleKind;

/// 2D affine transform `x' = a*x + c`, `y' = b*y + d` (axis-aligned subset).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub sx: f64,
    pub sy: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Affine {
    pub const IDENTITY: Self = Self {
        sx: 1.0,
        sy: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub const fn new(sx: f64, sy: f64, tx: f64, ty: f64) -> Self {
        Self { sx, sy, tx, ty }
    }

    pub fn apply(self, p: Point) -> Point {
        Point::new(self.sx * p.x + self.tx, self.sy * p.y + self.ty)
    }

    pub fn then(self, other: Self) -> Self {
        Self {
            sx: other.sx * self.sx,
            sy: other.sy * self.sy,
            tx: other.sx * self.tx + other.tx,
            ty: other.sy * self.ty + other.ty,
        }
    }
}

/// Full data→pixel mapping for a single axes panel.
#[derive(Debug, Clone, Copy)]
pub struct DataToPixel {
    x_scale: ScaleKind,
    y_scale: ScaleKind,
    pixel_rect: Rect,
    /// When true, low data-y maps to the **top** of the panel (matplotlib
    /// `imshow(origin='upper')` default index box / inverted y-axis).
    invert_y: bool,
}

impl DataToPixel {
    pub fn new(x_scale: ScaleKind, y_scale: ScaleKind, pixel_rect: Rect) -> Self {
        Self {
            x_scale,
            y_scale,
            pixel_rect,
            invert_y: false,
        }
    }

    /// Enable / disable inverted y mapping (matplotlib `Axes.invert_yaxis`).
    pub fn with_invert_y(mut self, invert: bool) -> Self {
        self.invert_y = invert;
        self
    }

    pub fn invert_y(self) -> bool {
        self.invert_y
    }

    pub fn pixel_rect(self) -> Rect {
        self.pixel_rect
    }

    pub fn x_scale(self) -> ScaleKind {
        self.x_scale
    }

    pub fn y_scale(self) -> ScaleKind {
        self.y_scale
    }

    /// Map a data point into pixel space (y grows downward on screen).
    pub fn map(&self, data: Point) -> Point {
        let ux = self.x_scale.normalize(data.x);
        let uy = self.y_scale.normalize(data.y);
        let py = if self.invert_y {
            self.pixel_rect.y0 + uy * self.pixel_rect.height()
        } else {
            self.pixel_rect.y1 - uy * self.pixel_rect.height()
        };
        Point::new(self.pixel_rect.x0 + ux * self.pixel_rect.width(), py)
    }

    pub fn map_x(&self, x: f64) -> f64 {
        let ux = self.x_scale.normalize(x);
        self.pixel_rect.x0 + ux * self.pixel_rect.width()
    }

    pub fn map_y(&self, y: f64) -> f64 {
        let uy = self.y_scale.normalize(y);
        if self.invert_y {
            self.pixel_rect.y0 + uy * self.pixel_rect.height()
        } else {
            self.pixel_rect.y1 - uy * self.pixel_rect.height()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{LinearScale, ScaleKind};

    #[test]
    fn maps_corners() {
        let x = ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap());
        let y = ScaleKind::Linear(LinearScale::new(0.0, 5.0).unwrap());
        let rect = Rect::new(100.0, 50.0, 500.0, 350.0);
        let t = DataToPixel::new(x, y, rect);

        let bl = t.map(Point::new(0.0, 0.0));
        assert!((bl.x - 100.0).abs() < 1e-9);
        assert!((bl.y - 350.0).abs() < 1e-9);

        let tr = t.map(Point::new(10.0, 5.0));
        assert!((tr.x - 500.0).abs() < 1e-9);
        assert!((tr.y - 50.0).abs() < 1e-9);
    }

    #[test]
    fn inverted_y_puts_low_data_at_top() {
        let x = ScaleKind::Linear(LinearScale::new(0.0, 1.0).unwrap());
        let y = ScaleKind::Linear(LinearScale::new(-0.5, 1.5).unwrap());
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let t = DataToPixel::new(x, y, rect).with_invert_y(true);
        let top = t.map(Point::new(0.0, -0.5));
        let bot = t.map(Point::new(0.0, 1.5));
        assert!((top.y - 0.0).abs() < 1e-9);
        assert!((bot.y - 200.0).abs() < 1e-9);
    }
}
