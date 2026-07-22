use kurbo::BezPath;
use plotine_core::{Cmap, Color, DataToPixel, Norm, Point};

use crate::recipes::heatmap::heatmap_limits;

/// One contour polyline in pixel space.
#[derive(Debug, Clone)]
pub struct ContourPath {
    pub path: BezPath,
    pub level: f64,
}

/// One filled band polygon in pixel space.
#[derive(Debug, Clone)]
pub struct ContourFill {
    pub path: BezPath,
    pub color: Color,
}

fn grid_xy(
    nrows: usize,
    ncols: usize,
    x: Option<&[f64]>,
    y: Option<&[f64]>,
) -> (Vec<f64>, Vec<f64>) {
    let xs = if let Some(x) = x {
        if x.len() == ncols {
            x.to_vec()
        } else {
            (0..ncols).map(|i| i as f64).collect()
        }
    } else {
        (0..ncols).map(|i| i as f64).collect()
    };
    let ys = if let Some(y) = y {
        if y.len() == nrows {
            y.to_vec()
        } else {
            (0..nrows).map(|i| i as f64).collect()
        }
    } else {
        (0..nrows).map(|i| i as f64).collect()
    };
    (xs, ys)
}

fn z_at(z: &[f64], ncols: usize, r: usize, c: usize) -> f64 {
    z.get(r * ncols + c).copied().unwrap_or(f64::NAN)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[allow(clippy::too_many_arguments)]
fn edge_point(
    side: u8,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    z2: f64,
    z3: f64,
    level: f64,
) -> Point {
    // Corners: 0=(x0,y0), 1=(x1,y0), 2=(x1,y1), 3=(x0,y1)
    match side {
        0 => {
            // bottom 0→1
            let t = ((level - z0) / (z1 - z0)).clamp(0.0, 1.0);
            Point::new(lerp(x0, x1, t), y0)
        }
        1 => {
            // right 1→2
            let t = ((level - z1) / (z2 - z1)).clamp(0.0, 1.0);
            Point::new(x1, lerp(y0, y1, t))
        }
        2 => {
            // top 2→3
            let t = ((level - z2) / (z3 - z2)).clamp(0.0, 1.0);
            Point::new(lerp(x1, x0, t), y1)
        }
        _ => {
            // left 3→0
            let t = ((level - z3) / (z0 - z3)).clamp(0.0, 1.0);
            Point::new(x0, lerp(y1, y0, t))
        }
    }
}

/// Marching-squares edge pairs for cases 1..14 (case 0/15 empty; 5/10 ambiguous → both pairs).
fn case_edges(case: u8) -> &'static [(u8, u8)] {
    match case {
        1 | 14 => &[(3, 0)],
        2 | 13 => &[(0, 1)],
        3 | 12 => &[(3, 1)],
        4 | 11 => &[(1, 2)],
        6 | 9 => &[(0, 2)],
        7 | 8 => &[(3, 2)],
        5 => &[(3, 0), (1, 2)],
        10 => &[(0, 1), (2, 3)],
        _ => &[],
    }
}

fn cell_case(z0: f64, z1: f64, z2: f64, z3: f64, level: f64) -> Option<u8> {
    if !(z0.is_finite() && z1.is_finite() && z2.is_finite() && z3.is_finite()) {
        return None;
    }
    let mut case = 0u8;
    if z0 >= level {
        case |= 1;
    }
    if z1 >= level {
        case |= 2;
    }
    if z2 >= level {
        case |= 4;
    }
    if z3 >= level {
        case |= 8;
    }
    Some(case)
}

/// Auto contour levels (matplotlib `ContourSet._autolev`).
///
/// Matplotlib uses `MaxNLocator(n + 1, min_n_ticks=1)` so integer `levels(n)`
/// requests `n+1` bins (e.g. `n=8` on `[0, 2]` → `0, 0.25, …, 2`;
/// `n=10` on a diverging field → step `0.08`).
pub fn auto_levels(z: &[f64], n: usize) -> Vec<f64> {
    let (lo, hi) = heatmap_limits(z, None, None);
    // mpl: MaxNLocator(N + 1, min_n_ticks=1)
    nice_levels(lo, hi, n.max(1).saturating_add(1))
}

/// Nice level boundaries between `vmin` and `vmax` (~`nbins` intervals).
///
/// Step candidates match matplotlib `MaxNLocator` defaults:
/// `[1, 1.5, 2, 2.5, 3, 4, 5, 6, 8, 10]`.
pub fn nice_levels(vmin: f64, vmax: f64, nbins: usize) -> Vec<f64> {
    let nbins = nbins.max(1);
    if !vmin.is_finite() || !vmax.is_finite() {
        return Vec::new();
    }
    if (vmax - vmin).abs() < 1e-15 {
        return vec![vmin];
    }
    let (vmin, vmax) = if vmin <= vmax {
        (vmin, vmax)
    } else {
        (vmax, vmin)
    };
    let raw = (vmax - vmin) / nbins as f64;
    let exp = raw.abs().log10().floor();
    let mag = 10f64.powf(exp);
    let norm = raw / mag;
    // matplotlib.ticker.MaxNLocator default `_steps`.
    const STEPS: [f64; 10] = [1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0];
    let nice_norm = STEPS.iter().copied().find(|&s| norm <= s).unwrap_or(10.0);
    let step = nice_norm * mag;
    if !step.is_finite() || step <= 0.0 {
        return vec![vmin, vmax];
    }
    // Snap to a grid that covers [vmin, vmax], like MaxNLocator / ContourSet._autolev:
    // keep the last tick ≤ data min and the first tick ≥ data max (may lie outside).
    let mut v = (vmin / step).floor() * step;
    // Guard against -0.0 noise.
    if v.abs() < step * 1e-12 {
        v = 0.0;
    }
    let end = (vmax / step).ceil() * step;
    let mut levels = Vec::new();
    let mut guard = 0;
    while v <= end + step * 1e-9 && guard < nbins + 16 {
        let mut x = v;
        if x.abs() < step * 1e-12 {
            x = 0.0;
        }
        let distinct = levels
            .last()
            .copied()
            .map(|p: f64| (p - x).abs() > step * 1e-9)
            .unwrap_or(true);
        if distinct {
            levels.push(x);
        }
        v += step;
        guard += 1;
    }
    if levels.is_empty() {
        vec![vmin, vmax]
    } else {
        levels
    }
}

/// One iso-segment in **data** XY (z = `level`). Used by 3D contour.
#[derive(Debug, Clone, Copy)]
pub struct ContourSegment {
    /// Segment start x.
    pub x0: f64,
    /// Segment start y.
    pub y0: f64,
    /// Segment end x.
    pub x1: f64,
    /// Segment end y.
    pub y1: f64,
    /// Contour level (constant z).
    pub level: f64,
}

/// Contour segments in data coordinates for one level (no pixel transform).
pub fn contour_level_segments(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    x: Option<&[f64]>,
    y: Option<&[f64]>,
    level: f64,
) -> Vec<ContourSegment> {
    if nrows < 2 || ncols < 2 || !level.is_finite() {
        return Vec::new();
    }
    let (xs, ys) = grid_xy(nrows, ncols, x, y);
    let mut out = Vec::new();
    for r in 0..nrows - 1 {
        for c in 0..ncols - 1 {
            let z0 = z_at(z, ncols, r, c);
            let z1 = z_at(z, ncols, r, c + 1);
            let z2 = z_at(z, ncols, r + 1, c + 1);
            let z3 = z_at(z, ncols, r + 1, c);
            let Some(case) = cell_case(z0, z1, z2, z3, level) else {
                continue;
            };
            if case == 0 || case == 15 {
                continue;
            }
            let x0 = xs[c];
            let x1 = xs[c + 1];
            let y0 = ys[r];
            let y1 = ys[r + 1];
            for &(a, b) in case_edges(case) {
                let p0 = edge_point(a, x0, y0, x1, y1, z0, z1, z2, z3, level);
                let p1 = edge_point(b, x0, y0, x1, y1, z0, z1, z2, z3, level);
                out.push(ContourSegment {
                    x0: p0.x,
                    y0: p0.y,
                    x1: p1.x,
                    y1: p1.y,
                    level,
                });
            }
        }
    }
    out
}

/// Contour line segments for one level, as disconnected 2-point paths.
pub fn contour_level_paths(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    x: Option<&[f64]>,
    y: Option<&[f64]>,
    level: f64,
    transform: &DataToPixel,
) -> Vec<ContourPath> {
    if nrows < 2 || ncols < 2 || !level.is_finite() {
        return Vec::new();
    }
    let (xs, ys) = grid_xy(nrows, ncols, x, y);
    let mut out = Vec::new();
    for r in 0..nrows - 1 {
        for c in 0..ncols - 1 {
            let z0 = z_at(z, ncols, r, c);
            let z1 = z_at(z, ncols, r, c + 1);
            let z2 = z_at(z, ncols, r + 1, c + 1);
            let z3 = z_at(z, ncols, r + 1, c);
            let Some(case) = cell_case(z0, z1, z2, z3, level) else {
                continue;
            };
            if case == 0 || case == 15 {
                continue;
            }
            let x0 = xs[c];
            let x1 = xs[c + 1];
            let y0 = ys[r];
            let y1 = ys[r + 1];
            for &(a, b) in case_edges(case) {
                let p0 = edge_point(a, x0, y0, x1, y1, z0, z1, z2, z3, level);
                let p1 = edge_point(b, x0, y0, x1, y1, z0, z1, z2, z3, level);
                let mut path = BezPath::new();
                path.move_to(transform.map(p0).to_kurbo());
                path.line_to(transform.map(p1).to_kurbo());
                out.push(ContourPath { path, level });
            }
        }
    }
    out
}

/// Contour lines for many levels.
pub fn contour_paths(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    x: Option<&[f64]>,
    y: Option<&[f64]>,
    levels: &[f64],
    transform: &DataToPixel,
) -> Vec<ContourPath> {
    let mut out = Vec::new();
    for &level in levels {
        out.extend(contour_level_paths(z, nrows, ncols, x, y, level, transform));
    }
    out
}

/// One contour level label in pixel space (matplotlib `clabel`).
#[derive(Debug, Clone)]
pub struct ContourLabel {
    pub pos: Point,
    pub level: f64,
    /// Screen rotation in degrees (y-down; text kept roughly upright).
    pub rotation_deg: f64,
}

/// Compact level string for contour labels (matplotlib `fmt='%.3g'`-like).
pub fn format_contour_level(v: f64) -> String {
    if !v.is_finite() {
        return String::new();
    }
    if v == 0.0 {
        return "0".into();
    }
    let abs = v.abs();
    // Prefer fixed form in a mid range; otherwise scientific (≈ 3 significant digits).
    if !(1e-4..1e6).contains(&abs) {
        return format!("{v:.2e}");
    }
    let exp = abs.log10().floor();
    let decimals = ((2.0 - exp).ceil() as i32).max(0) as usize;
    let mut s = format!("{v:.decimals$}");
    if let Some(dot) = s.find('.') {
        while s.ends_with('0') && s.len() > dot + 1 {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

/// Whether a pixel-space segment intersects a label's inline gap box.
///
/// The gap is an axis-aligned box in the label's rotated frame (baseline × height).
/// Any sample along the segment inside that box counts — not only the midpoint —
/// so short marching-squares edges under the glyph are all suppressed.
pub fn segment_in_label_gap(
    q0: Point,
    q1: Point,
    label: &ContourLabel,
    half_width: f64,
    half_height: f64,
) -> bool {
    let ang = label.rotation_deg.to_radians();
    let (c, s) = (ang.cos(), ang.sin());
    let in_box = |p: Point| {
        let dx = p.x - label.pos.x;
        let dy = p.y - label.pos.y;
        let lx = dx * c + dy * s;
        let ly = -dx * s + dy * c;
        lx.abs() <= half_width && ly.abs() <= half_height
    };
    // Dense enough for typical cell-edge segments (~10–40 px).
    for i in 0..=4 {
        let t = i as f64 / 4.0;
        let p = Point::new(q0.x + t * (q1.x - q0.x), q0.y + t * (q1.y - q0.y));
        if in_box(p) {
            return true;
        }
    }
    false
}

/// One label per level at the midpoint of the longest segment (pixel length ≥ `min_len_px`).
#[allow(clippy::too_many_arguments)]
pub fn contour_labels(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    x: Option<&[f64]>,
    y: Option<&[f64]>,
    levels: &[f64],
    transform: &DataToPixel,
    min_len_px: f64,
) -> Vec<ContourLabel> {
    if nrows < 2 || ncols < 2 {
        return Vec::new();
    }
    let (xs, ys) = grid_xy(nrows, ncols, x, y);
    let min_len_px = min_len_px.max(1.0);
    let mut out = Vec::new();
    for &level in levels {
        if !level.is_finite() {
            continue;
        }
        let mut best: Option<(f64, Point, Point)> = None;
        for r in 0..nrows - 1 {
            for c in 0..ncols - 1 {
                let z0 = z_at(z, ncols, r, c);
                let z1 = z_at(z, ncols, r, c + 1);
                let z2 = z_at(z, ncols, r + 1, c + 1);
                let z3 = z_at(z, ncols, r + 1, c);
                let Some(case) = cell_case(z0, z1, z2, z3, level) else {
                    continue;
                };
                if case == 0 || case == 15 {
                    continue;
                }
                let x0 = xs[c];
                let x1 = xs[c + 1];
                let y0 = ys[r];
                let y1 = ys[r + 1];
                for &(a, b) in case_edges(case) {
                    let p0 = edge_point(a, x0, y0, x1, y1, z0, z1, z2, z3, level);
                    let p1 = edge_point(b, x0, y0, x1, y1, z0, z1, z2, z3, level);
                    let q0 = transform.map(p0);
                    let q1 = transform.map(p1);
                    let len = ((q1.x - q0.x).powi(2) + (q1.y - q0.y).powi(2)).sqrt();
                    if len < min_len_px {
                        continue;
                    }
                    if best.as_ref().map(|(l, _, _)| len > *l).unwrap_or(true) {
                        best = Some((len, q0, q1));
                    }
                }
            }
        }
        if let Some((_, q0, q1)) = best {
            let pos = Point::new(0.5 * (q0.x + q1.x), 0.5 * (q0.y + q1.y));
            let mut deg = (q1.y - q0.y).atan2(q1.x - q0.x).to_degrees();
            if deg > 90.0 {
                deg -= 180.0;
            } else if deg < -90.0 {
                deg += 180.0;
            }
            out.push(ContourLabel {
                pos,
                level,
                rotation_deg: deg,
            });
        }
    }
    out
}

/// Filled contour bands (matplotlib `contourf`).
///
/// Paints low→high: for each boundary `levels[i]`, fill `z ≥ levels[i]` with the
/// color of band `[levels[i], levels[i+1]]`. Higher bands overpaint, matching
/// edge-linear marching-squares geometry used by `contour`.
///
/// Returns **one** [`ContourFill`] per band (multi-contour `BezPath`), so the
/// renderer pays O(levels) fill/stroke calls instead of O(cells × levels).
#[allow(clippy::too_many_arguments)]
pub fn contourf_fills(
    z: &[f64],
    nrows: usize,
    ncols: usize,
    x: Option<&[f64]>,
    y: Option<&[f64]>,
    levels: &[f64],
    cmap: &Cmap,
    norm: Norm,
    transform: &DataToPixel,
) -> Vec<ContourFill> {
    if nrows < 2 || ncols < 2 || levels.len() < 2 {
        return Vec::new();
    }
    let (xs, ys) = grid_xy(nrows, ncols, x, y);
    let vmin = levels.iter().copied().fold(f64::INFINITY, f64::min);
    let vmax = levels.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut out = Vec::with_capacity(levels.len().saturating_sub(1));
    // Skip the topmost boundary as a fill threshold; last band color is applied
    // when filling at `levels[n-2]` (covers up through `levels[n-1]`).
    for i in 0..levels.len().saturating_sub(1) {
        let lo = levels[i];
        let hi = levels[i + 1];
        if !(lo.is_finite() && hi.is_finite()) {
            continue;
        }
        let mid = 0.5 * (lo + hi);
        let color = cmap.map_norm(mid, vmin, vmax, norm);
        let mut path = BezPath::new();
        let mut any = false;
        for r in 0..nrows - 1 {
            for c in 0..ncols - 1 {
                let z0 = z_at(z, ncols, r, c);
                let z1 = z_at(z, ncols, r, c + 1);
                let z2 = z_at(z, ncols, r + 1, c + 1);
                let z3 = z_at(z, ncols, r + 1, c);
                if !(z0.is_finite() && z1.is_finite() && z2.is_finite() && z3.is_finite()) {
                    continue;
                }
                let x0 = xs[c];
                let x1 = xs[c + 1];
                let y0 = ys[r];
                let y1 = ys[r + 1];
                if append_cell_fill_above(&mut path, transform, x0, y0, x1, y1, z0, z1, z2, z3, lo)
                {
                    any = true;
                }
            }
        }
        if any {
            out.push(ContourFill { path, color });
        }
    }
    out
}

/// Append pixel-space polygons for `z ≥ level` inside one quad (marching squares).
///
/// Corners: 0=(x0,y0), 1=(x1,y0), 2=(x1,y1), 3=(x0,y1).
/// Returns `true` if at least one closed subpath was appended.
#[allow(clippy::too_many_arguments)]
fn append_cell_fill_above(
    path: &mut BezPath,
    transform: &DataToPixel,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    z2: f64,
    z3: f64,
    level: f64,
) -> bool {
    let Some(case) = cell_case(z0, z1, z2, z3, level) else {
        return false;
    };
    if case == 0 {
        return false;
    }
    let map = |p: Point| transform.map(p).to_kurbo();
    let c0 = Point::new(x0, y0);
    let c1 = Point::new(x1, y0);
    let c2 = Point::new(x1, y1);
    let c3 = Point::new(x0, y1);
    let e = |side: u8| edge_point(side, x0, y0, x1, y1, z0, z1, z2, z3, level);
    let mut append = |pts: &[Point]| {
        path.move_to(map(pts[0]));
        for p in pts.iter().skip(1) {
            path.line_to(map(*p));
        }
        path.close_path();
    };

    // Bitmask corners ≥ level: bit0=c0, bit1=c1, bit2=c2, bit3=c3.
    match case {
        15 => append(&[c0, c1, c2, c3]),
        1 => append(&[c0, e(0), e(3)]),
        2 => append(&[c1, e(1), e(0)]),
        3 => append(&[c0, c1, e(1), e(3)]),
        4 => append(&[c2, e(2), e(1)]),
        5 => {
            // Saddle: asymptotic decider on cell mean (matplotlib-style).
            let center = 0.25 * (z0 + z1 + z2 + z3);
            if center >= level {
                append(&[c0, e(0), e(1), c2, e(2), e(3)]);
            } else {
                append(&[c0, e(0), e(3)]);
                append(&[c2, e(1), e(2)]);
            }
        }
        6 => append(&[c1, c2, e(2), e(0)]),
        7 => append(&[c0, c1, c2, e(2), e(3)]),
        8 => append(&[c3, e(3), e(2)]),
        9 => append(&[c0, e(0), e(2), c3]),
        10 => {
            let center = 0.25 * (z0 + z1 + z2 + z3);
            if center >= level {
                append(&[c1, e(1), e(2), c3, e(3), e(0)]);
            } else {
                append(&[c1, e(1), e(0)]);
                append(&[c3, e(2), e(3)]);
            }
        }
        11 => append(&[c0, c1, e(1), e(2), c3]),
        12 => append(&[c2, c3, e(3), e(1)]),
        13 => append(&[c0, e(0), e(1), c2, c3]),
        14 => append(&[c1, c2, c3, e(3), e(0)]),
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn t() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 3.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 3.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        )
    }

    #[test]
    fn saddle_has_segments() {
        // Peak in center-ish
        let z = [
            0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0,
        ];
        let paths = contour_level_paths(&z, 3, 3, None, None, 0.5, &t());
        assert!(!paths.is_empty());
    }

    #[test]
    fn contourf_one_fill_per_band() {
        let z: Vec<f64> = (0..30 * 30)
            .map(|i| {
                let c = (i % 30) as f64;
                let r = (i / 30) as f64;
                let xx = c * 0.25 - 3.5;
                let yy = r * 0.25 - 3.5;
                (-xx * xx - yy * yy).exp() * 2.0
            })
            .collect();
        let levels = auto_levels(&z, 8);
        let fills = contourf_fills(
            &z,
            30,
            30,
            None,
            None,
            &levels,
            &Cmap::default(),
            Norm::Linear,
            &t(),
        );
        assert_eq!(fills.len(), levels.len().saturating_sub(1));
        // Dense field → many cell polygons merged into each band path.
        assert!(
            fills.iter().any(|f| f.path.elements().len() > 20),
            "expected multi-contour band paths"
        );
    }

    #[test]
    fn auto_levels_nice_like_matplotlib() {
        // Same field range as compare: ~[0, 2], levels(8) → 0.25 steps.
        let z: Vec<f64> = (0..30 * 30)
            .map(|i| {
                let c = (i % 30) as f64;
                let r = (i / 30) as f64;
                let xx = c * 0.25 - 3.5;
                let yy = r * 0.25 - 3.5;
                (-xx * xx - yy * yy).exp() * 2.0
            })
            .collect();
        let levels = auto_levels(&z, 8);
        assert!(levels.len() >= 7 && levels.len() <= 11, "got {levels:?}");
        assert!(
            (levels[1] - levels[0] - 0.25).abs() < 1e-9,
            "got {levels:?}"
        );
        assert!(
            levels.iter().any(|&v| (v - 1.0).abs() < 1e-9),
            "got {levels:?}"
        );
    }

    #[test]
    fn auto_levels_diverging_matches_mpl_contourf() {
        // compare coolwarm field; mpl contourf(levels=10) → step 0.08.
        let z: Vec<f64> = (0..24 * 24)
            .map(|i| {
                let c = (i % 24) as f64;
                let r = (i / 24) as f64;
                let xx = c * 0.3 - 3.5;
                let yy = r * 0.3 - 3.5;
                xx * (-xx * xx - yy * yy).exp()
            })
            .collect();
        let levels = auto_levels(&z, 10);
        assert!(
            (levels[1] - levels[0] - 0.08).abs() < 1e-9,
            "got {levels:?}"
        );
        assert!((levels[0] + 0.48).abs() < 1e-9, "got {levels:?}");
        assert!((levels.last().copied().unwrap_or(0.0) - 0.48).abs() < 1e-9);
    }

    #[test]
    fn labels_one_per_level() {
        let z = [
            0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0,
        ];
        let labels = contour_labels(&z, 3, 3, None, None, &[0.5], &t(), 1.0);
        assert_eq!(labels.len(), 1);
        assert!((labels[0].level - 0.5).abs() < 1e-12);
        assert_eq!(format_contour_level(0.5), "0.5");
        assert_eq!(format_contour_level(1.25), "1.25");
        assert_eq!(format_contour_level(1.0), "1");
    }

    #[test]
    fn inline_gap_blocks_nearby_segment() {
        let label = ContourLabel {
            pos: Point::new(50.0, 50.0),
            level: 1.0,
            rotation_deg: 0.0,
        };
        assert!(segment_in_label_gap(
            Point::new(48.0, 50.0),
            Point::new(52.0, 50.0),
            &label,
            6.0,
            4.0
        ));
        assert!(!segment_in_label_gap(
            Point::new(80.0, 50.0),
            Point::new(90.0, 50.0),
            &label,
            6.0,
            4.0
        ));
    }

    #[test]
    fn compare_field_inline_gaps_suppress_segments() {
        let z: Vec<f64> = (0..30 * 30)
            .map(|i| {
                let c = (i % 30) as f64;
                let r = (i / 30) as f64;
                let xx = c * 0.25 - 3.5;
                let yy = r * 0.25 - 3.5;
                (-xx * xx - yy * yy).exp() * 2.0
            })
            .collect();
        let levels = auto_levels(&z, 8);
        let transform = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 29.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 29.0).unwrap()),
            Rect::new(80.0, 60.0, 520.0, 420.0),
        );
        let px: f64 = 150.0 / 72.0;
        let labels = contour_labels(
            &z,
            30,
            30,
            None,
            None,
            &levels,
            &transform,
            (6.0 * px).max(8.0),
        );
        assert!(!labels.is_empty(), "expected clabels");
        let paths = contour_paths(&z, 30, 30, None, None, &levels, &transform);
        let mut blocked = 0usize;
        for path in &paths {
            let els = path.path.elements();
            let Some(kurbo::PathEl::MoveTo(a)) = els.first() else {
                continue;
            };
            let Some(kurbo::PathEl::LineTo(b)) = els.get(1) else {
                continue;
            };
            let q0 = Point::new(a.x, a.y);
            let q1 = Point::new(b.x, b.y);
            for lab in &labels {
                if (lab.level - path.level).abs() >= 1e-12 {
                    continue;
                }
                let text = format_contour_level(lab.level);
                let em = 7.0 * px;
                let half_w = (text.len() as f64 * 0.55 * em).max(6.0 * px);
                let half_h = (0.7 * em).max(4.0 * px);
                if segment_in_label_gap(q0, q1, lab, half_w, half_h) {
                    blocked += 1;
                    break;
                }
            }
        }
        assert!(
            blocked >= labels.len(),
            "expected ≥1 blocked segment per label; blocked={blocked} labels={} paths={}",
            labels.len(),
            paths.len()
        );
    }
}
