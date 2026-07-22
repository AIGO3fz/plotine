//! M1 demo: error bars + multi-series legend.

use plotine::prelude::*;

fn main() -> Result<()> {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [1.2, 1.8, 1.5, 2.4, 2.1, 2.8];
    let yerr = [0.2, 0.25, 0.15, 0.3, 0.22, 0.28];
    let trend: Vec<f64> = x.iter().map(|v| 1.1 + 0.3 * v).collect();

    Figure::new()
        .axes(|ax| {
            ax.line(x, &trend)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("trend");
            ax.errorbar(x, y, yerr)
                .color(Color::STEEL_BLUE)
                .capsize(5.0)
                .label("measured");
            ax.title("Error Bars")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopLeft)
                .grid(true);
        })
        .save("errorbar_demo.png")?;

    println!("wrote errorbar_demo.png");
    Ok(())
}
