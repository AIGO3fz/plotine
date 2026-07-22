use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

/// Filled area under a polyline down to `baseline` in data space.
pub fn area_path(x: &[f64], y: &[f64], baseline: f64, transform: &DataToPixel) -> BezPath {
    let mut path = BezPath::new();
    let mut pts: Vec<Point> = Vec::new();
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        if xi.is_finite() && yi.is_finite() {
            pts.push(Point::new(xi, yi));
        }
    }
    if pts.is_empty() {
        return path;
    }

    let first = pts[0];
    let last = *pts.last().unwrap();
    path.move_to(transform.map(Point::new(first.x, baseline)).to_kurbo());
    for p in &pts {
        path.line_to(transform.map(*p).to_kurbo());
    }
    path.line_to(transform.map(Point::new(last.x, baseline)).to_kurbo());
    path.close_path();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn closes_polygon() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let path = area_path(&[0.0, 1.0, 2.0], &[1.0, 2.0, 1.0], 0.0, &t);
        assert!(path.elements().len() >= 4);
    }
}
