//! Optional matplotlib.`pyplot`-style facade over [`plotine`].
//!
//! **This is not the primary plotine API.** Prefer
//! `Figure::new().axes(|ax| { … }).save(...)` for new code and agent codegen.
//! See the repository `AGENTS.md` and [`plotine`](https://docs.rs/plotine).
//!
//! Global figure/axes state lives **only** in this crate (thread-local), matching
//! a thin subset of `matplotlib.pyplot`.

#![warn(missing_docs)]

mod state;

pub use state::{clf, figure, gca_index, gcf_num, sca};

use plotine::prelude::{IntoSeries, Legend};
use plotine::{Figure, PlotError, Result};

/// Plot `y` versus `x` on the current axes (matplotlib `plt.plot` → [`Axes::line`](plotine::Axes::line)).
pub fn plot(x: impl IntoSeries, y: impl IntoSeries) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.line(x, y);
        Ok(())
    })
}

/// Scatter markers on the current axes (matplotlib `plt.scatter`).
pub fn scatter(x: impl IntoSeries, y: impl IntoSeries) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.scatter(x, y);
        Ok(())
    })
}

/// Create an `nrows × ncols` subplot grid on the current figure.
///
/// Selects axes index `0` afterward. Use [`sca`] to switch panels (0-based).
pub fn subplots(nrows: usize, ncols: usize) -> Result<()> {
    if nrows == 0 || ncols == 0 {
        return Err(PlotError::render("subplots nrows and ncols must be >= 1"));
    }
    let fig = Figure::new().subplots(nrows, ncols, |g| {
        for r in 0..nrows {
            for c in 0..ncols {
                g.at(r, c, |_| {});
            }
        }
    });
    state::set_current_figure(fig);
    Ok(())
}

/// Set the x-axis label (matplotlib `plt.xlabel`).
pub fn xlabel(label: impl Into<String>) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.x_label(label);
        Ok(())
    })
}

/// Set the y-axis label (matplotlib `plt.ylabel`).
pub fn ylabel(label: impl Into<String>) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.y_label(label);
        Ok(())
    })
}

/// Set the axes title (matplotlib `plt.title`).
pub fn title(text: impl Into<String>) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.title(text);
        Ok(())
    })
}

/// Show a legend using [`Legend::Best`] (matplotlib `plt.legend()`).
pub fn legend() -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.legend(Legend::Best);
        Ok(())
    })
}

/// Show a legend at an explicit location.
pub fn legend_loc(loc: Legend) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.legend(loc);
        Ok(())
    })
}

/// Toggle the major grid (matplotlib `plt.grid`).
pub fn grid(on: bool) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.grid(on);
        Ok(())
    })
}

/// Set x-axis limits (matplotlib `plt.xlim`).
pub fn xlim(min: f64, max: f64) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.x_range(min, max);
        Ok(())
    })
}

/// Set y-axis limits (matplotlib `plt.ylim`).
pub fn ylim(min: f64, max: f64) -> Result<()> {
    state::with_gca_mut(|ax| {
        ax.y_range(min, max);
        Ok(())
    })
}

/// Save the current figure (matplotlib `plt.savefig`).
///
/// Format is inferred from the path extension (`.png` / `.svg` / `.pdf`).
pub fn savefig(path: impl AsRef<std::path::Path>) -> Result<()> {
    state::with_gcf(|fig| fig.save(path))
}

/// Open an interactive window for the current figure (`feature = "gui"`).
///
/// Consumes the current figure into [`Figure::show`](plotine::Figure::show) and
/// replaces it with an empty figure afterward.
#[cfg(feature = "gui")]
pub fn show() -> Result<()> {
    let fig = state::take_gcf()?;
    fig.show()
}

/// Run `f` with a mutable borrow of the current [`Figure`].
///
/// Escape hatch for builder APIs not wrapped by this facade.
pub fn with_figure_mut<R>(f: impl FnOnce(&mut Figure) -> Result<R>) -> Result<R> {
    state::with_gcf_mut(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn plot_savefig_roundtrip() {
        clf();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.png");
        plot([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]).unwrap();
        xlabel("x").unwrap();
        ylabel("y").unwrap();
        title("test").unwrap();
        grid(true).unwrap();
        savefig(&path).unwrap();
        assert!(path.is_file());
        assert!(fs::metadata(&path).unwrap().len() > 32);
    }

    #[test]
    fn subplots_and_sca() {
        clf();
        subplots(2, 1).unwrap();
        assert_eq!(gca_index(), 0);
        plot([0.0, 1.0], [0.0, 1.0]).unwrap();
        sca(1).unwrap();
        assert_eq!(gca_index(), 1);
        scatter([0.0, 1.0], [1.0, 0.0]).unwrap();
        let dir = tempfile::tempdir().unwrap();
        savefig(dir.path().join("sub.png")).unwrap();
    }

    #[test]
    fn clf_resets_artists() {
        clf();
        plot([0.0, 1.0], [0.0, 1.0]).unwrap();
        clf();
        // Empty axes still render.
        let dir = tempfile::tempdir().unwrap();
        savefig(dir.path().join("empty.png")).unwrap();
    }

    #[test]
    fn figure_num_switches() {
        clf();
        let a = figure(None);
        plot([0.0, 1.0], [0.0, 1.0]).unwrap();
        let b = figure(None);
        assert_ne!(a, b);
        figure(Some(a));
        assert_eq!(gcf_num(), a);
    }

    /// TLS state must not cross threads (each thread gets its own gcf).
    #[test]
    fn thread_local_state_is_isolated() {
        std::thread::scope(|s| {
            let t1 = s.spawn(|| {
                clf();
                plot([0.0, 1.0], [0.0, 1.0]).unwrap();
                title("thread-a").unwrap();
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("a.png");
                savefig(&path).unwrap();
                assert!(path.is_file());
                assert!(fs::metadata(&path).unwrap().len() > 32);
                gcf_num()
            });
            let t2 = s.spawn(|| {
                clf();
                plot([0.0, 1.0], [1.0, 0.0]).unwrap();
                title("thread-b").unwrap();
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("b.png");
                savefig(&path).unwrap();
                assert!(path.is_file());
                assert!(fs::metadata(&path).unwrap().len() > 32);
                gcf_num()
            });
            // Both threads succeed independently; figure numbers are per-TLS and
            // may collide numerically — that is fine as long as neither panics.
            t1.join().expect("thread-a");
            t2.join().expect("thread-b");
        });
    }
}
