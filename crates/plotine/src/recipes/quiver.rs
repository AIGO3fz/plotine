use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

/// One arrow in pixel space: shaft + head outline.
#[derive(Debug, Clone)]
pub struct QuiverArrow {
    pub shaft: BezPath,
    pub head: BezPath,
}

/// Build quiver arrows for samples `(x, y, u, v)`.
///
/// `scale` maps data-vector length into data units (matplotlib-like: larger
/// scale → shorter arrows). Head size follows matplotlib `FancyArrow`:
/// `headlength` / `headwidth` are multiples of `shaft_width_px`.
#[allow(clippy::too_many_arguments)]
pub fn quiver_arrows(
    x: &[f64],
    y: &[f64],
    u: &[f64],
    v: &[f64],
    scale: f64,
    shaft_width_px: f64,
    head_length_width: f64,
    head_width_width: f64,
    transform: &DataToPixel,
) -> Vec<QuiverArrow> {
    let n = x.len().min(y.len()).min(u.len()).min(v.len());
    let scale = scale.max(1e-12);
    let shaft_w = shaft_width_px.max(0.5);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let xi = x[i];
        let yi = y[i];
        let ui = u[i];
        let vi = v[i];
        if !(xi.is_finite() && yi.is_finite() && ui.is_finite() && vi.is_finite()) {
            continue;
        }
        let dx = ui / scale;
        let dy = vi / scale;
        if dx * dx + dy * dy < 1e-24 {
            continue;
        }
        let p0 = transform.map(Point::new(xi, yi));
        let p1 = transform.map(Point::new(xi + dx, yi + dy));

        let sx = p1.x - p0.x;
        let sy = p1.y - p0.y;
        let len = (sx * sx + sy * sy).sqrt().max(1e-9);
        let ux = sx / len;
        let uy = sy / len;
        let px = -uy;
        let py = ux;
        // Matplotlib: headlength/headwidth are multiples of shaft width.
        let hl_hi = len * 0.9;
        let hw_hi = len * 0.5;
        let hl = (shaft_w * head_length_width).clamp(shaft_w.min(hl_hi), hl_hi);
        let hw = (shaft_w * head_width_width * 0.5).clamp((shaft_w * 0.5).min(hw_hi), hw_hi);
        // `headaxislength` ≈ 4.5/5 of headlength — shaft meets the head there.
        let axis_len = (hl * 4.5 / 5.0).min(len * 0.95);
        let tip = p1;
        let neck = Point::new(tip.x - ux * axis_len, tip.y - uy * axis_len);
        let base = Point::new(tip.x - ux * hl, tip.y - uy * hl);
        let half = shaft_w * 0.5;
        // Filled shaft rectangle (mpl `FancyArrow` polygon), not a stroked line.
        let mut shaft = BezPath::new();
        shaft.move_to(Point::new(p0.x + px * half, p0.y + py * half).to_kurbo());
        shaft.line_to(Point::new(neck.x + px * half, neck.y + py * half).to_kurbo());
        shaft.line_to(Point::new(neck.x - px * half, neck.y - py * half).to_kurbo());
        shaft.line_to(Point::new(p0.x - px * half, p0.y - py * half).to_kurbo());
        shaft.close_path();
        let left = Point::new(base.x + px * hw, base.y + py * hw);
        let right = Point::new(base.x - px * hw, base.y - py * hw);
        let mut head = BezPath::new();
        head.move_to(tip.to_kurbo());
        head.line_to(left.to_kurbo());
        head.line_to(right.to_kurbo());
        head.close_path();

        out.push(QuiverArrow { shaft, head });
    }
    out
}

/// Suggest a default scale from median vector length and axis span.
pub fn infer_quiver_scale(u: &[f64], v: &[f64], x_span: f64, y_span: f64) -> f64 {
    let mut lens: Vec<f64> = u
        .iter()
        .zip(v.iter())
        .filter_map(|(&ui, &vi)| {
            if ui.is_finite() && vi.is_finite() {
                Some((ui * ui + vi * vi).sqrt())
            } else {
                None
            }
        })
        .filter(|l| *l > 1e-12)
        .collect();
    if lens.is_empty() {
        return 1.0;
    }
    lens.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let med = lens[lens.len() / 2];
    let span = x_span.abs().max(y_span.abs()).max(1e-12);
    // Aim for arrows ~ 1/SPAN_DIV of the axes span (matplotlib-like density).
    let div = crate::mpl_policy::quiver::SPAN_DIV;
    (med * div / span).max(1e-6)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn one_arrow() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let arrows = quiver_arrows(&[0.0], &[0.0], &[1.0], &[0.5], 1.0, 2.0, 5.0, 3.0, &t);
        assert_eq!(arrows.len(), 1);
    }

    #[test]
    fn short_arrow_does_not_panic() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 12.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 12.0).unwrap()),
            Rect::new(0.0, 0.0, 200.0, 200.0),
        );
        // Tiny vectors → short pixel shafts; head clamp must stay ordered.
        let arrows = quiver_arrows(&[1.0], &[1.0], &[0.05], &[0.02], 1.0, 2.0, 5.0, 3.0, &t);
        assert_eq!(arrows.len(), 1);
    }
}
