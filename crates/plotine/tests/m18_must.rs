//! Must-have catch-up: TwoSlope norm, SegmentedColormap, annotated corr / regline CI.

use plotine::prelude::*;
use plotine::stats::{corr_heatmap, regline};

#[test]
fn two_slope_heatmap_renders() {
    let z = [-2.0, -1.0, 0.0, 1.0, 2.0, 3.0, -0.5, 0.5, 1.5];
    let png = Figure::new()
        .size(3.0, 2.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.heatmap(3, 3, z)
                .cmap(Colormap::Coolwarm)
                .vmin(-2.0)
                .vmax(3.0)
                .norm(Norm::TwoSlope { vcenter: 0.0 })
                .colorbar(true);
        })
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}

#[test]
fn segmented_cmap_on_heatmap() {
    let cmap = SegmentedColormap::from_colors([Color::STEEL_BLUE, Color::WHITE, Color::CRIMSON])
        .expect("stops");
    let z = [0.0, 0.5, 1.0, 0.25, 0.75, 0.1, 0.9, 0.4, 0.6];
    let png = Figure::new()
        .size(3.0, 2.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.heatmap(3, 3, z).cmap(cmap).colorbar(true);
        })
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}

#[test]
fn corr_heatmap_annotated_renders() {
    let a = [1.0, 2.0, 3.0, 4.0, 5.0];
    let b = [2.0, 1.0, 4.0, 3.0, 6.0];
    let c = [5.0, 4.0, 3.0, 2.0, 1.0];
    let png = corr_heatmap(&["a", "b", "c"], &[&a, &b, &c])
        .unwrap()
        .size(4.0, 3.5)
        .dpi(72.0)
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}

#[test]
fn regline_with_ci_renders() {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [0.9, 2.1, 2.8, 4.2, 5.1, 5.9];
    let png = Figure::new()
        .size(3.5, 2.8)
        .dpi(72.0)
        .axes(|ax| {
            regline(ax, &x, &y).unwrap();
        })
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}
