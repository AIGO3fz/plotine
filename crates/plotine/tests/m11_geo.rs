//! M11 geographic projection integration tests.

use plotine::prelude::*;

#[test]
fn plate_carree_map_renders() {
    let png = Figure::new()
        .size(6.0, 3.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.projection(GeoProjection::PlateCarree);
            ax.coastline().color(Color::rgb(0x55, 0x55, 0x55));
            ax.scatter([0.0, 116.4, -74.0], [51.5, 39.9, 40.7])
                .color(Color::CRIMSON)
                .size(5.0);
            ax.title("PlateCarree");
        })
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}

#[test]
fn mercator_map_renders() {
    let png = Figure::new()
        .size(6.0, 4.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.projection(GeoProjection::Mercator);
            ax.coastline();
            ax.line([-100.0, -80.0, -60.0], [20.0, 35.0, 40.0])
                .color(Color::STEEL_BLUE)
                .width(2.0);
            ax.title("Mercator");
        })
        .render_png()
        .expect("png");
    assert!(!png.is_empty());
}

#[test]
fn geo_and_polar_are_mutually_exclusive_last_wins() {
    let fig = Figure::new().size(3.0, 3.0).dpi(72.0).axes(|ax| {
        ax.projection(GeoProjection::PlateCarree);
        ax.polar_line([0.0, 1.0], [0.5, 1.0]);
    });
    // polar clears geo; should still render.
    assert!(!fig.render_png().unwrap().is_empty());
}
