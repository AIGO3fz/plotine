//! M1 demo: bar, histogram, and area charts.

use plotine::prelude::*;

fn main() -> Result<()> {
    // --- bars ---
    let cats = [1.0, 2.0, 3.0, 4.0, 5.0];
    let heights = [3.0, 7.0, 2.0, 5.0, 4.0];
    Figure::new()
        .axes(|ax| {
            ax.bar(cats, heights)
                .color(Color::STEEL_BLUE)
                .label("counts");
            ax.title("Bar Chart")
                .x_label("category")
                .y_label("value")
                .legend(Legend::TopRight);
        })
        .save("bar_demo.png")?;

    // --- histogram ---
    let mut data = Vec::new();
    for i in 0..500 {
        let t = i as f64 * 0.02;
        data.push((t * 0.7).sin() * 1.5 + (i as f64 * 0.13).cos() * 0.4);
    }
    Figure::new()
        .axes(|ax| {
            ax.hist(&data)
                .bins(20)
                .color(Color::MEDIUM_PURPLE)
                .label("samples");
            ax.title("Histogram")
                .x_label("value")
                .y_label("count")
                .legend(Legend::TopRight);
        })
        .save("hist_demo.png")?;

    // --- area ---
    let x: Vec<f64> = (0..80).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| (v * 0.8).sin().abs() + 0.2).collect();
    Figure::new()
        .axes(|ax| {
            ax.area(&x, &y)
                .color(Color::FOREST_GREEN)
                .alpha(0.4)
                .label("|sin|");
            ax.title("Area Chart")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight);
        })
        .save("area_demo.png")?;

    println!("wrote bar_demo.png, hist_demo.png, area_demo.png");
    Ok(())
}
