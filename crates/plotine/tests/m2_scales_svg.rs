use plotine::prelude::*;

#[test]
fn log_scale_rejects_non_positive_domain() {
    let err = Figure::new()
        .axes(|ax| {
            ax.line([1.0, 2.0], [-1.0, 3.0]);
            ax.y_scale(ScaleType::Log);
        })
        .render_png()
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("log scale") || msg.contains("non-positive"),
        "{msg}"
    );
    assert!(msg.contains("suggestion"), "{msg}");
}

#[test]
fn loglog_renders() {
    let x: Vec<f64> = (0..40)
        .map(|i| 10f64.powf(-1.0 + i as f64 * 0.08))
        .collect();
    let y: Vec<f64> = x.iter().map(|v| v * v).collect();
    Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            // Set log scales before artists so auto-limits use multiplicative padding.
            ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
            ax.line(&x, &y).width(2.0);
        })
        .render_png()
        .expect("loglog png");
}

#[test]
fn svg_is_deterministic() {
    let fig = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
        ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5])
            .color(Color::CRIMSON)
            .width(2.0);
        ax.title("SVG");
    });
    let a = fig.render_svg().expect("svg a");
    let b = fig.render_svg().expect("svg b");
    assert_eq!(a, b);
    assert!(a.starts_with("<?xml"));
    assert!(a.contains("<svg"));
    assert!(a.contains("</svg>"));
}

#[test]
fn themes_construct() {
    let _ = Theme::light();
    let _ = Theme::dark();
    let _ = Theme::paper();
}
