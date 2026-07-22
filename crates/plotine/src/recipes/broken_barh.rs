use crate::recipes::bar::BarRect;

/// Horizontal broken bars: each `(xmin, width)` spanning `[y, y+height]`
/// (matplotlib `broken_barh` yrange = `(ymin, height)`).
pub fn broken_barh_rects(xranges: &[(f64, f64)], y: f64, height: f64) -> Vec<BarRect> {
    let h = height.abs();
    let mut out = Vec::with_capacity(xranges.len());
    for &(xmin, width) in xranges {
        if !(xmin.is_finite() && width.is_finite() && y.is_finite()) {
            continue;
        }
        let x0 = xmin.min(xmin + width);
        let x1 = xmin.max(xmin + width);
        out.push(BarRect {
            x0,
            y0: y,
            x1,
            y1: y + h,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_bars() {
        let r = broken_barh_rects(&[(10.0, 50.0), (100.0, 20.0)], 20.0, 9.0);
        assert_eq!(r.len(), 2);
        assert!((r[0].x1 - r[0].x0 - 50.0).abs() < 1e-9);
    }
}
