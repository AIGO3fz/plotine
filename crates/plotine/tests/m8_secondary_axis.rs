use plotine::prelude::*;
use std::f64::consts::PI;

#[test]
fn secondary_x_degrees_renders() {
    let th = [0.0, PI / 2.0, PI];
    let y = [0.0, 1.0, 0.0];
    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(th, y).color(Color::STEEL_BLUE).width(2.0);
            ax.x_label("radians").y_label("y");
            ax.secondary_x(f64::to_degrees, f64::to_radians, |sec| {
                sec.label("degrees");
            });
            ax.title("Secondary X");
        })
        .render_png()
        .expect("secondary_x png");
    assert!(!png.is_empty());
}

#[test]
fn secondary_y_fahrenheit_renders() {
    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 50.0, 100.0], [0.0, 50.0, 100.0])
                .color(Color::CRIMSON)
                .width(2.0);
            ax.y_label("°C").x_label("sample");
            ax.secondary_y_linear(1.8, 32.0, |sec| {
                sec.label("°F");
            });
            ax.title("Secondary Y");
        })
        .render_png()
        .expect("secondary_y png");
    assert!(!png.is_empty());
}

#[test]
fn secondary_y_conflicts_with_twin_y() {
    let err = Figure::new()
        .size(3.0, 2.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0], [0.0, 1.0]);
            ax.twin_y(|ax2| {
                ax2.line([0.0, 1.0], [10.0, 20.0]);
            });
            ax.secondary_y_linear(2.0, 0.0, |_| {});
        })
        .render_png()
        .expect_err("secondary_y + twin_y");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("secondary_y") || msg.contains("twin_y"),
        "{msg}"
    );
}
