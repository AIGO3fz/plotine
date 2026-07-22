use plotine_core::{DataToPixel, Point};

use crate::recipes::spans::SpanSegment;

/// Vertical tick segments for horizontal eventplot orientation (matplotlib default).
///
/// Each row `positions[i]` is drawn at y = `i + 1` with half-length `lineoffset`.
pub fn eventplot_segments(
    positions: &[&[f64]],
    lineoffset: f64,
    transform: &DataToPixel,
) -> Vec<SpanSegment> {
    let half = lineoffset.abs().max(1e-9) * 0.5;
    let mut out = Vec::new();
    for (row, xs) in positions.iter().enumerate() {
        let y = (row + 1) as f64;
        for &x in *xs {
            if !(x.is_finite()) {
                continue;
            }
            let p0 = transform.map(Point::new(x, y - half));
            let p1 = transform.map(Point::new(x, y + half));
            out.push(SpanSegment {
                x0: p0.x,
                y0: p0.y,
                x1: p1.x,
                y1: p1.y,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn one_segment_per_event() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 3.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let a = [1.0, 2.0, 5.0];
        let b = [3.0];
        let segs = eventplot_segments(&[&a[..], &b[..]], 0.8, &t);
        assert_eq!(segs.len(), 4);
    }
}
