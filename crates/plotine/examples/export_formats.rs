//! Export the same figure to PNG / SVG / PDF / PGF (+ optional EPS).
//!
//! ```text
//! cargo run -p plotine --example export_formats
//! cargo run -p plotine --example export_formats --features eps
//! ```

use plotine::prelude::*;
use std::path::PathBuf;

fn main() -> plotine::Result<()> {
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/export_formats");
    std::fs::create_dir_all(&out).ok();

    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [0.0, 1.2, 0.4, 1.5, 0.8];
    let fig = Figure::new().size(4.0, 3.0).dpi(100.0).axes(|ax| {
        ax.line(x, y).color(Color::STEEL_BLUE).width(2.0);
        ax.title("export formats").x_label("x").y_label("y");
    });

    for ext in ["png", "svg", "pdf", "pgf"] {
        let path = out.join(format!("demo.{ext}"));
        fig.save(&path)?;
        println!("wrote {}", path.display());
    }

    #[cfg(feature = "eps")]
    {
        let path = out.join("demo.eps");
        match fig.save(&path) {
            Ok(()) => println!("wrote {}", path.display()),
            Err(e) => eprintln!("EPS skipped: {e}"),
        }
    }
    Ok(())
}
