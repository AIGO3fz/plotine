//! Documented evcxr usage (requires `--features evcxr`).
//!
//! In a Jupyter / evcxr notebook cell:
//!
//! ```ignore
//! :dep plotine = { version = "0.1", features = ["evcxr"] }
//! use plotine::prelude::*;
//!
//! let x: Vec<_> = (0..50).map(|i| i as f64 * 0.1).collect();
//! let y: Vec<_> = x.iter().map(|v| v.sin()).collect();
//! Figure::new()
//!     .axes(|ax| { ax.line(&x, &y).width(2.0); })
//!     .evcxr_display()?;
//! ```

use plotine::prelude::*;

fn main() -> Result<()> {
    let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.15).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let fig = Figure::new().size(5.0, 3.0).axes(|ax| {
        ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
        ax.title("evcxr demo");
    });

    // Smoke-test the MIME printer without a notebook.
    fig.evcxr_display()?;
    Ok(())
}
