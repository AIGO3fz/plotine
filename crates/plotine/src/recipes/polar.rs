use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};
use std::f64::consts::TAU;

use super::nice_levels;
use crate::mpl_policy::polar as polar_policy;

/// Convert polar `(theta, r)` samples to Cartesian `(x, y)`.
///
/// Convention matches matplotlib: θ = 0 at +x, increasing counter-clockwise.
pub fn polar_to_cartesian(theta: &[f64], r: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = theta.len().min(r.len());
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for i in 0..n {
        let t = theta[i];
        let ri = r[i];
        if t.is_finite() && ri.is_finite() {
            xs.push(ri * t.cos());
            ys.push(ri * t.sin());
        } else {
            xs.push(f64::NAN);
            ys.push(f64::NAN);
        }
    }
    (xs, ys)
}

/// Choose an outer radius and nice ring radii covering `data_rmax`.
///
/// Policy: [`crate::mpl_policy::polar`] — `rmax` is not snapped to the next nice
/// tick; ring labels stay strictly inside the spine.
pub fn polar_rings(data_rmax: f64, n_hint: usize) -> (f64, Vec<f64>) {
    let data_rmax = data_rmax.abs().max(1e-9);
    let n_hint = n_hint.clamp(3, 12);
    let rmax = polar_policy::rmax_from_data(data_rmax);
    let levels = nice_levels(0.0, polar_policy::ring_level_span(data_rmax), n_hint);
    let rings: Vec<f64> = levels
        .into_iter()
        .filter(|&r| r > 1e-12 && r < rmax * 0.999)
        .collect();
    let rings = if rings.is_empty() {
        let step = rmax / n_hint as f64;
        (1..n_hint)
            .map(|i| step * i as f64)
            .filter(|&r| r < rmax)
            .collect()
    } else {
        rings
    };
    (rmax, rings)
}

/// Horizontal text alignment hint for polar labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarLabelAlign {
    Left,
    Center,
    Right,
}

/// Vertical text baseline hint for polar labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarLabelBaseline {
    Top,
    Middle,
    Bottom,
}

/// One polar tick / grid label in pixel space.
#[derive(Debug, Clone)]
pub struct PolarLabel {
    pub pos: Point,
    pub text: String,
    /// Screen rotation in degrees (y-down; kept roughly upright).
    pub rotation_deg: f64,
    pub align: PolarLabelAlign,
    pub baseline: PolarLabelBaseline,
}

/// Concentric rings + radial spokes for a polar frame (data space → pixel).
pub fn polar_frame_paths(
    rmax: f64,
    ring_radii: &[f64],
    n_spokes: usize,
    transform: &DataToPixel,
) -> (Vec<BezPath>, Vec<BezPath>) {
    let rmax = rmax.abs().max(1e-9);
    let n_spokes = n_spokes.clamp(4, 24);
    let mut rings = Vec::new();
    let mut spokes = Vec::new();

    for &r in ring_radii {
        if !r.is_finite() || r <= 0.0 {
            continue;
        }
        rings.push(circle_path(r.min(rmax), transform));
    }
    // Ensure outer spine circle exists even if not in ring list.
    if ring_radii
        .last()
        .map(|&r| (r - rmax).abs() > rmax * 1e-9)
        .unwrap_or(true)
    {
        rings.push(circle_path(rmax, transform));
    }

    for i in 0..n_spokes {
        let a = TAU * i as f64 / n_spokes as f64;
        let mut path = BezPath::new();
        let p0 = transform.map(Point::new(0.0, 0.0));
        let p1 = transform.map(Point::new(rmax * a.cos(), rmax * a.sin()));
        path.move_to(p0.to_kurbo());
        path.line_to(p1.to_kurbo());
        spokes.push(path);
    }
    (rings, spokes)
}

fn circle_path(r: f64, transform: &DataToPixel) -> BezPath {
    let mut path = BezPath::new();
    let steps = 96usize;
    for k in 0..=steps {
        let a = TAU * k as f64 / steps as f64;
        let p = transform.map(Point::new(r * a.cos(), r * a.sin()));
        if k == 0 {
            path.move_to(p.to_kurbo());
        } else {
            path.line_to(p.to_kurbo());
        }
    }
    path
}

/// Angular labels (`0°` …) placed just outside the outer ring.
pub fn polar_angle_labels(
    rmax: f64,
    n_spokes: usize,
    transform: &DataToPixel,
    pad_px: f64,
) -> Vec<PolarLabel> {
    let rmax = rmax.abs().max(1e-9);
    let n_spokes = n_spokes.clamp(4, 24);
    let mut out = Vec::with_capacity(n_spokes);
    for i in 0..n_spokes {
        let deg = (360.0 * i as f64 / n_spokes as f64).round() as i32 % 360;
        let a = TAU * i as f64 / n_spokes as f64;
        let edge = transform.map(Point::new(rmax * a.cos(), rmax * a.sin()));
        let origin = transform.map(Point::new(0.0, 0.0));
        let dx = edge.x - origin.x;
        let dy = edge.y - origin.y;
        let len = (dx * dx + dy * dy).sqrt().max(1e-9);
        let pos = Point::new(edge.x + pad_px * dx / len, edge.y + pad_px * dy / len);
        // Anchor like matplotlib: outward side of the circle.
        let (align, baseline) = angle_label_anchor(deg);
        out.push(PolarLabel {
            pos,
            text: format!("{deg}°"),
            rotation_deg: 0.0,
            align,
            baseline,
        });
    }
    out
}

fn angle_label_anchor(deg: i32) -> (PolarLabelAlign, PolarLabelBaseline) {
    let d = deg.rem_euclid(360);
    match d {
        0 => (PolarLabelAlign::Left, PolarLabelBaseline::Middle),
        45 => (PolarLabelAlign::Left, PolarLabelBaseline::Bottom),
        90 => (PolarLabelAlign::Center, PolarLabelBaseline::Bottom),
        135 => (PolarLabelAlign::Right, PolarLabelBaseline::Bottom),
        180 => (PolarLabelAlign::Right, PolarLabelBaseline::Middle),
        225 => (PolarLabelAlign::Right, PolarLabelBaseline::Top),
        270 => (PolarLabelAlign::Center, PolarLabelBaseline::Top),
        315 => (PolarLabelAlign::Left, PolarLabelBaseline::Top),
        _ => (PolarLabelAlign::Center, PolarLabelBaseline::Middle),
    }
}

/// Radial labels along the mid-angle between the first two spokes (≈ 22.5° for 8 spokes).
///
/// Text stays horizontal (matplotlib polar default: tick label rotation = 0).
pub fn polar_radial_labels(
    ring_radii: &[f64],
    n_spokes: usize,
    transform: &DataToPixel,
) -> Vec<PolarLabel> {
    let n_spokes = n_spokes.clamp(4, 24);
    let a = TAU / (n_spokes as f64 * 2.0); // halfway to first spoke step
    let radii: Vec<f64> = ring_radii
        .iter()
        .copied()
        .filter(|&r| r.is_finite() && r > 0.0)
        .collect();
    let labels = plotine_core::ticks_from_values(&radii);
    let mut out = Vec::with_capacity(labels.len());
    for tick in labels {
        let pos = transform.map(Point::new(tick.value * a.cos(), tick.value * a.sin()));
        out.push(PolarLabel {
            pos,
            text: tick.label,
            rotation_deg: 0.0,
            align: PolarLabelAlign::Center,
            baseline: PolarLabelBaseline::Middle,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn t() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(-2.0, 2.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(-2.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 200.0, 200.0),
        )
    }

    #[test]
    fn unit_circle_point() {
        let (x, y) = polar_to_cartesian(&[0.0], &[1.0]);
        assert!((x[0] - 1.0).abs() < 1e-12);
        assert!(y[0].abs() < 1e-12);
    }

    #[test]
    fn rings_cover_data_like_matplotlib() {
        let (rmax, rings) = polar_rings(0.95, polar_policy::RING_N_HINT);
        assert!((rmax - polar_policy::rmax_from_data(0.95)).abs() < 1e-9);
        assert!(rings.iter().all(|&r| r < rmax));
        let expect = [0.2, 0.4, 0.6, 0.8];
        assert_eq!(rings.len(), expect.len(), "{rings:?}");
        for (got, want) in rings.iter().zip(expect) {
            assert!((got - want).abs() < 1e-9, "{rings:?}");
        }
    }

    #[test]
    fn angle_labels_eight_spokes() {
        let labels = polar_angle_labels(1.4, 8, &t(), 10.0);
        assert_eq!(labels.len(), 8);
        assert_eq!(labels[0].text, "0°");
        assert_eq!(labels[2].text, "90°");
    }
}
