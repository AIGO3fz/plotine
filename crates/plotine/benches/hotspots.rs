//! L2 Criterion microbenches for known hotspots (`docs/BENCHMARK.md` §P4).
//!
//! These are **recipe / chrome** timers for maintainers — not the product L1
//! suite (`examples/bench_suite.rs` / `scripts/benchmark.py`).
//!
//! ```bash
//! cargo bench -p plotine --bench hotspots
//! cargo bench -p plotine --bench hotspots -- recipe.contourf
//! ```
//!
//! HTML reports: `target/criterion/report/index.html`

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use plotine::mathtext;
use plotine::prelude::*;
use plotine::recipes::{auto_levels, contourf_fills, streamlines};
use plotine_backend_svg::SvgRenderer;
use plotine_core::{DataToPixel, LinearScale, Rect, ScaleKind};

fn transform_box(x0: f64, x1: f64, y0: f64, y1: f64) -> DataToPixel {
    let x = ScaleKind::Linear(LinearScale::new(x0, x1).unwrap());
    let y = ScaleKind::Linear(LinearScale::new(y0, y1).unwrap());
    DataToPixel::new(x, y, Rect::new(40.0, 40.0, 460.0, 340.0))
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

fn series_sin(n: usize) -> (Vec<f64>, Vec<f64>) {
    let x: Vec<f64> = (0..n)
        .map(|i| i as f64 * (10.0 / n.max(1) as f64))
        .collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    (x, y)
}

/// Pure geometry: contourf marching-squares fills (L1 bottleneck).
fn bench_recipe_contourf(c: &mut Criterion) {
    let n = 80usize;
    let z = grid_gauss(n);
    let levels = auto_levels(&z, 12);
    let transform = transform_box(-4.0, 4.0, -4.0, 4.0);
    let cmap: Cmap = Colormap::Viridis.into();

    c.bench_function("recipe.contourf_fills_80", |b| {
        b.iter(|| {
            contourf_fills(
                black_box(&z),
                n,
                n,
                None,
                None,
                black_box(&levels),
                &cmap,
                Norm::Linear,
                &transform,
            )
        });
    });
}

/// Pure geometry: streamplot integration.
fn bench_recipe_streamplot(c: &mut Criterion) {
    let n = 20usize;
    let (u, v) = vortex_uv(n);
    let transform = transform_box(0.0, (n - 1) as f64, 0.0, (n - 1) as f64);
    let px = 150.0 / 72.0;

    c.bench_function("recipe.streamlines_20", |b| {
        b.iter(|| streamlines(black_box(&u), black_box(&v), n, n, 1.0, 1.0, &transform, px));
    });
}

/// Mathtext measure (text shaping / layout, no full figure).
fn bench_mathtext(c: &mut Criterion) {
    let renderer = SvgRenderer::new(400, 300).expect("svg");
    let samples = [
        r"$\int_0^1 x^2\,dx$",
        r"$\frac{\alpha+\beta}{\gamma}$",
        r"$10^{n}$ vs plain",
        r"$\sum_{i=1}^{N} x_i$",
    ];

    c.bench_function("mathtext.measure_mixed", |b| {
        b.iter(|| {
            for s in &samples {
                let _ = mathtext::measure_text(&renderer, black_box(s), 14.0);
            }
        });
    });
}

/// End-to-end PNG for the four L1 hotspot families (Criterion stats).
fn bench_e2e_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_render_png");

    group.bench_function("contourf_80", |b| {
        let z = grid_gauss(80);
        b.iter(|| {
            Figure::new()
                .size(5.0, 3.5)
                .dpi(150.0)
                .theme(Theme::light())
                .axes(|ax| {
                    ax.contourf(80, 80, black_box(&z))
                        .levels(12)
                        .cmap(Colormap::Viridis);
                })
                .render_png()
                .expect("png")
        });
    });

    group.bench_function("streamplot_20", |b| {
        let n = 20usize;
        let (u, v) = vortex_uv(n);
        b.iter(|| {
            Figure::new()
                .size(5.0, 3.5)
                .dpi(150.0)
                .theme(Theme::light())
                .axes(|ax| {
                    ax.streamplot(n, n, black_box(&u), black_box(&v))
                        .density(1.0)
                        .color(Color::CRIMSON);
                })
                .render_png()
                .expect("png")
        });
    });

    group.bench_function("mathtext_title", |b| {
        let (x, y) = series_sin(200);
        b.iter(|| {
            Figure::new()
                .size(5.0, 3.5)
                .dpi(150.0)
                .theme(Theme::light())
                .axes(|ax| {
                    ax.line(black_box(&x), black_box(&y))
                        .color(Color::STEEL_BLUE);
                    ax.title(r"$\int_0^1 x^2\,dx$")
                        .x_label(r"$x$")
                        .y_label(r"$f(x)$");
                })
                .render_png()
                .expect("png")
        });
    });

    group.bench_function("subplots_2x2", |b| {
        let (x, y) = series_sin(500);
        b.iter(|| {
            Figure::new()
                .size(5.0, 3.5)
                .dpi(150.0)
                .theme(Theme::light())
                .subplots(2, 2, |g| {
                    for r in 0..2 {
                        for c in 0..2 {
                            g.at(r, c, |ax| {
                                ax.line(black_box(&x), black_box(&y))
                                    .color(Color::STEEL_BLUE);
                                ax.title(format!("p{r}{c}"));
                            });
                        }
                    }
                })
                .render_png()
                .expect("png")
        });
    });

    group.finish();
}

/// Layout chrome: empty axes with title/labels (mpl-bench analogue).
fn bench_layout_chrome(c: &mut Criterion) {
    c.bench_function("e2e.chrome_empty", |b| {
        b.iter(|| {
            Figure::new()
                .size(5.0, 3.5)
                .dpi(150.0)
                .theme(Theme::light())
                .axes(|ax| {
                    ax.title("Empty").x_label("x").y_label("y").grid(true);
                })
                .render_png()
                .expect("png")
        });
    });
}

criterion_group!(
    hotspots,
    bench_recipe_contourf,
    bench_recipe_streamplot,
    bench_mathtext,
    bench_e2e_render,
    bench_layout_chrome
);
criterion_main!(hotspots);
