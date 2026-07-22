//! M2 demo: Unix-timestamp x-axis with date tick labels.

use plotine::prelude::*;

fn main() -> Result<()> {
    // Daily samples through January 2020 (UTC).
    let start = 1_577_836_800_f64; // 2020-01-01
    let x: Vec<f64> = (0..31).map(|i| start + i as f64 * 86_400.0).collect();
    let y: Vec<f64> = (0..31)
        .map(|i| 10.0 + (i as f64 * 0.35).sin() * 2.0 + i as f64 * 0.05)
        .collect();

    Figure::new()
        .size(8.0, 4.5)
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("metric");
            ax.x_datetime(true)
                .title("Datetime X Axis")
                .x_label("date (UTC)")
                .y_label("value")
                .legend(Legend::TopLeft)
                .grid(true);
        })
        .save("datetime_axis.png")?;

    println!("wrote datetime_axis.png");
    Ok(())
}
