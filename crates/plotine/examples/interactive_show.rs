//! Interactive GUI demo (`feature = "gui"`).
//!
//! ```text
//! cargo run -p plotine --example interactive_show --features gui
//! ```
//!
//! Controls (matplotlib NavigationToolbar2 subset):
//! - Pan mode (p): left-drag pan; scroll wheel zoom
//! - Zoom mode (o): left-drag box zoom
//! - 3D: left-drag rotate elev/azim; scroll zoom data box
//! - Home (h/r), Back (←), Forward (→), Save (s), Quit (q)

use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    let x: Vec<f64> = (0..200).map(|i| i as f64 * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    // Window 1 semantics: build then show (blocking). Subplots + log demo.
    println!("Opening 2D interactive window (close it to continue to 3D)…");
    Figure::new()
        .size(6.4, 4.8)
        .dpi(120.0)
        .subplots(1, 2, |g| {
            g.at(0, 0, |ax| {
                ax.line(&x, &y)
                    .color(Color::CRIMSON)
                    .width(2.0)
                    .label("sin");
                ax.title("Pan / zoom")
                    .x_label("x")
                    .y_label("y")
                    .legend(Legend::TopRight)
                    .grid(true);
            });
            g.at(0, 1, |ax| {
                let xp: Vec<f64> = (1..50).map(|i| i as f64).collect();
                let yp: Vec<f64> = xp.iter().map(|v| v * v).collect();
                ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
                ax.line(&xp, &yp).color(Color::STEEL_BLUE).width(2.0);
                ax.title("Log–log").x_label("x").y_label("x²").grid(true);
            });
        })
        .show()?;

    println!("Opening 3D interactive window…");
    let t: Vec<f64> = (0..120).map(|i| i as f64 * 0.1).collect();
    let x3: Vec<f64> = t.iter().map(|v| v.cos()).collect();
    let y3: Vec<f64> = t.iter().map(|v| v.sin()).collect();
    let z3 = t.clone();
    Figure::new()
        .size(6.4, 4.8)
        .dpi(120.0)
        .axes3d(|ax| {
            ax.plot3d(&x3, &y3, &z3)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("helix");
            ax.title("Drag to rotate").elev(30.0).azim(-60.0);
            ax.legend(Legend::TopRight);
        })
        .show()?;

    Ok(())
}
