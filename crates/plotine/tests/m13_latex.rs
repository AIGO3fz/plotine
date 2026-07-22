//! M13 external LaTeX integration tests.

use plotine::prelude::*;

#[cfg(not(feature = "latex"))]
#[test]
fn usetex_requires_latex_feature() {
    let err = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .usetex(true)
        .axes(|ax| {
            ax.title(r"$x^2$");
            ax.line([0.0, 1.0], [0.0, 1.0]);
        })
        .render_png()
        .expect_err("usetex without feature \"latex\"");
    assert!(
        err.to_string().contains("latex") || err.to_string().contains("LaTeX"),
        "{}",
        err
    );
    assert!(!err.suggestion().is_empty());
}

#[cfg(feature = "latex")]
#[test]
fn usetex_errors_when_tools_missing() {
    if plotine::latex::tools_available() {
        return;
    }
    let err = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .usetex(true)
        .axes(|ax| {
            ax.title(r"$x^2$");
            ax.line([0.0, 1.0], [0.0, 1.0]);
        })
        .render_png()
        .expect_err("usetex without latex/dvipng");
    match err {
        PlotError::LatexUnavailable { .. } => {}
        other => panic!("expected LatexUnavailable, got {other}"),
    }
    assert!(!err.suggestion().is_empty());
}

#[cfg(feature = "latex")]
#[test]
fn default_path_still_uses_mathtext() {
    let png = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .usetex(false)
        .axes(|ax| {
            ax.title(r"mathtext $x^2$");
            ax.line([0.0, 1.0], [0.0, 1.0]);
        })
        .render_png()
        .expect("mathtext default");
    assert!(!png.is_empty());
}

#[cfg(feature = "latex")]
#[test]
#[ignore = "requires latex + dvipng on PATH"]
fn usetex_renders_when_tex_present() {
    assert!(
        plotine::latex::tools_available(),
        "enable only when TeX is installed"
    );
    let png = Figure::new()
        .size(4.0, 3.0)
        .dpi(100.0)
        .usetex(true)
        .axes(|ax| {
            ax.title(r"$\alpha+\beta$");
            ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
        })
        .render_png()
        .expect("usetex png");
    assert!(!png.is_empty());
}
