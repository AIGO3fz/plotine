use plotine_core::{DataToPixel, Point, Rect};

/// Axis-aligned bar in data coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarRect {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
}

impl BarRect {
    pub fn to_pixel_rect(self, transform: &DataToPixel) -> Rect {
        let p0 = transform.map(Point::new(self.x0, self.y0));
        let p1 = transform.map(Point::new(self.x1, self.y1));
        Rect::new(
            p0.x.min(p1.x),
            p0.y.min(p1.y),
            p0.x.max(p1.x),
            p0.y.max(p1.y),
        )
    }
}

/// Infer a data-space bar width from x positions (`relative` ∈ (0, 1]).
pub fn infer_bar_width(x: &[f64], relative: f64) -> f64 {
    let relative = relative.clamp(0.05, 1.0);
    let mut xs: Vec<f64> = x.iter().copied().filter(|v| v.is_finite()).collect();
    if xs.len() < 2 {
        return relative;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut gaps = Vec::new();
    for w in xs.windows(2) {
        let d = w[1] - w[0];
        if d > 1e-12 {
            gaps.push(d);
        }
    }
    let spacing = if gaps.is_empty() {
        1.0
    } else {
        gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        gaps[gaps.len() / 2]
    };
    spacing * relative
}

/// Build bar rectangles centered on each `x[i]` with height `heights[i]`.
pub fn bar_rects(x: &[f64], heights: &[f64], width: f64, baseline: f64) -> Vec<BarRect> {
    let half = width.abs() * 0.5;
    x.iter()
        .zip(heights.iter())
        .filter_map(|(&xi, &h)| {
            if !xi.is_finite() || !h.is_finite() {
                return None;
            }
            Some(BarRect {
                x0: xi - half,
                x1: xi + half,
                y0: baseline,
                y1: baseline + h,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_bars() {
        let bars = bar_rects(&[0.0, 2.0], &[1.0, 3.0], 1.0, 0.0);
        assert_eq!(bars.len(), 2);
        assert!((bars[0].x0 - (-0.5)).abs() < 1e-12);
        assert!((bars[0].y1 - 1.0).abs() < 1e-12);
    }
}
