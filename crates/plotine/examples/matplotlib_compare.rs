//! Generate the plotine half of a side-by-side comparison with matplotlib.
//!
//! ```bash
//! cargo run -p plotine --example matplotlib_compare
//! python scripts/matplotlib_compare.py   # requires matplotlib
//! ```
//!
//! Outputs land in `./compare/` as `plotine_*.png` and `mpl_*.png`.

use std::f64::consts::PI;
use std::fs;
use std::path::PathBuf;

use plotine::prelude::*;

fn out_dir() -> PathBuf {
    PathBuf::from("compare")
}

fn ensure_out() -> Result<()> {
    fs::create_dir_all(out_dir()).map_err(|e| PlotError::io(e.to_string()))
}

fn sine_xy(n: usize) -> (Vec<f64>, Vec<f64>) {
    let x: Vec<f64> = (0..n).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    (x, y)
}

fn main() -> Result<()> {
    ensure_out()?;
    let dir = out_dir();
    let (x, y) = sine_xy(100);

    // --- line ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
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
        .save(dir.join("plotine_line.png"))?;

    // --- scatter ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.scatter(&x, &y)
                // matplotlib compare uses s=12 → diameter = 2·√(s/π)
                .size(plotine::mpl_policy::scatter::diameter_from_area_pt2(12.0))
                .color(Color::STEEL_BLUE)
                .label("samples");
            ax.title("Scatter")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight);
        })
        .save(dir.join("plotine_scatter.png"))?;

    // --- bar ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.bar([1.0, 2.0, 3.0, 4.0], [3.0, 7.0, 2.0, 5.0])
                .color(Color::STEEL_BLUE)
                .label("counts");
            ax.title("Bar")
                .x_label("category")
                .y_label("value")
                .legend(Legend::TopRight)
                .grid(true)
                .grid_axis(GridAxis::Y);
        })
        .save(dir.join("plotine_bar.png"))?;

    // --- hist ---
    let hist_data: Vec<f64> = (0..200)
        .map(|i| {
            let t = i as f64 / 40.0;
            (t * 0.7).sin() + 0.15 * ((i % 17) as f64 / 17.0)
        })
        .collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.hist(&hist_data)
                .bins(12)
                .color(Color::FOREST_GREEN)
                .label("n");
            ax.title("Histogram")
                .x_label("value")
                .y_label("count")
                .legend(Legend::TopRight)
                .grid(true)
                .grid_axis(GridAxis::Y);
        })
        .save(dir.join("plotine_hist.png"))?;

    // --- area ---
    let area_y: Vec<f64> = x.iter().map(|v| (v * 0.8).sin().abs() + 0.2).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.area(&x, &area_y)
                .color(Color::STEEL_BLUE)
                .alpha(0.45)
                .label("area");
            ax.title("Area")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight);
        })
        .save(dir.join("plotine_area.png"))?;

    // --- errorbar ---
    let ex = [0.0, 1.0, 2.0, 3.0, 4.0];
    let ey = [1.0, 1.5, 1.2, 2.0, 1.8];
    let ee = [0.2, 0.25, 0.15, 0.3, 0.2];
    let exerr = [0.12, 0.1, 0.15, 0.1, 0.12];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.errorbar(ex, ey, ee)
                .xerr(exerr)
                .color(Color::STEEL_BLUE)
                .label("data");
            ax.title("Errorbar")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopLeft);
        })
        .save(dir.join("plotine_errorbar.png"))?;

    // --- loglog ---
    let x_log: Vec<f64> = (0..40)
        .map(|i| 10f64.powf(-1.0 + i as f64 * 0.08))
        .collect();
    let y_log: Vec<f64> = x_log.iter().map(|v| 2.0 * v.powf(1.5)).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
            ax.line(&x_log, &y_log).color(Color::CRIMSON).width(2.0);
            ax.title("Log-log").x_label("x").y_label("y");
        })
        .save(dir.join("plotine_loglog.png"))?;

    // --- dark + symlog ---
    let sx: Vec<f64> = (-40..41).map(|i| i as f64 * 0.25).collect();
    let sy: Vec<f64> = sx.iter().map(|v| v + 0.3 * v.sin()).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .theme(Theme::dark())
        .axes(|ax| {
            ax.y_scale(ScaleType::Symlog { linthresh: 1.0 });
            ax.line(&sx, &sy).color(Color::CRIMSON).width(2.0);
            ax.title("Dark + Symlog").x_label("x").y_label("y");
        })
        .save(dir.join("plotine_dark_symlog.png"))?;

    // --- paper theme ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin");
            ax.title("Paper Theme")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight);
        })
        .save(dir.join("plotine_paper.png"))?;

    // --- subplots ---
    Figure::new()
        .size(7.0, 5.0)
        .dpi(150.0)
        .subplots(2, 2, |g| {
            // Post-`tight_layout` gaps on this 2×2 are ~0.09 fig-fraction; with
            // axes≈cells that is wspace/hspace ≈ 0.2 (mpl GridSpec defaults).
            g.hspace(0.2).wspace(0.2);
            g.at(0, 0, |ax| {
                ax.line(&x, &y).color(Color::CRIMSON).width(1.5);
                ax.title("A: line").grid(false);
            });
            g.at(0, 1, |ax| {
                // mpl `s=8` ≈ marker diameter √8 pt
                ax.scatter(&x, &y).size(3.2).color(Color::STEEL_BLUE);
                ax.title("B: scatter").grid(false);
            });
            g.at(1, 0, |ax| {
                ax.bar([1.0, 2.0, 3.0], [2.0, 4.0, 3.0])
                    .color(Color::FOREST_GREEN);
                ax.title("C: bar").grid(false);
            });
            g.at(1, 1, |ax| {
                ax.hist(&hist_data).bins(10).color(Color::MEDIUM_PURPLE);
                ax.title("D: hist").grid(false);
            });
        })
        .save(dir.join("plotine_subplots.png"))?;

    // --- datetime ---
    let start = 1_577_836_800_f64; // 2020-01-01
    let dx: Vec<f64> = (0..12).map(|i| start + i as f64 * 86_400.0).collect();
    let dy: Vec<f64> = (0..12).map(|i| (i as f64 * 0.5).sin() + 1.0).collect();
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&dx, &dy).color(Color::STEEL_BLUE).width(2.0);
            ax.x_datetime(true)
                .title("Datetime")
                .x_label("date")
                .y_label("value");
        })
        .save(dir.join("plotine_datetime.png"))?;

    // --- heatmap ---
    let values: Vec<f64> = (0..64)
        .map(|i| {
            let r = (i / 8) as f64;
            let c = (i % 8) as f64;
            (r * 0.6).sin() + (c * 0.7).cos()
        })
        .collect();
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.heatmap(8, 8, &values)
                .cmap(Colormap::Viridis)
                .colorbar(true);
            ax.title("Heatmap");
        })
        .save(dir.join("plotine_heatmap.png"))?;

    // --- boxplot ---
    let a = [1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 7.0];
    let b = [2.0, 2.5, 3.0, 3.2, 3.8, 4.5];
    let c = [0.5, 1.0, 1.5, 2.0, 2.2, 2.8, 3.0];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.boxplot([&a[..], &b[..], &c[..]])
                .color(Color::STEEL_BLUE)
                .widths(0.5);
            ax.title("Boxplot").grid(true).grid_axis(GridAxis::Y);
        })
        .save(dir.join("plotine_boxplot.png"))?;

    // --- violin ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.violin([&a[..], &b[..], &c[..]])
                .color(Color::MEDIUM_PURPLE)
                .alpha(0.55);
            ax.title("Violin").grid(true).grid_axis(GridAxis::Y);
        })
        .save(dir.join("plotine_violin.png"))?;

    // ========== M5 / M6 additions ==========

    // --- fill_between ---
    let y1: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let y2: Vec<f64> = x.iter().map(|v| 0.5 * v.cos()).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.fill_between(&x, &y1, &y2)
                .color(Color::STEEL_BLUE)
                .alpha(0.4)
                .label("band");
            ax.line(&x, &y1)
                .color(Color::CRIMSON)
                .width(1.5)
                .label("y1");
            ax.line(&x, &y2)
                .color(Color::FOREST_GREEN)
                .width(1.5)
                .label("y2");
            ax.title("Fill Between").legend(Legend::TopRight).grid(true);
        })
        .save(dir.join("plotine_fill_between.png"))?;

    // --- step ---
    let stx = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let sty = [1.0, 2.0, 1.5, 3.0, 2.2, 2.8];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.step(stx, sty)
                .mode(StepMode::Mid)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("mid");
            ax.title("Step").legend(Legend::TopLeft).grid(true);
        })
        .save(dir.join("plotine_step.png"))?;

    // --- pie ---
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.pie([35.0, 25.0, 20.0, 20.0])
                .labels(["A", "B", "C", "D"]);
            ax.title("Pie").legend(Legend::TopRight);
        })
        .save(dir.join("plotine_pie.png"))?;

    // --- stackplot ---
    let sx_stack: Vec<f64> = (0..40).map(|i| i as f64 * 0.25).collect();
    let s0: Vec<f64> = sx_stack.iter().map(|v| 1.0 + 0.3 * v.sin()).collect();
    let s1: Vec<f64> = sx_stack
        .iter()
        .map(|v| 1.5 + 0.2 * (v * 0.7).cos())
        .collect();
    let s2: Vec<f64> = sx_stack
        .iter()
        .map(|v| 0.8 + 0.15 * (v * 1.3).sin())
        .collect();
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.stackplot(&sx_stack, [&s0[..], &s1[..], &s2[..]])
                .labels(["low", "mid", "high"])
                .alpha(0.85);
            ax.title("Stackplot").legend(Legend::TopLeft).grid(true);
        })
        .save(dir.join("plotine_stackplot.png"))?;

    // --- contour + clabel ---
    let mut z_c = Vec::with_capacity(30 * 30);
    for r in 0..30 {
        for c in 0..30 {
            let xx = c as f64 * 0.25 - 3.5;
            let yy = r as f64 * 0.25 - 3.5;
            z_c.push((-xx * xx - yy * yy).exp() * 2.0);
        }
    }
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.contourf(30, 30, &z_c)
                .levels(8)
                .cmap(Colormap::Viridis)
                .colorbar(false);
            ax.contour(30, 30, &z_c)
                .levels(8)
                .color(Color::rgb(51, 51, 51)) // matplotlib colors="0.2"
                .width(0.8)
                .clabel(true)
                .clabel_size(7.0);
            ax.title("Contour + Clabel");
        })
        .save(dir.join("plotine_contour.png"))?;

    // --- hist2d ---
    let mut hx = Vec::with_capacity(400);
    let mut hy = Vec::with_capacity(400);
    for i in 0..400 {
        let t = i as f64 * 0.05;
        hx.push(t.sin() + 0.15 * ((i * 3) % 11) as f64 / 11.0);
        hy.push(t.cos() + 0.15 * ((i * 5) % 13) as f64 / 13.0);
    }
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.hist2d(&hx, &hy)
                .bins(20)
                .cmap(Colormap::Viridis)
                .colorbar(true);
            ax.title("Hist2D");
        })
        .save(dir.join("plotine_hist2d.png"))?;

    // --- quiver ---
    let nq = 8usize;
    let mut qx = Vec::new();
    let mut qy = Vec::new();
    let mut qu = Vec::new();
    let mut qv = Vec::new();
    for r in 0..nq {
        for c in 0..nq {
            let xx = c as f64;
            let yy = r as f64;
            let cx = xx - 3.5;
            let cy = yy - 3.5;
            qx.push(xx);
            qy.push(yy);
            qu.push(-cy * 0.35);
            qv.push(cx * 0.35);
        }
    }
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.quiver(&qx, &qy, &qu, &qv)
                .color(Color::STEEL_BLUE)
                .quiverkey(1.0, "1");
            ax.title("Quiver");
        })
        .save(dir.join("plotine_quiver.png"))?;

    // --- barbs ---
    let mut bx = Vec::new();
    let mut by = Vec::new();
    let mut bu = Vec::new();
    let mut bv = Vec::new();
    for r in 0..5 {
        for c in 0..6 {
            let xx = c as f64;
            let yy = r as f64;
            let speed = 10.0 + c as f64 * 12.0 + r as f64 * 8.0;
            let ang = (r + c) as f64 * 0.4;
            bx.push(xx);
            by.push(yy);
            bu.push(speed * ang.cos());
            bv.push(speed * ang.sin());
        }
    }
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.barbs(&bx, &by, &bu, &bv).color(Color::STEEL_BLUE);
            ax.title("Barbs").grid(true);
        })
        .save(dir.join("plotine_barbs.png"))?;

    // --- streamplot ---
    let ns = 12usize;
    let mut su = vec![0.0; ns * ns];
    let mut sv = vec![0.0; ns * ns];
    for r in 0..ns {
        for c in 0..ns {
            let cx = c as f64 - 5.5;
            let cy = r as f64 - 5.5;
            su[r * ns + c] = -cy;
            sv[r * ns + c] = cx;
        }
    }
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.streamplot(ns, ns, &su, &sv)
                .density(1.2)
                .arrow_size(1.0)
                .color(Color::CRIMSON)
                .width(0.9);
            ax.title("Streamplot");
        })
        .save(dir.join("plotine_streamplot.png"))?;

    // --- polar ---
    let th: Vec<f64> = (0..120).map(|i| i as f64 * PI / 60.0).collect();
    let pr: Vec<f64> = th.iter().map(|t| 1.0 + 0.35 * (2.0 * t).cos()).collect();
    Figure::new()
        .size(4.5, 4.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.polar_line(&th, &pr).color(Color::CRIMSON).width(2.0);
            ax.title("Polar");
        })
        .save(dir.join("plotine_polar.png"))?;

    // --- twin_y (twinx) ---
    let ty: [f64; 6] = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let t_left: Vec<f64> = ty.iter().map(|&v| v.sin() + 1.5).collect();
    let t_right: Vec<f64> = ty.iter().map(|&v| 20.0 + 5.0 * v.cos()).collect();
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(ty, &t_left)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("left");
            ax.y_label("left y");
            ax.twin_y(|ax2| {
                ax2.line(ty, &t_right)
                    .color(Color::CRIMSON)
                    .width(2.0)
                    .label("right");
                ax2.y_label("right y");
            });
            ax.title("Twin Y")
                .x_label("x")
                .legend(Legend::TopLeft)
                .grid(true);
        })
        .save(dir.join("plotine_twin_y.png"))?;

    // --- twin_x (twiny) ---
    let bottom_x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let top_x = [1.0, 4.0, 9.0, 16.0, 25.0];
    let ty2 = [0.0, 1.0, 2.0, 3.0, 4.0];
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(bottom_x, ty2)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("linear x");
            ax.x_label("linear");
            ax.y_label("y");
            ax.twin_x(|ax2| {
                ax2.line(top_x, ty2)
                    .color(Color::CRIMSON)
                    .width(2.0)
                    .label("quad x");
                ax2.x_label("quadratic");
            });
            ax.title("Twin X").legend(Legend::BottomRight).grid(true);
        })
        .save(dir.join("plotine_twin_x.png"))?;

    // --- categories ---
    let cats = ["A", "B", "C", "D"];
    let heights = [3.0, 7.0, 2.0, 5.0];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.x_categories(cats);
            ax.bar(category_indices(cats.len()), heights)
                .color(Color::STEEL_BLUE);
            ax.title("Categories")
                .y_label("value")
                .grid(true)
                .grid_axis(GridAxis::Y);
        })
        .save(dir.join("plotine_categories.png"))?;

    // --- LogNorm heatmap ---
    let z_log = [
        1.0, 10.0, 100.0, 3.0, 30.0, 300.0, 5.0, 50.0, 500.0, 2.0, 20.0, 200.0, 4.0, 40.0, 400.0,
        6.0,
    ];
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.heatmap(4, 4, z_log)
                .cmap(Colormap::Viridis)
                .norm(Norm::Log)
                .colorbar(true);
            ax.title("LogNorm Heatmap");
        })
        .save(dir.join("plotine_lognorm.png"))?;

    // --- annotate ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
            ax.annotate("peak", (PI / 2.0, 1.0), (2.5, 1.15))
                .arrow_style(ArrowStyle::Simple)
                .color(Color::CRIMSON);
            ax.title("Annotate").grid(true);
        })
        .save(dir.join("plotine_annotate.png"))?;

    // --- stem ---
    let stem_x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let stem_y = [0.5, 1.2, 0.8, 1.5, 1.1, 0.6, 0.9];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.stem(stem_x, stem_y).color(Color::STEEL_BLUE);
            ax.title("Stem").grid(true);
        })
        .save(dir.join("plotine_stem.png"))?;

    // --- stairs ---
    let edges = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let vals = [1.0, 2.5, 1.5, 3.0, 2.0];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.stairs(edges, vals)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("bins");
            ax.title("Stairs").legend(Legend::TopRight).grid(true);
        })
        .save(dir.join("plotine_stairs.png"))?;

    // --- barh ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.barh([1.0, 2.0, 3.0, 4.0], [3.0, 7.0, 2.0, 5.0])
                .color(Color::STEEL_BLUE);
            ax.title("BarH")
                .x_label("value")
                .y_label("category")
                .grid(true)
                .grid_axis(GridAxis::X);
        })
        .save(dir.join("plotine_barh.png"))?;

    // --- hexbin ---
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.hexbin(&hx, &hy)
                .gridsize(12)
                .cmap(Colormap::Viridis)
                .colorbar(true);
            ax.title("Hexbin");
        })
        .save(dir.join("plotine_hexbin.png"))?;

    // --- spy ---
    let mut sparse = vec![0.0; 10 * 10];
    for i in 0..10 {
        sparse[i * 10 + i] = 1.0 + i as f64;
        if i + 2 < 10 {
            sparse[i * 10 + i + 2] = 0.5;
        }
    }
    Figure::new()
        .size(4.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.spy(10, 10, &sparse).color(Color::STEEL_BLUE);
            ax.title("Spy");
        })
        .save(dir.join("plotine_spy.png"))?;

    // --- eventplot ---
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.eventplot([
                [1.0, 2.0, 5.0, 7.0].as_slice(),
                [0.5, 3.0, 4.5].as_slice(),
                [2.5, 6.0].as_slice(),
            ])
            .labels(["r1", "r2", "r3"]);
            ax.title("Eventplot").legend(Legend::TopRight);
        })
        .save(dir.join("plotine_eventplot.png"))?;

    // --- broken_barh ---
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.broken_barh([(10.0, 50.0), (100.0, 20.0), (150.0, 40.0)], (20.0, 9.0))
                .color(Color::STEEL_BLUE)
                .label("jobs");
            ax.broken_barh([(40.0, 30.0), (120.0, 50.0)], (35.0, 9.0))
                .color(Color::CRIMSON)
                .label("tasks");
            ax.title("Broken BarH").legend(Legend::TopRight).grid(true);
        })
        .save(dir.join("plotine_broken_barh.png"))?;

    // --- polygon + spans ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.axvspan(1.0, 2.0)
                .color(Color::STEEL_BLUE)
                .alpha(0.25)
                .label("vspan");
            ax.axhspan(-0.2, 0.2)
                .color(Color::CRIMSON)
                .alpha(0.25)
                .label("hspan");
            ax.polygon([0.5, 2.5, 1.5], [0.5, 0.5, 1.5])
                .color(Color::FOREST_GREEN)
                .alpha(0.55)
                .label("poly");
            ax.x_range(0.0, 3.0).y_range(-0.5, 2.0);
            ax.title("Polygon + Spans")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(dir.join("plotine_polygon.png"))?;

    // --- pcolormesh ---
    let mut pc = Vec::with_capacity(8 * 8);
    for r in 0..8 {
        for c in 0..8 {
            pc.push((r as f64 * 0.5).sin() + (c as f64 * 0.4).cos());
        }
    }
    let x_edges: Vec<f64> = (0..=8).map(|i| i as f64).collect();
    let y_edges: Vec<f64> = (0..=8).map(|i| i as f64).collect();
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.pcolormesh(&x_edges, &y_edges, &pc)
                .cmap(Colormap::Plasma)
                .colorbar(true);
            ax.title("Pcolormesh");
        })
        .save(dir.join("plotine_pcolormesh.png"))?;

    // --- multiline ---
    let y_a: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let y_b: Vec<f64> = x.iter().map(|v| 0.7 * (v * 0.8).cos()).collect();
    let y_c: Vec<f64> = x.iter().map(|v| 0.4 * (v * 1.2).sin() + 0.3).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y_a)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin");
            ax.line(&x, &y_b)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("cos");
            ax.line(&x, &y_c)
                .color(Color::FOREST_GREEN)
                .width(2.0)
                .label("mix");
            ax.title("Multiline").legend(Legend::TopRight).grid(true);
        })
        .save(dir.join("plotine_multiline.png"))?;

    // --- hlines / vlines ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.hlines([0.5, 1.0, 1.5], 0.0, 5.0)
                .color(Color::STEEL_BLUE)
                .width(1.5)
                .label("hlines");
            ax.vlines([1.0, 2.5, 4.0], 0.0, 2.0)
                .color(Color::CRIMSON)
                .width(1.5)
                .label("vlines");
            ax.title("HLines / VLines")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(dir.join("plotine_hlines_vlines.png"))?;

    // --- fill_betweenx ---
    let fbx_y: Vec<f64> = (0..40).map(|i| i as f64 * 0.15).collect();
    let fbx_x1: Vec<f64> = fbx_y.iter().map(|v| (v * 0.8).sin()).collect();
    let fbx_x2: Vec<f64> = fbx_y.iter().map(|v| 0.5 * (v * 0.6).cos()).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.fill_betweenx(&fbx_y, &fbx_x1, &fbx_x2)
                .color(Color::STEEL_BLUE)
                .alpha(0.4)
                .label("band");
            ax.line(&fbx_x1, &fbx_y)
                .color(Color::CRIMSON)
                .width(1.5)
                .label("x1");
            ax.line(&fbx_x2, &fbx_y)
                .color(Color::FOREST_GREEN)
                .width(1.5)
                .label("x2");
            ax.title("Fill Between X")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(dir.join("plotine_fill_betweenx.png"))?;

    // --- axhline / axvline ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
            ax.axhline(0.0)
                .color(Color::CRIMSON)
                .width(1.2)
                .label("y=0");
            ax.axvline(PI)
                .color(Color::FOREST_GREEN)
                .width(1.2)
                .label("x=π");
            ax.title("AxHLine / AxVLine")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(dir.join("plotine_axhline_axvline.png"))?;

    // --- asymmetric errorbar ---
    let aex = [0.0, 1.0, 2.0, 3.0, 4.0];
    let aey = [1.0, 1.5, 1.2, 2.0, 1.8];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.errorbar(aex, aey, [0.1, 0.1, 0.1, 0.1, 0.1])
                .yerr_asym([0.3, 0.15, 0.4, 0.2, 0.25], [0.1, 0.35, 0.15, 0.4, 0.2])
                .xerr_asym([0.12, 0.08, 0.15, 0.1, 0.12], [0.08, 0.14, 0.1, 0.16, 0.09])
                .color(Color::STEEL_BLUE)
                .capsize(4.0)
                .label("asym");
            ax.title("Asymmetric Errorbar")
                .legend(Legend::TopLeft)
                .grid(true);
        })
        .save(dir.join("plotine_errorbar_asym.png"))?;

    // --- annotate arrow styles ---
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::STEEL_BLUE).width(1.5);
            ax.annotate("tri", (1.5, 1.0), (0.4, 0.3))
                .arrow_style(ArrowStyle::Triangle)
                .color(Color::CRIMSON);
            ax.annotate("simple", (3.0, 0.2), (4.2, 0.9))
                .arrow_style(ArrowStyle::Simple)
                .color(Color::FOREST_GREEN);
            ax.annotate("bracket", (5.0, -0.8), (6.5, -0.2))
                .arrow_style(ArrowStyle::Bracket)
                .color(Color::MEDIUM_PURPLE);
            ax.annotate("both", (7.5, 0.9), (8.8, 0.2))
                .arrow_style(ArrowStyle::BothEnds)
                .color(Color::CRIMSON);
            ax.title("Annotate Styles").grid(true);
        })
        .save(dir.join("plotine_annotate_styles.png"))?;

    // --- heatmap extent + alpha ---
    let hext: Vec<f64> = (0..16)
        .map(|i| {
            let r = (i / 4) as f64;
            let c = (i % 4) as f64;
            (r * 0.7).sin() + (c * 0.9).cos()
        })
        .collect();
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.heatmap(4, 4, &hext)
                .extent([0.0, 10.0, 0.0, 4.0])
                .alpha(0.75)
                .cmap(Colormap::Viridis)
                .colorbar(true);
            ax.title("Heatmap Extent");
        })
        .save(dir.join("plotine_heatmap_extent.png"))?;

    // --- inset_axes ---
    let ix: Vec<f64> = (0..80).map(|i| i as f64 * 0.15).collect();
    let iy: Vec<f64> = ix
        .iter()
        .map(|v| (v * 0.9).sin() + 0.15 * (v * 2.3).cos())
        .collect();
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&ix, &iy).color(Color::STEEL_BLUE).width(2.0);
            ax.title("Inset Axes").x_label("x").y_label("y");
            ax.inset_axes([0.55, 0.55, 0.4, 0.4], |inset| {
                inset
                    .line(&ix[..20], &iy[..20])
                    .color(Color::CRIMSON)
                    .width(1.5);
                inset.title("zoom");
            });
        })
        .save(dir.join("plotine_inset_axes.png"))?;

    // --- secondary axes ---
    let th_sec: Vec<f64> = (0..60).map(|i| i as f64 * PI / 30.0).collect();
    let y_sec: Vec<f64> = th_sec.iter().map(|t| t.sin()).collect();
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&th_sec, &y_sec).color(Color::STEEL_BLUE).width(2.0);
            ax.title("Secondary Axes")
                .x_label("radians")
                .y_label("amplitude");
            ax.secondary_x(f64::to_degrees, f64::to_radians, |sec| {
                sec.label("degrees");
            });
        })
        .save(dir.join("plotine_secondary_axes.png"))?;

    // --- subplot span ---
    let sx_span: Vec<f64> = (0..40).map(|i| i as f64 * 0.15).collect();
    let sy_span: Vec<f64> = sx_span.iter().map(|v| v.sin()).collect();
    Figure::new()
        .size(6.5, 4.5)
        .dpi(150.0)
        .subplots(2, 2, |g| {
            g.hspace(0.28).wspace(0.22);
            g.at_span(0, 0, 2, 1, |ax| {
                ax.line(&sx_span, &sy_span)
                    .color(Color::STEEL_BLUE)
                    .width(2.0);
                ax.title("Span (tall)").y_label("y").grid(false);
            });
            g.at(0, 1, |ax| {
                ax.scatter(&sx_span, &sy_span)
                    .color(Color::CRIMSON)
                    .size(3.0);
                ax.title("top-right").grid(false);
            });
            g.at(1, 1, |ax| {
                ax.hist(&sy_span).bins(10).color(Color::FOREST_GREEN);
                ax.title("bottom-right").grid(false);
            });
        })
        .save(dir.join("plotine_subplot_span.png"))?;

    // --- tripcolor + tricontour ---
    let tx = [0.0, 1.0, 2.0, 0.5, 1.5, 1.0];
    let ty = [0.0, 0.0, 0.0, 0.9, 0.9, 1.6];
    let tz = [0.0, 0.4, 0.1, 0.8, 1.0, 0.6];
    let ttris = [[0usize, 1, 3], [1, 2, 4], [1, 3, 4], [3, 4, 5]];
    Figure::new()
        .size(6.0, 4.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.tripcolor(tx, ty, tz)
                .triangles(ttris)
                .cmap(Colormap::RdBuR)
                .colorbar(true);
            ax.tricontour(tx, ty, tz)
                .triangles(ttris)
                .levels(7)
                .color(Color::rgb(0x22, 0x22, 0x22))
                .width(0.9);
            ax.title("Tripcolor + Tricontour");
        })
        .save(dir.join("plotine_tripcolor.png"))?;

    // --- nested inset_axes ---
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&ix, &iy).color(Color::STEEL_BLUE).width(2.0);
            ax.title("Nested Inset");
            ax.inset_axes([0.48, 0.48, 0.48, 0.48], |outer| {
                outer
                    .line(&ix[..30], &iy[..30])
                    .color(Color::CRIMSON)
                    .width(1.5);
                outer.title("outer");
                outer.inset_axes([0.5, 0.5, 0.45, 0.45], |inner| {
                    inner
                        .line(&ix[..12], &iy[..12])
                        .color(Color::FOREST_GREEN)
                        .width(1.2);
                    inner.title("inner");
                });
            });
        })
        .save(dir.join("plotine_nested_inset.png"))?;

    // --- secondary_y (°C ↔ °F) ---
    let t_sec: Vec<f64> = (0..40).map(|i| i as f64 * 0.25).collect();
    let c_sec: Vec<f64> = t_sec.iter().map(|t| 10.0 + 8.0 * (t * 0.7).sin()).collect();
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&t_sec, &c_sec).color(Color::STEEL_BLUE).width(2.0);
            ax.title("Secondary Y").x_label("t").y_label("°C");
            ax.secondary_y_linear(1.8, 32.0, |sec| {
                sec.label("°F");
            });
        })
        .save(dir.join("plotine_secondary_y.png"))?;

    // --- text + annotate callouts ---
    let (imax, ymax) = y
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, v)| (i, *v))
        .unwrap();
    let xmax = x[imax];
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("sin");
            ax.text(0.4, -0.6, "trough region")
                .color(Color::LABEL)
                .size(10.0)
                .ha(TextAlign::Left);
            ax.annotate("peak", (xmax, ymax), (xmax + 0.8, ymax + 0.35))
                .arrow(true)
                .color(Color::CRIMSON)
                .arrow_color(Color::SPINE)
                .ha(TextAlign::Left)
                .va(TextBaseline::Bottom);
            ax.title("Text + Annotate")
                .x_label("x")
                .y_label("y")
                .legend(Legend::BottomRight)
                .grid(true);
        })
        .save(dir.join("plotine_text.png"))?;

    // --- mathtext (real layout: scripts + frac; not unicode rewrite) ---
    let mx: Vec<f64> = (0..100).map(|i| i as f64 * 2.0 * PI / 99.0).collect();
    let my: Vec<f64> = mx
        .iter()
        .map(|t| (2.0 * t).sin() * (-0.15 * t).exp())
        .collect();
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&mx, &my)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label(r"$e^{-0.15t}\sin(2t)$");
            ax.title(r"Damped oscillator: $\alpha$-decay")
                .x_label(r"$t$ (s)")
                .y_label(r"$\theta$ (rad)")
                .legend(Legend::TopRight)
                .grid(true);
            ax.text(3.6, 0.45, r"$H_2O:\frac{1}{2}mv^2$")
                .color(Color::LABEL)
                .size(11.0);
        })
        .save(dir.join("plotine_math_labels.png"))?;

    // --- table ---
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.bar([1.0, 2.0, 3.0], [3.0, 5.0, 2.0])
                .color(Color::STEEL_BLUE)
                .label("counts");
            ax.table([["A", "3"], ["B", "5"], ["C", "2"]])
                .col_labels(["Item", "Value"])
                .loc(TableLoc::UpperRight)
                .fontsize(9.0);
            ax.title("Table")
                .legend(Legend::BottomLeft)
                .grid(true)
                .grid_axis(GridAxis::Y);
        })
        .save(dir.join("plotine_table.png"))?;

    // --- polar_scatter ---
    let th_sc: Vec<f64> = (0..36).map(|i| i as f64 * PI / 18.0).collect();
    let pr_sc: Vec<f64> = th_sc.iter().map(|t| 0.6 + 0.35 * (3.0 * t).cos()).collect();
    Figure::new()
        .size(4.5, 4.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.polar_scatter(&th_sc, &pr_sc)
                .color(Color::STEEL_BLUE)
                // matplotlib compare uses s=16
                .size(plotine::mpl_policy::scatter::diameter_from_area_pt2(16.0));
            ax.title("Polar Scatter");
        })
        .save(dir.join("plotine_polar_scatter.png"))?;

    // --- Coolwarm colormap ---
    let mut z_cw = Vec::with_capacity(24 * 24);
    for r in 0..24 {
        for c in 0..24 {
            let xx = c as f64 * 0.3 - 3.5;
            let yy = r as f64 * 0.3 - 3.5;
            z_cw.push(xx * (-xx * xx - yy * yy).exp());
        }
    }
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.contourf(24, 24, &z_cw)
                .levels(10)
                .cmap(Colormap::Coolwarm)
                .colorbar(true);
            ax.title("Coolwarm");
        })
        .save(dir.join("plotine_coolwarm.png"))?;

    // --- step modes (pre / mid / post) ---
    let stx = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let sty = [1.0, 2.0, 1.5, 3.0, 2.2, 2.8];
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.step(stx, sty)
                .mode(StepMode::Pre)
                .color(Color::CRIMSON)
                .width(1.8)
                .label("pre");
            ax.step(stx, sty)
                .mode(StepMode::Mid)
                .color(Color::STEEL_BLUE)
                .width(1.8)
                .label("mid");
            ax.step(stx, sty)
                .mode(StepMode::Post)
                .color(Color::FOREST_GREEN)
                .width(1.8)
                .label("post");
            ax.title("Step Modes").legend(Legend::TopLeft).grid(true);
        })
        .save(dir.join("plotine_step_modes.png"))?;

    // --- axhspan / axvspan (dedicated) ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::SPINE).width(1.5);
            ax.axvspan(1.0, 2.5)
                .color(Color::STEEL_BLUE)
                .alpha(0.25)
                .label("vspan");
            ax.axhspan(-0.4, 0.4)
                .color(Color::CRIMSON)
                .alpha(0.25)
                .label("hspan");
            ax.title("AxHSpan / AxVSpan")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(dir.join("plotine_axspan.png"))?;

    // --- empty axes chrome ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.title("Empty")
                .x_label("x")
                .y_label("y")
                .x_range(0.0, 1.0)
                .y_range(0.0, 1.0);
        })
        .save(dir.join("plotine_empty.png"))?;

    // --- bar + bottom legend ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.bar([1.0, 2.0, 3.0], [3.0, 5.0, 4.0])
                .color(Color::DARK_ORANGE)
                .label("A");
            ax.title("Bar Legend").legend(Legend::BottomRight);
        })
        .save(dir.join("plotine_bar_legend.png"))?;

    // --- contour clabel (dedicated paper-style) ---
    let mut z_cl = Vec::with_capacity(30 * 30);
    for r in 0..30 {
        for c in 0..30 {
            let xx = c as f64 * 0.25 - 3.5;
            let yy = r as f64 * 0.25 - 3.5;
            z_cl.push((-xx * xx - yy * yy).exp() * 2.0);
        }
    }
    Figure::new()
        .size(5.5, 4.5)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.contourf(30, 30, &z_cl)
                .levels(8)
                .cmap(Colormap::Viridis)
                .colorbar(false);
            ax.contour(30, 30, &z_cl)
                .levels(8)
                .color(Color::SPINE)
                .width(0.9)
                .clabel(true)
                .clabel_size(8.0)
                .clabel_color(Color::LABEL);
            ax.title("Contour Labels").grid(false);
        })
        .save(dir.join("plotine_clabel.png"))?;

    // --- scatter + line overlay ---
    let xs_ov: Vec<f64> = (0..35).map(|i| i as f64 * 8.0 / 34.0).collect();
    let ys_ov: Vec<f64> = xs_ov
        .iter()
        .enumerate()
        .map(|(i, v)| 0.2 * v + (i as f64).sin() * 0.4)
        .collect();
    let trend: Vec<f64> = xs_ov.iter().map(|v| 0.2 * v).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.scatter(&xs_ov, &ys_ov).size(5.0).label("data");
            ax.line(&xs_ov, &trend).width(2.0).label("trend");
            ax.title("Scatter + Line").legend(Legend::TopLeft);
        })
        .save(dir.join("plotine_scatter_line.png"))?;

    // --- area + line overlay ---
    let ya_ov: Vec<f64> = x.iter().map(|v| (v * 0.7).sin().abs() + 0.15).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.area(&x, &ya_ov).alpha(0.35).label("fill");
            ax.line(&x, &ya_ov).width(1.8).label("edge");
            ax.title("Area Overlay").legend(Legend::TopRight);
        })
        .save(dir.join("plotine_area_line.png"))?;

    // --- barh + h/vlines combo ---
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.barh([1.0, 2.0, 3.0], [4.0, 7.0, 2.5])
                .color(Color::STEEL_BLUE)
                .label("barh");
            ax.vlines([2.0, 5.0], 0.5, 3.5)
                .color(Color::CRIMSON)
                .width(1.5)
                .label("vlines");
            ax.hlines([2.5], 0.0, 8.0)
                .color(Color::DARK_ORANGE)
                .label("hline");
            ax.title("BarH / Spans").legend(Legend::TopRight).grid(true);
        })
        .save(dir.join("plotine_barh_spans.png"))?;

    // --- y_categories ---
    let ycats = ["low", "mid", "high"];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.y_categories(ycats);
            ax.barh(category_indices(ycats.len()), [4.0, 7.0, 2.5])
                .color(Color::STEEL_BLUE);
            ax.title("Y Categories")
                .x_label("value")
                .grid(true)
                .grid_axis(GridAxis::X);
        })
        .save(dir.join("plotine_y_categories.png"))?;

    // --- y_datetime ---
    let yd: Vec<f64> = (0..12).map(|i| start + i as f64 * 86_400.0).collect();
    let xd_val: Vec<f64> = (0..12).map(|i| (i as f64 * 0.5).sin() + 1.0).collect();
    Figure::new()
        .size(5.5, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&xd_val, &yd).color(Color::STEEL_BLUE).width(2.0);
            ax.y_datetime(true)
                .title("Y Datetime")
                .x_label("value")
                .y_label("date");
        })
        .save(dir.join("plotine_y_datetime.png"))?;

    // --- heatmap origin=lower ---
    let z_origin: Vec<f64> = (0..16)
        .map(|i| {
            let r = (i / 4) as f64;
            let c = (i % 4) as f64;
            r + c * 0.25
        })
        .collect();
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.heatmap(4, 4, &z_origin)
                .origin(HeatmapOrigin::Lower)
                .cmap(Colormap::Viridis)
                .colorbar(true);
            ax.title("Heatmap Origin Lower");
        })
        .save(dir.join("plotine_heatmap_origin.png"))?;

    // --- semilogy ---
    let sx_log: Vec<f64> = (1..=40).map(|i| i as f64).collect();
    let sy_log: Vec<f64> = sx_log.iter().map(|v| (0.15 * v).exp() * 0.05).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.y_scale(ScaleType::Log);
            ax.line(&sx_log, &sy_log).color(Color::CRIMSON).width(2.0);
            ax.title("Semilogy").x_label("x").y_label("y");
        })
        .save(dir.join("plotine_semilogy.png"))?;

    // --- Tab10 colormap ---
    let z_tab: Vec<f64> = (0..10).map(|i| i as f64).collect();
    Figure::new()
        .size(5.0, 3.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.heatmap(2, 5, &z_tab)
                .cmap(Colormap::Tab10)
                .colorbar(true);
            ax.title("Tab10");
        })
        .save(dir.join("plotine_tab10.png"))?;

    // --- Inferno heatmap ---
    let z_inf: Vec<f64> = (0..48)
        .map(|i| {
            let r = (i / 8) as f64;
            let c = (i % 8) as f64;
            (r * 0.55).sin() + (c * 0.65).cos()
        })
        .collect();
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.heatmap(6, 8, &z_inf)
                .cmap(Colormap::Inferno)
                .colorbar(true);
            ax.title("Inferno");
        })
        .save(dir.join("plotine_inferno.png"))?;

    // --- quiver + streamplot subplot ---
    let nqs = 12usize;
    let mut qxs = Vec::new();
    let mut qys = Vec::new();
    let mut qus = Vec::new();
    let mut qvs = Vec::new();
    let mut sus = vec![0.0; nqs * nqs];
    let mut svs = vec![0.0; nqs * nqs];
    for r in 0..nqs {
        for c in 0..nqs {
            let xx = c as f64;
            let yy = r as f64;
            let cx = xx - 5.5;
            let cy = yy - 5.5;
            qxs.push(xx);
            qys.push(yy);
            qus.push(-cy * 0.3);
            qvs.push(cx * 0.3);
            sus[r * nqs + c] = -cy;
            svs[r * nqs + c] = cx;
        }
    }
    Figure::new()
        .size(7.0, 3.5)
        .dpi(150.0)
        .subplots(1, 2, |g| {
            g.at(0, 0, |ax| {
                ax.quiver(&qxs, &qys, &qus, &qvs)
                    .color(Color::STEEL_BLUE)
                    .quiverkey(1.0, "1 unit");
                ax.title("Quiver");
            });
            g.at(0, 1, |ax| {
                ax.streamplot(nqs, nqs, &sus, &svs)
                    .density(1.2)
                    .color(Color::CRIMSON)
                    .width(0.9);
                ax.title("Streamplot");
            });
        })
        .save(dir.join("plotine_quiver_stream.png"))?;

    // --- polar + cartesian mix ---
    Figure::new()
        .size(7.0, 3.5)
        .dpi(150.0)
        .subplots(1, 2, |g| {
            g.wspace(0.3);
            g.at(0, 0, |ax| {
                ax.polar_line(&th, &pr)
                    .color(Color::MEDIUM_PURPLE)
                    .width(2.0);
                ax.title("Polar");
            });
            g.at(0, 1, |ax| {
                ax.line(&th, &pr).color(Color::STEEL_BLUE).width(1.75);
                ax.title("Cartesian")
                    .x_label("theta")
                    .y_label("r")
                    .grid(true);
            });
        })
        .save(dir.join("plotine_polar_mix.png"))?;

    // --- hist2d + hexbin subplot ---
    let mut hx_mix = Vec::new();
    let mut hy_mix = Vec::new();
    for i in 0..400 {
        let t = i as f64 * 0.05;
        hx_mix.push(t.sin() * 2.0 + (i % 17) as f64 * 0.05);
        hy_mix.push(t.cos() * 2.0 + (i % 13) as f64 * 0.04);
    }
    Figure::new()
        .size(7.0, 3.5)
        .dpi(150.0)
        .subplots(1, 2, |g| {
            g.at(0, 0, |ax| {
                ax.hist2d(&hx_mix, &hy_mix).bins(16).cmap(Colormap::Viridis);
                ax.title("Hist2D");
            });
            g.at(0, 1, |ax| {
                ax.hexbin(&hx_mix, &hy_mix)
                    .gridsize(12)
                    .cmap(Colormap::Plasma);
                ax.title("Hexbin");
            });
        })
        .save(dir.join("plotine_hist2d_hexbin.png"))?;

    // --- 3D helix (matplotlib gallery: Parametric curve) ---
    let t3: Vec<f64> = (0..200).map(|i| i as f64 * 4.0 * PI / 199.0).collect();
    let hx: Vec<f64> = t3.iter().map(|t| t.cos()).collect();
    let hy: Vec<f64> = t3.iter().map(|t| t.sin()).collect();
    let hz = t3.clone();
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.plot3d(&hx, &hy, &hz)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("helix");
            ax.title("3D Helix").legend(Legend::TopRight);
        })
        .save(dir.join("plotine_helix_3d.png"))?;

    // --- 3D scatter ---
    let n3 = 200usize;
    let sx3: Vec<f64> = (0..n3)
        .map(|i| ((i as f64) * 0.1).cos() + (i as f64 * 0.037).sin() * 0.3)
        .collect();
    let sy3: Vec<f64> = (0..n3)
        .map(|i| ((i as f64) * 0.1).sin() + (i as f64 * 0.029).cos() * 0.3)
        .collect();
    let sz3: Vec<f64> = (0..n3).map(|i| i as f64 / n3 as f64 * 10.0).collect();
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.scatter3d(&sx3, &sy3, &sz3).color(Color::STEEL_BLUE);
            ax.title("3D Scatter");
        })
        .save(dir.join("plotine_scatter_3d.png"))?;

    // --- 3D surface (matplotlib gallery: surface3d coolwarm sombrero) ---
    let sn = 40usize; // arange(-5,5,0.25) → 40 samples
    let surf_x: Vec<f64> = (0..sn).map(|i| -5.0 + i as f64 * 0.25).collect();
    let surf_y = surf_x.clone();
    let mut surf_z = Vec::with_capacity(sn * sn);
    for &yv in &surf_y {
        for &xv in &surf_x {
            let r = (xv * xv + yv * yv).sqrt();
            surf_z.push(r.sin());
        }
    }
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.surface(sn, sn, &surf_z)
                .x(&surf_x)
                .y(&surf_y)
                .cmap(Colormap::Plasma)
                .alpha(0.95);
            ax.title("3D Surface").elev(30.0).azim(-60.0);
        })
        .save(dir.join("plotine_surface_3d.png"))?;

    // --- 3D gaussian surface (gallery 44 style) ---
    let gn = 25usize;
    let g_x: Vec<f64> = (0..gn)
        .map(|i| (i as f64 / (gn - 1) as f64) * 4.0 - 2.0)
        .collect();
    let g_y = g_x.clone();
    let mut g_z = Vec::with_capacity(gn * gn);
    for &yv in &g_y {
        for &xv in &g_x {
            g_z.push((-(xv * xv + yv * yv) * 0.5).exp());
        }
    }
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.surface(gn, gn, &g_z)
                .x(&g_x)
                .y(&g_y)
                .cmap(Colormap::Plasma)
                .alpha(0.9);
            ax.title("3D Gaussian").elev(35.0).azim(-50.0);
        })
        .save(dir.join("plotine_gaussian_3d.png"))?;

    // --- 3D wireframe (matplotlib gallery: wire3d) ---
    let wn = 30usize;
    let wire_x: Vec<f64> = (0..wn)
        .map(|i| -3.0 + i as f64 * 6.0 / (wn - 1) as f64)
        .collect();
    let wire_y = wire_x.clone();
    let mut wire_z = Vec::with_capacity(wn * wn);
    for &yv in &wire_y {
        for &xv in &wire_x {
            wire_z.push(xv.sin() * yv.cos());
        }
    }
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.wireframe(wn, wn, &wire_z)
                .x(&wire_x)
                .y(&wire_y)
                .color(Color::STEEL_BLUE)
                .width(0.7);
            ax.title("3D Wireframe").elev(25.0).azim(-70.0);
        })
        .save(dir.join("plotine_wireframe_3d.png"))?;

    // --- 3D bar (matplotlib gallery: bars3d) ---
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.bar3d(
                [0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0],
                [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                [3.0, 5.0, 2.0, 4.0, 1.0, 6.0, 3.0, 2.0],
            )
            .dx(0.6)
            .dy(0.6)
            .color(Color::STEEL_BLUE)
            .alpha(0.85);
            ax.title("3D Bar").elev(30.0).azim(-55.0);
        })
        .save(dir.join("plotine_bar_3d.png"))?;

    // =====================================================================
    // M9–M13 feature pixel-align pairs (see scripts/pixel_align_features.py)
    // =====================================================================

    // --- M9: static render path shared with Figure::show (pixel path) ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
            ax.title("M9 static render")
                .x_label("x")
                .y_label("y")
                .grid(true);
        })
        .save(dir.join("plotine_m9_static.png"))?;

    // --- M10: animation frame 0 (fixed ylim; same as FuncAnimation) ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
            ax.title("M10 anim frame").y_range(-1.2, 1.2).grid(true);
        })
        .save(dir.join("plotine_m10_anim_frame.png"))?;

    // --- M11: PlateCarree + embedded NE 110m coastline ---
    // World extent is 360×180 (data aspect 2:1). Stock 5×3.5 axes (~1.44:1)
    // letterbox under `aspect_equal`; 7×3.5 stock box ≈ 2:1 so the map fills.
    Figure::new()
        .size(7.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.projection(GeoProjection::PlateCarree);
            ax.coastline()
                .color(Color::rgb(0x55, 0x55, 0x55))
                .width(0.7);
            ax.scatter([0.0, 116.4, -74.0], [51.5, 39.9, 40.7])
                .color(Color::CRIMSON)
                .size(4.5);
            // Lock ticks to match mpl pixel-align pair (avoid AutoLocator drift).
            ax.x_ticks([-150.0, -100.0, -50.0, 0.0, 50.0, 100.0, 150.0]);
            ax.y_ticks([-80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0]);
            ax.title("M11 Geo PlateCarree").grid(true);
        })
        .save(dir.join("plotine_m11_geo.png"))?;

    // --- M12: pyplot facade emits identical bytes to builder (see pyplot example);
    //     pixel-align target is the same figure as `line` / builder path. ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin(x)");
            ax.title("M12 builder (== pyplot)")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(dir.join("plotine_m12_pyplot.png"))?;

    // --- M13: inline mathtext integral (textstyle side limits, like mpl titles) ---
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
            ax.title(r"M13 mathtext $\int_0^1 x^2\,dx$")
                .x_label(r"$x$")
                .y_label(r"$y$")
                .grid(true);
        })
        .save(dir.join("plotine_m13_mathtext.png"))?;

    println!(
        "Wrote plotine comparison figures to {} (full static coverage + M9–M13)",
        dir.display()
    );
    println!("Next: python scripts/matplotlib_compare.py");
    println!("Then:  python scripts/pixel_align_features.py");
    Ok(())
}
