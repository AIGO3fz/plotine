//! Hexagonal binning matching matplotlib `Axes.hexbin` (pointy-top lattice).
//!
//! Matplotlib's default hex vertices are pointy-top (vertical flats on the
//! left/right). Flat-top requires rotating those vertices.

use kurbo::BezPath;
use plotine_core::{Cmap, Color, DataToPixel, Norm, Point};

use crate::mpl_policy::hexbin as hex_policy;
use crate::recipes::heatmap::heatmap_limits;

/// One filled hexagon in pixel space.
#[derive(Debug, Clone)]
pub struct HexCell {
    pub path: BezPath,
    pub color: Color,
    pub count: f64,
}

/// Pointy-top hexagon vertices in data space (matplotlib polygon scaled by sx, sy/3).
fn hex_polygon_data(cx: f64, cy: f64, sx: f64, sy: f64) -> [Point; 6] {
    let hx = sx;
    let hy = sy / 3.0;
    // [[.5,-.5],[.5,.5],[0,1],[-.5,.5],[-.5,-.5],[0,-1]] * [hx, hy]
    [
        Point::new(cx + 0.5 * hx, cy - 0.5 * hy),
        Point::new(cx + 0.5 * hx, cy + 0.5 * hy),
        Point::new(cx, cy + hy),
        Point::new(cx - 0.5 * hx, cy + 0.5 * hy),
        Point::new(cx - 0.5 * hx, cy - 0.5 * hy),
        Point::new(cx, cy - hy),
    ]
}

fn path_from_verts(verts: &[Point], transform: &DataToPixel) -> BezPath {
    let mut path = BezPath::new();
    if let Some(first) = verts.first() {
        path.move_to(transform.map(*first).to_kurbo());
        for p in verts.iter().skip(1) {
            path.line_to(transform.map(*p).to_kurbo());
        }
        path.close_path();
    }
    path
}

/// Matplotlib `transforms.nonsingular(..., expander=0.1)` used by `Axes.hexbin`.
///
/// Only expands when the interval is singular / near-zero — **not** a blanket ±10%
/// pad on every non-empty sample (that was causing denser bins than mpl).
fn nonsingular(vmin: f64, vmax: f64, expander: f64) -> (f64, f64) {
    if !vmin.is_finite() || !vmax.is_finite() {
        return (-expander, expander);
    }
    let (mut vmin, mut vmax) = if vmax < vmin {
        (vmax, vmin)
    } else {
        (vmin, vmax)
    };
    let tiny = 1e-15_f64;
    let maxabs = vmin.abs().max(vmax.abs());
    let float_tiny = f64::MIN_POSITIVE;
    if maxabs < (1e6 / tiny) * float_tiny {
        return (-expander, expander);
    }
    if vmax - vmin <= maxabs * tiny {
        if vmax == 0.0 && vmin == 0.0 {
            return (-expander, expander);
        }
        vmin -= expander * vmin.abs();
        vmax += expander * vmax.abs();
    }
    (vmin, vmax)
}

/// Resolve the matplotlib-style data extent used for hexbin binning.
pub fn hexbin_extent(x: &[f64], y: &[f64]) -> Option<(f64, f64, f64, f64)> {
    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        if xi.is_finite() && yi.is_finite() {
            xmin = xmin.min(xi);
            xmax = xmax.max(xi);
            ymin = ymin.min(yi);
            ymax = ymax.max(yi);
        }
    }
    if !xmin.is_finite() {
        return None;
    }
    let (xmin, xmax) = nonsingular(xmin, xmax, 0.1);
    let (ymin, ymax) = nonsingular(ymin, ymax, 0.1);
    Some((xmin, xmax, ymin, ymax))
}

/// Hexagonal binning of `(x, y)` with approximate grid spacing `gridsize` along x.
///
/// Geometry follows matplotlib: two interlaced pointy-top lattices, `ny = nx / √3`.
#[allow(clippy::too_many_arguments)]
pub fn hexbin_cells(
    x: &[f64],
    y: &[f64],
    gridsize: usize,
    cmap: &Cmap,
    vmin: Option<f64>,
    vmax: Option<f64>,
    norm: Norm,
    transform: &DataToPixel,
) -> (Vec<HexCell>, f64, f64) {
    let nx = gridsize.max(2);
    // matplotlib: ny = int(nx / sqrt(3))  (truncate toward zero)
    let ny = ((nx as f64) / 3.0_f64.sqrt()).max(1.0) as usize;

    let Some((mut xmin, mut xmax, ymin, ymax)) = hexbin_extent(x, y) else {
        return (Vec::new(), 0.0, 1.0);
    };
    let dx = xmax - xmin;
    let padding = 1e-9 * dx;
    xmin -= padding;
    xmax += padding;
    let sx = (xmax - xmin) / nx as f64;
    let sy = (ymax - ymin) / ny as f64;

    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let nx2 = nx;
    let ny2 = ny;
    let n1 = nx1 * ny1;
    let n2 = nx2 * ny2;
    let mut counts1 = vec![0.0; n1];
    let mut counts2 = vec![0.0; n2];

    for (&xi, &yi) in x.iter().zip(y.iter()) {
        if !(xi.is_finite() && yi.is_finite()) {
            continue;
        }
        let ix = (xi - xmin) / sx;
        let iy = (yi - ymin) / sy;
        let ix1 = ix.round() as i32;
        let iy1 = iy.round() as i32;
        let ix2 = ix.floor() as i32;
        let iy2 = iy.floor() as i32;
        let d1 = (ix - ix1 as f64).powi(2) + 3.0 * (iy - iy1 as f64).powi(2);
        let d2 = (ix - ix2 as f64 - 0.5).powi(2) + 3.0 * (iy - iy2 as f64 - 0.5).powi(2);
        if d1 < d2 {
            if ix1 >= 0 && iy1 >= 0 && (ix1 as usize) < nx1 && (iy1 as usize) < ny1 {
                counts1[ix1 as usize * ny1 + iy1 as usize] += 1.0;
            }
        } else if ix2 >= 0 && iy2 >= 0 && (ix2 as usize) < nx2 && (iy2 as usize) < ny2 {
            counts2[ix2 as usize * ny2 + iy2 as usize] += 1.0;
        }
    }

    // Matplotlib default `mincnt=None` draws empty hexes (count=0) with the
    // colormap low end — keep that stock behaviour for visual parity.
    let mut values: Vec<f64> = counts1.iter().chain(counts2.iter()).copied().collect();
    values.push(0.0);
    let (lo, hi) = heatmap_limits(&values, vmin, vmax);

    let mut cells = Vec::with_capacity(n1 + n2);
    // Lattice 1: integer centers
    for ix in 0..nx1 {
        for iy in 0..ny1 {
            let count = counts1[ix * ny1 + iy];
            let cx = xmin + ix as f64 * sx;
            let cy = ymin + iy as f64 * sy;
            let scale = hex_policy::POLYGON_SCALE;
            let verts = hex_polygon_data(cx, cy, sx * scale, sy * scale);
            cells.push(HexCell {
                path: path_from_verts(&verts, transform),
                color: cmap.map_norm(count, lo, hi, norm),
                count,
            });
        }
    }
    // Lattice 2: half-offset centers
    for ix in 0..nx2 {
        for iy in 0..ny2 {
            let count = counts2[ix * ny2 + iy];
            let cx = xmin + (ix as f64 + 0.5) * sx;
            let cy = ymin + (iy as f64 + 0.5) * sy;
            let scale = hex_policy::POLYGON_SCALE;
            let verts = hex_polygon_data(cx, cy, sx * scale, sy * scale);
            cells.push(HexCell {
                path: path_from_verts(&verts, transform),
                color: cmap.map_norm(count, lo, hi, norm),
                count,
            });
        }
    }
    (cells, lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{Cmap, Colormap};
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn bins_some_points() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 10.0).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let x = [1.0, 1.1, 5.0, 5.2, 9.0];
        let y = [1.0, 1.2, 5.0, 4.8, 9.0];
        let (cells, _, _) = hexbin_cells(
            &x,
            &y,
            5,
            &Cmap::from(Colormap::Viridis),
            None,
            None,
            Norm::Linear,
            &t,
        );
        assert!(!cells.is_empty());
    }

    #[test]
    fn extent_does_not_pad_nonsingular_ranges() {
        // Matplotlib `nonsingular(expander=0.1)` leaves healthy ranges alone.
        let x = [-1.0_f64, 1.0];
        let y = [-0.5_f64, 0.5];
        let (xmin, xmax, ymin, ymax) = hexbin_extent(&x, &y).unwrap();
        assert!((xmin - -1.0).abs() < 1e-12);
        assert!((xmax - 1.0).abs() < 1e-12);
        assert!((ymin - -0.5).abs() < 1e-12);
        assert!((ymax - 0.5).abs() < 1e-12);
    }

    #[test]
    fn extent_expands_singular_axis() {
        let x = [2.0_f64, 2.0];
        let y = [0.0_f64, 1.0];
        let (xmin, xmax, _, _) = hexbin_extent(&x, &y).unwrap();
        assert!(xmin < 2.0 && xmax > 2.0);
    }
}
