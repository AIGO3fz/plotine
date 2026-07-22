//! Box-and-whisker statistics and geometry.

use crate::recipes::BarRect;

/// Summary statistics for one boxplot group.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxStats {
    /// Category x-position (1-based by default).
    pub x: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub whisker_lo: f64,
    pub whisker_hi: f64,
    pub fliers: Vec<f64>,
}

/// Compute Tukey boxplot stats for each group (`x = 1..n`).
pub fn boxplot_stats(groups: &[&[f64]], width: f64) -> Vec<(BoxStats, BarRect)> {
    let width = width.clamp(0.1, 0.95);
    groups
        .iter()
        .enumerate()
        .filter_map(|(i, sample)| {
            let stats = summarize(i as f64 + 1.0, sample)?;
            let half = width * 0.5;
            let box_rect = BarRect {
                x0: stats.x - half,
                x1: stats.x + half,
                y0: stats.q1,
                y1: stats.q3,
            };
            Some((stats, box_rect))
        })
        .collect()
}

fn summarize(x: f64, sample: &[f64]) -> Option<BoxStats> {
    let mut vals: Vec<f64> = sample.iter().copied().filter(|v| v.is_finite()).collect();
    if vals.is_empty() {
        return None;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q1 = quantile_sorted(&vals, 0.25);
    let median = quantile_sorted(&vals, 0.5);
    let q3 = quantile_sorted(&vals, 0.75);
    let iqr = (q3 - q1).max(0.0);
    let lo_fence = q1 - 1.5 * iqr;
    let hi_fence = q3 + 1.5 * iqr;

    let mut whisker_lo = vals[0];
    let mut whisker_hi = *vals.last().unwrap();
    for &v in &vals {
        if v >= lo_fence {
            whisker_lo = v;
            break;
        }
    }
    for &v in vals.iter().rev() {
        if v <= hi_fence {
            whisker_hi = v;
            break;
        }
    }

    let fliers: Vec<f64> = vals
        .into_iter()
        .filter(|&v| v < lo_fence || v > hi_fence)
        .collect();

    Some(BoxStats {
        x,
        q1,
        median,
        q3,
        whisker_lo,
        whisker_hi,
        fliers,
    })
}

/// Linear-interpolated quantile (numpy `linear` / Type 7).
pub(crate) fn quantile_sorted(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let q = q.clamp(0.0, 1.0);
    let pos = q * (n - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let f = pos - lo as f64;
    sorted[lo] * (1.0 - f) + sorted[hi] * f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_of_odd_sample() {
        let s = [1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = summarize(1.0, &s).unwrap();
        assert!((stats.median - 3.0).abs() < 1e-12);
        assert!((stats.q1 - 2.0).abs() < 1e-12);
        assert!((stats.q3 - 4.0).abs() < 1e-12);
    }

    #[test]
    fn fliers_detected() {
        let s = [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 10.0];
        let stats = summarize(1.0, &s).unwrap();
        assert!(stats.fliers.iter().any(|v| (*v - 10.0).abs() < 1e-12));
    }
}
