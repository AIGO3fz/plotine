//! CJK font feature smoke tests (skipped when no system CJK face is present).

#![cfg(feature = "cjk")]

use plotine::prelude::*;

#[test]
fn load_system_cjk_and_render_title() {
    let loaded = match plotine::fonts::load_system_cjk() {
        Ok(f) => f,
        Err(_) => {
            eprintln!("skip: no system CJK font on this machine");
            return;
        }
    };
    assert!(!loaded.is_empty());
    assert!(!plotine::fonts::registered_families().is_empty());

    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0], [0.0, 1.0]);
            ax.title("测试标题").x_label("横轴").y_label("纵轴");
        })
        .render_png()
        .expect("png");
    assert!(png.len() > 100);
}
