use plotine::prelude::*;

#[test]
fn subplot_at_span_renders() {
    let png = Figure::new()
        .size(5.0, 3.5)
        .dpi(72.0)
        .subplots(2, 2, |g| {
            g.hspace(0.3).wspace(0.25);
            g.at_span(0, 0, 2, 1, |ax| {
                ax.line([0.0, 1.0, 2.0, 3.0], [0.0, 1.0, 0.5, 1.2])
                    .color(Color::STEEL_BLUE)
                    .width(2.0);
                ax.title("tall");
            });
            g.at(0, 1, |ax| {
                ax.scatter([0.0, 1.0, 2.0], [1.0, 0.2, 0.8])
                    .color(Color::CRIMSON)
                    .size(4.0);
                ax.title("A");
            });
            g.at(1, 1, |ax| {
                ax.bar([1.0, 2.0], [3.0, 2.0]).color(Color::FOREST_GREEN);
                ax.title("B");
            });
        })
        .render_png()
        .expect("span png");
    assert!(!png.is_empty());
}
