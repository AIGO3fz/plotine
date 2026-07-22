//! M0 acceptance demo: publication-quality empty axes with ticks, labels, and title.

use plotine::prelude::*;

fn main() -> Result<()> {
    Figure::new()
        .size(7.0, 5.0)
        .dpi(120.0)
        .axes(|ax| {
            ax.title("Empty Axes (M0 Acceptance)")
                .x_label("time (s)")
                .y_label("amplitude")
                .x_range(0.0, 10.0)
                .y_range(-1.0, 1.0)
                .grid(true);
        })
        .save("empty_axes.png")?;

    println!("wrote empty_axes.png");
    Ok(())
}
