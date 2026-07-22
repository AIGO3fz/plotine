use plotine_core::Cmap;

use crate::recipes::bar::BarRect;
use crate::recipes::heatmap::{heatmap_limits, HeatmapCell};

/// Resolve count limits for colormap scaling.
pub fn hist2d_limits(values: &[f64], vmin: Option<f64>, vmax: Option<f64>) -> (f64, f64) {
    heatmap_limits(values, vmin, vmax)
}

/// 2D histogram binning result.
#[derive(Debug, Clone)]
pub struct Hist2dBins {
    pub x_edges: Vec<f64>,
    pub y_edges: Vec<f64>,
    /// Row-major counts: `ny * nx`, row 0 is the lowest y-bin.
    pub counts: Vec<f64>,
    pub nx: usize,
    pub ny: usize,
}

/// Bin `(x, y)` samples into an `nx × ny` grid over the data extent.
pub fn hist2d_bins(x: &[f64], y: &[f64], nx: usize, ny: usize) -> Hist2dBins {
    let nx = nx.max(1);
    let ny = ny.max(1);
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
        return Hist2dBins {
            x_edges: vec![0.0, 1.0],
            y_edges: vec![0.0, 1.0],
            counts: vec![0.0],
            nx: 1,
            ny: 1,
        };
    }
    if (xmax - xmin).abs() < 1e-12 {
        xmin -= 0.5;
        xmax += 0.5;
    }
    if (ymax - ymin).abs() < 1e-12 {
        ymin -= 0.5;
        ymax += 0.5;
    }

    let mut x_edges = Vec::with_capacity(nx + 1);
    let mut y_edges = Vec::with_capacity(ny + 1);
    for i in 0..=nx {
        x_edges.push(xmin + (xmax - xmin) * i as f64 / nx as f64);
    }
    for i in 0..=ny {
        y_edges.push(ymin + (ymax - ymin) * i as f64 / ny as f64);
    }

    let mut counts = vec![0.0; nx * ny];
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        if !(xi.is_finite() && yi.is_finite()) {
            continue;
        }
        let mut cx = ((xi - xmin) / (xmax - xmin) * nx as f64).floor() as isize;
        let mut cy = ((yi - ymin) / (ymax - ymin) * ny as f64).floor() as isize;
        if xi >= xmax {
            cx = nx as isize - 1;
        }
        if yi >= ymax {
            cy = ny as isize - 1;
        }
        if cx >= 0 && cy >= 0 && (cx as usize) < nx && (cy as usize) < ny {
            counts[cy as usize * nx + cx as usize] += 1.0;
        }
    }

    Hist2dBins {
        x_edges,
        y_edges,
        counts,
        nx,
        ny,
    }
}

/// Colored cells for a 2D histogram.
pub fn hist2d_cells(
    bins: &Hist2dBins,
    cmap: &Cmap,
    vmin: f64,
    vmax: f64,
    norm: plotine_core::Norm,
) -> Vec<HeatmapCell> {
    let mut out = Vec::with_capacity(bins.nx * bins.ny);
    for row in 0..bins.ny {
        for col in 0..bins.nx {
            let value = bins.counts[row * bins.nx + col];
            out.push(HeatmapCell {
                rect: BarRect {
                    x0: bins.x_edges[col],
                    x1: bins.x_edges[col + 1],
                    y0: bins.y_edges[row],
                    y1: bins.y_edges[row + 1],
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

    #[test]
    fn counts_sum_to_n() {
        let x = [0.1, 0.2, 0.8, 0.9];
        let y = [0.1, 0.9, 0.1, 0.9];
        let bins = hist2d_bins(&x, &y, 2, 2);
        let sum: f64 = bins.counts.iter().sum();
        assert!((sum - 4.0).abs() < 1e-9);
    }
}
