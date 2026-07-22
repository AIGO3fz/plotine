use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

/// Build a polyline in pixel space from aligned x/y samples.
pub fn line_path(x: &[f64], y: &[f64], transform: &DataToPixel) -> BezPath {
    let mut path = BezPath::new();
    let mut started = false;
    // Collapse runs that round to the same pixel — critical for N≫width (e.g. 1e6
    // points into a ~500px axes box) without changing the visible polyline.
    let mut last_key: Option<(i32, i32)> = None;
    let mut pending: Option<Point> = None;
    let flush_pending = |path: &mut BezPath, pending: &mut Option<Point>, started: &mut bool| {
        if let Some(p) = pending.take() {
            if *started {
                path.line_to(p.to_kurbo());
            }
        }
    };
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        if !xi.is_finite() || !yi.is_finite() {
            flush_pending(&mut path, &mut pending, &mut started);
            started = false;
            last_key = None;
            continue;
        }
        let p = transform.map(Point::new(xi, yi));
        let key = (p.x.round() as i32, p.y.round() as i32);
        if !started {
            path.move_to(p.to_kurbo());
            started = true;
            last_key = Some(key);
            pending = None;
            continue;
        }
        if last_key == Some(key) {
            pending = Some(p);
            continue;
        }
        flush_pending(&mut path, &mut pending, &mut started);
        path.line_to(p.to_kurbo());
        last_key = Some(key);
        pending = None;
    }
    flush_pending(&mut path, &mut pending, &mut started);
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn breaks_on_nan() {
        let x = [0.0, 1.0, 2.0, 3.0];
        let y = [0.0, f64::NAN, 1.0, 2.0];
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 3.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 300.0, 200.0),
        );
        let path = line_path(&x, &y, &t);
        // NaN breaks the polyline → at least 2 subpaths (move + lines)
        assert!(path.elements().len() >= 3);
    }
}
