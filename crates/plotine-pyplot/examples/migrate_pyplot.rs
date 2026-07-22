//! Migrate from matplotlib.pyplot-style calls to plotine via `plotine-pyplot`.
//!
//! ```bash
//! cargo run -p plotine-pyplot --example migrate_pyplot
//! ```
//!
//! Prefer the builder API for new code:
//! `Figure::new().axes(|ax| { … }).save(...)`.

use plotine_pyplot as plt;
use std::f64::consts::PI;
use std::path::PathBuf;

fn main() -> plotine::Result<()> {
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/pyplot_migrate");
    std::fs::create_dir_all(&out).ok();

    // --- Single axes (classic pyplot script) ---
    plt::clf();
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| (v * PI / 5.0).sin()).collect();
    plt::plot(&x, &y)?;
    plt::xlabel("time")?;
    plt::ylabel("amplitude")?;
    plt::title("plotine-pyplot migrate")?;
    plt::grid(true)?;
    plt::legend()?; // no labels yet — harmless empty/best placement
    plt::savefig(out.join("01_line.png"))?;

    // --- Subplots ---
    plt::clf();
    plt::subplots(2, 1)?;
    plt::sca(0)?;
    plt::plot(&x, &y)?;
    plt::title("top")?;
    plt::ylabel("sin")?;
    plt::sca(1)?;
    let y2: Vec<f64> = x.iter().map(|v| (v * PI / 5.0).cos()).collect();
    plt::scatter(&x, &y2)?;
    plt::title("bottom")?;
    plt::xlabel("x")?;
    plt::ylabel("cos")?;
    plt::savefig(out.join("02_subplots.png"))?;

    // --- Escape hatch: touch the underlying Figure ---
    plt::clf();
    plt::plot(&x, &y)?;
    plt::with_figure_mut(|fig| {
        if let Some(ax) = fig.axes_at_mut(0) {
            ax.line(&x, &y2)
                .color(plotine::Color::STEEL_BLUE)
                .width(1.5)
                .label("cos");
            ax.legend(plotine::Legend::TopRight);
        }
        Ok(())
    })?;
    plt::savefig(out.join("03_escape.png"))?;

    println!("wrote PNGs under {}", out.display());
    println!("(primary API remains plotine::Figure builder — see AGENTS.md)");
    Ok(())
}
