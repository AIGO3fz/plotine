//! Product L1 benchmark suite (see `docs/BENCHMARK.md`).
//!
//! Protocol: warmup 2 + measure 7 → median / p95. Prefer `--release`.
//!
//! ```bash
//! cargo run -p plotine --example bench_suite --release
//! python scripts/benchmark.py
//! ```
//!
//! Optional: `BENCH_SAVE=1` writes sample artifacts under `compare/bench/`.
//! Filter: `BENCH_FILTER=series.line` runs only matching scenario names.
//! Tier: `BENCH_TIER=smoke|default|stress` (default excludes Tier B stress cases).

use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use plotine::prelude::*;

const OUT: &str = "compare/bench";
const FIG_W: f64 = 5.0;
const FIG_H: f64 = 3.5;
const DPI: f64 = 150.0;
const WARMUP: usize = 2;
const ITERS: usize = 7;

/// Tier S — fast smoke / CI (`docs/BENCHMARK.md`).
const TIER_SMOKE: &[&str] = &[
    "chrome.empty",
    "series.line_n1000",
    "series.scatter_n1000",
    "stat.heatmap_64",
    "layout.subplots_2x2",
    "math.mathtext",
    "fmt.svg_line_n1000",
];

/// Tier B — stress / nightly only.
const TIER_STRESS: &[&str] = &[
    "series.line_n1e6",
    "stat.heatmap_512",
    "field.streamplot_40",
    "d3.surface_80",
    "layout.subplots_4x4",
    "composite.dashboard",
];

type CaseFn = Box<dyn FnMut() -> Result<CaseOut>>;

fn main() -> Result<()> {
    let save = env::var_os("BENCH_SAVE").is_some();
    let filter = env::var("BENCH_FILTER").unwrap_or_default();
    let tier = env::var("BENCH_TIER").unwrap_or_else(|_| "default".into());
    let dir = PathBuf::from(OUT);
    if save {
        fs::create_dir_all(&dir).map_err(|e| PlotError::io(e.to_string()))?;
    }

    let mut cases: Vec<(String, CaseFn)> = Vec::new();
    register_all(&mut cases);
    register_stress(&mut cases);

    let selected: Vec<_> = cases
        .iter()
        .map(|(n, _)| n.clone())
        .filter(|n| tier_allows(&tier, n))
        .filter(|n| filter.is_empty() || n.contains(&filter))
        .collect();

    println!(
        "BENCH_META warmup={WARMUP} iters={ITERS} figsize={FIG_W}x{FIG_H} dpi={DPI} \
         tier={tier} save={} filter={:?} n_cases={} n_selected={}",
        save,
        if filter.is_empty() {
            "*"
        } else {
            filter.as_str()
        },
        cases.len(),
        selected.len()
    );

    for (name, case) in &mut cases {
        if !selected.iter().any(|s| s == name) {
            continue;
        }
        run_case(name, case, save, &dir)?;
    }
    Ok(())
}

fn tier_allows(tier: &str, name: &str) -> bool {
    let stress = TIER_STRESS.contains(&name);
    match tier.to_ascii_lowercase().as_str() {
        "smoke" | "s" => TIER_SMOKE.contains(&name),
        "stress" | "all" | "b" => true,
        // default / a: Tier A (+S), exclude stress
        _ => !stress,
    }
}

fn register_all(cases: &mut Vec<(String, CaseFn)>) {
    // ── chrome / series ──────────────────────────────────────────────
    push(
        cases,
        "chrome.empty",
        Box::new(|| {
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.title("Empty").x_label("x").y_label("y").grid(true);
                    })
                    .render_png()?,
            ))
        }),
    );

    for n in [1_000usize, 10_000, 100_000] {
        let title = format!("series.line_n{n}");
        let name = title.clone();
        push(
            cases,
            name,
            Box::new(move || {
                let (x, y) = series_sin(n);
                Ok(CaseOut::png(
                    Figure::new()
                        .size(FIG_W, FIG_H)
                        .dpi(DPI)
                        .theme(Theme::light())
                        .axes(|ax| {
                            ax.line(&x, &y)
                                .color(Color::CRIMSON)
                                .width(1.5)
                                .label("sin");
                            ax.title(&title)
                                .x_label("x")
                                .y_label("y")
                                .legend(Legend::TopRight)
                                .grid(true);
                        })
                        .render_png()?,
                ))
            }),
        );
    }

    for n in [1_000usize, 10_000] {
        let title = format!("series.scatter_n{n}");
        let name = title.clone();
        push(
            cases,
            name,
            Box::new(move || {
                let (x, y) = series_sin(n);
                Ok(CaseOut::png(
                    Figure::new()
                        .size(FIG_W, FIG_H)
                        .dpi(DPI)
                        .theme(Theme::light())
                        .axes(|ax| {
                            ax.scatter(&x, &y).color(Color::STEEL_BLUE).size(3.0);
                            ax.title(&title).x_label("x").y_label("y");
                        })
                        .render_png()?,
                ))
            }),
        );
    }

    push(
        cases,
        "series.bar_n50",
        Box::new(|| {
            let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
            let h: Vec<f64> = (0..50)
                .map(|i| 1.0 + ((i as f64) * 0.37).sin().abs() * 4.0)
                .collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.bar(&x, &h).color(Color::STEEL_BLUE);
                        ax.title("bar_n50");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "series.hist_n1e4",
        Box::new(|| {
            let data = seeded_samples(10_000);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.hist(&data).bins(30).color(Color::MEDIUM_PURPLE);
                        ax.title("hist_n1e4");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "series.area_n1e3",
        Box::new(|| {
            let (x, y) = series_sin(1_000);
            let y: Vec<f64> = y.iter().map(|v| v.abs() + 0.15).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.area(&x, &y)
                            .color(Color::FOREST_GREEN)
                            .alpha(0.45)
                            .label("area");
                        ax.title("area_n1e3").legend(Legend::TopRight);
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "series.errorbar_n200",
        Box::new(|| {
            let (x, y) = series_sin(200);
            let e: Vec<f64> = y.iter().map(|_| 0.15).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.errorbar(&x, &y, &e).color(Color::STEEL_BLUE);
                        ax.title("errorbar_n200");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "series.multiline_5x1e3",
        Box::new(|| {
            let (x, _) = series_sin(1_000);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        for k in 0..5 {
                            let y: Vec<f64> = x
                                .iter()
                                .map(|v| (v + k as f64 * 0.4).sin() + k as f64 * 0.15)
                                .collect();
                            ax.line(&x, &y).width(1.2).label(format!("s{k}"));
                        }
                        ax.title("multiline_5x1e3").legend(Legend::TopRight);
                    })
                    .render_png()?,
            ))
        }),
    );

    // ── stats ────────────────────────────────────────────────────────
    push(
        cases,
        "stat.heatmap_64",
        Box::new(|| Ok(CaseOut::png(heatmap_png(64)?))),
    );
    push(
        cases,
        "stat.heatmap_128",
        Box::new(|| Ok(CaseOut::png(heatmap_png(128)?))),
    );

    push(
        cases,
        "stat.boxplot",
        Box::new(|| {
            let g = box_groups(4, 100);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.boxplot([&g[0][..], &g[1][..], &g[2][..], &g[3][..]])
                            .color(Color::STEEL_BLUE);
                        ax.title("boxplot");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "stat.violin",
        Box::new(|| {
            let g = box_groups(4, 100);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.violin([&g[0][..], &g[1][..], &g[2][..], &g[3][..]])
                            .color(Color::MEDIUM_PURPLE);
                        ax.title("violin");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "stat.hist2d_1e4",
        Box::new(|| {
            let (x, y) = cloud_xy(10_000);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.hist2d(&x, &y).bins(30).cmap(Colormap::Viridis);
                        ax.title("hist2d_1e4");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "stat.hexbin_1e4",
        Box::new(|| {
            let (x, y) = cloud_xy(10_000);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.hexbin(&x, &y).gridsize(20).cmap(Colormap::Plasma);
                        ax.title("hexbin_1e4");
                    })
                    .render_png()?,
            ))
        }),
    );

    // ── field / mesh ─────────────────────────────────────────────────
    push(
        cases,
        "field.contourf_80",
        Box::new(|| {
            let z = grid_gauss(80);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.contourf(80, 80, &z).levels(12).cmap(Colormap::Viridis);
                        ax.title("contourf_80");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "field.pcolormesh_80",
        Box::new(|| {
            let n = 80usize;
            let xe: Vec<f64> = (0..=n).map(|i| i as f64 * 8.0 / n as f64 - 4.0).collect();
            let ye = xe.clone();
            let z = grid_gauss(n);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.pcolormesh(&xe, &ye, &z).cmap(Colormap::Viridis);
                        ax.title("pcolormesh_80");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "field.quiver_20",
        Box::new(|| {
            let n = 20usize;
            let (qx, qy, qu, qv) = quiver_grid(n);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.quiver(&qx, &qy, &qu, &qv).color(Color::STEEL_BLUE);
                        ax.title("quiver_20");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "field.streamplot_20",
        Box::new(|| {
            let n = 20usize;
            let (su, sv) = vortex_uv(n);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.streamplot(n, n, &su, &sv)
                            .density(1.0)
                            .color(Color::CRIMSON)
                            .width(0.9);
                        ax.title("streamplot_20");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "field.tripcolor_1e3",
        Box::new(|| {
            // ~32² verts; explicit grid tris (finalize still requires .triangles).
            let (x, y, z, tris) = tri_grid(32);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.tripcolor(&x, &y, &z)
                            .triangles(tris)
                            .cmap(Colormap::Viridis);
                        ax.title("tripcolor_1e3");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "field.spy_40",
        Box::new(|| {
            let n = 40usize;
            let mut sparse = vec![0.0; n * n];
            for i in 0..n {
                sparse[i * n + i] = 1.0;
                if i + 3 < n {
                    sparse[i * n + i + 3] = 0.5;
                }
            }
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.spy(n, n, &sparse)
                            .color(Color::STEEL_BLUE)
                            .marker_size(3.0);
                        ax.title("spy_40");
                    })
                    .render_png()?,
            ))
        }),
    );

    // ── layout / scales ──────────────────────────────────────────────
    push(
        cases,
        "layout.subplots_2x2",
        Box::new(|| {
            let (x, y) = series_sin(500);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .subplots(2, 2, |g| {
                        for r in 0..2 {
                            for c in 0..2 {
                                g.at(r, c, |ax| {
                                    ax.line(&x, &y).color(Color::STEEL_BLUE).width(1.2);
                                    ax.title(format!("p{r}{c}"));
                                });
                            }
                        }
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "layout.mosaic",
        Box::new(|| {
            let (x, y) = series_sin(400);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .subplot_mosaic("AAB;CCD", |name, ax| match name {
                        'A' | 'B' => {
                            ax.line(&x, &y).color(Color::STEEL_BLUE);
                            ax.title(format!("{name}"));
                        }
                        'C' => {
                            ax.scatter(&x, &y).size(2.0);
                            ax.title("C");
                        }
                        'D' => {
                            ax.hist(&y).bins(12);
                            ax.title("D");
                        }
                        _ => {}
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "layout.twin_y",
        Box::new(|| {
            let (x, left) = series_sin(200);
            let right: Vec<f64> = x.iter().map(|t| t * t * 0.05 + 1.0).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&x, &left)
                            .color(Color::STEEL_BLUE)
                            .width(1.5)
                            .label("L");
                        ax.twin_y(|ax2| {
                            ax2.line(&x, &right)
                                .color(Color::CRIMSON)
                                .width(1.5)
                                .label("R");
                        });
                        ax.title("twin_y").legend(Legend::TopLeft).grid(true);
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "layout.inset",
        Box::new(|| {
            let (x, y) = series_sin(200);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::STEEL_BLUE).width(1.5);
                        ax.title("inset");
                        ax.inset_axes([0.55, 0.55, 0.4, 0.4], |inset| {
                            inset
                                .line(&x[..40], &y[..40])
                                .color(Color::CRIMSON)
                                .width(1.2);
                            inset.title("zoom");
                        });
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "layout.secondary_x",
        Box::new(|| {
            let th: Vec<f64> = (0..80).map(|i| i as f64 * PI / 40.0).collect();
            let y: Vec<f64> = th.iter().map(|t| t.sin()).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&th, &y).color(Color::STEEL_BLUE).width(1.5);
                        ax.title("secondary_x").x_label("rad");
                        ax.secondary_x(f64::to_degrees, f64::to_radians, |sec| {
                            sec.label("deg");
                        });
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "scale.loglog",
        Box::new(|| {
            let x: Vec<f64> = (0..50)
                .map(|i| 10f64.powf(-1.0 + i as f64 * 0.06))
                .collect();
            let y: Vec<f64> = x.iter().map(|v| 3.0 * v.powf(1.4)).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
                        ax.line(&x, &y).width(1.5).label("power");
                        ax.title("loglog").legend(Legend::TopLeft);
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "scale.datetime",
        Box::new(|| {
            let t0 = 1_577_836_800_f64;
            let x: Vec<f64> = (0..60).map(|i| t0 + i as f64 * 86_400.0).collect();
            let y: Vec<f64> = (0..60)
                .map(|i| 8.0 + (i as f64 * 0.25).sin() * 1.5)
                .collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::STEEL_BLUE).width(1.5);
                        ax.x_datetime(true).title("datetime").y_label("y");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "polar.line",
        Box::new(|| {
            let th: Vec<f64> = (0..120).map(|i| i as f64 * PI / 60.0).collect();
            let pr: Vec<f64> = th.iter().map(|t| 1.0 + 0.35 * (2.0 * t).cos()).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.polar_line(&th, &pr).color(Color::CRIMSON).width(1.5);
                        ax.title("polar.line");
                    })
                    .render_png()?,
            ))
        }),
    );

    // ── annotation / proportion ──────────────────────────────────────
    push(
        cases,
        "anno.annotate_styles",
        Box::new(|| {
            let (x, y) = series_sin(80);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::STEEL_BLUE).width(1.5);
                        let styles = [
                            (ArrowStyle::Triangle, (2.0, 0.9), (3.0, 1.2)),
                            (ArrowStyle::Simple, (4.0, -0.5), (5.0, -0.9)),
                            (ArrowStyle::Bracket, (6.0, 0.4), (7.0, 0.8)),
                            (ArrowStyle::BothEnds, (1.0, -0.8), (0.2, -1.1)),
                        ];
                        for (style, xy, xytext) in styles {
                            ax.annotate("x", xy, xytext)
                                .arrow(true)
                                .arrow_style(style)
                                .color(Color::CRIMSON);
                        }
                        ax.title("annotate_styles");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "anno.table",
        Box::new(|| {
            let (x, y) = series_sin(40);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::STEEL_BLUE);
                        ax.table([["A", "3"], ["B", "5"], ["C", "2"]])
                            .col_labels(["name", "n"]);
                        ax.title("table");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "prop.pie",
        Box::new(|| {
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.pie([35.0, 25.0, 20.0, 12.0, 8.0])
                            .labels(["A", "B", "C", "D", "E"]);
                        ax.title("pie");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "prop.stackplot",
        Box::new(|| {
            let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.25).collect();
            let s0: Vec<f64> = x.iter().map(|v| 1.0 + 0.3 * v.sin()).collect();
            let s1: Vec<f64> = x.iter().map(|v| 1.5 + 0.2 * (v * 0.7).cos()).collect();
            let s2: Vec<f64> = x.iter().map(|v| 0.8 + 0.15 * (v * 1.3).sin()).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.stackplot(&x, [&s0[..], &s1[..], &s2[..]])
                            .labels(["a", "b", "c"])
                            .alpha(0.85);
                        ax.title("stackplot").legend(Legend::TopLeft);
                    })
                    .render_png()?,
            ))
        }),
    );

    // ── 3D ───────────────────────────────────────────────────────────
    push(
        cases,
        "d3.helix",
        Box::new(|| {
            let t: Vec<f64> = (0..200).map(|i| i as f64 * 0.1).collect();
            let x: Vec<f64> = t.iter().map(|v| v.cos()).collect();
            let y: Vec<f64> = t.iter().map(|v| v.sin()).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes3d(|ax| {
                        ax.plot3d(&x, &y, &t).color(Color::CRIMSON).width(1.5);
                        ax.title("helix").elev(30.0).azim(-60.0);
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "d3.scatter",
        Box::new(|| {
            let n = 200usize;
            let x: Vec<f64> = (0..n).map(|i| (i as f64 * 0.1).cos()).collect();
            let y: Vec<f64> = (0..n).map(|i| (i as f64 * 0.1).sin()).collect();
            let z: Vec<f64> = (0..n).map(|i| i as f64 / n as f64 * 10.0).collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes3d(|ax| {
                        ax.scatter3d(&x, &y, &z).color(Color::STEEL_BLUE).size(3.0);
                        ax.title("scatter3d");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "d3.surface_40",
        Box::new(|| Ok(CaseOut::png(surface_png(40)?))),
    );

    push(
        cases,
        "d3.wireframe_40",
        Box::new(|| {
            let n = 40usize;
            let (xs, ys, z) = surface_xyz(n);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes3d(|ax| {
                        ax.wireframe(n, n, &z)
                            .x(&xs)
                            .y(&ys)
                            .color(Color::STEEL_BLUE);
                        ax.title("wireframe_40").elev(35.0).azim(-50.0);
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "d3.bar",
        Box::new(|| {
            let x: Vec<f64> = (0..20).map(|i| (i % 5) as f64).collect();
            let y: Vec<f64> = (0..20).map(|i| (i / 5) as f64).collect();
            let z: Vec<f64> = (0..20)
                .map(|i| 1.0 + ((i as f64) * 0.4).sin().abs() * 3.0)
                .collect();
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes3d(|ax| {
                        ax.bar3d(&x, &y, &z).color(Color::STEEL_BLUE);
                        ax.title("bar3d").elev(30.0).azim(-55.0);
                    })
                    .render_png()?,
            ))
        }),
    );

    // ── formats / features ───────────────────────────────────────────
    push(
        cases,
        "math.mathtext",
        Box::new(|| {
            let (x, y) = series_sin(200);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::STEEL_BLUE).width(1.5);
                        ax.title(r"$\int_0^1 x^2\,dx$")
                            .x_label(r"$x$")
                            .y_label(r"$f(x)$");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "fmt.svg_line_n1000",
        Box::new(|| {
            let (x, y) = series_sin(1_000);
            Ok(CaseOut::svg(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::CRIMSON).width(1.5);
                        ax.title("svg_line_n1000");
                    })
                    .render_svg()?,
            ))
        }),
    );

    push(
        cases,
        "fmt.pdf_line_n1000",
        Box::new(|| {
            let (x, y) = series_sin(1_000);
            let pdf = Figure::new()
                .size(FIG_W, FIG_H)
                .dpi(DPI)
                .theme(Theme::light())
                .axes(|ax| {
                    ax.line(&x, &y).color(Color::CRIMSON).width(1.5);
                    ax.title("pdf_line_n1000");
                })
                .render_pdf()?;
            Ok(CaseOut {
                bytes: pdf.len() as u64,
                fmt: "pdf",
                sample: Some(pdf),
            })
        }),
    );

    push(
        cases,
        "feat.anim_20f",
        Box::new(|| {
            let (x, y) = series_sin(100);
            let fig = Figure::new()
                .size(FIG_W, FIG_H)
                .dpi(DPI)
                .theme(Theme::light())
                .axes(|ax| {
                    ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
                    ax.title("anim").y_range(-1.2, 1.2).grid(true);
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
            // Fair bytes vs mpl: encode PNG sequence (not raw RGBA).
            let tmp = env::temp_dir().join(format!("plotine_bench_anim_{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);
            fs::create_dir_all(&tmp).map_err(|e| PlotError::io(e.to_string()))?;
            anim.save_png_sequence(&tmp)?;
            let bytes: u64 = fs::read_dir(&tmp)
                .map_err(|e| PlotError::io(e.to_string()))?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum();
            let _ = fs::remove_dir_all(&tmp);
            Ok(CaseOut {
                bytes,
                fmt: "png_seq",
                sample: None,
            })
        }),
    );

    push(
        cases,
        "feat.geo",
        Box::new(|| {
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.projection(GeoProjection::PlateCarree);
                        ax.coastline()
                            .color(Color::rgb(0x55, 0x55, 0x55))
                            .width(0.7);
                        ax.scatter([0.0, 116.4, -74.0], [51.5, 39.9, 40.7])
                            .color(Color::CRIMSON)
                            .size(4.5);
                        ax.title("geo").grid(true);
                    })
                    .render_png()?,
            ))
        }),
    );
}

/// Tier B stress cases (`BENCH_TIER=stress`).
fn register_stress(cases: &mut Vec<(String, CaseFn)>) {
    push(
        cases,
        "series.line_n1e6",
        Box::new(|| {
            let (x, y) = series_sin(1_000_000);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.line(&x, &y).color(Color::CRIMSON).width(1.0);
                        ax.title("line_n1e6");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "stat.heatmap_512",
        Box::new(|| Ok(CaseOut::png(heatmap_png(512)?))),
    );

    push(
        cases,
        "field.streamplot_40",
        Box::new(|| {
            let n = 40usize;
            let (su, sv) = vortex_uv(n);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .axes(|ax| {
                        ax.streamplot(n, n, &su, &sv)
                            .density(1.0)
                            .color(Color::CRIMSON)
                            .width(0.8);
                        ax.title("streamplot_40");
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "d3.surface_80",
        Box::new(|| Ok(CaseOut::png(surface_png(80)?))),
    );

    push(
        cases,
        "layout.subplots_4x4",
        Box::new(|| {
            let (x, y) = series_sin(200);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .subplots(4, 4, |g| {
                        for r in 0..4 {
                            for c in 0..4 {
                                g.at(r, c, |ax| {
                                    ax.line(&x, &y).color(Color::STEEL_BLUE).width(1.0);
                                    ax.title(format!("{r}{c}"));
                                });
                            }
                        }
                    })
                    .render_png()?,
            ))
        }),
    );

    push(
        cases,
        "composite.dashboard",
        Box::new(|| {
            let (x, left) = series_sin(400);
            let right: Vec<f64> = x.iter().map(|t| t * t * 0.02 + 0.5).collect();
            let z = grid_gauss(48);
            Ok(CaseOut::png(
                Figure::new()
                    .size(FIG_W, FIG_H)
                    .dpi(DPI)
                    .theme(Theme::light())
                    .subplots(1, 2, |g| {
                        g.wspace(0.35);
                        g.at(0, 0, |ax| {
                            ax.line(&x, &left)
                                .color(Color::STEEL_BLUE)
                                .width(1.5)
                                .label("amp");
                            ax.twin_y(|ax2| {
                                ax2.line(&x, &right)
                                    .color(Color::CRIMSON)
                                    .width(1.2)
                                    .label("energy");
                            });
                            ax.title("twin")
                                .legend(Legend::OutsideUpperRight)
                                .grid(true);
                        });
                        g.at(0, 1, |ax| {
                            ax.heatmap(48, 48, &z)
                                .cmap(Colormap::Viridis)
                                .colorbar(true);
                            ax.title("heat");
                        });
                    })
                    .render_png()?,
            ))
        }),
    );
}

fn heatmap_png(n: usize) -> Result<Vec<u8>> {
    let z = grid_gauss(n);
    Figure::new()
        .size(FIG_W, FIG_H)
        .dpi(DPI)
        .theme(Theme::light())
        .axes(|ax| {
            ax.heatmap(n, n, &z).cmap(Colormap::Viridis).colorbar(true);
            ax.title(format!("heatmap_{n}"));
        })
        .render_png()
}

fn surface_png(n: usize) -> Result<Vec<u8>> {
    let (xs, ys, z) = surface_xyz(n);
    Figure::new()
        .size(FIG_W, FIG_H)
        .dpi(DPI)
        .theme(Theme::light())
        .axes3d(|ax| {
            ax.surface(n, n, &z)
                .x(&xs)
                .y(&ys)
                .cmap(Colormap::Viridis)
                .alpha(0.9);
            ax.title(format!("surface_{n}")).elev(35.0).azim(-50.0);
        })
        .render_png()
}

struct CaseOut {
    bytes: u64,
    fmt: &'static str,
    sample: Option<Vec<u8>>,
}

impl CaseOut {
    fn png(png: Vec<u8>) -> Self {
        let bytes = png.len() as u64;
        Self {
            bytes,
            fmt: "png",
            sample: Some(png),
        }
    }

    fn svg(svg: String) -> Self {
        let sample = svg.into_bytes();
        let bytes = sample.len() as u64;
        Self {
            bytes,
            fmt: "svg",
            sample: Some(sample),
        }
    }
}

fn push(cases: &mut Vec<(String, CaseFn)>, name: impl Into<String>, f: CaseFn) {
    cases.push((name.into(), f));
}

fn run_case(name: &str, case: &mut CaseFn, save: bool, dir: &Path) -> Result<()> {
    for _ in 0..WARMUP {
        let _ = case()?;
    }
    let mut times = Vec::with_capacity(ITERS);
    let mut last = CaseOut {
        bytes: 0,
        fmt: "png",
        sample: None,
    };
    for _ in 0..ITERS {
        let t0 = Instant::now();
        last = case()?;
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = percentile(&times, 0.50);
    let p95 = percentile(&times, 0.95);
    println!(
        "BENCH name={name} median_ms={median:.3} p95_ms={p95:.3} bytes={} fmt={}",
        last.bytes, last.fmt
    );
    if save {
        if let Some(bytes) = last.sample {
            let path = dir.join(format!("plotine_{name}.{}", last.fmt));
            fs::write(&path, bytes).map_err(|e| PlotError::io(e.to_string()))?;
        }
    }
    Ok(())
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn series_sin(n: usize) -> (Vec<f64>, Vec<f64>) {
    let x: Vec<f64> = (0..n)
        .map(|i| i as f64 * (10.0 / n.max(1) as f64))
        .collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    (x, y)
}

/// Deterministic N(0,1) samples for L1 bench (must stay in sync with
/// `scripts/benchmark.py::seeded_samples` — LCG + Box–Muller, not numpy RNG).
fn seeded_samples(n: usize) -> Vec<f64> {
    let mut state = 42u64;
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let u1 = ((state >> 33) as f64 / (u32::MAX as f64)).clamp(1e-12, 1.0);
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let u2 = ((state >> 33) as f64 / (u32::MAX as f64)).clamp(1e-12, 1.0);
        let r = (-2.0 * u1.ln()).sqrt();
        out.push(r * (2.0 * PI * u2).cos());
    }
    out
}

fn box_groups(ng: usize, n: usize) -> Vec<Vec<f64>> {
    let all = seeded_samples(ng * n);
    (0..ng).map(|g| all[g * n..(g + 1) * n].to_vec()).collect()
}

fn cloud_xy(n: usize) -> (Vec<f64>, Vec<f64>) {
    let s = seeded_samples(n * 2);
    let x = s[..n].to_vec();
    let y: Vec<f64> = s[n..]
        .iter()
        .zip(s[..n].iter())
        .map(|(a, b)| 0.6 * a + 0.4 * b)
        .collect();
    (x, y)
}

fn grid_gauss(n: usize) -> Vec<f64> {
    let mut z = Vec::with_capacity(n * n);
    let scale = 8.0 / (n.max(2) - 1) as f64;
    for r in 0..n {
        for c in 0..n {
            let x = c as f64 * scale - 4.0;
            let y = r as f64 * scale - 4.0;
            z.push((-x * x - y * y).exp() * 2.0 + 0.3 * (x * 0.8).sin());
        }
    }
    z
}

fn vortex_uv(n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut u = vec![0.0; n * n];
    let mut v = vec![0.0; n * n];
    let mid = (n as f64 - 1.0) * 0.5;
    for r in 0..n {
        for c in 0..n {
            let cx = c as f64 - mid;
            let cy = r as f64 - mid;
            u[r * n + c] = -cy;
            v[r * n + c] = cx;
        }
    }
    (u, v)
}

fn quiver_grid(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut qx = Vec::with_capacity(n * n);
    let mut qy = Vec::with_capacity(n * n);
    let mut qu = Vec::with_capacity(n * n);
    let mut qv = Vec::with_capacity(n * n);
    let mid = (n as f64 - 1.0) * 0.5;
    for r in 0..n {
        for c in 0..n {
            let x = c as f64;
            let y = r as f64;
            qx.push(x);
            qy.push(y);
            qu.push(-(y - mid) * 0.3);
            qv.push((x - mid) * 0.3);
        }
    }
    (qx, qy, qu, qv)
}

fn tri_grid(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<[usize; 3]>) {
    let mut x = Vec::with_capacity(n * n);
    let mut y = Vec::with_capacity(n * n);
    let mut z = Vec::with_capacity(n * n);
    for r in 0..n {
        for c in 0..n {
            let xv = c as f64;
            let yv = r as f64;
            x.push(xv);
            y.push(yv);
            z.push((-(xv * xv + yv * yv) * 0.02).exp());
        }
    }
    let mut tris = Vec::with_capacity((n - 1) * (n - 1) * 2);
    for r in 0..n - 1 {
        for c in 0..n - 1 {
            let i = r * n + c;
            tris.push([i, i + 1, i + n]);
            tris.push([i + 1, i + n + 1, i + n]);
        }
    }
    (x, y, z, tris)
}

fn surface_xyz(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let xs: Vec<f64> = (0..n)
        .map(|i| (i as f64 / (n - 1) as f64) * 4.0 - 2.0)
        .collect();
    let ys = xs.clone();
    let mut z = Vec::with_capacity(n * n);
    for &yv in &ys {
        for &xv in &xs {
            z.push((-(xv * xv + yv * yv) * 0.5).exp());
        }
    }
    (xs, ys, z)
}
