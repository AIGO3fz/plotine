use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

/// Closed polygon filling the region between `(x, y1)` and `(x, y2)`.
///
/// Walks along `y1` left→right, then along `y2` right→left (matplotlib-style).
pub fn fill_between_path(x: &[f64], y1: &[f64], y2: &[f64], transform: &DataToPixel) -> BezPath {
    let mut path = BezPath::new();
    let n = x.len().min(y1.len()).min(y2.len());
    let mut upper: Vec<Point> = Vec::new();
    let mut lower: Vec<Point> = Vec::new();
    for i in 0..n {
        let xi = x[i];
        let a = y1[i];
        let b = y2[i];
        if xi.is_finite() && a.is_finite() && b.is_finite() {
            upper.push(Point::new(xi, a));
            lower.push(Point::new(xi, b));
        }
    }
    if upper.is_empty() {
        return path;
    }

    path.move_to(transform.map(upper[0]).to_kurbo());
    for p in upper.iter().skip(1) {
        path.line_to(transform.map(*p).to_kurbo());
    }
    for p in lower.iter().rev() {
        path.line_to(transform.map(*p).to_kurbo());
    }
    path.close_path();
    path
}

/// Closed polygon filling between `(x1, y)` and `(x2, y)` (vertical bands).
pub fn fill_betweenx_path(y: &[f64], x1: &[f64], x2: &[f64], transform: &DataToPixel) -> BezPath {
    let mut path = BezPath::new();
    let n = y.len().min(x1.len()).min(x2.len());
    let mut left: Vec<Point> = Vec::new();
    let mut right: Vec<Point> = Vec::new();
    for i in 0..n {
        let yi = y[i];
        let a = x1[i];
        let b = x2[i];
        if yi.is_finite() && a.is_finite() && b.is_finite() {
            left.push(Point::new(a, yi));
            right.push(Point::new(b, yi));
        }
    }
    if left.is_empty() {
        return path;
    }

    path.move_to(transform.map(left[0]).to_kurbo());
    for p in left.iter().skip(1) {
        path.line_to(transform.map(*p).to_kurbo());
    }
    for p in right.iter().rev() {
        path.line_to(transform.map(*p).to_kurbo());
    }
    path.close_path();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn unit_transform() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        )
    }

    #[test]
    fn fill_between_closes() {
        let t = unit_transform();
        let path = fill_between_path(&[0.0, 1.0, 2.0], &[1.0, 2.0, 1.0], &[0.0, 0.5, 0.0], &t);
        assert!(path.elements().len() >= 5);
    }

    #[test]
    fn fill_betweenx_closes() {
        let t = unit_transform();
        let path = fill_betweenx_path(&[0.0, 1.0, 2.0], &[0.0, 0.0, 0.0], &[1.0, 1.5, 1.0], &t);
        assert!(path.elements().len() >= 5);
    }
}
