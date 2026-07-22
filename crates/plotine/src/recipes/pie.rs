use kurbo::BezPath;
use plotine_core::Point;
use std::f64::consts::TAU;

/// One pie wedge as a closed path in pixel space.
#[derive(Debug, Clone)]
pub struct PieWedge {
    pub path: BezPath,
    /// Mid-angle in radians (screen: 0 = +x, CCW; y grows down so sin is flipped).
    pub mid_angle: f64,
}

/// Build pie wedges centered at `center` with `radius` (pixel space).
///
/// `start_angle_deg` is from +x. When `counterclock` is true, slices advance
/// counter-clockwise (math convention); when false, clockwise (matplotlib default).
pub fn pie_wedges(
    sizes: &[f64],
    center: Point,
    radius: f64,
    start_angle_deg: f64,
    counterclock: bool,
    arc_steps: usize,
) -> Vec<PieWedge> {
    let total: f64 = sizes
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .sum();
    if total <= 0.0 || radius <= 0.0 {
        return Vec::new();
    }
    let steps = arc_steps.max(8);
    let mut angle = start_angle_deg.to_radians();
    let dir = if counterclock { 1.0 } else { -1.0 };
    let mut out = Vec::new();
    for &size in sizes {
        if !(size.is_finite() && size > 0.0) {
            continue;
        }
        let sweep = TAU * (size / total) * dir;
        let mid = angle + 0.5 * sweep;
        let mut path = BezPath::new();
        path.move_to(center.to_kurbo());
        let n = ((sweep.abs() / TAU) * steps as f64).ceil() as usize + 1;
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let a = angle + sweep * t;
            let p = Point::new(center.x + radius * a.cos(), center.y - radius * a.sin());
            path.line_to(p.to_kurbo());
        }
        path.close_path();
        out.push(PieWedge {
            path,
            mid_angle: mid,
        });
        angle += sweep;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_equal_slices() {
        let wedges = pie_wedges(
            &[1.0, 1.0, 1.0],
            Point::new(50.0, 50.0),
            40.0,
            90.0,
            true,
            48,
        );
        assert_eq!(wedges.len(), 3);
    }

    #[test]
    fn skips_non_positive() {
        let wedges = pie_wedges(
            &[1.0, 0.0, -2.0, 1.0],
            Point::new(0.0, 0.0),
            10.0,
            0.0,
            false,
            24,
        );
        assert_eq!(wedges.len(), 2);
    }
}
