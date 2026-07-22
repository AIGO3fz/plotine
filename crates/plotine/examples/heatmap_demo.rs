//! M3 demo: heatmap with Viridis colormap and colorbar.

use plotine::prelude::*;

fn main() -> Result<()> {
    let nrows = 8usize;
    let ncols = 10usize;
    let mut values = Vec::with_capacity(nrows * ncols);
    for r in 0..nrows {
        for c in 0..ncols {
            let x = c as f64 / (ncols - 1) as f64;
            let y = r as f64 / (nrows - 1) as f64;
            values.push((x * 3.0).sin() * (y * 2.5).cos() + 0.15 * y);
        }
    }

    Figure::new()
        .size(7.0, 5.0)
        .axes(|ax| {
            ax.heatmap(nrows, ncols, &values)
                .cmap(Colormap::Viridis)
                .colorbar(true);
            ax.title("Heatmap").x_label("column").y_label("row");
        })
        .save("heatmap_demo.png")?;

    println!("wrote heatmap_demo.png");
    Ok(())
}
