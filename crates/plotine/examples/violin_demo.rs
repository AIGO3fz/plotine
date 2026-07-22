//! M3 demo: grouped violin plot (Gaussian KDE).

use plotine::prelude::*;

fn main() -> Result<()> {
    let a = [1.0, 2.0, 2.5, 3.0, 3.2, 3.5, 4.0, 4.2, 3.8, 2.8];
    let b = [2.0, 2.2, 2.8, 3.1, 3.4, 3.8, 4.2, 4.5, 5.0, 3.6];
    let c = [0.5, 1.0, 1.5, 2.0, 2.1, 2.4, 2.8, 3.0, 2.2, 1.8];

    Figure::new()
        .size(6.5, 4.5)
        .axes(|ax| {
            ax.violin([&a[..], &b[..], &c[..]])
                .color(Color::MEDIUM_PURPLE)
                .widths(0.85)
                .label("kde");
            ax.title("Violin")
                .x_label("group")
                .y_label("value")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save("violin_demo.png")?;

    println!("wrote violin_demo.png");
    Ok(())
}
