//! Data-space patch geometry (matplotlib `Rectangle` / `Circle` / `Ellipse`).

use kurbo::{BezPath, Point as KurboPoint};
use plotine_core::{DataToPixel, Point, Rect};

/// Axis-aligned rectangle `[x, x+width] × [y, y+height]` in data coords.
pub fn rectangle_data_rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Option<(f64, f64, f64, f64)> {
    if ![x, y, width, height].iter().all(|v| v.is_finite()) {
        return None;
    }
    let x0 = x.min(x + width);
    let x1 = x.max(x + width);
    let y0 = y.min(y + height);
    let y1 = y.max(y + height);
    if (x1 - x0) < 1e-15 || (y1 - y0) < 1e-15 {
        return None;
    }
    Some((x0, y0, x1, y1))
}

/// Pixel rect for an axis-aligned data rectangle.
pub fn rectangle_pixel_rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    transform: &DataToPixel,
) -> Option<Rect> {
    let (x0, y0, x1, y1) = rectangle_data_rect(x, y, width, height)?;
    let p00 = transform.map(Point::new(x0, y0));
    let p11 = transform.map(Point::new(x1, y1));
    Some(Rect::new(
        p00.x.min(p11.x),
        p00.y.min(p11.y),
        p00.x.max(p11.x),
        p00.y.max(p11.y),
    ))
}

/// Closed ellipse path centered at `(cx, cy)` with data-space diameters `width` / `height`.
pub fn ellipse_path(
    cx: f64,
    cy: f64,
    width: f64,
    height: f64,
    transform: &DataToPixel,
    segments: usize,
) -> BezPath {
    let mut path = BezPath::new();
    if ![cx, cy, width, height].iter().all(|v| v.is_finite()) {
        return path;
    }
    let rx = width.abs() * 0.5;
    let ry = height.abs() * 0.5;
    if rx < 1e-15 || ry < 1e-15 {
        return path;
    }
    let n = segments.max(8);
    for i in 0..=n {
        let t = std::f64::consts::TAU * (i as f64) / (n as f64);
        let p = transform.map(Point::new(cx + rx * t.cos(), cy + ry * t.sin()));
        let k = KurboPoint::new(p.x, p.y);
        if i == 0 {
            path.move_to(k);
        } else {
            path.line_to(k);
        }
    }
    path.close_path();
    path
}

/// Circle path (data-space radius) — ellipse with equal diameters.
pub fn circle_path(
    cx: f64,
    cy: f64,
    radius: f64,
    transform: &DataToPixel,
    segments: usize,
) -> BezPath {
    let d = radius.abs() * 2.0;
    ellipse_path(cx, cy, d, d, transform, segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, ScaleKind};

    fn unit_t() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        )
    }

    #[test]
    fn rectangle_normalizes_negative_size() {
        let (x0, y0, x1, y1) = rectangle_data_rect(3.0, 4.0, -2.0, -1.0).unwrap();
        assert!((x0 - 1.0).abs() < 1e-12);
        assert!((y0 - 3.0).abs() < 1e-12);
        assert!((x1 - 3.0).abs() < 1e-12);
        assert!((y1 - 4.0).abs() < 1e-12);
    }

    #[test]
    fn circle_path_closes() {
        let path = circle_path(5.0, 5.0, 2.0, &unit_t(), 32);
        assert!(path.elements().len() > 8);
    }
}
