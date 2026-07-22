//! M2 demo: log/symlog scales and built-in themes.

use plotine::prelude::*;

fn main() -> Result<()> {
    let x: Vec<f64> = (0..60)
        .map(|i| 10f64.powf(-1.0 + i as f64 * 0.05))
        .collect();
    let y: Vec<f64> = x.iter().map(|v| v.powf(1.5) * 2.0).collect();

    Figure::new()
        .theme(Theme::light())
        .axes(|ax| {
            ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("y = 2 x^1.5");
            ax.title("Log–Log Scale")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopLeft);
        })
        .save("loglog.png")?;

    let xs: Vec<f64> = (-40..=40).map(|i| i as f64 * 0.25).collect();
    let ys: Vec<f64> = xs
        .iter()
        .map(|v| {
            if v.abs() < 1e-9 {
                0.0
            } else {
                v.signum() * v.abs().powf(1.2)
            }
        })
        .collect();

    Figure::new()
        .theme(Theme::dark())
        .axes(|ax| {
            ax.y_scale(ScaleType::Symlog { linthresh: 1.0 });
            ax.line(&xs, &ys)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("signed power");
            ax.title("Dark Theme + Symlog Y")
                .x_label("x")
                .y_label("y")
                .legend(Legend::BottomRight);
        })
        .save("dark_symlog.png")?;

    Figure::new()
        .theme(Theme::paper())
        .axes(|ax| {
            ax.x_scale(ScaleType::Log);
            ax.line(&x, &y).color(Color::FOREST_GREEN).width(2.2);
            ax.title("Paper Theme").x_label("x (log)").y_label("y");
        })
        .save("paper_theme.png")?;

    // SVG export of the log-log figure
    Figure::new()
        .axes(|ax| {
            ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
            ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
            ax.title("SVG Log–Log").x_label("x").y_label("y");
        })
        .save("loglog.svg")?;

    println!("wrote loglog.png, dark_symlog.png, paper_theme.png, loglog.svg");
    Ok(())
}
