use crate::recipes::bar::{infer_bar_width, BarRect};

/// Horizontal bars centered on `y` with the given widths (extent along x).
pub fn barh_rects(y: &[f64], widths: &[f64], height: f64, baseline: f64) -> Vec<BarRect> {
    let n = y.len().min(widths.len());
    let half = height.abs() * 0.5;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let yi = y[i];
        let w = widths[i];
        if !(yi.is_finite() && w.is_finite()) {
            continue;
        }
        let x0 = baseline.min(baseline + w);
        let x1 = baseline.max(baseline + w);
        out.push(BarRect {
            x0,
            y0: yi - half,
            x1,
            y1: yi + half,
        });
    }
    out
}

/// Infer bar height from median spacing of y positions.
pub fn infer_barh_height(ys: &[f64], relative: f64) -> f64 {
    infer_bar_width(ys, relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_extent_follows_width() {
        let bars = barh_rects(&[1.0, 2.0], &[3.0, -1.0], 0.4, 0.0);
        assert_eq!(bars.len(), 2);
        assert!((bars[0].x1 - bars[0].x0 - 3.0).abs() < 1e-9);
        assert!((bars[1].x1 - bars[1].x0 - 1.0).abs() < 1e-9);
    }
}
