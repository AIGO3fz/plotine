use plotine::prelude::*;

#[test]
fn renders_empty_axes_png_bytes() {
    let (w, h, rgba) = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.title("Snapshot")
                .x_label("x")
                .y_label("y")
                .x_range(0.0, 1.0)
                .y_range(0.0, 1.0);
        })
        .render_rgba()
        .expect("render");

    assert_eq!(w, 288);
    assert_eq!(h, 216);
    assert_eq!(rgba.len(), (w * h * 4) as usize);
    // Background should be near-white in the corner.
    assert!(rgba[0] > 240 && rgba[1] > 240 && rgba[2] > 240);
}

#[test]
fn empty_figure_errors_with_suggestion() {
    let err = Figure::new().render_rgba().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("suggestion"), "{msg}");
}
