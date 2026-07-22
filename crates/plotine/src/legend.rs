//! Legend placement for labeled artists.

use plotine_core::Rect;

/// Where to anchor the legend box (inside the axes, or outside on the right).
///
/// Only artists with a `.label(...)` appear in the legend. Call
/// [`Axes::legend`](crate::Axes::legend) to enable it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Legend {
    /// Upper-right corner of the axes (default).
    #[default]
    TopRight,
    /// Upper-left corner of the axes.
    TopLeft,
    /// Lower-right corner of the axes.
    BottomRight,
    /// Lower-left corner of the axes.
    BottomLeft,
    /// Right edge, vertically centered.
    Right,
    /// Left edge, vertically centered.
    CenterLeft,
    /// Right edge, vertically centered.
    CenterRight,
    /// Bottom edge, horizontally centered.
    LowerCenter,
    /// Top edge, horizontally centered.
    UpperCenter,
    /// Dead center of the axes.
    Center,
    /// Pick the fixed location with the least overlap against sampled data
    /// points (matplotlib `loc='best'`). Falls back to [`Legend::TopRight`]
    /// when no samples are available.
    Best,
    /// Outside the axes on the right, top-aligned
    /// (matplotlib `loc='upper left', bbox_to_anchor=(1.02, 1)`).
    OutsideUpperRight,
    /// Outside the axes on the right, vertically centered.
    OutsideRight,
    /// Outside the axes on the right, bottom-aligned.
    OutsideLowerRight,
}

impl Legend {
    /// `true` when the legend is drawn outside the axes box.
    pub fn is_outside(self) -> bool {
        matches!(
            self,
            Self::OutsideUpperRight | Self::OutsideRight | Self::OutsideLowerRight
        )
    }

    /// Candidate locations considered by [`Legend::Best`] (excludes `Best` /
    /// `Center` / outside placements).
    pub fn best_candidates() -> &'static [Legend] {
        &[
            Self::TopRight,
            Self::TopLeft,
            Self::BottomRight,
            Self::BottomLeft,
            Self::Right,
            Self::CenterLeft,
            Self::CenterRight,
            Self::UpperCenter,
            Self::LowerCenter,
        ]
    }

    /// Top-left corner of the legend box for a fixed (non-`Best`) location.
    pub fn anchor(self, axes: Rect, box_w: f64, box_h: f64, inset: f64) -> (f64, f64) {
        let axes_cx = (axes.x0 + axes.x1) / 2.0;
        let axes_cy = (axes.y0 + axes.y1) / 2.0;
        match self {
            Self::TopRight | Self::Best => (axes.x1 - box_w - inset, axes.y0 + inset),
            Self::TopLeft => (axes.x0 + inset, axes.y0 + inset),
            Self::BottomRight => (axes.x1 - box_w - inset, axes.y1 - box_h - inset),
            Self::BottomLeft => (axes.x0 + inset, axes.y1 - box_h - inset),
            Self::Right => (axes.x1 - box_w - inset, axes_cy - box_h / 2.0),
            Self::CenterLeft => (axes.x0 + inset, axes_cy - box_h / 2.0),
            Self::CenterRight => (axes.x1 - box_w - inset, axes_cy - box_h / 2.0),
            Self::LowerCenter => (axes_cx - box_w / 2.0, axes.y1 - box_h - inset),
            Self::UpperCenter => (axes_cx - box_w / 2.0, axes.y0 + inset),
            Self::Center => (axes_cx - box_w / 2.0, axes_cy - box_h / 2.0),
            Self::OutsideUpperRight => (axes.x1 + inset, axes.y0 + inset),
            Self::OutsideRight => (axes.x1 + inset, axes_cy - box_h / 2.0),
            Self::OutsideLowerRight => (axes.x1 + inset, axes.y1 - box_h - inset),
        }
    }

    /// Resolve [`Legend::Best`] to a concrete corner by minimizing sample hits
    /// inside the legend box. Non-`Best` locations are returned unchanged.
    pub fn resolve_best(
        self,
        axes: Rect,
        box_w: f64,
        box_h: f64,
        inset: f64,
        samples: &[(f64, f64)],
    ) -> Self {
        if !matches!(self, Self::Best) {
            return self;
        }
        if samples.is_empty() {
            return Self::TopRight;
        }
        let mut best = Self::TopRight;
        let mut best_score = usize::MAX;
        for &cand in Self::best_candidates() {
            let (x0, y0) = cand.anchor(axes, box_w, box_h, inset);
            let x1 = x0 + box_w;
            let y1 = y0 + box_h;
            let mut score = 0usize;
            for &(px, py) in samples {
                if px >= x0 && px <= x1 && py >= y0 && py <= y1 {
                    score += 1;
                }
            }
            if score < best_score {
                best_score = score;
                best = cand;
                if score == 0 {
                    break;
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn best_picks_empty_corner() {
        let axes = Rect::new(0.0, 0.0, 100.0, 100.0);
        // Cover top band + right edge so the free corner is bottom-left.
        let mut samples = Vec::new();
        for i in 0..30 {
            let t = i as f64;
            samples.push((10.0 + t * 2.5, 8.0 + t * 0.4)); // top strip
            samples.push((78.0 + t * 0.5, 15.0 + t * 2.0)); // right strip
        }
        let loc = Legend::Best.resolve_best(axes, 20.0, 20.0, 5.0, &samples);
        assert!(
            matches!(
                loc,
                Legend::BottomLeft | Legend::BottomRight | Legend::LowerCenter
            ),
            "got {loc:?}"
        );
        assert_ne!(loc, Legend::TopRight);
        assert_ne!(loc, Legend::TopLeft);
    }

    #[test]
    fn outside_upper_right_is_right_of_axes() {
        let axes = Rect::new(0.0, 0.0, 100.0, 80.0);
        let (x0, y0) = Legend::OutsideUpperRight.anchor(axes, 30.0, 20.0, 4.0);
        assert!(x0 >= axes.x1);
        assert!((y0 - 4.0).abs() < 1e-9);
        assert!(Legend::OutsideUpperRight.is_outside());
        assert!(!Legend::TopRight.is_outside());
    }
}
