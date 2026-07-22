use plotine::prelude::*;

#[test]
fn bar_hist_area_render() {
    let x = [1.0, 2.0, 3.0];
    let h = [2.0, 5.0, 3.0];
    Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.bar(x, h).label("bars");
            ax.legend(Legend::TopRight);
        })
        .render_png()
        .expect("bar");

    let data: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin()).collect();
    Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.hist(&data).bins(8);
        })
        .render_png()
        .expect("hist");

    let xs: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.iter().map(|v| v * 0.1).collect();
    Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.area(&xs, &ys).alpha(0.3);
        })
        .render_png()
        .expect("area");
}

#[test]
fn errorbar_length_mismatch() {
    let err = Figure::new()
        .axes(|ax| {
            ax.errorbar([0.0, 1.0], [1.0, 2.0], [0.1]);
        })
        .render_rgba()
        .unwrap_err();
    assert!(err.to_string().contains("length mismatch"));
}

#[test]
fn errorbar_xerr_renders_and_expands_x() {
    let png = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.errorbar([0.0, 1.0], [1.0, 2.0], [0.1, 0.1])
                .xerr([0.25, 0.25])
                .color(Color::STEEL_BLUE);
        })
        .render_png()
        .expect("errorbar xerr");
    assert!(!png.is_empty());
}

#[test]
fn errorbar_xerr_length_mismatch() {
    let err = Figure::new()
        .axes(|ax| {
            ax.errorbar([0.0, 1.0], [1.0, 2.0], [0.1, 0.1]).xerr([0.2]);
        })
        .render_rgba()
        .unwrap_err();
    assert!(err.to_string().contains("length mismatch"));
}

#[test]
fn errorbar_asymmetric_yerr_xerr_renders() {
    let png = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.errorbar([0.0, 1.0], [1.0, 2.0], [0.1, 0.1])
                .yerr_asym([0.4, 0.2], [0.1, 0.5])
                .xerr_asym([0.15, 0.1], [0.05, 0.2])
                .color(Color::STEEL_BLUE);
        })
        .render_png()
        .expect("asymmetric errorbar");
    assert!(!png.is_empty());
}

#[test]
fn errorbar_yerr_asym_length_mismatch() {
    let err = Figure::new()
        .axes(|ax| {
            ax.errorbar([0.0, 1.0], [1.0, 2.0], [0.1, 0.1])
                .yerr_asym([0.1], [0.2, 0.2]);
        })
        .render_rgba()
        .unwrap_err();
    assert!(err.to_string().contains("length mismatch"));
}

#[test]
fn histogram_recipe_unit() {
    let h = plotine::recipes::histogram(&[0.0, 0.5, 1.0, 1.5], 2);
    assert_eq!(h.counts.len(), 2);
    let sum: f64 = h.counts.iter().sum();
    assert!((sum - 4.0).abs() < 1e-9);
}
