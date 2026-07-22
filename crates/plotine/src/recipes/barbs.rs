use kurbo::{BezPath, Circle, Shape};
use plotine_core::{DataToPixel, Point};

/// One wind barb in pixel space.
#[derive(Debug, Clone)]
pub struct BarbGeom {
    /// Staff from tip toward the feathered end (absent for empty/calm).
    pub shaft: Option<BezPath>,
    /// Full / half barb strokes.
    pub feathers: Vec<BezPath>,
    /// Filled flag triangles.
    pub flags: Vec<BezPath>,
    /// Calm (near-zero magnitude) circle at the tip.
    pub empty: Option<BezPath>,
}

/// Decompose magnitude into flags / full barbs / half / empty (matplotlib defaults).
pub fn barb_components(
    mag: f64,
    half: f64,
    full: f64,
    flag: f64,
    rounding: bool,
) -> (usize, usize, bool, bool) {
    let half = half.max(1e-12);
    let full = full.max(half);
    let flag = flag.max(full);
    let mut mag = mag.max(0.0);
    if rounding {
        mag = half * (mag / half).round();
    }
    let n_flags = (mag / flag).floor() as usize;
    mag -= n_flags as f64 * flag;
    let n_full = (mag / full).floor() as usize;
    mag -= n_full as f64 * full;
    let has_half = mag >= half - 1e-12;
    let empty = !(has_half || n_flags > 0 || n_full > 0);
    (n_flags, n_full, has_half, empty)
}

/// Build wind barbs for samples `(x, y, u, v)`.
///
/// `length_px` is the staff length in pixels. Feathers point to the right of the
/// staff (Northern Hemisphere); set `flip` for the opposite side.
#[allow(clippy::too_many_arguments)]
pub fn barb_geoms(
    x: &[f64],
    y: &[f64],
    u: &[f64],
    v: &[f64],
    length_px: f64,
    half: f64,
    full: f64,
    flag: f64,
    flip: bool,
    transform: &DataToPixel,
) -> Vec<BarbGeom> {
    let n = x.len().min(y.len()).min(u.len()).min(v.len());
    let length_px = length_px.max(4.0);
    let spacing = length_px * 0.125;
    let full_height = length_px * 0.4;
    let full_width = length_px * 0.25;
    let empty_rad = length_px * 0.15;
    let mut out = Vec::with_capacity(n);

    for i in 0..n {
        let xi = x[i];
        let yi = y[i];
        let ui = u[i];
        let vi = v[i];
        if !(xi.is_finite() && yi.is_finite() && ui.is_finite() && vi.is_finite()) {
            continue;
        }
        let mag = (ui * ui + vi * vi).sqrt();
        let (n_flags, n_full, has_half, empty) = barb_components(mag, half, full, flag, true);
        let tip = transform.map(Point::new(xi, yi));

        if empty {
            let circ = Circle::new(tip.to_kurbo(), empty_rad).to_path(0.1);
            out.push(BarbGeom {
                shaft: None,
                feathers: Vec::new(),
                flags: Vec::new(),
                empty: Some(circ),
            });
            continue;
        }

        // Screen-space wind direction (tip points along the vector).
        let tip2 = transform.map(Point::new(xi + ui, yi + vi));
        let mut dx = tip2.x - tip.x;
        let mut dy = tip2.y - tip.y;
        let dlen = (dx * dx + dy * dy).sqrt();
        if dlen < 1e-12 {
            // Magnitude rounded up but (u,v) maps to a degenerate pixel dir —
            // fall back to data-space angle.
            dx = ui;
            dy = -vi; // y-down screen
            let dlen = (dx * dx + dy * dy).sqrt().max(1e-12);
            dx /= dlen;
            dy /= dlen;
        } else {
            dx /= dlen;
            dy /= dlen;
        }

        // Staff from tip toward feathers = opposite wind.
        let sx = -dx;
        let sy = -dy;
        let mut px = -sy;
        let mut py = sx;
        if flip {
            px = -px;
            py = -py;
        }
        let height = full_height;

        let feather_end = Point::new(tip.x + sx * length_px, tip.y + sy * length_px);
        let mut shaft = BezPath::new();
        shaft.move_to(tip.to_kurbo());
        shaft.line_to(feather_end.to_kurbo());

        let mut flags = Vec::new();
        let mut feathers = Vec::new();
        let mut offset = length_px;

        for _ in 0..n_flags {
            if (offset - length_px).abs() > 1e-9 {
                offset += spacing * 0.5;
            }
            // Triangle: base on staff spanning `full_width`, tip at +height.
            let p0 = Point::new(tip.x + sx * offset, tip.y + sy * offset);
            let p1 = Point::new(
                tip.x + sx * (offset - full_width * 0.5) + px * height,
                tip.y + sy * (offset - full_width * 0.5) + py * height,
            );
            let p2 = Point::new(
                tip.x + sx * (offset - full_width),
                tip.y + sy * (offset - full_width),
            );
            let mut tri = BezPath::new();
            tri.move_to(p0.to_kurbo());
            tri.line_to(p1.to_kurbo());
            tri.line_to(p2.to_kurbo());
            tri.close_path();
            flags.push(tri);
            offset -= full_width + spacing;
        }

        for _ in 0..n_full {
            let base = Point::new(tip.x + sx * offset, tip.y + sy * offset);
            let tip_f = Point::new(
                tip.x + sx * (offset + full_width * 0.5) + px * height,
                tip.y + sy * (offset + full_width * 0.5) + py * height,
            );
            let mut line = BezPath::new();
            line.move_to(base.to_kurbo());
            line.line_to(tip_f.to_kurbo());
            feathers.push(line);
            offset -= spacing;
        }

        if has_half {
            if (offset - length_px).abs() < 1e-9 {
                offset -= 1.5 * spacing;
            }
            let base = Point::new(tip.x + sx * offset, tip.y + sy * offset);
            let tip_f = Point::new(
                tip.x + sx * (offset + full_width * 0.25) + px * (height * 0.5),
                tip.y + sy * (offset + full_width * 0.25) + py * (height * 0.5),
            );
            let mut line = BezPath::new();
            line.move_to(base.to_kurbo());
            line.line_to(tip_f.to_kurbo());
            feathers.push(line);
        }

        out.push(BarbGeom {
            shaft: Some(shaft),
            feathers,
            flags,
            empty: None,
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
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            Rect::new(0.0, 0.0, 200.0, 200.0),
        )
    }

    #[test]
    fn components_standard() {
        // 65 → 1 flag + 1 full + 1 half
        let (f, b, h, e) = barb_components(65.0, 5.0, 10.0, 50.0, true);
        assert_eq!((f, b, h, e), (1, 1, true, false));
        // 2 rounds to 0 → empty calm circle
        let (f, b, h, e) = barb_components(2.0, 5.0, 10.0, 50.0, true);
        assert!(e && f == 0 && b == 0 && !h);
        let (_, _, _, empty0) = barb_components(0.0, 5.0, 10.0, 50.0, true);
        assert!(empty0);
    }

    #[test]
    fn one_barb_with_flag() {
        let geoms = barb_geoms(
            &[5.0],
            &[5.0],
            &[50.0],
            &[0.0],
            40.0,
            5.0,
            10.0,
            50.0,
            false,
            &t(),
        );
        assert_eq!(geoms.len(), 1);
        assert!(geoms[0].shaft.is_some());
        assert_eq!(geoms[0].flags.len(), 1);
        assert!(geoms[0].empty.is_none());
    }

    #[test]
    fn calm_is_circle() {
        let geoms = barb_geoms(
            &[1.0],
            &[1.0],
            &[0.0],
            &[0.0],
            40.0,
            5.0,
            10.0,
            50.0,
            false,
            &t(),
        );
        assert_eq!(geoms.len(), 1);
        assert!(geoms[0].empty.is_some());
        assert!(geoms[0].shaft.is_none());
    }
}
