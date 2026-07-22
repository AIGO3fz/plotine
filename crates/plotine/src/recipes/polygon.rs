use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

/// Closed polygon through `(x[i], y[i])` (auto-closes).
pub fn polygon_path(x: &[f64], y: &[f64], transform: &DataToPixel) -> BezPath {
    let mut path = BezPath::new();
    let n = x.len().min(y.len());
    let mut pts = Vec::new();
    for i in 0..n {
        if x[i].is_finite() && y[i].is_finite() {
            pts.push(Point::new(x[i], y[i]));
        }
    }
    if pts.len() < 3 {
        return path;
    }
    path.move_to(transform.map(pts[0]).to_kurbo());
    for p in pts.iter().skip(1) {
        path.line_to(transform.map(*p).to_kurbo());
    }
    path.close_path();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn triangle_closes() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let path = polygon_path(&[0.0, 1.0, 0.5], &[0.0, 0.0, 1.0], &t);
        assert!(path.elements().len() >= 4);
    }
}
