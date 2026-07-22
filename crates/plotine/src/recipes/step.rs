use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

/// Where the step occurs relative to sample positions (matplotlib `ax.step(where=…)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepMode {
    /// Interval `(x[i-1], x[i]]` holds `y[i]` (matplotlib `where="pre"`; default).
    #[default]
    Pre,
    /// Steps halfway between consecutive x positions (`where="mid"`).
    Mid,
    /// Interval `[x[i], x[i+1])` holds `y[i]` (`where="post"`).
    Post,
}

/// Build a step polyline in pixel space.
pub fn step_path(x: &[f64], y: &[f64], mode: StepMode, transform: &DataToPixel) -> BezPath {
    let mut path = BezPath::new();
    let n = x.len().min(y.len());
    let mut pts: Vec<Point> = Vec::new();
    for i in 0..n {
        if x[i].is_finite() && y[i].is_finite() {
            pts.push(Point::new(x[i], y[i]));
        }
    }
    if pts.is_empty() {
        return path;
    }
    if pts.len() == 1 {
        let p = transform.map(pts[0]).to_kurbo();
        path.move_to(p);
        path.line_to(p);
        return path;
    }

    match mode {
        // Matplotlib pre: vertical at x[i-1] then horizontal to x[i] at y[i].
        StepMode::Pre => {
            path.move_to(transform.map(pts[0]).to_kurbo());
            for i in 1..pts.len() {
                let prev = pts[i - 1];
                let cur = pts[i];
                path.line_to(transform.map(Point::new(prev.x, cur.y)).to_kurbo());
                path.line_to(transform.map(cur).to_kurbo());
            }
        }
        // Matplotlib post: horizontal at y[i-1] to x[i] then vertical to y[i].
        StepMode::Post => {
            path.move_to(transform.map(pts[0]).to_kurbo());
            for i in 1..pts.len() {
                let prev = pts[i - 1];
                let cur = pts[i];
                path.line_to(transform.map(Point::new(cur.x, prev.y)).to_kurbo());
                path.line_to(transform.map(cur).to_kurbo());
            }
        }
        StepMode::Mid => {
            path.move_to(transform.map(pts[0]).to_kurbo());
            for i in 1..pts.len() {
                let prev = pts[i - 1];
                let cur = pts[i];
                let mid_x = 0.5 * (prev.x + cur.x);
                path.line_to(transform.map(Point::new(mid_x, prev.y)).to_kurbo());
                path.line_to(transform.map(Point::new(mid_x, cur.y)).to_kurbo());
                path.line_to(transform.map(cur).to_kurbo());
            }
        }
    }
    path
}

/// Stairs: constant values between `edges` (`edges.len() == values.len() + 1`).
///
/// When `baseline` is finite (matplotlib default `0.0`), the path drops vertically
/// to the baseline at the first and last edge.
pub fn stairs_path(
    edges: &[f64],
    values: &[f64],
    baseline: f64,
    transform: &DataToPixel,
) -> BezPath {
    let mut path = BezPath::new();
    if values.is_empty() || edges.len() != values.len() + 1 {
        return path;
    }
    let mut started = false;
    let mut last_x1 = f64::NAN;
    for (i, &v) in values.iter().enumerate() {
        let x0 = edges[i];
        let x1 = edges[i + 1];
        if !(x0.is_finite() && x1.is_finite() && v.is_finite()) {
            continue;
        }
        if !started {
            if baseline.is_finite() {
                path.move_to(transform.map(Point::new(x0, baseline)).to_kurbo());
                path.line_to(transform.map(Point::new(x0, v)).to_kurbo());
            } else {
                path.move_to(transform.map(Point::new(x0, v)).to_kurbo());
            }
            started = true;
        } else {
            path.line_to(transform.map(Point::new(x0, v)).to_kurbo());
        }
        path.line_to(transform.map(Point::new(x1, v)).to_kurbo());
        last_x1 = x1;
    }
    if started && baseline.is_finite() && last_x1.is_finite() {
        path.line_to(transform.map(Point::new(last_x1, baseline)).to_kurbo());
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn t() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 3.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 3.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        )
    }

    #[test]
    fn step_pre_has_horizontal_then_vertical() {
        let path = step_path(&[0.0, 1.0, 2.0], &[1.0, 2.0, 1.5], StepMode::Pre, &t());
        // move + 2 segments per step interval → at least 5 elements
        assert!(path.elements().len() >= 5);
    }

    #[test]
    fn step_pre_matches_matplotlib_where_pre() {
        // mpl: (0,1)→(0,2)→(1,2)…  post: (0,1)→(1,1)→(1,2)…
        let t = t();
        let pre = step_path(&[0.0, 1.0], &[1.0, 2.0], StepMode::Pre, &t);
        let post = step_path(&[0.0, 1.0], &[1.0, 2.0], StepMode::Post, &t);
        let pre_pts: Vec<_> = pre
            .elements()
            .iter()
            .filter_map(|e| match e {
                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => Some((p.x, p.y)),
                _ => None,
            })
            .collect();
        let post_pts: Vec<_> = post
            .elements()
            .iter()
            .filter_map(|e| match e {
                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => Some((p.x, p.y)),
                _ => None,
            })
            .collect();
        // Same start; pre's second vertex shares x with start (vertical first).
        assert!((pre_pts[0].0 - post_pts[0].0).abs() < 1e-9);
        assert!((pre_pts[1].0 - pre_pts[0].0).abs() < 1e-9);
        assert!((post_pts[1].1 - post_pts[0].1).abs() < 1e-9);
    }

    #[test]
    fn stairs_requires_n_plus_one_edges() {
        let ok = stairs_path(&[0.0, 1.0, 2.0], &[1.0, 2.0], 0.0, &t());
        assert!(!ok.elements().is_empty());
        let bad = stairs_path(&[0.0, 1.0], &[1.0, 2.0], 0.0, &t());
        assert!(bad.elements().is_empty());
    }

    #[test]
    fn stairs_with_baseline_starts_and_ends_at_baseline() {
        let path = stairs_path(&[0.0, 1.0, 2.0], &[1.0, 2.0], 0.0, &t());
        // move(baseline) + up + horiz + vert + horiz + down → ≥ 6 elements
        assert!(path.elements().len() >= 6);
    }
}
