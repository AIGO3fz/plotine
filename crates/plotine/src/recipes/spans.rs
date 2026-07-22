use plotine_core::{DataToPixel, Point, Rect};

/// Pixel-space line segment.
#[derive(Debug, Clone, Copy)]
pub struct SpanSegment {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

fn broadcast(values: &[f64], n: usize) -> Option<Vec<f64>> {
    if values.len() == n {
        Some(values.to_vec())
    } else if values.len() == 1 {
        Some(vec![values[0]; n])
    } else {
        None
    }
}

/// Horizontal segments at each `y[i]` from `xmin[i]` to `xmax[i]`.
///
/// `xmin` / `xmax` may be length 1 (broadcast) or match `y`.
pub fn hline_segments(
    y: &[f64],
    xmin: &[f64],
    xmax: &[f64],
    transform: &DataToPixel,
) -> Option<Vec<SpanSegment>> {
    let n = y.len();
    let xmin = broadcast(xmin, n)?;
    let xmax = broadcast(xmax, n)?;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let yi = y[i];
        let x0 = xmin[i];
        let x1 = xmax[i];
        if !(yi.is_finite() && x0.is_finite() && x1.is_finite()) {
            continue;
        }
        let p0 = transform.map(Point::new(x0, yi));
        let p1 = transform.map(Point::new(x1, yi));
        out.push(SpanSegment {
            x0: p0.x,
            y0: p0.y,
            x1: p1.x,
            y1: p1.y,
        });
    }
    Some(out)
}

/// Vertical segments at each `x[i]` from `ymin[i]` to `ymax[i]`.
pub fn vline_segments(
    x: &[f64],
    ymin: &[f64],
    ymax: &[f64],
    transform: &DataToPixel,
) -> Option<Vec<SpanSegment>> {
    let n = x.len();
    let ymin = broadcast(ymin, n)?;
    let ymax = broadcast(ymax, n)?;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let xi = x[i];
        let y0 = ymin[i];
        let y1 = ymax[i];
        if !(xi.is_finite() && y0.is_finite() && y1.is_finite()) {
            continue;
        }
        let p0 = transform.map(Point::new(xi, y0));
        let p1 = transform.map(Point::new(xi, y1));
        out.push(SpanSegment {
            x0: p0.x,
            y0: p0.y,
            x1: p1.x,
            y1: p1.y,
        });
    }
    Some(out)
}

/// Full-width horizontal band in pixel space using the current x-domain.
pub fn axhspan_rect(ymin: f64, ymax: f64, transform: &DataToPixel) -> Option<Rect> {
    if !(ymin.is_finite() && ymax.is_finite()) {
        return None;
    }
    let (x0, x1) = transform.x_scale().domain();
    let p0 = transform.map(Point::new(x0, ymin.min(ymax)));
    let p1 = transform.map(Point::new(x1, ymin.max(ymax)));
    Some(Rect::new(
        p0.x.min(p1.x),
        p0.y.min(p1.y),
        p0.x.max(p1.x),
        p0.y.max(p1.y),
    ))
}

/// Full-height vertical band in pixel space using the current y-domain.
pub fn axvspan_rect(xmin: f64, xmax: f64, transform: &DataToPixel) -> Option<Rect> {
    if !(xmin.is_finite() && xmax.is_finite()) {
        return None;
    }
    let (y0, y1) = transform.y_scale().domain();
    let p0 = transform.map(Point::new(xmin.min(xmax), y0));
    let p1 = transform.map(Point::new(xmin.max(xmax), y1));
    Some(Rect::new(
        p0.x.min(p1.x),
        p0.y.min(p1.y),
        p0.x.max(p1.x),
        p0.y.max(p1.y),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn t() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        )
    }

    #[test]
    fn broadcast_scalar_xmin_xmax() {
        let segs = hline_segments(&[1.0, 2.0], &[0.0], &[5.0], &t()).unwrap();
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn reject_bad_broadcast() {
        assert!(hline_segments(&[1.0, 2.0], &[0.0, 1.0, 2.0], &[5.0], &t()).is_none());
    }
}
