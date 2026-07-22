//! Seaborn-style statistical helpers (thin layer over core artists).
//!
//! These are convenience builders — not a full seaborn / Grammar of Graphics DSL.

use plotine_core::{Color, Colormap, Norm, PlotError, Result};
use plotine_render::{TextAlign, TextBaseline};

use crate::axes::Axes;
use crate::figure::Figure;
use crate::legend::Legend;
use crate::theme::Theme;

/// Pearson correlation matrix for columns of equal length.
///
/// Returns an `n×n` row-major matrix. Non-finite pairs are skipped pairwise.
pub fn corrcoef(columns: &[&[f64]]) -> Result<Vec<f64>> {
    let n = columns.len();
    if n == 0 {
        return Err(PlotError::render(
            "corrcoef requires at least one column; pass non-empty column slices",
        ));
    }
    let len = columns[0].len();
    if len < 2 {
        return Err(PlotError::render(
            "corrcoef needs at least 2 samples per column",
        ));
    }
    for c in columns {
        if c.len() != len {
            return Err(PlotError::length_mismatch(len, c.len()));
        }
    }
    let mut out = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            out[i * n + j] = pearson(columns[i], columns[j]);
        }
    }
    Ok(out)
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (&x, &y) in a.iter().zip(b.iter()) {
        if x.is_finite() && y.is_finite() {
            xs.push(x);
            ys.push(y);
        }
    }
    let n = xs.len() as f64;
    if n < 2.0 {
        return f64::NAN;
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut dx2 = 0.0;
    let mut dy2 = 0.0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let dx = x - mx;
        let dy = y - my;
        num += dx * dy;
        dx2 += dx * dx;
        dy2 += dy * dy;
    }
    let den = (dx2 * dy2).sqrt();
    if den < 1e-15 {
        return f64::NAN;
    }
    num / den
}

/// OLS slope/intercept for `y ≈ a + b·x`. Returns `(intercept, slope)`.
pub fn linregress(x: &[f64], y: &[f64]) -> Result<(f64, f64)> {
    let fit = ols_fit(x, y)?;
    Ok((fit.intercept, fit.slope))
}

struct OlsFit {
    intercept: f64,
    slope: f64,
    /// Residual standard error.
    sigma: f64,
    n: f64,
    x_mean: f64,
    sxx: f64,
}

fn ols_fit(x: &[f64], y: &[f64]) -> Result<OlsFit> {
    if x.len() != y.len() {
        return Err(PlotError::length_mismatch(x.len(), y.len()));
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (&a, &b) in x.iter().zip(y.iter()) {
        if a.is_finite() && b.is_finite() {
            xs.push(a);
            ys.push(b);
        }
    }
    let n = xs.len() as f64;
    if n < 2.0 {
        return Err(PlotError::render(
            "linregress needs at least 2 finite (x,y) pairs",
        ));
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    for (&a, &b) in xs.iter().zip(ys.iter()) {
        let dx = a - mx;
        sxx += dx * dx;
        sxy += dx * (b - my);
    }
    if sxx < 1e-15 {
        return Err(PlotError::render(
            "linregress: x has near-zero variance; cannot fit a slope",
        ));
    }
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let mut sse = 0.0;
    for (&a, &b) in xs.iter().zip(ys.iter()) {
        let pred = intercept + slope * a;
        let e = b - pred;
        sse += e * e;
    }
    let dof = (n - 2.0).max(1.0);
    let sigma = (sse / dof).sqrt();
    Ok(OlsFit {
        intercept,
        slope,
        sigma,
        n,
        x_mean: mx,
        sxx,
    })
}

/// Correlation heatmap figure (seaborn `heatmap` of a corr matrix).
///
/// Uses [`Norm::TwoSlope`] centered at 0, Coolwarm, and annotates each cell.
pub fn corr_heatmap(labels: &[&str], columns: &[&[f64]]) -> Result<Figure> {
    if labels.len() != columns.len() {
        return Err(PlotError::length_mismatch(labels.len(), columns.len()));
    }
    let n = columns.len();
    let mat = corrcoef(columns)?;
    let label_owned: Vec<String> = labels.iter().map(|s| (*s).to_string()).collect();
    let mat_draw = mat.clone();
    Ok(Figure::new().theme(Theme::light()).axes(move |ax| {
        ax.heatmap(n, n, &mat_draw)
            .cmap(Colormap::Coolwarm)
            .vmin(-1.0)
            .vmax(1.0)
            .norm(Norm::TwoSlope { vcenter: 0.0 })
            .colorbar(true);
        for i in 0..n {
            for j in 0..n {
                let v = mat_draw[i * n + j];
                if v.is_finite() {
                    ax.text(j as f64, i as f64, format!("{v:.2}"))
                        .ha(TextAlign::Center)
                        .va(TextBaseline::Middle)
                        .color(Color::rgb(0x20, 0x20, 0x20));
                }
            }
        }
        ax.title("Correlation");
        if !label_owned.is_empty() {
            ax.x_categories(label_owned.clone());
            ax.y_categories(label_owned);
        }
    }))
}

/// Pairwise scatter grid with histograms on the diagonal (seaborn `pairplot` subset).
pub fn pair_scatter(labels: &[&str], columns: &[&[f64]]) -> Result<Figure> {
    if labels.len() != columns.len() {
        return Err(PlotError::length_mismatch(labels.len(), columns.len()));
    }
    let n = columns.len();
    if n == 0 {
        return Err(PlotError::render("pair_scatter needs at least one column"));
    }
    let len = columns[0].len();
    for c in columns {
        if c.len() != len {
            return Err(PlotError::length_mismatch(len, c.len()));
        }
    }
    let labels_owned: Vec<String> = labels.iter().map(|s| (*s).to_string()).collect();
    let cols: Vec<Vec<f64>> = columns.iter().map(|c| c.to_vec()).collect();
    Ok(Figure::new()
        .theme(Theme::light())
        .subplots(n, n, move |g| {
            g.hspace(0.35).wspace(0.35);
            for i in 0..n {
                for j in 0..n {
                    let yi = cols[i].clone();
                    let xj = cols[j].clone();
                    let title = if i == 0 {
                        labels_owned.get(j).cloned().unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let ylabel = if j == 0 {
                        labels_owned.get(i).cloned().unwrap_or_default()
                    } else {
                        String::new()
                    };
                    g.at(i, j, move |ax| {
                        if i == j {
                            ax.hist(&yi).bins(12);
                        } else {
                            ax.scatter(&xj, &yi).size(2.5);
                        }
                        if !title.is_empty() {
                            ax.title(title);
                        }
                        if !ylabel.is_empty() {
                            ax.y_label(ylabel);
                        }
                    });
                }
            }
        }))
}

/// Scatter + OLS regression line on an axes (seaborn `regplot` subset).
///
/// Draws a 95%-ish confidence band (`±1.96 · SE`) around the fit via `fill_between`.
pub fn regline(ax: &mut Axes, x: &[f64], y: &[f64]) -> Result<()> {
    let fit = ols_fit(x, y)?;
    ax.scatter(x, y).size(3.0).label("data");
    let (xmin, xmax) = x
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(None, |acc, v| match acc {
            None => Some((v, v)),
            Some((a, b)) => Some((a.min(v), b.max(v))),
        })
        .unwrap_or((0.0, 1.0));
    const N: usize = 64;
    let mut xs = Vec::with_capacity(N);
    let mut yhat = Vec::with_capacity(N);
    let mut ylo = Vec::with_capacity(N);
    let mut yhi = Vec::with_capacity(N);
    for i in 0..N {
        let t = i as f64 / (N - 1) as f64;
        let xv = xmin + t * (xmax - xmin);
        let yh = fit.intercept + fit.slope * xv;
        // Mean-response SE: σ √(1/n + (x-x̄)² / Sxx)
        let se = fit.sigma * (1.0 / fit.n + (xv - fit.x_mean).powi(2) / fit.sxx.max(1e-15)).sqrt();
        let half = 1.96 * se;
        xs.push(xv);
        yhat.push(yh);
        ylo.push(yh - half);
        yhi.push(yh + half);
    }
    ax.fill_between(&xs, &ylo, &yhi)
        .color(Color::CRIMSON)
        .alpha(0.2)
        .label("95% CI");
    ax.line(&xs, &yhat)
        .width(2.0)
        .color(Color::CRIMSON)
        .label(format!("y={:.3}+{:.3}x", fit.intercept, fit.slope));
    ax.legend(Legend::TopLeft);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corr_identity_diagonal() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [2.0, 4.0, 6.0, 8.0];
        let m = corrcoef(&[&a, &b]).unwrap();
        assert!((m[0] - 1.0).abs() < 1e-9);
        assert!((m[3] - 1.0).abs() < 1e-9);
        assert!((m[1] - 1.0).abs() < 1e-9); // perfect correlation
    }

    #[test]
    fn linregress_known_line() {
        let x = [0.0, 1.0, 2.0, 3.0];
        let y = [1.0, 3.0, 5.0, 7.0]; // y = 1 + 2x
        let (a, b) = linregress(&x, &y).unwrap();
        assert!((a - 1.0).abs() < 1e-9);
        assert!((b - 2.0).abs() < 1e-9);
    }
}
