use plotine::prelude::*;

#[test]
fn tripcolor_and_tricontour_render() {
    // Simple mesh: square split into two triangles.
    let x = [0.0, 1.0, 1.0, 0.0];
    let y = [0.0, 0.0, 1.0, 1.0];
    let z = [0.0, 1.0, 0.5, 0.25];
    let tris = [[0usize, 1, 2], [0, 2, 3]];

    let png = Figure::new()
        .size(4.0, 3.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.tripcolor(x, y, z)
                .triangles(tris)
                .cmap(Colormap::Coolwarm)
                .colorbar(true);
            ax.tricontour(x, y, z)
                .triangles(tris)
                .levels(6)
                .color(Color::BLACK)
                .width(0.8);
            ax.title("Tri mesh");
        })
        .render_png()
        .expect("tri png");
    assert!(!png.is_empty());
}

#[test]
fn tripcolor_requires_triangles() {
    let err = Figure::new()
        .size(3.0, 2.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.tripcolor([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.5]);
        })
        .render_png()
        .expect_err("missing triangles");
    let msg = format!("{err:?}");
    assert!(msg.contains("triangles") || msg.contains("empty"), "{msg}");
}
