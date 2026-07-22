//! M3 demo: ndarray Array1 / Array2 → line + heatmap (requires `--features ndarray`).

use ndarray::{Array1, Array2};
use plotine::prelude::*;

fn main() -> Result<()> {
    let x = Array1::linspace(0.0, std::f64::consts::TAU, 120);
    let y = x.mapv(f64::sin);

    Figure::new()
        .size(6.0, 3.5)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
            ax.title("ndarray Line");
        })
        .save("ndarray_line.png")?;

    let mut z = Array2::<f64>::zeros((6, 8));
    for ((r, c), v) in z.indexed_iter_mut() {
        *v = (r as f64 * 0.6).sin() * (c as f64 * 0.5).cos();
    }
    Figure::new()
        .size(6.0, 4.0)
        .axes(|ax| {
            ax.heatmap_array(&z).cmap(Colormap::Plasma).colorbar(true);
            ax.title("ndarray Heatmap");
        })
        .save("ndarray_heatmap.png")?;

    println!("wrote ndarray_line.png and ndarray_heatmap.png");
    Ok(())
}
