//! M1 demo: scatter + line overlay with automatic color cycle.

use plotine::prelude::*;

fn main() -> Result<()> {
    let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.25).collect();
    let trend: Vec<f64> = x.iter().map(|v| 0.15 * v + 0.2).collect();
    let noise: Vec<f64> = x
        .iter()
        .enumerate()
        .map(|(i, v)| 0.15 * v + 0.2 + ((i as f64 * 1.7).sin() * 0.35))
        .collect();

    Figure::new()
        .axes(|ax| {
            ax.scatter(&x, &noise).size(6.0).label("samples");
            ax.line(&x, &trend).width(2.0).label("trend");
            ax.title("Scatter + Line")
                .x_label("x")
                .y_label("y")
                .grid(true);
        })
        .save("scatter_demo.png")?;

    println!("wrote scatter_demo.png");
    Ok(())
}
