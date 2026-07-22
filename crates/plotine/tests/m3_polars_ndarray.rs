#![cfg(all(feature = "polars", feature = "ndarray"))]

use ndarray::{Array1, Array2};
use plotine::prelude::*;
use polars::prelude::*;

#[test]
fn polars_xy_three_line_plot() {
    let df = df! {
        "x" => &[0.0_f64, 1.0, 2.0, 3.0],
        "y" => &[0.0_f64, 1.0, 0.5, 1.5],
    }
    .unwrap();
    let (x, y) = plotine::polars::xy(&df, "x", "y").unwrap();
    Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(&x, &y).width(1.5);
        })
        .render_png()
        .expect("polars plot");
}

#[test]
fn polars_missing_column_suggests_fix() {
    let df = df! { "x" => &[1.0_f64] }.unwrap();
    let err = plotine::polars::column(&df, "missing").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("not found"));
    assert!(msg.contains("suggestion"));
}

#[test]
fn ndarray_line_and_heatmap() {
    let x = Array1::linspace(0.0, 1.0, 10);
    let y = &x * 2.0;
    Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(&x, &y).width(1.5);
        })
        .render_png()
        .expect("ndarray line");

    let z = Array2::from_shape_fn((3, 4), |(r, c)| (r + c) as f64);
    Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.heatmap_array(&z).colorbar(false);
        })
        .render_png()
        .expect("ndarray heatmap");
}
