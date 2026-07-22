//! CJK labels via optional `cjk` feature (system / user font — not embedded).
//!
//! ```bash
//! cargo run -p plotine --example cjk_labels --features cjk
//! ```

use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    let families = plotine::fonts::load_system_cjk()?;
    println!("loaded CJK families: {families:?}");

    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [0.0, 1.2, 0.8, 1.5, 1.1];

    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("序列 A");
            ax.title("中文标题示例")
                .x_label("横轴")
                .y_label("纵轴")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save("cjk_labels.png")?;

    println!("wrote cjk_labels.png");
    Ok(())
}
