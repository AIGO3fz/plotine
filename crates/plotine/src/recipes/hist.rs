use crate::recipes::bar::BarRect;

/// Result of binning a 1D sample.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBins {
    pub edges: Vec<f64>,
    pub counts: Vec<f64>,
}

impl HistogramBins {
    pub fn bars(&self) -> Vec<BarRect> {
        self.edges
            .windows(2)
            .zip(self.counts.iter())
            .map(|(edge, &count)| BarRect {
                x0: edge[0],
                x1: edge[1],
                y0: 0.0,
                y1: count,
            })
            .collect()
    }
}

/// Equal-width histogram over finite samples.
pub fn histogram(data: &[f64], bins: usize) -> HistogramBins {
    let bins = bins.max(1);
    let values: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
    if values.is_empty() {
        return HistogramBins {
            edges: vec![0.0, 1.0],
            counts: vec![0.0],
        };
    }
    let mut min = values[0];
    let mut max = values[0];
    for &v in &values {
        min = min.min(v);
        max = max.max(v);
    }
    if (max - min).abs() < f64::EPSILON {
        max = min + 1.0;
    }
    let width = (max - min) / bins as f64;
    let mut edges = Vec::with_capacity(bins + 1);
    for i in 0..=bins {
        edges.push(min + width * i as f64);
    }
    // Ensure the last edge includes max under floating error.
    if let Some(last) = edges.last_mut() {
        *last = max;
    }

    let mut counts = vec![0.0; bins];
    for v in values {
        let mut idx = ((v - min) / width).floor() as isize;
        if idx < 0 {
            idx = 0;
        }
        if idx as usize >= bins {
            idx = (bins - 1) as isize;
        }
        counts[idx as usize] += 1.0;
    }

    HistogramBins { edges, counts }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_sum_to_n() {
        let data = [0.0, 0.1, 0.2, 0.9, 1.0, 1.1];
        let h = histogram(&data, 2);
        let sum: f64 = h.counts.iter().sum();
        assert!((sum - 6.0).abs() < 1e-12);
        assert_eq!(h.edges.len(), 3);
    }
}
