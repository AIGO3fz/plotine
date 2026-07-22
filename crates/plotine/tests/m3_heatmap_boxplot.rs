use plotine::prelude::*;

#[test]
fn heatmap_renders_with_colorbar() {
    let values: Vec<f64> = (0..20).map(|i| i as f64 * 0.1).collect();
    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.heatmap(4, 5, &values)
                .cmap(Colormap::Plasma)
                .colorbar(true);
            ax.title("hm");
        })
        .render_png()
        .expect("heatmap");
    assert!(png.len() > 200);
}

#[test]
fn heatmap_size_mismatch_errors() {
    let err = Figure::new()
        .axes(|ax| {
            ax.heatmap(2, 2, [1.0, 2.0, 3.0]);
        })
        .render_png()
        .unwrap_err();
    assert!(err.to_string().contains("heatmap size mismatch"));
    assert!(err.to_string().contains("suggestion"));
}

#[test]
fn heatmap_extent_and_alpha_render() {
    let values = [0.0, 0.5, 1.0, 0.25];
    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.heatmap(2, 2, values)
                .extent([0.0, 10.0, 0.0, 4.0])
                .alpha(0.6)
                .colorbar(false);
            ax.x_range(0.0, 10.0).y_range(0.0, 4.0);
        })
        .render_png()
        .expect("heatmap extent/alpha");
    assert!(png.len() > 200);
}

#[test]
fn boxplot_renders() {
    let a = [1.0, 2.0, 3.0, 4.0, 5.0];
    let b = [2.0, 2.5, 3.0, 3.5, 8.0];
    Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.boxplot([&a[..], &b[..]]).color(Color::CRIMSON);
            ax.title("box");
        })
        .render_png()
        .expect("boxplot");
}

#[test]
fn colormap_endpoints() {
    let a = Colormap::Viridis.sample(0.0);
    let b = Colormap::Viridis.sample(1.0);
    assert_ne!(a, b);
}
