use plotine_core::{DataToPixel, Point};

/// One vertical error bar in pixel space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErrorBarGeom {
    pub x: f64,
    pub y_lo: f64,
    pub y_hi: f64,
    pub y_mid: f64,
}

/// One horizontal error bar in pixel space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErrorBarXGeom {
    pub y: f64,
    pub x_lo: f64,
    pub x_hi: f64,
    pub x_mid: f64,
}

/// Map `(x, y - lower … y + upper)` into pixel geometry. Non-finite samples are skipped.
///
/// Symmetric bars use `lower == upper` (matplotlib 1-D `yerr`). Asymmetric bars
/// match matplotlib `yerr` of shape `(2, N)` where row 0 is the lower error and
/// row 1 is the upper error (both non-negative magnitudes).
pub fn errorbar_geoms_asym(
    x: &[f64],
    y: &[f64],
    lower: &[f64],
    upper: &[f64],
    transform: &DataToPixel,
) -> Vec<ErrorBarGeom> {
    x.iter()
        .zip(y.iter())
        .zip(lower.iter().zip(upper.iter()))
        .filter_map(|((&xi, &yi), (&lo_e, &hi_e))| {
            if !(xi.is_finite() && yi.is_finite() && lo_e.is_finite() && hi_e.is_finite()) {
                return None;
            }
            let lo = transform.map(Point::new(xi, yi - lo_e.abs()));
            let hi = transform.map(Point::new(xi, yi + hi_e.abs()));
            let mid = transform.map(Point::new(xi, yi));
            Some(ErrorBarGeom {
                x: mid.x,
                // screen y grows downward; data-lo may map above data-hi
                y_lo: lo.y.max(hi.y),
                y_hi: lo.y.min(hi.y),
                y_mid: mid.y,
            })
        })
        .collect()
}

/// Map `(x, y ± yerr)` into pixel geometry (symmetric).
pub fn errorbar_geoms(
    x: &[f64],
    y: &[f64],
    yerr: &[f64],
    transform: &DataToPixel,
) -> Vec<ErrorBarGeom> {
    errorbar_geoms_asym(x, y, yerr, yerr, transform)
}

/// Map `(x - lower … x + upper, y)` into pixel geometry.
pub fn errorbar_x_geoms_asym(
    x: &[f64],
    y: &[f64],
    lower: &[f64],
    upper: &[f64],
    transform: &DataToPixel,
) -> Vec<ErrorBarXGeom> {
    x.iter()
        .zip(y.iter())
        .zip(lower.iter().zip(upper.iter()))
        .filter_map(|((&xi, &yi), (&lo_e, &hi_e))| {
            if !(xi.is_finite() && yi.is_finite() && lo_e.is_finite() && hi_e.is_finite()) {
                return None;
            }
            let lo = transform.map(Point::new(xi - lo_e.abs(), yi));
            let hi = transform.map(Point::new(xi + hi_e.abs(), yi));
            let mid = transform.map(Point::new(xi, yi));
            Some(ErrorBarXGeom {
                y: mid.y,
                x_lo: lo.x.min(hi.x),
                x_hi: lo.x.max(hi.x),
                x_mid: mid.x,
            })
        })
        .collect()
}

/// Map `(x ± xerr, y)` into pixel geometry (symmetric).
pub fn errorbar_x_geoms(
    x: &[f64],
    y: &[f64],
    xerr: &[f64],
    transform: &DataToPixel,
) -> Vec<ErrorBarXGeom> {
    errorbar_x_geoms_asym(x, y, xerr, xerr, transform)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn unit_transform() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 1.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        )
    }

    #[test]
    fn skips_nan() {
        let t = unit_transform();
        let g = errorbar_geoms(&[0.0, 1.0], &[1.0, f64::NAN], &[0.2, 0.2], &t);
        assert_eq!(g.len(), 1);
    }

    #[test]
    fn horizontal_skips_nan() {
        let t = unit_transform();
        let g = errorbar_x_geoms(&[0.0, f64::NAN], &[1.0, 1.0], &[0.1, 0.1], &t);
        assert_eq!(g.len(), 1);
        assert!(g[0].x_lo < g[0].x_hi);
    }

    #[test]
    fn asymmetric_vertical_uses_separate_arms() {
        let t = unit_transform();
        // y=1, lower=0.5 → data 0.5; upper=0.25 → data 1.25
        let g = errorbar_geoms_asym(&[0.5], &[1.0], &[0.5], &[0.25], &t);
        assert_eq!(g.len(), 1);
        let mid = t.map(Point::new(0.5, 1.0)).y;
        let lo = t.map(Point::new(0.5, 0.5)).y;
        let hi = t.map(Point::new(0.5, 1.25)).y;
        assert!((g[0].y_mid - mid).abs() < 1e-9);
        assert!((g[0].y_lo - lo.max(hi)).abs() < 1e-9);
        assert!((g[0].y_hi - lo.min(hi)).abs() < 1e-9);
    }
}
