use plotine::prelude::*;

#[test]
fn table_renders() {
    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.bar([1.0, 2.0, 3.0], [3.0, 5.0, 2.0])
                .color(Color::STEEL_BLUE);
            ax.table([["A", "3"], ["B", "5"], ["C", "2"]])
                .col_labels(["Item", "Value"])
                .loc(TableLoc::UpperRight)
                .fontsize(8.0);
            ax.title("Table");
        })
        .render_png()
        .expect("table");
    assert!(!png.is_empty());
}

#[test]
fn mathtext_frac_and_scripts_render() {
    let png = Figure::new()
        .size(5.0, 3.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0, 2.0], [0.2, 0.9, 0.4]);
            ax.title(r"Damped: $\alpha + \frac{1}{2}e^{-t}$")
                .x_label(r"$t$ (s)")
                .y_label(r"$\theta$");
            ax.text(0.2, 0.7, r"$x^{2}_{i}$");
        })
        .render_png()
        .expect("mathtext");
    assert!(!png.is_empty());
}

#[test]
fn mathtext_integral_limits_render() {
    // Display-style ∫ with limits above/below (mpl mathtext alignment).
    let png = Figure::new()
        .size(5.0, 3.5)
        .dpi(100.0)
        .axes(|ax| {
            ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
            ax.title(r"$\int_0^1 x^2\,dx$");
        })
        .render_png()
        .expect("integral limits");
    assert!(!png.is_empty());
}
