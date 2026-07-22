use plotine_core::{DataToPixel, Point};

use crate::recipes::scatter::Marker;

/// Markers at cells where `|values[i]| > precision` (row-major matrix).
pub fn spy_markers(
    nrows: usize,
    ncols: usize,
    values: &[f64],
    precision: f64,
    marker_radius: f64,
    transform: &DataToPixel,
) -> Vec<Marker> {
    let precision = precision.abs();
    let mut out = Vec::new();
    for row in 0..nrows {
        for col in 0..ncols {
            let v = values.get(row * ncols + col).copied().unwrap_or(0.0);
            if v.is_finite() && v.abs() > precision {
                let p = transform.map(Point::new(col as f64, row as f64));
                out.push(Marker {
                    center: p,
                    radius: marker_radius,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    #[test]
    fn finds_nonzero() {
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(-0.5, 2.5).unwrap()),
            ScaleKind::Linear(LinearScale::new(-0.5, 2.5).unwrap()),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let z = [0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0];
        let m = spy_markers(3, 3, &z, 1e-8, 2.0, &t);
        assert_eq!(m.len(), 2);
    }
}
