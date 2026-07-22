//! Size + render-time bench for core charts and Post-M8 features (M9–M13).
//!
//! ```bash
//! cargo run -p plotine --example size_benchmark --release
//! cargo run -p plotine --example size_benchmark --release --features "gif,latex"
//! python scripts/size_benchmark.py
//! ```
//!
//! Prefer `bench_suite` / `scripts/benchmark.py` for median/p95 product numbers.
//!
//! Writes under `compare/size_bench/` and prints `TIMING …` lines for the Python harness.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use plotine::prelude::*;

const OUT: &str = "compare/size_bench";
const FIG_W: f64 = 5.0;
const FIG_H: f64 = 3.5;
const FEATURE_DPI: f64 = 150.0;

fn main() -> Result<()> {
    let dir = PathBuf::from(OUT);
    fs::create_dir_all(&dir).map_err(|e| PlotError::io(e.to_string()))?;

    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    // --- Baseline: line @ several DPIs (historical size_bench) ---
    for dpi in [100.0, 150.0, 200.0, 300.0] {
        let name = format!("line_{}", dpi as u32);
        let path = dir.join(format!("plotine_{name}.png"));
        timed(&name, || {
            Figure::new()
                .size(FIG_W, FIG_H)
                .dpi(dpi)
                .axes(|ax| {
                    ax.line(&x, &y)
                        .color(Color::CRIMSON)
                        .width(2.0)
                        .label("sin(x)");
                    ax.title("Line")
                        .x_label("x")
                        .y_label("y")
                        .legend(Legend::TopRight);
                })
                .save(&path)?;
            Ok(path_meta(&path))
        })?;
    }

    // --- M9 static render path (shared with GUI; NOT toolbar UX — see docs/GUI_TOOLBAR.md) ---
    {
        let name = "static_render_150";
        let path = dir.join(format!("plotine_{name}.png"));
        timed(name, || {
            let png = Figure::new()
                .size(FIG_W, FIG_H)
                .dpi(FEATURE_DPI)
                .axes(|ax| {
                    ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
                    ax.title("static render (show pixel path)")
                        .x_label("x")
                        .y_label("y")
                        .grid(true);
                })
                .render_png()?;
            fs::write(&path, &png).map_err(|e| PlotError::io(e.to_string()))?;
            Ok(Meta {
                bytes: png.len() as u64,
                path: path.clone(),
            })
        })?;
    }

    // --- M10 animation: N frames + optional GIF ---
    {
        let name = "anim_20f_150";
        let frame_dir = dir.join("plotine_anim_frames");
        let _ = fs::remove_dir_all(&frame_dir);
        timed(name, || {
            let fig = Figure::new()
                .size(FIG_W, FIG_H)
                .dpi(FEATURE_DPI)
                .axes(|ax| {
                    ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
                    ax.title("Animation").y_range(-1.2, 1.2).grid(true);
                });
            let anim = fig.animate(0..20, |fig, i| {
                let t = i as f64 * 0.15;
                let yy: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
                fig.axes_at_mut(0)
                    .expect("axes")
                    .line_at_mut(0)
                    .expect("line")
                    .set_y(&yy)?;
                Ok(())
            })?;
            anim.save_png_sequence(&frame_dir)?;
            // Representative single-frame copy for size table
            let src = frame_dir.join("frame_0000.png");
            let dst = dir.join("plotine_anim_frame_150.png");
            fs::copy(&src, &dst).map_err(|e| PlotError::io(e.to_string()))?;
            let total: u64 = fs::read_dir(&frame_dir)
                .map_err(|e| PlotError::io(e.to_string()))?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum();
            Ok(Meta {
                bytes: total,
                path: dst,
            })
        })?;

        #[cfg(feature = "gif")]
        {
            let name = "anim_gif_20f_150";
            let path = dir.join("plotine_anim_20f_150.gif");
            timed(name, || {
                let fig = Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(FEATURE_DPI)
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
                        ax.title("Animation GIF").y_range(-1.2, 1.2);
                    });
                let anim = fig
                    .animate(0..20, |fig, i| {
                        let t = i as f64 * 0.15;
                        let yy: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
                        fig.axes_at_mut(0)
                            .expect("axes")
                            .line_at_mut(0)
                            .expect("line")
                            .set_y(&yy)?;
                        Ok(())
                    })?
                    .interval_ms(50);
                anim.save_gif(&path)?;
                Ok(path_meta(&path))
            })?;
        }
        #[cfg(not(feature = "gif"))]
        println!("SKIP name=anim_gif_20f_150 reason=feature_gif");
    }

    // --- M11 geographic projection + coastline ---
    {
        let name = "geo_150";
        let path = dir.join(format!("plotine_{name}.png"));
        timed(name, || {
            Figure::new()
                .size(FIG_W, FIG_H)
                .dpi(FEATURE_DPI)
                .axes(|ax| {
                    ax.projection(GeoProjection::PlateCarree);
                    ax.coastline()
                        .color(Color::rgb(0x55, 0x55, 0x55))
                        .width(0.7);
                    ax.scatter([0.0, 116.4, -74.0], [51.5, 39.9, 40.7])
                        .color(Color::CRIMSON)
                        .size(4.5);
                    ax.title("Geo PlateCarree").grid(true);
                })
                .save(&path)?;
            Ok(path_meta(&path))
        })?;
    }

    // --- M13 mathtext (default) + optional usetex ---
    {
        let name = "mathtext_150";
        let path = dir.join(format!("plotine_{name}.png"));
        timed(name, || {
            Figure::new()
                .size(FIG_W, FIG_H)
                .dpi(FEATURE_DPI)
                .axes(|ax| {
                    ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
                    ax.title(r"mathtext $\int_0^1 x^2\,dx$")
                        .x_label(r"$x$")
                        .y_label(r"$y$");
                })
                .save(&path)?;
            Ok(path_meta(&path))
        })?;
    }

    #[cfg(feature = "latex")]
    {
        if plotine::latex::tools_available() {
            let name = "usetex_150";
            let path = dir.join(format!("plotine_{name}.png"));
            timed(name, || {
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(FEATURE_DPI)
                    .usetex(true)
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
                        ax.title(r"usetex $\int_0^1 x^2\,dx$")
                            .x_label(r"$x$")
                            .y_label(r"$y$");
                    })
                    .save(&path)?;
                Ok(path_meta(&path))
            })?;
        } else {
            println!("SKIP name=usetex_150 reason=no_latex_tools");
        }
    }
    #[cfg(not(feature = "latex"))]
    println!("SKIP name=usetex_150 reason=feature_latex");

    println!(
        "NOTE m12_pyplot: run `cargo run -p plotine-pyplot --example pyplot_size_benchmark` \
         (harness does this)"
    );
    Ok(())
}

struct Meta {
    bytes: u64,
    path: PathBuf,
}

fn path_meta(path: &Path) -> Meta {
    Meta {
        bytes: fs::metadata(path).map(|m| m.len()).unwrap_or(0),
        path: path.to_path_buf(),
    }
}

fn timed(name: &str, f: impl FnOnce() -> Result<Meta>) -> Result<()> {
    let t0 = Instant::now();
    let meta = f()?;
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "TIMING name={name} ms={ms:.3} bytes={} path={}",
        meta.bytes,
        meta.path.display()
    );
    Ok(())
}
