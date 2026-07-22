//! M3 demo: grouped box-and-whisker plot.

use plotine::prelude::*;

fn main() -> Result<()> {
    let a = [1.0, 2.0, 2.5, 3.0, 3.2, 3.5, 4.0, 8.0];
    let b = [2.0, 2.2, 2.8, 3.1, 3.4, 3.8, 4.2, 4.5, 5.0];
    let c = [0.5, 1.0, 1.5, 2.0, 2.1, 2.4, 2.8, 3.0, 6.5];

    Figure::new()
        .size(6.5, 4.5)
        .axes(|ax| {
            ax.boxplot([&a[..], &b[..], &c[..]])
                .color(Color::STEEL_BLUE)
                .widths(0.55)
                .label("groups");
            ax.title("Boxplot")
                .x_label("group")
                .y_label("value")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save("boxplot_demo.png")?;

    println!("wrote boxplot_demo.png");
    Ok(())
}
