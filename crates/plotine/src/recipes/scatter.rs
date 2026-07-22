use kurbo::{BezPath, Circle, Point as KurboPoint, Shape};
use plotine_core::{DataToPixel, Point};

use crate::style::MarkerStyle;

/// Marker geometry in pixel space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Marker {
    pub center: Point,
    pub radius: f64,
}

/// Place circular markers for each finite (x, y) pair.
pub fn scatter_markers(x: &[f64], y: &[f64], transform: &DataToPixel, radius: f64) -> Vec<Marker> {
    x.iter()
        .zip(y.iter())
        .filter_map(|(&xi, &yi)| {
            if xi.is_finite() && yi.is_finite() {
                Some(Marker {
                    center: transform.map(Point::new(xi, yi)),
                    radius,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Bézier flatten tolerance for disk markers (pixels).
fn circle_tol(radius: f64) -> f64 {
    // ~8–12 segments for typical marker sizes; `0.1` was far denser than visible.
    (radius * 0.35).clamp(0.35, 1.25)
}

/// Build a pixel-space path for `marker` using the given [`MarkerStyle`].
pub fn marker_path(marker: Marker, style: MarkerStyle) -> BezPath {
    let c = marker.center;
    let r = marker.radius.max(0.5);
    match style {
        // Coarser tessellation than 0.1: tiny markers do not need dense cubics
        // (was a major cost at N≈1e4 after path batching).
        MarkerStyle::Circle => Circle::new(c.to_kurbo(), r).to_path(circle_tol(r)),
        MarkerStyle::Point => Circle::new(c.to_kurbo(), r * 0.35).to_path(circle_tol(r * 0.35)),
        MarkerStyle::Square => {
            let mut path = BezPath::new();
            path.move_to(KurboPoint::new(c.x - r, c.y - r));
            path.line_to(KurboPoint::new(c.x + r, c.y - r));
            path.line_to(KurboPoint::new(c.x + r, c.y + r));
            path.line_to(KurboPoint::new(c.x - r, c.y + r));
            path.close_path();
            path
        }
        MarkerStyle::Triangle => {
            // Equilateral-ish: tip up, base below center.
            let h = r * 1.732; // ≈ √3 · r for height of equilateral with side 2r
            let mut path = BezPath::new();
            path.move_to(KurboPoint::new(c.x, c.y - h * 0.6));
            path.line_to(KurboPoint::new(c.x + r, c.y + h * 0.4));
            path.line_to(KurboPoint::new(c.x - r, c.y + h * 0.4));
            path.close_path();
            path
        }
        MarkerStyle::TriangleDown => {
            let h = r * 1.732;
            let mut path = BezPath::new();
            path.move_to(KurboPoint::new(c.x, c.y + h * 0.6));
            path.line_to(KurboPoint::new(c.x + r, c.y - h * 0.4));
            path.line_to(KurboPoint::new(c.x - r, c.y - h * 0.4));
            path.close_path();
            path
        }
        MarkerStyle::Diamond => {
            let mut path = BezPath::new();
            path.move_to(KurboPoint::new(c.x, c.y - r));
            path.line_to(KurboPoint::new(c.x + r, c.y));
            path.line_to(KurboPoint::new(c.x, c.y + r));
            path.line_to(KurboPoint::new(c.x - r, c.y));
            path.close_path();
            path
        }
        MarkerStyle::Plus => {
            let mut path = BezPath::new();
            path.move_to(KurboPoint::new(c.x - r, c.y));
            path.line_to(KurboPoint::new(c.x + r, c.y));
            path.move_to(KurboPoint::new(c.x, c.y - r));
            path.line_to(KurboPoint::new(c.x, c.y + r));
            path
        }
        MarkerStyle::Cross => {
            let mut path = BezPath::new();
            let d = r * std::f64::consts::FRAC_1_SQRT_2;
            path.move_to(KurboPoint::new(c.x - d, c.y - d));
            path.line_to(KurboPoint::new(c.x + d, c.y + d));
            path.move_to(KurboPoint::new(c.x + d, c.y - d));
            path.line_to(KurboPoint::new(c.x - d, c.y + d));
            path
        }
        MarkerStyle::Star => star_path(c, r),
    }
}

fn star_path(center: Point, radius: f64) -> BezPath {
    // 5-point star: outer radius `radius`, inner ≈ 0.4 · outer.
    let outer = radius;
    let inner = radius * 0.4;
    let mut pts = [(0.0_f64, 0.0_f64); 10];
    for (i, pt) in pts.iter_mut().enumerate() {
        let ang = -std::f64::consts::FRAC_PI_2 + i as f64 * std::f64::consts::PI / 5.0;
        let rad = if i % 2 == 0 { outer } else { inner };
        *pt = (center.x + rad * ang.cos(), center.y + rad * ang.sin());
    }
    let mut path = BezPath::new();
    path.move_to(KurboPoint::new(pts[0].0, pts[0].1));
    for p in pts.iter().skip(1) {
        path.line_to(KurboPoint::new(p.0, p.1));
    }
    path.close_path();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::PathEl;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn skips_non_finite() {
        let x = [0.0, 1.0];
        let y = [0.0, f64::INFINITY];
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 1.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 1.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        assert_eq!(scatter_markers(&x, &y, &t, 3.0).len(), 1);
    }

    #[test]
    fn each_marker_style_produces_non_empty_path() {
        let m = Marker {
            center: Point::new(10.0, 20.0),
            radius: 4.0,
        };
        let styles = [
            MarkerStyle::Circle,
            MarkerStyle::Square,
            MarkerStyle::Triangle,
            MarkerStyle::TriangleDown,
            MarkerStyle::Diamond,
            MarkerStyle::Plus,
            MarkerStyle::Cross,
            MarkerStyle::Star,
            MarkerStyle::Point,
        ];
        for style in styles {
            let path = marker_path(m, style);
            assert!(
                path.elements()
                    .iter()
                    .any(|el| !matches!(el, PathEl::ClosePath)),
                "empty path for {style:?}"
            );
        }
    }
}
