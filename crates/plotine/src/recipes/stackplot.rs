use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

/// Cumulative stacked bands: returns one closed path per series (bottom→top).
pub fn stackplot_paths(x: &[f64], ys: &[&[f64]], transform: &DataToPixel) -> Vec<BezPath> {
    if ys.is_empty() {
        return Vec::new();
    }
    let n = x.len();
    let mut cum_lo = vec![0.0_f64; n];
    let mut out = Vec::with_capacity(ys.len());

    for series in ys {
        let m = n.min(series.len());
        let mut cum_hi = cum_lo.clone();
        for i in 0..m {
            let v = series[i];
            if x[i].is_finite() && v.is_finite() {
                cum_hi[i] = cum_lo[i] + v;
            }
        }
        let mut upper = Vec::new();
        let mut lower = Vec::new();
        for i in 0..m {
            if x[i].is_finite() && series[i].is_finite() {
                upper.push(Point::new(x[i], cum_hi[i]));
                lower.push(Point::new(x[i], cum_lo[i]));
            }
        }
        let mut path = BezPath::new();
        if let Some(first) = upper.first() {
            path.move_to(transform.map(*first).to_kurbo());
            for p in upper.iter().skip(1) {
                path.line_to(transform.map(*p).to_kurbo());
            }
            for p in lower.iter().rev() {
                path.line_to(transform.map(*p).to_kurbo());
            }
            path.close_path();
            out.push(path);
        }
        cum_lo = cum_hi;
    }
    out
}

/// Max cumulative height for autoscaling.
pub fn stackplot_ymax(ys: &[&[f64]]) -> Option<f64> {
    if ys.is_empty() {
        return None;
    }
    let n = ys.iter().map(|s| s.len()).min().unwrap_or(0);
    if n == 0 {
        return None;
    }
    let mut max = f64::NEG_INFINITY;
    for i in 0..n {
        let mut sum = 0.0;
        for s in ys {
            let v = s[i];
            if v.is_finite() {
                sum += v;
            }
        }
        max = max.max(sum);
    }
    if max.is_finite() {
        Some(max)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn two_layers() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 5.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let y0 = [1.0, 1.0, 1.0];
        let y1 = [2.0, 2.0, 2.0];
        let paths = stackplot_paths(&[0.0, 1.0, 2.0], &[&y0[..], &y1[..]], &t);
        assert_eq!(paths.len(), 2);
        assert_eq!(stackplot_ymax(&[&y0[..], &y1[..]]), Some(3.0));
    }
}
