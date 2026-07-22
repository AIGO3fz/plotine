use plotine_core::{DataToPixel, Point};

/// One stem: baseline → head, plus head marker center in pixel space.
#[derive(Debug, Clone, Copy)]
pub struct StemGeom {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub head: Point,
}

/// Vertical stems from `baseline` to each `(x[i], y[i])`.
pub fn stem_geoms(x: &[f64], y: &[f64], baseline: f64, transform: &DataToPixel) -> Vec<StemGeom> {
    let n = x.len().min(y.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let xi = x[i];
        let yi = y[i];
        if !(xi.is_finite() && yi.is_finite() && baseline.is_finite()) {
            continue;
        }
        let p0 = transform.map(Point::new(xi, baseline));
        let p1 = transform.map(Point::new(xi, yi));
        out.push(StemGeom {
            x0: p0.x,
            y0: p0.y,
            x1: p1.x,
            y1: p1.y,
            head: p1,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn one_stem_per_finite_point() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let g = stem_geoms(&[0.0, 1.0, f64::NAN], &[1.0, 2.0, 3.0], 0.0, &t);
        assert_eq!(g.len(), 2);
    }
}
