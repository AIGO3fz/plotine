use plotine_core::{Cmap, Color, Norm};

use crate::recipes::bar::BarRect;
use crate::recipes::heatmap::{heatmap_limits, HeatmapCell};

/// Build pcolormesh cells from edge arrays.
///
/// `x_edges.len() == ncols + 1`, `y_edges.len() == nrows + 1`,
/// `values` is row-major `nrows * ncols`.
pub fn pcolormesh_cells(
    x_edges: &[f64],
    y_edges: &[f64],
    values: &[f64],
    cmap: &Cmap,
    vmin: f64,
    vmax: f64,
    norm: Norm,
) -> Vec<HeatmapCell> {
    if x_edges.len() < 2 || y_edges.len() < 2 {
        return Vec::new();
    }
    let ncols = x_edges.len() - 1;
    let nrows = y_edges.len() - 1;
    let mut out = Vec::with_capacity(nrows * ncols);
    for row in 0..nrows {
        for col in 0..ncols {
            let value = values.get(row * ncols + col).copied().unwrap_or(f64::NAN);
            out.push(HeatmapCell {
                rect: BarRect {
                    x0: x_edges[col],
                    x1: x_edges[col + 1],
                    y0: y_edges[row],
                    y1: y_edges[row + 1],
                },
                color: if value.is_finite() {
                    cmap.map_norm(value, vmin, vmax, norm)
                } else {
                    Color::rgba(0, 0, 0, 0)
                },
            });
        }
    }
    out
}

pub fn pcolormesh_limits(values: &[f64], vmin: Option<f64>, vmax: Option<f64>) -> (f64, f64) {
    heatmap_limits(values, vmin, vmax)
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{Cmap, Colormap};

    #[test]
    fn cell_count() {
        let cells = pcolormesh_cells(
            &[0.0, 1.0, 3.0],
            &[0.0, 2.0, 4.0],
            &[1.0, 2.0, 3.0, 4.0],
            &Cmap::from(Colormap::Viridis),
            1.0,
            4.0,
            Norm::Linear,
        );
        assert_eq!(cells.len(), 4);
        assert!((cells[1].rect.x1 - cells[1].rect.x0 - 2.0).abs() < 1e-9);
    }
}
