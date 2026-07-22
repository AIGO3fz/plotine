//! M14 output formats: PGF always; EPS/MP4 when tools are present.

use plotine::prelude::*;

#[test]
fn save_pgf_contains_pgfpicture() {
    let x = [0.0, 1.0, 2.0];
    let y = [0.0, 1.0, 0.5];
    let pgf = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line(x, y);
        })
        .render_pgf()
        .expect("pgf");
    assert!(pgf.contains("\\begin{pgfpicture}"));
    assert!(pgf.contains("\\end{pgfpicture}"));
}

#[test]
#[cfg(feature = "eps")]
fn save_eps_errors_or_writes_when_gs_missing() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let path = dir.path().join("out.eps");
    let fig = Figure::new().size(2.0, 1.5).dpi(72.0).axes(|ax| {
        ax.line(&[0.0, 1.0], &[0.0, 1.0]);
    });
    match fig.save_eps(&path) {
        Ok(()) => assert!(path.is_file()),
        Err(PlotError::ExternalToolUnavailable { tool, .. }) => {
            assert!(tool == "gs" || tool.starts_with("gs"));
        }
        Err(other) => panic!("unexpected error: {other}"),
    }
}

#[test]
#[cfg(feature = "mp4")]
fn save_mp4_errors_or_writes_when_ffmpeg_missing() {
    let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.1).collect();
    let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let fig = Figure::new().size(2.0, 1.5).dpi(72.0).axes(|ax| {
        ax.line(&x, &y0);
        ax.y_range(-1.2, 1.2);
    });
    let anim = fig
        .animate(0..3, |fig, i| {
            let t = i as f64 * 0.2;
            let y: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
            fig.axes_at_mut(0)
                .unwrap()
                .line_at_mut(0)
                .unwrap()
                .set_y(&y)?;
            Ok(())
        })
        .expect("anim");
    let dir = tempfile::tempdir().expect("tmpdir");
    let path = dir.path().join("wave.mp4");
    match anim.save_mp4(&path) {
        Ok(()) => assert!(path.is_file()),
        Err(PlotError::ExternalToolUnavailable { tool, .. }) => assert_eq!(tool, "ffmpeg"),
        Err(other) => panic!("unexpected error: {other}"),
    }
}
