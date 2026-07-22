//! M16 ecosystem: stats helpers + GeoJSON.

use plotine::prelude::*;
use plotine::stats::{corr_heatmap, linregress, pair_scatter, regline};

#[test]
fn corr_heatmap_renders() {
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
fn pair_scatter_renders() {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [0.1, 1.2, 1.8, 3.1, 3.9, 5.2];
    let png = pair_scatter(&["x", "y"], &[&x, &y])
        .unwrap()
        .size(5.0, 5.0)
        .dpi(72.0)
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}

#[test]
fn regline_on_axes() {
    let x = [0.0, 1.0, 2.0, 3.0];
    let y = [1.0, 3.0, 5.0, 7.0];
    let (a, b) = linregress(&x, &y).unwrap();
    assert!((a - 1.0).abs() < 1e-9 && (b - 2.0).abs() < 1e-9);
    let png = Figure::new()
        .size(3.0, 2.5)
        .dpi(72.0)
        .axes(|ax| {
            regline(ax, &x, &y).unwrap();
        })
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}

#[test]
fn geojson_draws_polygon() {
    let js = br#"{
      "type":"FeatureCollection",
      "features":[{
        "type":"Feature",
        "properties":{},
        "geometry":{
          "type":"Polygon",
          "coordinates":[[[-10,-5],[10,-5],[10,5],[-10,5],[-10,-5]]]
        }
      }]
    }"#;
    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.projection(GeoProjection::PlateCarree);
            let n = ax.geojson(js).expect("geojson");
            assert_eq!(n, 1);
            ax.x_range(-20.0, 20.0).y_range(-15.0, 15.0);
        })
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}
