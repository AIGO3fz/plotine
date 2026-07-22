//! Violin plot geometry via Gaussian KDE (Scott bandwidth, matplotlib-compatible).

use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

use crate::recipes::boxplot::quantile_sorted;

/// One violin outline in data coordinates (ready to transform).
#[derive(Debug, Clone)]
pub struct ViolinGeom {
    pub center: f64,
    pub median: f64,
    /// Sample minimum (extrema stem bottom).
    pub ymin: f64,
    /// Sample maximum (extrema stem top).
    pub ymax: f64,
    pub y: Vec<f64>,
    pub density: Vec<f64>,
    pub max_density: f64,
}

/// Build violin geometries for each group (`x = 1..n`).
pub fn violin_geoms(groups: &[&[f64]], points: usize) -> Vec<ViolinGeom> {
    let points = points.clamp(16, 256);
    groups
        .iter()
        .enumerate()
        .filter_map(|(i, sample)| violin_one(i as f64 + 1.0, sample, points))
        .collect()
}

/// Closed mirrored violin path in pixel space.
pub fn violin_path(geom: &ViolinGeom, width: f64, transform: &DataToPixel) -> BezPath {
    let half = width.clamp(0.1, 0.95) * 0.5;
    let scale = if geom.max_density > 1e-12 {
        half / geom.max_density
    } else {
        0.0
    };

    let mut path = BezPath::new();
    if geom.y.is_empty() {
        return path;
    }

    // Right edge bottom → top, then left edge top → bottom.
    let first_y = geom.y[0];
    let first_d = geom.density[0] * scale;
    path.move_to(
        transform
            .map(Point::new(geom.center + first_d, first_y))
            .to_kurbo(),
    );
    for (&y, &d) in geom.y.iter().zip(geom.density.iter()).skip(1) {
        path.line_to(
            transform
                .map(Point::new(geom.center + d * scale, y))
                .to_kurbo(),
        );
    }
    for (&y, &d) in geom.y.iter().zip(geom.density.iter()).rev() {
        path.line_to(
            transform
                .map(Point::new(geom.center - d * scale, y))
                .to_kurbo(),
        );
    }
    path.close_path();
    path
}

fn violin_one(center: f64, sample: &[f64], points: usize) -> Option<ViolinGeom> {
    let mut vals: Vec<f64> = sample.iter().copied().filter(|v| v.is_finite()).collect();
    if vals.is_empty() {
        return None;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let ymin = vals[0];
    let ymax = *vals.last().unwrap();
    let bw = scott_bandwidth(&vals);
    // Matplotlib evaluates the KDE over the data range (no Silverman pad).
    let y0 = ymin;
    let y1 = ymax;
    let span = (y1 - y0).max(1e-12);

    let mut y = Vec::with_capacity(points);
    let mut density = Vec::with_capacity(points);
    let mut max_d = 0.0_f64;
    for i in 0..points {
        let yi = y0 + span * i as f64 / (points - 1).max(1) as f64;
        let d = kde(&vals, yi, bw);
        max_d = max_d.max(d);
        y.push(yi);
        density.push(d);
    }
    let median = quantile_sorted(&vals, 0.5);
    Some(ViolinGeom {
        center,
        median,
        ymin,
        ymax,
        y,
        density,
        max_density: max_d.max(1e-12),
    })
}

/// Scott's rule (scipy `gaussian_kde` / matplotlib `bw_method=None` default).
fn scott_bandwidth(sorted: &[f64]) -> f64 {
    let n = sorted.len() as f64;
    if n < 2.0 {
        return 1.0;
    }
    let mean = sorted.iter().sum::<f64>() / n;
    let var = sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = var.sqrt().max(1e-12);
    (std * n.powf(-0.2)).max(1e-12)
}

fn kde(samples: &[f64], x: f64, bw: f64) -> f64 {
    let n = samples.len() as f64;
    let inv = 1.0 / (n * bw * (std::f64::consts::TAU).sqrt());
    let mut sum = 0.0;
    for &xi in samples {
        let u = (x - xi) / bw;
        sum += (-0.5 * u * u).exp();
    }
    inv * sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn violin_has_positive_density() {
        let s = [1.0, 2.0, 2.5, 3.0, 3.2, 3.5, 4.0];
        let geoms = violin_geoms(&[&s], 32);
        assert_eq!(geoms.len(), 1);
        assert!(geoms[0].max_density > 0.0);
        assert!(geoms[0].density.iter().any(|&d| d > 0.0));
        assert!((geoms[0].ymin - 1.0).abs() < 1e-12);
        assert!((geoms[0].ymax - 4.0).abs() < 1e-12);
    }
}
