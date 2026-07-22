//! M2 demo: 2×2 subplot grid without overlapping labels.

use plotine::prelude::*;

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    if n <= 1 {
        return vec![start];
    }
    let step = (end - start) / (n - 1) as f64;
    (0..n).map(|i| start + step * i as f64).collect()
}

fn main() -> Result<()> {
    let x = linspace(0.0, 10.0, 200);
    let sine: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let cosy: Vec<f64> = x.iter().map(|v| v.cos()).collect();

    Figure::new()
        .size(9.0, 7.0)
        .dpi(110.0)
        .subplots(2, 2, |g| {
            g.hspace(0.32).wspace(0.28);
            g.at(0, 0, |ax| {
                ax.line(&x, &sine)
                    .color(Color::CRIMSON)
                    .width(2.0)
                    .label("sin");
                ax.title("Line")
                    .x_label("x")
                    .y_label("y")
                    .legend(Legend::TopRight);
            });
            g.at(0, 1, |ax| {
                ax.scatter(&x, &cosy).size(3.5).color(Color::STEEL_BLUE);
                ax.title("Scatter").x_label("x").y_label("y");
            });
            g.at(1, 0, |ax| {
                ax.bar([1.0, 2.0, 3.0, 4.0], [3.0, 5.0, 2.0, 4.0])
                    .color(Color::FOREST_GREEN)
                    .label("n");
                ax.title("Bar").legend(Legend::TopLeft);
            });
            g.at(1, 1, |ax| {
                let data: Vec<f64> = (0..200)
                    .map(|i| ((i as f64) * 0.13).sin() + ((i as f64) * 0.04).cos())
                    .collect();
                ax.hist(&data)
                    .bins(14)
                    .color(Color::MEDIUM_PURPLE)
                    .label("hist");
                ax.title("Histogram").legend(Legend::TopRight);
            });
        })
        .save("subplots_2x2.png")?;

    println!("wrote subplots_2x2.png");
    Ok(())
}
