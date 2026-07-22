use plotine::prelude::*;

#[test]
fn inset_axes_renders_zoom_window() {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let png = Figure::new()
        .size(5.0, 3.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
            ax.title("Host");
            ax.inset_axes([0.55, 0.55, 0.4, 0.4], |inset| {
                inset
                    .line(&x[..15], &y[..15])
                    .color(Color::CRIMSON)
                    .width(1.5);
                inset.title("zoom");
            });
        })
        .render_png()
        .expect("inset png");
    assert!(!png.is_empty());
}

#[test]
fn inset_axes_rejects_colorbar() {
    let z = [1.0, 2.0, 3.0, 4.0];
    let err = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0], [0.0, 1.0]);
            ax.inset_axes([0.5, 0.5, 0.4, 0.4], |inset| {
                inset.heatmap(2, 2, z).colorbar(true);
            });
        })
        .render_png()
        .expect_err("colorbar on inset");
    assert!(
        err.to_string().contains("colorbar") || format!("{err:?}").contains("colorbar"),
        "{err}"
    );
}

#[test]
fn nested_inset_axes_renders() {
    let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.25).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let png = Figure::new()
        .size(5.0, 3.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::STEEL_BLUE);
            ax.inset_axes([0.5, 0.5, 0.45, 0.45], |outer| {
                outer.line(&x[..20], &y[..20]).color(Color::CRIMSON);
                outer.inset_axes([0.55, 0.55, 0.4, 0.4], |inner| {
                    inner.line(&x[..8], &y[..8]).color(Color::FOREST_GREEN);
                });
            });
        })
        .render_png()
        .expect("nested inset");
    assert!(!png.is_empty());
}

#[test]
fn annotate_arrow_styles_render() {
    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
            ax.annotate("tri", (1.0, 1.0), (0.2, 0.2))
                .arrow_style(ArrowStyle::Triangle);
            ax.annotate("simple", (1.5, 0.7), (1.8, 0.2))
                .arrow_style(ArrowStyle::Simple);
            ax.annotate("bracket", (0.5, 0.5), (0.1, 0.8))
                .arrow_style(ArrowStyle::Bracket);
            ax.annotate("both", (1.2, 0.4), (1.6, 0.9))
                .arrow_style(ArrowStyle::BothEnds);
        })
        .render_png()
        .expect("annotate styles");
    assert!(!png.is_empty());
}
