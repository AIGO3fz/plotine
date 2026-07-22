use plotine::prelude::*;

#[test]
fn violin_renders() {
    let a = [1.0, 2.0, 2.5, 3.0, 3.5, 4.0];
    let b = [2.0, 2.5, 3.0, 3.5, 4.5, 5.0];
    Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.violin([&a[..], &b[..]])
                .color(Color::STEEL_BLUE)
                .widths(0.8);
            ax.title("violin");
        })
        .render_png()
        .expect("violin");
}

#[cfg(feature = "evcxr")]
#[test]
fn evcxr_display_succeeds() {
    Figure::new()
        .size(2.0, 1.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0], [0.0, 1.0]);
        })
        .evcxr_display()
        .expect("evcxr");
}
