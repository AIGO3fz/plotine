//! Emit a pyplot-facade line chart into `compare/size_bench` for size parity vs builder API.
//!
//! ```bash
//! cargo run -p plotine-pyplot --example pyplot_size_benchmark
//! ```
//!
//! Named differently from `plotine`'s `size_benchmark` so example binaries do not
//! collide under `target/debug/examples/`.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use plotine::prelude::*;
use plotine_pyplot as plt;

fn main() -> plotine::Result<()> {
    let dir = PathBuf::from("compare/size_bench");
    fs::create_dir_all(&dir).map_err(|e| PlotError::io(e.to_string()))?;
    let path = dir.join("pyplot_line_150.png");

    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    let t0 = Instant::now();
    plt::clf();
    // Match builder size_benchmark line: 5×3.5 in @ 150 DPI, same artists.
    plt::with_figure_mut(|fig| {
        *fig = Figure::new().size(5.0, 3.5).dpi(150.0).axes(|ax| {
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin(x)");
            ax.title("Line")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight);
        });
        Ok(())
    })?;
    plt::savefig(&path)?;
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    let bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    println!(
        "TIMING name=pyplot_line_150 ms={ms:.3} bytes={bytes} path={}",
        path.display()
    );
    Ok(())
}
