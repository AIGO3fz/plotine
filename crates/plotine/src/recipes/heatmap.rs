//! Heatmap geometry: grid cells + value → color.

use plotine_core::{Cmap, Color, Norm};

use crate::recipes::BarRect;

/// Vertical placement of row 0 (matplotlib `imshow` `origin`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeatmapOrigin {
    /// Row 0 at the **top** of the image (matplotlib `imshow` default).
    ///
    /// With the default index box this inverts the y-axis so tick `-0.5` is at
    /// the top; with an explicit `.extent([...])` the axis stays upright and
    /// row 0 is placed at high data-y.
    #[default]
    Upper,
    /// Row 0 at the **bottom** (upright y-axis).
    Lower,
}

/// One colored cell of a heatmap in data coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatmapCell {
    pub rect: BarRect,
    pub color: Color,
}

/// Resolve value limits from data (ignoring non-finite entries).
pub fn heatmap_limits(values: &[f64], vmin: Option<f64>, vmax: Option<f64>) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &v in values {
        if v.is_finite() {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return (0.0, 1.0);
    }
    let vmin = vmin.unwrap_or(lo);
    let vmax = vmax.unwrap_or(hi);
    if (vmax - vmin).abs() < 1e-12 {
        (vmin - 0.5, vmax + 0.5)
    } else if vmin <= vmax {
        (vmin, vmax)
    } else {
        (vmax, vmin)
    }
}

/// Build row-major heatmap cells.
///
/// Default cells are centered on integer indices (matplotlib `imshow` extent
/// `(-0.5, ncols-0.5, …)`). Pass `extent = Some([left, right, bottom, top])` to
/// match `imshow(..., extent=…)`.
///
/// Row placement matches matplotlib:
/// - **Default index box + [`HeatmapOrigin::Upper`]**: row 0 at low data-y; the
///   axes invert y so that row appears at the top (mpl `ylim=(top,bottom)`).
/// - **Explicit `extent` + Upper**, or **Lower**: row 0 at the top / bottom of
///   the extent respectively, with upright y.
#[allow(clippy::too_many_arguments)]
pub fn heatmap_cells(
    nrows: usize,
    ncols: usize,
    values: &[f64],
    cmap: &Cmap,
    vmin: f64,
    vmax: f64,
    norm: Norm,
    origin: HeatmapOrigin,
    extent: Option<[f64; 4]>,
) -> Vec<HeatmapCell> {
    let nrows = nrows.max(1);
    let ncols = ncols.max(1);
    let explicit_extent = extent.is_some();
    let (x0, x1, y0, y1) = match extent {
        Some([l, r, b, t]) if l.is_finite() && r.is_finite() && b.is_finite() && t.is_finite() => {
            (l, r, b, t)
        }
        _ => (-0.5, (ncols as f64) - 0.5, -0.5, (nrows as f64) - 0.5),
    };
    let dx = (x1 - x0) / ncols as f64;
    let dy = (y1 - y0) / nrows as f64;
    let mut out = Vec::with_capacity(nrows * ncols);
    for row in 0..nrows {
        // Default Upper uses low data-y + inverted axes (mpl index-box imshow).
        // Explicit extent Upper keeps upright axes and puts row 0 at high y.
        let ry = match origin {
            HeatmapOrigin::Upper if explicit_extent => (nrows - 1 - row) as f64,
            HeatmapOrigin::Upper | HeatmapOrigin::Lower => row as f64,
        };
        let cy0 = y0 + ry * dy;
        let cy1 = cy0 + dy;
        for col in 0..ncols {
            let idx = row * ncols + col;
            let value = values.get(idx).copied().unwrap_or(f64::NAN);
            let cx0 = x0 + col as f64 * dx;
            out.push(HeatmapCell {
                rect: BarRect {
                    x0: cx0,
                    x1: cx0 + dx,
                    y0: cy0.min(cy1),
                    y1: cy0.max(cy1),
                },
                color: cmap.map_norm(value, vmin, vmax, norm),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{Cmap, Colormap};

    #[test]
    fn cells_cover_grid_upper() {
        let cells = heatmap_cells(
            2,
            3,
            &[0.0, 0.5, 1.0, 0.25, 0.75, 0.1],
            &Cmap::from(Colormap::Viridis),
            0.0,
            1.0,
            Norm::Linear,
            HeatmapOrigin::Upper,
            None,
        );
        assert_eq!(cells.len(), 6);
        // Default Upper: row 0 at low data-y (axes invert y so it appears on top).
        assert!((cells[0].rect.y0 - (-0.5)).abs() < 1e-12);
        assert!((cells[0].rect.y1 - 0.5).abs() < 1e-12);
        // Last cell x extent.
        assert!((cells[5].rect.x1 - 2.5).abs() < 1e-12);
    }

    #[test]
    fn cells_cover_grid_lower() {
        let cells = heatmap_cells(
            2,
            3,
            &[0.0, 0.5, 1.0, 0.25, 0.75, 0.1],
            &Cmap::from(Colormap::Viridis),
            0.0,
            1.0,
            Norm::Linear,
            HeatmapOrigin::Lower,
            None,
        );
        assert!((cells[0].rect.y0 + 0.5).abs() < 1e-12);
        assert!((cells[3].rect.y0 - 0.5).abs() < 1e-12);
    }

    #[test]
    fn extent_maps_into_data_box() {
        let cells = heatmap_cells(
            2,
            2,
            &[0.0, 1.0, 0.5, 0.25],
            &Cmap::from(Colormap::Viridis),
            0.0,
            1.0,
            Norm::Linear,
            HeatmapOrigin::Lower,
            Some([0.0, 10.0, 0.0, 4.0]),
        );
        assert!((cells[0].rect.x0 - 0.0).abs() < 1e-12);
        assert!((cells[0].rect.x1 - 5.0).abs() < 1e-12);
        assert!((cells[0].rect.y0 - 0.0).abs() < 1e-12);
        assert!((cells[0].rect.y1 - 2.0).abs() < 1e-12);
        assert!((cells[3].rect.x1 - 10.0).abs() < 1e-12);
        assert!((cells[3].rect.y1 - 4.0).abs() < 1e-12);
    }
}
