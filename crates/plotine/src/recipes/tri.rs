//! Triangular mesh geometry for `tripcolor` / `tricontour`.
//!
//! MVP requires explicit triangle indices (no automatic Delaunay).

use kurbo::BezPath;
use plotine_core::{Cmap, Color, DataToPixel, Norm, Point};

use super::contour::{nice_levels, ContourPath};

/// One filled triangle in pixel space.
#[derive(Debug, Clone)]
pub struct TriFill {
    pub path: BezPath,
    pub color: Color,
}

/// Validate triangle indices against vertex count.
pub fn validate_triangles(n_verts: usize, triangles: &[[usize; 3]]) -> Result<(), String> {
    if triangles.is_empty() {
        return Err("triangles is empty".into());
    }
    for (i, t) in triangles.iter().enumerate() {
        for &idx in t {
            if idx >= n_verts {
                return Err(format!(
                    "triangle {i} references vertex {idx} but only {n_verts} vertices exist"
                ));
            }
        }
        if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
            return Err(format!("triangle {i} has duplicate vertex indices"));
        }
    }
    Ok(())
}

/// Face-mean z limits for flat-shaded `tripcolor` (matplotlib default clim).
pub fn tripcolor_face_limits(z: &[f64], triangles: &[[usize; 3]]) -> Option<(f64, f64)> {
    let mut zmin = f64::INFINITY;
    let mut zmax = f64::NEG_INFINITY;
    for t in triangles {
        let mut sum = 0.0;
        let mut n = 0usize;
        for &i in t {
            if let Some(&zi) = z.get(i) {
                if zi.is_finite() {
                    sum += zi;
                    n += 1;
                }
            }
        }
        if n == 0 {
            continue;
        }
        let zmean = sum / n as f64;
        zmin = zmin.min(zmean);
        zmax = zmax.max(zmean);
    }
    if zmin.is_finite() {
        Some((zmin, zmax))
    } else {
        None
    }
}

/// Data limits of vertices used by triangles.
pub fn tri_limits(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    triangles: &[[usize; 3]],
) -> Option<(f64, f64, f64, f64, f64, f64)> {
    let n = x.len().min(y.len()).min(z.len());
    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    let mut zmin = f64::INFINITY;
    let mut zmax = f64::NEG_INFINITY;
    for t in triangles {
        for &i in t {
            if i >= n {
                continue;
            }
            let (xi, yi, zi) = (x[i], y[i], z[i]);
            if xi.is_finite() {
                xmin = xmin.min(xi);
                xmax = xmax.max(xi);
            }
            if yi.is_finite() {
                ymin = ymin.min(yi);
                ymax = ymax.max(yi);
            }
            if zi.is_finite() {
                zmin = zmin.min(zi);
                zmax = zmax.max(zi);
            }
        }
    }
    if xmin.is_finite() && ymin.is_finite() && zmin.is_finite() {
        Some((xmin, xmax, ymin, ymax, zmin, zmax))
    } else {
        None
    }
}

/// Constant-color filled triangles (matplotlib `tripcolor`, faceted).
#[allow(clippy::too_many_arguments)]
pub fn tripcolor_fills(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    triangles: &[[usize; 3]],
    vmin: f64,
    vmax: f64,
    cmap: &Cmap,
    norm: Norm,
    transform: &DataToPixel,
) -> Vec<TriFill> {
    let n = x.len().min(y.len()).min(z.len());
    let mut out = Vec::with_capacity(triangles.len());
    for t in triangles {
        let (i, j, k) = (t[0], t[1], t[2]);
        if i >= n || j >= n || k >= n {
            continue;
        }
        let (xi, yi, zi) = (x[i], y[i], z[i]);
        let (xj, yj, zj) = (x[j], y[j], z[j]);
        let (xk, yk, zk) = (x[k], y[k], z[k]);
        if !(xi.is_finite()
            && yi.is_finite()
            && xj.is_finite()
            && yj.is_finite()
            && xk.is_finite()
            && yk.is_finite())
        {
            continue;
        }
        let zvals = [zi, zj, zk];
        let finite: Vec<f64> = zvals.into_iter().filter(|v| v.is_finite()).collect();
        if finite.is_empty() {
            continue;
        }
        let zmean = finite.iter().sum::<f64>() / finite.len() as f64;
        let p0 = transform.map(Point::new(xi, yi));
        let p1 = transform.map(Point::new(xj, yj));
        let p2 = transform.map(Point::new(xk, yk));
        let mut path = BezPath::new();
        path.move_to(p0.to_kurbo());
        path.line_to(p1.to_kurbo());
        path.line_to(p2.to_kurbo());
        path.close_path();
        out.push(TriFill {
            path,
            color: cmap.map_norm(zmean, vmin, vmax, norm),
        });
    }
    out
}

fn edge_crossing(
    xa: f64,
    ya: f64,
    za: f64,
    xb: f64,
    yb: f64,
    zb: f64,
    level: f64,
) -> Option<Point> {
    if !(za.is_finite() && zb.is_finite()) {
        return None;
    }
    let da = za - level;
    let db = zb - level;
    if da == 0.0 {
        return Some(Point::new(xa, ya));
    }
    if db == 0.0 {
        return Some(Point::new(xb, yb));
    }
    if da * db > 0.0 {
        return None;
    }
    let t = (da / (da - db)).clamp(0.0, 1.0);
    Some(Point::new(xa + (xb - xa) * t, ya + (yb - ya) * t))
}

/// Marching-triangles contour segments for one level.
fn tri_level_segments(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    triangles: &[[usize; 3]],
    level: f64,
) -> Vec<(Point, Point)> {
    let n = x.len().min(y.len()).min(z.len());
    let mut segs = Vec::new();
    for t in triangles {
        let (i, j, k) = (t[0], t[1], t[2]);
        if i >= n || j >= n || k >= n {
            continue;
        }
        let verts = [(x[i], y[i], z[i]), (x[j], y[j], z[j]), (x[k], y[k], z[k])];
        let edges = [(0usize, 1usize), (1, 2), (2, 0)];
        let mut crossings = Vec::with_capacity(3);
        for (a, b) in edges {
            let (xa, ya, za) = verts[a];
            let (xb, yb, zb) = verts[b];
            if let Some(p) = edge_crossing(xa, ya, za, xb, yb, zb, level) {
                // Dedup identical endpoints (vertex hits).
                if crossings
                    .last()
                    .map(|q: &Point| (q.x - p.x).abs() > 1e-12 || (q.y - p.y).abs() > 1e-12)
                    .unwrap_or(true)
                {
                    crossings.push(p);
                }
            }
        }
        if crossings.len() >= 2 {
            segs.push((crossings[0], crossings[1]));
        }
    }
    segs
}

/// Contour polylines on a triangular mesh (matplotlib `tricontour`).
pub fn tricontour_paths(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    triangles: &[[usize; 3]],
    levels: &[f64],
    transform: &DataToPixel,
) -> Vec<ContourPath> {
    let mut out = Vec::new();
    for &level in levels {
        if !level.is_finite() {
            continue;
        }
        for (a, b) in tri_level_segments(x, y, z, triangles, level) {
            let p0 = transform.map(a);
            let p1 = transform.map(b);
            let mut path = BezPath::new();
            path.move_to(p0.to_kurbo());
            path.line_to(p1.to_kurbo());
            out.push(ContourPath { path, level });
        }
    }
    out
}

/// Filled contour regions on a triangular mesh (matplotlib `tricontourf`).
///
/// Each triangle face is colored by the mean z-value of its vertices,
/// binned into the provided levels. This is equivalent to `tripcolor`
/// with level-quantized coloring.
#[allow(clippy::too_many_arguments)]
pub fn tricontourf_fills(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    triangles: &[[usize; 3]],
    levels: &[f64],
    cmap: &Cmap,
    norm: Norm,
    transform: &DataToPixel,
) -> Vec<TriFill> {
    if levels.len() < 2 {
        return Vec::new();
    }
    let n = x.len().min(y.len()).min(z.len());
    let vmin = levels.first().copied().unwrap_or(0.0);
    let vmax = levels.last().copied().unwrap_or(1.0);
    let mut out = Vec::with_capacity(triangles.len());
    for t in triangles {
        let (i, j, k) = (t[0], t[1], t[2]);
        if i >= n || j >= n || k >= n {
            continue;
        }
        let (xi, yi, zi) = (x[i], y[i], z[i]);
        let (xj, yj, zj) = (x[j], y[j], z[j]);
        let (xk, yk, zk) = (x[k], y[k], z[k]);
        if !(xi.is_finite()
            && yi.is_finite()
            && xj.is_finite()
            && yj.is_finite()
            && xk.is_finite()
            && yk.is_finite())
        {
            continue;
        }
        let zvals = [zi, zj, zk];
        let finite: Vec<f64> = zvals.into_iter().filter(|v| v.is_finite()).collect();
        if finite.is_empty() {
            continue;
        }
        let zmean = finite.iter().sum::<f64>() / finite.len() as f64;
        // Find which level bin the mean falls in
        let bin = levels
            .windows(2)
            .position(|w| zmean >= w[0] && zmean < w[1])
            .unwrap_or_else(|| {
                if zmean < levels[0] {
                    0
                } else {
                    levels.len().saturating_sub(2)
                }
            });
        let mid = 0.5 * (levels[bin] + levels[bin + 1]);
        let color = cmap.map_norm(mid, vmin, vmax, norm);
        let p0 = transform.map(Point::new(xi, yi));
        let p1 = transform.map(Point::new(xj, yj));
        let p2 = transform.map(Point::new(xk, yk));
        let mut path = BezPath::new();
        path.move_to(p0.to_kurbo());
        path.line_to(p1.to_kurbo());
        path.line_to(p2.to_kurbo());
        path.close_path();
        out.push(TriFill { path, color });
    }
    out
}

/// Resolve contour levels from explicit list or auto count.
pub fn resolve_tri_levels(
    z: &[f64],
    triangles: &[[usize; 3]],
    explicit: Option<&[f64]>,
    n: usize,
) -> Vec<f64> {
    if let Some(levels) = explicit {
        return levels.iter().copied().filter(|v| v.is_finite()).collect();
    }
    let mut zmin = f64::INFINITY;
    let mut zmax = f64::NEG_INFINITY;
    for t in triangles {
        for &i in t {
            if let Some(&zi) = z.get(i) {
                if zi.is_finite() {
                    zmin = zmin.min(zi);
                    zmax = zmax.max(zi);
                }
            }
        }
    }
    if !zmin.is_finite() {
        return Vec::new();
    }
    nice_levels(zmin, zmax, n.max(1))
}

/// Test if point (px, py) lies inside the circumcircle of triangle (a, b, c).
///
/// Uses the determinant sign test (positive when inside for CCW-ordered vertices).
#[allow(clippy::too_many_arguments)]
fn circumcircle_contains(
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    cx: f64,
    cy: f64,
    px: f64,
    py: f64,
) -> bool {
    let dax = ax - px;
    let day = ay - py;
    let dbx = bx - px;
    let dby = by - py;
    let dcx = cx - px;
    let dcy = cy - py;

    let det = dax * (dby * (dcx * dcx + dcy * dcy) - dcy * (dbx * dbx + dby * dby))
        - day * (dbx * (dcx * dcx + dcy * dcy) - dcx * (dbx * dbx + dby * dby))
        + (dax * dax + day * day) * (dbx * dcy - dby * dcx);

    det > 0.0
}

/// Ensure triangle vertices are in counter-clockwise order.
fn ensure_ccw(tri: &mut [usize; 3], x: &[f64], y: &[f64]) {
    let (ax, ay) = (x[tri[0]], y[tri[0]]);
    let (bx, by) = (x[tri[1]], y[tri[1]]);
    let (cx, cy) = (x[tri[2]], y[tri[2]]);
    let cross = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
    if cross < 0.0 {
        tri.swap(1, 2);
    }
}

/// Compute Delaunay triangulation of 2D points (Bowyer-Watson algorithm).
///
/// Returns triangle indices `[[i, j, k], ...]` suitable for `tripcolor` / `tricontour`.
/// Points must have at least 3 non-collinear entries.
pub fn delaunay(x: &[f64], y: &[f64]) -> Result<Vec<[usize; 3]>, String> {
    let n = x.len().min(y.len());
    if n < 3 {
        return Err("need at least 3 points for triangulation".into());
    }

    // Check for all-collinear points.
    let mut all_collinear = true;
    for i in 2..n {
        let cross = (x[1] - x[0]) * (y[i] - y[0]) - (y[1] - y[0]) * (x[i] - x[0]);
        if cross.abs() > 1e-10 {
            all_collinear = false;
            break;
        }
    }
    if all_collinear {
        return Err("all points are collinear".into());
    }

    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for i in 0..n {
        if !x[i].is_finite() || !y[i].is_finite() {
            return Err(format!("point {i} has non-finite coordinates"));
        }
        xmin = xmin.min(x[i]);
        xmax = xmax.max(x[i]);
        ymin = ymin.min(y[i]);
        ymax = ymax.max(y[i]);
    }

    let dx = (xmax - xmin).max(1e-10);
    let dy = (ymax - ymin).max(1e-10);
    let dmax = dx.max(dy);
    let mid_x = (xmin + xmax) * 0.5;
    let mid_y = (ymin + ymax) * 0.5;

    // Super-triangle vertices (indices n, n+1, n+2 in the extended point set).
    let margin = 20.0;
    let st0 = (mid_x - margin * dmax, mid_y - margin * dmax);
    let st1 = (mid_x + margin * dmax, mid_y - margin * dmax);
    let st2 = (mid_x, mid_y + margin * dmax);

    // Extended coordinates: original points + 3 super-triangle vertices.
    let mut all_x: Vec<f64> = x[..n].to_vec();
    let mut all_y: Vec<f64> = y[..n].to_vec();
    all_x.extend_from_slice(&[st0.0, st1.0, st2.0]);
    all_y.extend_from_slice(&[st0.1, st1.1, st2.1]);

    let mut triangles: Vec<[usize; 3]> = vec![[n, n + 1, n + 2]];

    for i in 0..n {
        let px = all_x[i];
        let py = all_y[i];

        // Find all triangles whose circumcircle contains this point.
        let mut bad = Vec::new();
        for (ti, tri) in triangles.iter().enumerate() {
            if circumcircle_contains(
                all_x[tri[0]],
                all_y[tri[0]],
                all_x[tri[1]],
                all_y[tri[1]],
                all_x[tri[2]],
                all_y[tri[2]],
                px,
                py,
            ) {
                bad.push(ti);
            }
        }

        // Collect boundary edges of the polygonal hole.
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for &bi in &bad {
            let tri = triangles[bi];
            let tri_edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
            for (a, b) in tri_edges {
                let is_shared = bad.iter().any(|&bj| {
                    bj != bi && {
                        let other = triangles[bj];
                        let oe = [
                            (other[0], other[1]),
                            (other[1], other[2]),
                            (other[2], other[0]),
                        ];
                        oe.contains(&(b, a)) || oe.contains(&(a, b))
                    }
                });
                if !is_shared {
                    edges.push((a, b));
                }
            }
        }

        // Remove bad triangles (reverse order to keep indices valid).
        bad.sort_unstable();
        for &bi in bad.iter().rev() {
            triangles.swap_remove(bi);
        }

        // Re-triangulate: connect the new point to each boundary edge.
        for (a, b) in edges {
            let mut tri = [i, a, b];
            ensure_ccw(&mut tri, &all_x, &all_y);
            triangles.push(tri);
        }
    }

    // Remove all triangles referencing super-triangle vertices.
    triangles.retain(|tri| tri[0] < n && tri[1] < n && tri[2] < n);

    if triangles.is_empty() {
        return Err("triangulation produced no triangles".into());
    }

    Ok(triangles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{Cmap, Colormap};
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn transform() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 2.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        )
    }

    #[test]
    fn tripcolor_one_triangle() {
        let x = [0.0, 1.0, 0.0];
        let y = [0.0, 0.0, 1.0];
        let z = [0.0, 1.0, 0.5];
        let tris = [[0, 1, 2]];
        let fills = tripcolor_fills(
            &x,
            &y,
            &z,
            &tris,
            0.0,
            1.0,
            &Cmap::from(Colormap::Viridis),
            Norm::Linear,
            &transform(),
        );
        assert_eq!(fills.len(), 1);
    }

    #[test]
    fn tripcolor_face_limits_use_means() {
        let z = [0.0, 0.4, 0.1, 0.8, 1.0, 0.6];
        let tris = [[0, 1, 3], [1, 2, 4], [1, 3, 4], [3, 4, 5]];
        let (lo, hi) = tripcolor_face_limits(&z, &tris).unwrap();
        assert!((lo - 0.4).abs() < 1e-12);
        assert!((hi - 0.8).abs() < 1e-12);
    }

    #[test]
    fn tricontour_crosses_mid() {
        let x = [0.0, 2.0, 0.0];
        let y = [0.0, 0.0, 2.0];
        let z = [0.0, 1.0, 0.0];
        let tris = [[0, 1, 2]];
        let paths = tricontour_paths(&x, &y, &z, &tris, &[0.5], &transform());
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn validate_rejects_oob() {
        assert!(validate_triangles(3, &[[0, 1, 3]]).is_err());
    }

    #[test]
    fn delaunay_three_points() {
        let x = [0.0, 1.0, 0.5];
        let y = [0.0, 0.0, 1.0];
        let tris = delaunay(&x, &y).unwrap();
        assert_eq!(tris.len(), 1);
        for tri in &tris {
            for &idx in tri {
                assert!(idx < 3);
            }
        }
    }

    #[test]
    fn delaunay_square_four_points() {
        let x = [0.0, 1.0, 1.0, 0.0];
        let y = [0.0, 0.0, 1.0, 1.0];
        let tris = delaunay(&x, &y).unwrap();
        assert_eq!(tris.len(), 2);
        for tri in &tris {
            for &idx in tri {
                assert!(idx < 4);
            }
        }
    }

    #[test]
    fn delaunay_collinear_error() {
        let x = [0.0, 1.0, 2.0];
        let y = [0.0, 1.0, 2.0];
        assert!(delaunay(&x, &y).is_err());
    }

    #[test]
    fn delaunay_too_few_points() {
        let x = [0.0, 1.0];
        let y = [0.0, 1.0];
        assert!(delaunay(&x, &y).is_err());
    }
}
