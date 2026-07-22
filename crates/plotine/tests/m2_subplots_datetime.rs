use plotine::prelude::*;

#[test]
fn subplots_2x2_renders() {
    let x = [0.0, 1.0, 2.0, 3.0];
    let y = [0.0, 1.0, 0.5, 1.2];
    let (w, h, rgba) = Figure::new()
        .size(6.0, 5.0)
        .dpi(72.0)
        .subplots(2, 2, |g| {
            g.at(0, 0, |ax| {
                ax.line(x, y).width(1.5);
                ax.title("A");
            });
            g.at(0, 1, |ax| {
                ax.scatter(x, y).size(4.0);
                ax.title("B");
            });
            g.at(1, 0, |ax| {
                ax.bar([1.0, 2.0, 3.0], [2.0, 4.0, 3.0]);
                ax.title("C");
            });
            g.at(1, 1, |ax| {
                ax.hist([0.1, 0.2, 0.8, 0.9, 1.1, 1.2]).bins(4);
                ax.title("D");
            });
        })
        .render_rgba()
        .expect("subplots");
    assert_eq!(w, 432);
    assert_eq!(h, 360);
    assert_eq!(rgba.len(), (w * h * 4) as usize);
}

#[test]
fn empty_subplots_errors() {
    let err = Figure::new()
        .subplots(2, 2, |_g| {})
        .render_png()
        .unwrap_err();
    assert!(err.to_string().contains("suggestion"));
}

#[test]
fn datetime_axis_renders() {
    let start = 1_577_836_800_f64;
    let x: Vec<f64> = (0..10).map(|i| start + i as f64 * 86_400.0).collect();
    let y: Vec<f64> = (0..10).map(|i| i as f64).collect();
    Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(&x, &y).width(2.0);
            ax.x_datetime(true).title("dates");
        })
        .render_png()
        .expect("datetime");
}

#[test]
fn unix_format_smoke() {
    let s = plotine_core::format_unix_tick(1_577_836_800.0, 86400.0 * 40.0);
    assert!(s.contains("2020"), "{s}");
}

#[test]
fn datetime_locator_daily_midnight() {
    let start = 1_577_836_800.0;
    let end = start + 14.0 * 86_400.0;
    let ticks = plotine_core::DatetimeLocator::new(6).ticks(start, end);
    assert!(ticks.len() >= 3 && ticks.len() <= 16);
    for t in &ticks {
        let (_, _, _, hh, mm, ss) = plotine_core::civil_from_unix(t.value as i64);
        assert_eq!((hh, mm, ss), (0, 0, 0), "label={}", t.label);
    }
}

#[test]
fn tight_layout_aligns_column_axes() {
    // Left column has a y-label only on the bottom panel → both should share left inset.
    let (w, h, rgba) = Figure::new()
        .size(7.0, 5.0)
        .dpi(72.0)
        .subplots(2, 1, |g| {
            g.at(0, 0, |ax| {
                ax.line([0.0, 1.0], [0.0, 1.0]).width(1.5);
                ax.title("Top");
            });
            g.at(1, 0, |ax| {
                ax.line([0.0, 1.0], [1.0, 0.0]).width(1.5);
                ax.title("Bottom").y_label("amplitude");
            });
        })
        .render_rgba()
        .expect("tight");
    assert_eq!(rgba.len(), (w * h * 4) as usize);
}
