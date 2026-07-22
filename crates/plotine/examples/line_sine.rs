//! M1 demo: styled sine line with publication defaults.

use plotine::prelude::*;

fn main() -> Result<()> {
    let x: Vec<f64> = (0..200).map(|i| i as f64 * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    Figure::new()
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin(x)");
            ax.title("Sine Wave")
                .x_label("time (s)")
                .y_label("amplitude");
        })
        .save("line_sine.png")?;

    println!("wrote line_sine.png");
    Ok(())
}
