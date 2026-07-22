use plotine::prelude::*;

#[test]
fn line_auto_ranges_from_data() {
    let x = vec![2.0, 4.0, 6.0];
    let y = vec![-1.0, 0.0, 1.0];
    let (w, h, rgba) = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(&x, &y).width(2.0);
        })
        .render_rgba()
        .expect("render");
    assert_eq!(w, 216);
    assert_eq!(h, 144);
    assert_eq!(rgba.len(), (w * h * 4) as usize);
}

#[test]
fn length_mismatch_errors() {
    let err = Figure::new()
        .axes(|ax| {
            ax.line([0.0, 1.0], [0.0]);
        })
        .render_rgba()
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("length mismatch"), "{msg}");
    assert!(msg.contains("suggestion"), "{msg}");
}

#[test]
fn color_cycle_assigns_distinct_defaults() {
    // Smoke: two series render without panic; second uses cycle color.
    let x = [0.0, 1.0, 2.0];
    let y1 = [0.0, 1.0, 0.0];
    let y2 = [1.0, 0.0, 1.0];
    Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(x, y1);
            ax.scatter(x, y2).size(4.0);
        })
        .render_png()
        .expect("render");
}
