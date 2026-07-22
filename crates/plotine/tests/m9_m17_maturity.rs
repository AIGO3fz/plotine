//! M9–M17 maturity: content assertions beyond “does not panic”.
//!
//! Visual binary snapshots remain Linux-gated in `visual_snapshots.rs`.
//! This file proves behavioral contracts that hold on every CI platform.

use plotine::prelude::*;
use plotine::stats::{corr_heatmap, pair_scatter, regline};
use plotine::NavMode;
use plotine::ViewHistory;

fn wave_anim(n_frames: usize) -> Animation {
    let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.1).collect();
    let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let fig = Figure::new().size(3.0, 2.0).dpi(48.0).axes(|ax| {
        ax.line(&x, &y0).color(Color::CRIMSON);
        ax.y_range(-1.2, 1.2);
    });
    fig.animate(0..n_frames, |fig, i| {
        let t = i as f64 * 0.35;
        let y: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
        fig.axes_at_mut(0)
            .expect("panel")
            .line_at_mut(0)
            .expect("line")
            .set_y(&y)?;
        Ok(())
    })
    .expect("animate")
}

// --- M10 Animation ----------------------------------------------------------

#[test]
fn m10_frames_change_and_png_sequence_valid() {
    let anim = wave_anim(3).interval_ms(40);
    assert_eq!(anim.len(), 3);
    assert_ne!(anim.frames()[0].rgba, anim.frames()[1].rgba);

    let dir = tempfile::tempdir().expect("tmpdir");
    anim.save_png_sequence(dir.path()).expect("png sequence");
    for i in 0..3 {
        let path = dir.path().join(format!("frame_{i:04}.png"));
        let bytes = std::fs::read(&path).expect("read frame");
        assert!(
            bytes.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']),
            "frame {i} missing PNG signature"
        );
        assert!(bytes.len() > 64);
    }
}

#[test]
#[cfg(feature = "gif")]
fn m10_gif_has_magic_header() {
    let anim = wave_anim(3).interval_ms(50);
    let dir = tempfile::tempdir().expect("tmpdir");
    let path = dir.path().join("wave.gif");
    anim.save_gif(&path).expect("gif");
    let bytes = std::fs::read(&path).expect("read gif");
    assert!(
        bytes.starts_with(b"GIF89a") || bytes.starts_with(b"GIF87a"),
        "invalid GIF header: {:?}",
        &bytes[..bytes.len().min(8)]
    );
}

// --- M11 Geo / M16 stats ----------------------------------------------------

#[test]
fn m11_geo_maps_render_nontrivial() {
    for proj in [GeoProjection::PlateCarree, GeoProjection::Mercator] {
        let png = Figure::new()
            .size(5.0, 3.0)
            .dpi(72.0)
            .axes(|ax| {
                ax.projection(proj);
                ax.coastline().color(Color::rgb(0x44, 0x44, 0x44));
                ax.scatter([0.0, 116.4], [51.5, 39.9])
                    .color(Color::CRIMSON)
                    .size(4.0);
            })
            .render_png()
            .expect("geo png");
        assert!(png.len() > 500, "{proj:?} PNG too small");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']));
    }
}

#[test]
fn m16_stats_helpers_render_nontrivial() {
    let a = [1.0, 2.0, 3.0, 4.0, 5.0];
    let b = [2.0, 1.0, 4.0, 3.0, 6.0];
    let c = [5.0, 4.0, 3.0, 2.0, 1.0];
    let corr = corr_heatmap(&["a", "b", "c"], &[&a, &b, &c])
        .unwrap()
        .size(3.5, 3.0)
        .dpi(72.0)
        .render_png()
        .expect("corr");
    assert!(corr.len() > 500);

    let pair = pair_scatter(&["a", "b"], &[&a, &b])
        .unwrap()
        .size(4.0, 4.0)
        .dpi(72.0)
        .render_png()
        .expect("pair");
    assert!(pair.len() > 500);

    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [0.9, 2.1, 2.8, 4.2, 5.1];
    let reg = Figure::new()
        .size(3.5, 2.5)
        .dpi(72.0)
        .axes(|ax| {
            regline(ax, &x, &y).unwrap();
        })
        .render_png()
        .expect("regline");
    assert!(reg.len() > 500);
}

// --- M14 PGF (content contract; EPS/MP4 covered in m14_formats) -------------

#[test]
fn m14_pgf_has_picture_env() {
    let pgf = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
        })
        .render_pgf()
        .expect("pgf");
    assert!(pgf.contains("\\begin{pgfpicture}"));
    assert!(pgf.contains("\\end{pgfpicture}"));
    assert!(pgf.contains("\\pgfpath") || pgf.contains("\\pgfline"));
}

// --- M9 view/nav (GUI without opening a window) -----------------------------

#[test]
fn m9_view_history_home_back_forward() {
    let mut fig = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
        ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
        ax.x_range(0.0, 2.0).y_range(-1.0, 2.0);
    });
    let home = fig.capture_view();
    let mut hist = ViewHistory::new(home.clone());

    let mut zoomed = home.clone();
    if let Some(p) = zoomed.panels.get_mut(0) {
        p.x_min = 0.5;
        p.x_max = 1.5;
        p.y_min = 0.0;
        p.y_max = 1.0;
    }
    fig.apply_view(&zoomed);
    hist.push(fig.capture_view());

    let back = hist.back().expect("back");
    fig.apply_view(&back);
    assert_eq!(fig.capture_view(), home);

    let fwd = hist.forward().expect("forward");
    fig.apply_view(&fwd);
    let cur = fig.capture_view();
    assert!((cur.panels[0].x_min - 0.5).abs() < 1e-12);
    assert!((cur.panels[0].x_max - 1.5).abs() < 1e-12);

    assert_eq!(NavMode::default(), NavMode::Pan);
    let _ = NavMode::Zoom;
}

// --- M15 GUI smoke (optional display) ---------------------------------------

/// Requires a real display. Run locally:
/// `cargo test -p plotine --features gui --test m9_m17_maturity -- --ignored`
#[test]
#[cfg(feature = "gui")]
#[ignore = "requires a display; run with --features gui -- --ignored"]
fn m15_show_nonblocking_close_or_skip_headless() {
    let result = Figure::new()
        .size(2.0, 1.5)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0], [0.0, 1.0]);
        })
        .show_nonblocking();
    match result {
        Ok(handle) => {
            std::thread::sleep(std::time::Duration::from_millis(400));
            handle.close();
            // Headless CI may still open then fail on join; accept GUI-class errors.
            if let Err(e) = handle.join() {
                let msg = e.to_string().to_lowercase();
                assert!(
                    msg.contains("display")
                        || msg.contains("gui")
                        || msg.contains("window")
                        || msg.contains("winit")
                        || msg.contains("egl")
                        || msg.contains("wayland")
                        || msg.contains("x11")
                        || msg.contains("panic"),
                    "unexpected GUI join error: {e}"
                );
            }
        }
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            assert!(
                msg.contains("display")
                    || msg.contains("gui")
                    || msg.contains("window")
                    || msg.contains("spawn")
                    || msg.contains("winit")
                    || msg.contains("egl"),
                "unexpected show_nonblocking error: {e}"
            );
        }
    }
}
