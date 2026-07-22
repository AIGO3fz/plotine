//! M3 demo: three-line Polars DataFrame → PNG (requires `--features polars`).

use plotine::prelude::*;
use polars::prelude::*;

fn main() -> Result<()> {
    let df = df! {
        "x" => (0..80).map(|i| i as f64 * 0.1).collect::<Vec<_>>(),
        "y" => (0..80).map(|i| (i as f64 * 0.1).sin()).collect::<Vec<_>>(),
    }
    .expect("df");

    let (x, y) = plotine::polars::xy(&df, "x", "y")?;
    Figure::new()
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin");
            ax.title("Polars DataFrame").legend(Legend::TopRight);
        })
        .save("polars_line.png")?;

    println!("wrote polars_line.png");
    Ok(())
}
