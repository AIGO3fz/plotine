//! Render the plotine gallery into `./gallery/`.
//!
//! ```bash
//! cargo run -p plotine --example gallery
//! ```

use std::f64::consts::PI;
use std::fs;
use std::path::Path;

use plotine::prelude::*;

fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| PlotError::io(e.to_string()))
}

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    if n <= 1 {
        return vec![start];
    }
    let step = (end - start) / (n - 1) as f64;
    (0..n).map(|i| start + step * i as f64).collect()
}

fn main() -> Result<()> {
    let out = Path::new("gallery");
    ensure_dir(out)?;

    // 01 empty
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.title("01 Empty Axes")
                .x_label("x")
                .y_label("y")
                .x_range(0.0, 1.0)
                .y_range(0.0, 1.0);
        })
        .save(out.join("01_empty.png"))?;

    // 02 sine
    let x = linspace(0.0, 10.0, 200);
    let sine: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &sine)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin");
            ax.title("02 Line")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight);
        })
        .save(out.join("02_line.png"))?;

    // 03 scatter
    let xs = linspace(0.0, 8.0, 35);
    let ys: Vec<f64> = xs
        .iter()
        .enumerate()
        .map(|(i, v)| 0.2 * v + (i as f64).sin() * 0.4)
        .collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.scatter(&xs, &ys).size(6.0).label("pts");
            ax.title("03 Scatter").legend(Legend::TopLeft);
        })
        .save(out.join("03_scatter.png"))?;

    // 04 bar
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.bar([1.0, 2.0, 3.0, 4.0], [4.0, 7.0, 2.0, 5.0])
                .label("n");
            ax.title("04 Bar").legend(Legend::TopRight);
        })
        .save(out.join("04_bar.png"))?;

    // 05 hist
    let data: Vec<f64> = (0..300)
        .map(|i| ((i as f64) * 0.11).sin() + ((i as f64) * 0.03).cos())
        .collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.hist(&data)
                .bins(16)
                .color(Color::MEDIUM_PURPLE)
                .label("hist");
            ax.title("05 Histogram").legend(Legend::TopRight);
        })
        .save(out.join("05_hist.png"))?;

    // 06 area
    let ya: Vec<f64> = x.iter().map(|v| (v * 0.7).sin().abs() + 0.15).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.area(&x, &ya)
                .color(Color::FOREST_GREEN)
                .alpha(0.4)
                .label("area");
            ax.title("06 Area").legend(Legend::TopRight);
        })
        .save(out.join("06_area.png"))?;

    // 07 errorbar
    let xe = [0.0, 1.0, 2.0, 3.0, 4.0];
    let ye = [1.1, 1.6, 1.3, 2.0, 1.8];
    let ee = [0.15, 0.2, 0.12, 0.25, 0.18];
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.errorbar(xe, ye, ee).label("err");
            ax.title("07 Errorbar").legend(Legend::TopLeft);
        })
        .save(out.join("07_errorbar.png"))?;

    // 08 multi-line
    let c = linspace(-std::f64::consts::PI, std::f64::consts::PI, 220);
    let siny: Vec<f64> = c.iter().map(|v| v.sin()).collect();
    let cosy: Vec<f64> = c.iter().map(|v| v.cos()).collect();
    let tany: Vec<f64> = c.iter().map(|v| v.tanh()).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&c, &siny).label("sin").width(2.0);
            ax.line(&c, &cosy).label("cos").width(2.0);
            ax.line(&c, &tany).label("tanh").width(2.0);
            ax.title("08 Multi-line")
                .y_range(-1.2, 1.2)
                .legend(Legend::BottomLeft);
        })
        .save(out.join("08_multiline.png"))?;

    // 09 loglog
    let xl: Vec<f64> = (0..50)
        .map(|i| 10f64.powf(-1.0 + i as f64 * 0.06))
        .collect();
    let yl: Vec<f64> = xl.iter().map(|v| 3.0 * v.powf(1.4)).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
            ax.line(&xl, &yl).width(2.0).label("power");
            ax.title("09 Log–Log").legend(Legend::TopLeft);
        })
        .save(out.join("09_loglog.png"))?;

    // 10 symlog dark
    let xd = linspace(-8.0, 8.0, 160);
    let yd: Vec<f64> = xd.iter().map(|v| v * v * v.signum()).collect();
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .theme(Theme::dark())
        .axes(|ax| {
            ax.y_scale(ScaleType::Symlog { linthresh: 1.0 });
            ax.line(&xd, &yd).width(2.0).label("x|x|");
            ax.title("10 Dark + Symlog").legend(Legend::TopRight);
        })
        .save(out.join("10_dark_symlog.png"))?;

    // 11 paper theme
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&x, &sine).color(Color::STEEL_BLUE).width(2.2);
            ax.title("11 Paper Theme").x_label("t").y_label("amp");
        })
        .save(out.join("11_paper.png"))?;

    // 12 scatter+line
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.scatter(&xs, &ys).size(5.0).label("data");
            let trend: Vec<f64> = xs.iter().map(|v| 0.2 * v).collect();
            ax.line(&xs, &trend).width(2.0).label("trend");
            ax.title("12 Scatter + Line").legend(Legend::TopLeft);
        })
        .save(out.join("12_scatter_line.png"))?;

    // 13 area+line overlay
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.area(&x, &ya).alpha(0.35).label("fill");
            ax.line(&x, &ya).width(1.8).label("edge");
            ax.title("13 Area Overlay").legend(Legend::TopRight);
        })
        .save(out.join("13_area_line.png"))?;

    // 14 SVG twin of sine
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&x, &sine).color(Color::CRIMSON).width(2.0);
            ax.title("14 SVG Line").x_label("x").y_label("y");
        })
        .save(out.join("14_line.svg"))?;

    // 15 bar+legend bottom
    Figure::new()
        .size(5.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.bar([1.0, 2.0, 3.0], [3.0, 5.0, 4.0])
                .color(Color::DARK_ORANGE)
                .label("A");
            ax.title("15 Bar Legend").legend(Legend::BottomRight);
        })
        .save(out.join("15_bar_legend.png"))?;

    // 16 subplots 2×2
    Figure::new()
        .size(7.0, 5.5)
        .dpi(150.0)
        .subplots(2, 2, |g| {
            g.hspace(0.3).wspace(0.25);
            g.at(0, 0, |ax| {
                ax.line(&x, &sine).color(Color::CRIMSON).width(1.8);
                ax.title("16a Line");
            });
            g.at(0, 1, |ax| {
                ax.scatter(&xs, &ys).size(4.0);
                ax.title("16b Scatter");
            });
            g.at(1, 0, |ax| {
                ax.bar([1.0, 2.0, 3.0], [3.0, 5.0, 2.0]);
                ax.title("16c Bar");
            });
            g.at(1, 1, |ax| {
                ax.hist(&ys).bins(10).color(Color::MEDIUM_PURPLE);
                ax.title("16d Hist");
            });
        })
        .save(out.join("16_subplots.png"))?;

    // 17 datetime x-axis
    let t0 = 1_577_836_800_f64;
    let tx: Vec<f64> = (0..20).map(|i| t0 + i as f64 * 86_400.0).collect();
    let ty: Vec<f64> = (0..20)
        .map(|i| 8.0 + (i as f64 * 0.4).sin() * 1.5)
        .collect();
    Figure::new()
        .size(6.0, 3.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&tx, &ty).color(Color::STEEL_BLUE).width(2.0);
            ax.x_datetime(true)
                .title("17 Datetime Axis")
                .x_label("date (UTC)")
                .y_label("value");
        })
        .save(out.join("17_datetime.png"))?;

    // 18 heatmap
    let hm: Vec<f64> = (0..48)
        .map(|i| {
            let r = (i / 8) as f64 / 5.0;
            let c = (i % 8) as f64 / 7.0;
            (c * 3.0).sin() * (r * 2.0).cos()
        })
        .collect();
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.heatmap(6, 8, &hm).cmap(Colormap::Inferno).colorbar(true);
            ax.title("18 Heatmap").x_label("x").y_label("y");
        })
        .save(out.join("18_heatmap.png"))?;

    // 19 boxplot
    let g1 = [1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 7.5];
    let g2 = [2.0, 2.4, 2.8, 3.2, 3.6, 4.2, 4.8];
    let g3 = [0.8, 1.2, 1.8, 2.2, 2.6, 3.0, 5.5];
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.boxplot([&g1[..], &g2[..], &g3[..]])
                .color(Color::STEEL_BLUE)
                .label("samples");
            ax.title("19 Boxplot")
                .x_label("group")
                .y_label("value")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(out.join("19_boxplot.png"))?;

    // 20 violin
    let v1 = [1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 5.0];
    let v2 = [2.0, 2.4, 2.8, 3.2, 3.6, 4.0, 4.5];
    let v3 = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 4.5];
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.violin([&v1[..], &v2[..], &v3[..]])
                .color(Color::MEDIUM_PURPLE)
                .alpha(0.5)
                .label("kde");
            ax.title("20 Violin")
                .x_label("group")
                .y_label("value")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(out.join("20_violin.png"))?;

    // 21 fill_between
    let x = linspace(0.0, 2.0 * PI, 80);
    let y1: Vec<f64> = x.iter().map(|v| v.sin() + 0.5).collect();
    let y2: Vec<f64> = x.iter().map(|v| v.sin() - 0.5).collect();
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.fill_between(&x, &y1, &y2)
                .color(Color::STEEL_BLUE)
                .alpha(0.35)
                .label("band");
            ax.line(&x, &y1).color(Color::CRIMSON).width(1.5);
            ax.line(&x, &y2).color(Color::CRIMSON).width(1.5);
            ax.title("21 Fill Between")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(out.join("21_fill_between.png"))?;

    // 22 step + stairs
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.step([0.0, 1.0, 2.0, 3.0], [1.0, 2.5, 1.5, 3.0])
                .mode(StepMode::Pre)
                .color(Color::STEEL_BLUE)
                .label("step");
            ax.stairs([0.0, 1.0, 2.0, 3.0, 4.0], [0.5, 1.5, 1.0, 2.0])
                .color(Color::CRIMSON)
                .label("stairs");
            ax.title("22 Step / Stairs")
                .legend(Legend::TopLeft)
                .grid(true);
        })
        .save(out.join("22_step_stairs.png"))?;

    // 23 stem
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.stem([0.0, 1.0, 2.0, 3.0, 4.0], [1.2, -0.8, 1.5, 0.3, -1.1])
                .color(Color::MEDIUM_PURPLE)
                .label("stem");
            ax.axhline(0.0).color(Color::SPINE).width(0.8);
            ax.title("23 Stem").legend(Legend::TopRight).grid(true);
        })
        .save(out.join("23_stem.png"))?;

    // 24 hlines / vlines / barh
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
            ax.title("24 BarH / Spans")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(out.join("24_barh_spans.png"))?;

    // 25 pie
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.pie([35.0, 25.0, 20.0, 20.0])
                .labels(["A", "B", "C", "D"]);
            ax.title("25 Pie").legend(Legend::TopRight);
        })
        .save(out.join("25_pie.png"))?;

    // 26 stackplot
    let sx = linspace(0.0, 10.0, 40);
    let s0: Vec<f64> = sx.iter().map(|v| 1.0 + 0.3 * v.sin()).collect();
    let s1: Vec<f64> = sx.iter().map(|v| 1.5 + 0.2 * (v * 0.7).cos()).collect();
    let s2: Vec<f64> = sx.iter().map(|v| 0.8 + 0.15 * (v * 1.3).sin()).collect();
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.stackplot(&sx, [&s0[..], &s1[..], &s2[..]])
                .labels(["low", "mid", "high"])
                .alpha(0.85);
            ax.title("26 Stackplot").legend(Legend::TopLeft).grid(true);
        })
        .save(out.join("26_stackplot.png"))?;

    // 27 eventplot + broken_barh
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .subplots(2, 1, |g| {
            g.at(0, 0, |ax| {
                ax.eventplot([
                    [1.0, 2.0, 5.0, 7.0].as_slice(),
                    [0.5, 3.0, 4.5].as_slice(),
                    [2.5, 6.0].as_slice(),
                ])
                .labels(["r1", "r2", "r3"]);
                ax.title("27a Eventplot").legend(Legend::TopRight);
            });
            g.at(1, 0, |ax| {
                ax.broken_barh([(10.0, 50.0), (100.0, 20.0), (150.0, 40.0)], (20.0, 9.0))
                    .color(Color::STEEL_BLUE)
                    .label("jobs");
                ax.broken_barh([(40.0, 30.0), (120.0, 50.0)], (35.0, 9.0))
                    .color(Color::CRIMSON)
                    .label("tasks");
                ax.title("27b Broken BarH")
                    .legend(Legend::TopRight)
                    .grid(true);
            });
        })
        .save(out.join("27_event_broken.png"))?;

    // 28 polygon + axhspan / axvspan
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.axvspan(1.0, 2.0)
                .color(Color::STEEL_BLUE)
                .alpha(0.2)
                .label("vspan");
            ax.axhspan(-0.2, 0.2)
                .color(Color::CRIMSON)
                .alpha(0.2)
                .label("hspan");
            ax.polygon([0.5, 2.5, 1.5], [0.5, 0.5, 1.5])
                .color(Color::FOREST_GREEN)
                .alpha(0.45)
                .label("poly");
            ax.line([0.0, 1.0, 2.0, 3.0], [0.0, 1.0, 0.2, 0.8])
                .color(Color::SPINE)
                .width(1.5);
            ax.title("28 Polygon / Spans")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(out.join("28_polygon_spans.png"))?;

    // 29 hist2d + hexbin
    let mut hx = Vec::new();
    let mut hy = Vec::new();
    for i in 0..400 {
        let t = i as f64 * 0.05;
        hx.push(t.sin() * 2.0 + (i % 17) as f64 * 0.05);
        hy.push(t.cos() * 2.0 + (i % 13) as f64 * 0.04);
    }
    Figure::new()
        .size(7.0, 3.5)
        .dpi(150.0)
        .subplots(1, 2, |g| {
            g.at(0, 0, |ax| {
                ax.hist2d(&hx, &hy).bins(16).cmap(Colormap::Viridis);
                ax.title("29a Hist2D");
            });
            g.at(0, 1, |ax| {
                ax.hexbin(&hx, &hy).gridsize(12).cmap(Colormap::Plasma);
                ax.title("29b Hexbin");
            });
        })
        .save(out.join("29_hist2d_hexbin.png"))?;

    // 30 contour / contourf
    let mut z = Vec::with_capacity(40 * 40);
    for r in 0..40 {
        for c in 0..40 {
            let x = c as f64 * 0.2 - 4.0;
            let y = r as f64 * 0.2 - 4.0;
            z.push((-x * x - y * y).exp() * 2.0 + 0.3 * (x * 0.8).sin());
        }
    }
    Figure::new()
        .size(7.0, 3.5)
        .dpi(150.0)
        .subplots(1, 2, |g| {
            g.at(0, 0, |ax| {
                ax.contourf(40, 40, &z).levels(12).cmap(Colormap::Viridis);
                ax.contour(40, 40, &z)
                    .levels(12)
                    .color(Color::SPINE.with_alpha(0.55))
                    .width(0.6);
                ax.title("30a Contourf + Contour");
            });
            g.at(0, 1, |ax| {
                ax.pcolormesh(linspace(-4.0, 4.0, 21), linspace(-4.0, 4.0, 21), {
                    let mut v = Vec::with_capacity(20 * 20);
                    for r in 0..20 {
                        for c in 0..20 {
                            let x = c as f64 * 0.4 - 4.0;
                            let y = r as f64 * 0.4 - 4.0;
                            v.push(x * y * 0.1);
                        }
                    }
                    v
                })
                .cmap(Colormap::Plasma);
                ax.title("30b Pcolormesh");
            });
        })
        .save(out.join("30_contour_pcolor.png"))?;

    // 31 spy
    let mut sparse = vec![0.0; 12 * 12];
    for i in 0..12 {
        sparse[i * 12 + i] = 1.0;
        if i + 2 < 12 {
            sparse[i * 12 + i + 2] = 0.5;
        }
    }
    Figure::new()
        .size(4.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.spy(12, 12, &sparse)
                .color(Color::STEEL_BLUE)
                .marker_size(5.0);
            ax.title("31 Spy");
        })
        .save(out.join("31_spy.png"))?;

    // 32 quiver + streamplot
    let n = 12usize;
    let mut qx = Vec::new();
    let mut qy = Vec::new();
    let mut qu = Vec::new();
    let mut qv = Vec::new();
    let mut su = vec![0.0; n * n];
    let mut sv = vec![0.0; n * n];
    for r in 0..n {
        for c in 0..n {
            let x = c as f64;
            let y = r as f64;
            let cx = x - 5.5;
            let cy = y - 5.5;
            qx.push(x);
            qy.push(y);
            qu.push(-cy * 0.3);
            qv.push(cx * 0.3);
            su[r * n + c] = -cy;
            sv[r * n + c] = cx;
        }
    }
    Figure::new()
        .size(7.0, 3.5)
        .dpi(150.0)
        .subplots(1, 2, |g| {
            g.at(0, 0, |ax| {
                ax.quiver(&qx, &qy, &qu, &qv)
                    .color(Color::STEEL_BLUE)
                    .quiverkey(1.0, "1 unit");
                ax.title("32a Quiver");
            });
            g.at(0, 1, |ax| {
                ax.streamplot(n, n, &su, &sv)
                    .density(1.2)
                    .color(Color::CRIMSON)
                    .width(0.9);
                ax.title("32b Streamplot");
            });
        })
        .save(out.join("32_quiver_stream.png"))?;

    // 33 polar + cartesian subplot mix
    let th: Vec<f64> = (0..120).map(|i| i as f64 * PI / 60.0).collect();
    let pr: Vec<f64> = th.iter().map(|t| 1.0 + 0.35 * (2.0 * t).cos()).collect();
    Figure::new()
        .size(7.0, 3.5)
        .dpi(150.0)
        .subplots(1, 2, |g| {
            g.wspace(0.3);
            g.at(0, 0, |ax| {
                ax.polar_line(&th, &pr)
                    .color(Color::MEDIUM_PURPLE)
                    .width(2.0);
                ax.title("33a Polar");
            });
            g.at(0, 1, |ax| {
                ax.line(&th, &pr).color(Color::STEEL_BLUE).width(1.75);
                ax.title("33b Cartesian")
                    .x_label("theta")
                    .y_label("r")
                    .grid(true);
            });
        })
        .save(out.join("33_polar_mix.png"))?;

    // 34 text + annotate (paper-style callouts)
    let x = linspace(0.0, 2.0 * PI, 80);
    let y: Vec<f64> = x.iter().map(|t| t.sin()).collect();
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
            ax.title("34 Text + Annotate")
                .x_label("x")
                .y_label("y")
                .legend(Legend::BottomRight)
                .grid(true);
        })
        .save(out.join("34_annotate.png"))?;

    // 35 twin Y (shared x, independent right axis)
    let tx = linspace(0.0, 10.0, 60);
    let left: Vec<f64> = tx.iter().map(|t| (t * 0.7).sin() * 2.0 + 2.0).collect();
    let right: Vec<f64> = tx.iter().map(|t| t * t * 0.8 + 5.0).collect();
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&tx, &left)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("amplitude");
            ax.y_label("amplitude");
            ax.twin_y(|ax2| {
                ax2.line(&tx, &right)
                    .color(Color::CRIMSON)
                    .width(2.0)
                    .label("energy");
                ax2.y_label("energy");
            });
            ax.title("35 Twin Y")
                .x_label("t")
                .legend(Legend::TopLeft)
                .grid(true);
        })
        .save(out.join("35_twin_y.png"))?;

    // Also emit a vector PDF (LaTeX-friendly) for the twin-y paper figure.
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&tx, &left)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("amplitude");
            ax.y_label("amplitude");
            ax.twin_y(|ax2| {
                ax2.line(&tx, &right)
                    .color(Color::CRIMSON)
                    .width(2.0)
                    .label("energy");
                ax2.y_label("energy");
            });
            ax.title("35 Twin Y (PDF)")
                .x_label("t")
                .legend(Legend::TopLeft)
                .grid(true);
        })
        .save(out.join("35_twin_y.pdf"))?;

    // 36 Unicode math labels (Greek + super/subscripts; no LaTeX engine)
    let mx = linspace(0.0, 2.0 * PI, 100);
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
            ax.title(r"36 Mathtext: $\alpha$-decay")
                .x_label(r"$t$ (s)")
                .y_label(r"$\theta$ (rad)")
                .legend(Legend::TopRight)
                .grid(true);
            ax.text(4.2, 0.35, r"$H_2O$ ref")
                .color(Color::LABEL)
                .size(10.0);
        })
        .save(out.join("36_math_labels.png"))?;

    // 37 categorical x-axis
    let cats = ["A", "B", "C", "D"];
    let cx = category_indices(cats.len());
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.x_categories(cats);
            ax.bar(&cx, [4.0, 7.0, 3.0, 5.5])
                .color(Color::STEEL_BLUE)
                .label("counts");
            ax.title("37 Categories")
                .x_label("group")
                .y_label("value")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(out.join("37_categories.png"))?;

    // 38 LogNorm heatmap
    let mut z = Vec::with_capacity(8 * 8);
    for r in 0..8 {
        for c in 0..8 {
            z.push(10f64.powf((r + c) as f64 * 0.35));
        }
    }
    Figure::new()
        .size(5.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.heatmap(8, 8, &z)
                .cmap(Colormap::Viridis)
                .norm(Norm::Log)
                .colorbar(true);
            ax.title("38 LogNorm Heatmap");
        })
        .save(out.join("38_lognorm_heatmap.png"))?;

    // 39 twin X (shared y, independent top x)
    let ty = linspace(0.0, 5.0, 40);
    let bottom_x: Vec<f64> = ty.iter().map(|v| v * 0.8).collect();
    let top_x: Vec<f64> = ty.iter().map(|v| 10.0 + v * v).collect();
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&bottom_x, &ty)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("linear x");
            ax.x_label("linear");
            ax.y_label("y");
            ax.twin_x(|ax2| {
                ax2.line(&top_x, &ty)
                    .color(Color::CRIMSON)
                    .width(2.0)
                    .label("quad x");
                ax2.x_label("quadratic");
            });
            ax.title("39 Twin X").legend(Legend::BottomRight).grid(true);
        })
        .save(out.join("39_twin_x.png"))?;

    // 40 contour clabel
    let mut z40 = Vec::with_capacity(30 * 30);
    for r in 0..30 {
        for c in 0..30 {
            let x = c as f64 * 0.25 - 3.5;
            let y = r as f64 * 0.25 - 3.5;
            z40.push((-x * x - y * y).exp() * 2.0);
        }
    }
    Figure::new()
        .size(5.5, 4.5)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.contourf(30, 30, &z40)
                .levels(8)
                .cmap(Colormap::Viridis)
                .colorbar(false);
            ax.contour(30, 30, &z40)
                .levels(8)
                .color(Color::SPINE)
                .width(0.9)
                .clabel(true)
                .clabel_size(8.0)
                .clabel_color(Color::LABEL);
            ax.title("40 Contour Labels").grid(false);
        })
        .save(out.join("40_clabel.png"))?;

    // 41 wind barbs
    let mut bx = Vec::new();
    let mut by = Vec::new();
    let mut bu = Vec::new();
    let mut bv = Vec::new();
    for r in 0..6 {
        for c in 0..8 {
            let x = c as f64;
            let y = r as f64;
            let speed = 5.0 + (c as f64) * 10.0 + (r as f64) * 5.0;
            let ang = (r as f64 + c as f64) * 0.35;
            bx.push(x);
            by.push(y);
            bu.push(speed * ang.cos());
            bv.push(speed * ang.sin());
        }
    }
    Figure::new()
        .size(6.5, 4.5)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.barbs(&bx, &by, &bu, &bv)
                .length(10.0)
                .width(1.1)
                .color(Color::STEEL_BLUE);
            ax.title("41 Wind Barbs")
                .x_label("x")
                .y_label("y")
                .grid(true);
        })
        .save(out.join("41_barbs.png"))?;

    // ─── 3D plots ─────────────────────────────────────────────────────────────

    // 42 3D helix
    let t3 = linspace(0.0, 4.0 * PI, 200);
    let hx: Vec<f64> = t3.iter().map(|t| t.cos()).collect();
    let hy: Vec<f64> = t3.iter().map(|t| t.sin()).collect();
    let hz: Vec<f64> = t3.clone();
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.plot3d(&hx, &hy, &hz)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("helix");
            ax.title("42 3D Helix");
            ax.legend(Legend::TopRight);
        })
        .save(out.join("42_helix_3d.png"))?;

    // 43 3D scatter
    let n3 = 200;
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
            ax.title("43 3D Scatter");
        })
        .save(out.join("43_scatter_3d.png"))?;

    // 44 3D surface
    let sn = 25;
    let surf_x: Vec<f64> = (0..sn)
        .map(|i| (i as f64 / (sn - 1) as f64) * 4.0 - 2.0)
        .collect();
    let surf_y = surf_x.clone();
    let mut surf_z = Vec::with_capacity(sn * sn);
    for &yv in &surf_y {
        for &xv in &surf_x {
            surf_z.push((-(xv * xv + yv * yv) * 0.5).exp());
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
                .alpha(0.9);
            ax.title("44 3D Surface");
            ax.elev(35.0).azim(-50.0);
        })
        .save(out.join("44_surface_3d.png"))?;

    // 45 3D wireframe
    let wn = 15;
    let wire_x: Vec<f64> = (0..wn)
        .map(|i| (i as f64 / (wn - 1) as f64) * 6.0 - 3.0)
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
                .width(0.9);
            ax.title("45 3D Wireframe");
            ax.elev(25.0).azim(-70.0);
        })
        .save(out.join("45_wireframe_3d.png"))?;

    // 46 3D bar
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
            ax.title("46 3D Bar");
            ax.elev(30.0).azim(-55.0);
        })
        .save(out.join("46_bar_3d.png"))?;

    // 47 inset_axes zoom
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
            ax.title("47 Inset Axes").x_label("x").y_label("y");
            ax.inset_axes([0.55, 0.55, 0.4, 0.4], |inset| {
                inset
                    .line(&ix[..20], &iy[..20])
                    .color(Color::CRIMSON)
                    .width(1.5);
                inset.title("zoom");
            });
        })
        .save(out.join("47_inset_axes.png"))?;

    // 48 secondary axes (rad↔deg + °C↔°F)
    let th48: Vec<f64> = (0..60).map(|i| i as f64 * PI / 30.0).collect();
    let y48: Vec<f64> = th48.iter().map(|t| t.sin()).collect();
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&th48, &y48).color(Color::STEEL_BLUE).width(2.0);
            ax.title("48 Secondary Axes")
                .x_label("radians")
                .y_label("amplitude");
            ax.secondary_x(f64::to_degrees, f64::to_radians, |sec| {
                sec.label("degrees");
            });
        })
        .save(out.join("48_secondary_axes.png"))?;

    // 49 subplot span (tall left + two right)
    let sx: Vec<f64> = (0..40).map(|i| i as f64 * 0.15).collect();
    let sy: Vec<f64> = sx.iter().map(|v| v.sin()).collect();
    Figure::new()
        .size(6.5, 4.5)
        .dpi(150.0)
        .subplots(2, 2, |g| {
            g.hspace(0.28).wspace(0.22);
            g.at_span(0, 0, 2, 1, |ax| {
                ax.line(&sx, &sy).color(Color::STEEL_BLUE).width(2.0);
                ax.title("49 Span (tall)").y_label("y");
            });
            g.at(0, 1, |ax| {
                ax.scatter(&sx, &sy).color(Color::CRIMSON).size(3.0);
                ax.title("top-right");
            });
            g.at(1, 1, |ax| {
                ax.hist(&sy).bins(10).color(Color::FOREST_GREEN);
                ax.title("bottom-right");
            });
        })
        .save(out.join("49_subplot_span.png"))?;

    // 50 triangular mesh (tripcolor + tricontour) + new colormaps
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
            ax.title("50 Tripcolor + Tricontour");
        })
        .save(out.join("50_tripcolor_tricontour.png"))?;

    // 51 table
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
            ax.title("51 Table")
                .legend(Legend::BottomLeft)
                .grid(true)
                .grid_axis(GridAxis::Y);
        })
        .save(out.join("51_table.png"))?;

    // 52 mathtext (layout engine: scripts + frac + sqrt + matrix)
    let mx52 = linspace(0.0, 2.0 * PI, 100);
    let my52: Vec<f64> = mx52
        .iter()
        .map(|t| (2.0 * t).sin() * (-0.15 * t).exp())
        .collect();
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&mx52, &my52)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label(r"$e^{-0.15t}\sin(2t)$");
            ax.title(r"52 Mathtext: $\sqrt{a^2+b^2}$")
                .x_label(r"$t$ (s)")
                .y_label(r"$\theta$ (rad)")
                .legend(Legend::TopRight)
                .grid(true);
            ax.text(3.2, 0.55, r"$H_2O:\frac{1}{2}mv^2$")
                .color(Color::LABEL)
                .size(11.0);
            ax.text(3.2, 0.25, r"$\sqrt[3]{8}=2$")
                .color(Color::LABEL)
                .size(11.0);
            ax.text(
                0.2,
                -0.55,
                r"$R=\begin{pmatrix} a & b \\ c & d \end{pmatrix}$",
            )
            .color(Color::LABEL)
            .size(11.0);
        })
        .save(out.join("52_mathtext.png"))?;

    // 53 3D contour (iso-z curves)
    let cn = 30;
    let cx53: Vec<f64> = (0..cn)
        .map(|i| (i as f64 / (cn - 1) as f64) * 4.0 - 2.0)
        .collect();
    let cy53 = cx53.clone();
    let mut cz53 = Vec::with_capacity(cn * cn);
    for &yv in &cy53 {
        for &xv in &cx53 {
            cz53.push((-(xv * xv + yv * yv) * 0.5).exp());
        }
    }
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.contour3d(cn, cn, &cz53)
                .x(&cx53)
                .y(&cy53)
                .levels(10)
                .width(1.2);
            ax.title("53 3D Contour");
            ax.elev(30.0).azim(-60.0);
        })
        .save(out.join("53_contour_3d.png"))?;

    // 54 3D quiver
    let mut qx54 = Vec::new();
    let mut qy54 = Vec::new();
    let mut qz54 = Vec::new();
    let mut qu54 = Vec::new();
    let mut qv54 = Vec::new();
    let mut qw54 = Vec::new();
    for i in 0..5 {
        for j in 0..5 {
            for k in 0..3 {
                let x = i as f64 - 2.0;
                let y = j as f64 - 2.0;
                let z = k as f64;
                qx54.push(x);
                qy54.push(y);
                qz54.push(z);
                qu54.push(-y * 0.3);
                qv54.push(x * 0.3);
                qw54.push(0.15);
            }
        }
    }
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.quiver3d(&qx54, &qy54, &qz54, &qu54, &qv54, &qw54)
                .scale(1.0)
                .color(Color::STEEL_BLUE)
                .width(1.2);
            ax.title("54 3D Quiver");
            ax.elev(25.0).azim(-50.0);
        })
        .save(out.join("54_quiver_3d.png"))?;

    // 55 linestyles (Solid / Dashed / Dotted / DashDot)
    let xl = linspace(0.0, 2.0 * PI, 80);
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&xl, xl.iter().map(|t| t.sin()).collect::<Vec<_>>())
                .width(2.0)
                .linestyle(LineStyle::Solid)
                .label("Solid");
            ax.line(&xl, xl.iter().map(|t| t.sin() + 0.4).collect::<Vec<_>>())
                .width(2.0)
                .linestyle(LineStyle::Dashed)
                .label("Dashed");
            ax.line(&xl, xl.iter().map(|t| t.sin() + 0.8).collect::<Vec<_>>())
                .width(2.0)
                .linestyle(LineStyle::Dotted)
                .label("Dotted");
            ax.line(&xl, xl.iter().map(|t| t.sin() + 1.2).collect::<Vec<_>>())
                .width(2.0)
                .linestyle(LineStyle::DashDot)
                .label("DashDot");
            ax.title("55 LineStyles")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(out.join("55_linestyles.png"))?;

    // 56 marker styles
    let markers = [
        MarkerStyle::Circle,
        MarkerStyle::Square,
        MarkerStyle::Triangle,
        MarkerStyle::TriangleDown,
        MarkerStyle::Diamond,
        MarkerStyle::Plus,
        MarkerStyle::Cross,
        MarkerStyle::Star,
        MarkerStyle::Point,
    ];
    Figure::new()
        .size(6.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            for (i, m) in markers.iter().enumerate() {
                let y0 = i as f64;
                let xm: Vec<f64> = (0..8).map(|j| j as f64).collect();
                let ym: Vec<f64> = xm.iter().map(|_| y0).collect();
                ax.scatter(&xm, &ym)
                    .marker(*m)
                    .size(10.0)
                    .label(format!("{m:?}"));
            }
            ax.title("56 MarkerStyles")
                .y_range(-0.5, 8.5)
                .legend(Legend::Right)
                .legend_ncol(1)
                .grid(true)
                .grid_axis(GridAxis::X);
        })
        .save(out.join("56_markers.png"))?;

    // 57 despine + minor ticks + axline + aspect_equal
    let xa = linspace(-1.0, 1.0, 40);
    let ya: Vec<f64> = xa.iter().map(|v| v * v).collect();
    Figure::new()
        .size(5.0, 5.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.scatter(&xa, &ya)
                .marker(MarkerStyle::Diamond)
                .size(7.0)
                .color(Color::STEEL_BLUE)
                .label("y=x²");
            ax.axline((-1.0, -1.0), (1.0, 1.0))
                .color(Color::CRIMSON)
                .linestyle(LineStyle::Dashed)
                .width(1.5)
                .label("y=x");
            ax.aspect_equal(true)
                .despine()
                .minor_ticks(true)
                .title("57 Despine · Minor · Axline · Equal")
                .legend(Legend::UpperCenter)
                .grid(true);
        })
        .save(out.join("57_style_chrome.png"))?;

    // 58 subplot_mosaic + sharex/sharey + suptitle
    let xm58 = linspace(0.0, 4.0 * PI, 120);
    let s1: Vec<f64> = xm58.iter().map(|t| t.sin()).collect();
    let s2: Vec<f64> = xm58.iter().map(|t| (1.5 * t).cos()).collect();
    Figure::new()
        .size(7.0, 5.0)
        .dpi(150.0)
        .suptitle("58 Mosaic + Suptitle")
        .subplot_mosaic("AAB;CCD", |name, ax| match name {
            'A' => {
                ax.line(&xm58, &s1).color(Color::CRIMSON).width(1.8);
                ax.title("A").grid(true);
            }
            'B' => {
                ax.scatter(&xm58, &s2)
                    .marker(MarkerStyle::Triangle)
                    .size(4.0)
                    .color(Color::STEEL_BLUE);
                ax.title("B");
            }
            'C' => {
                ax.area(&xm58, &s1).color(Color::FOREST_GREEN).alpha(0.35);
                ax.title("C").grid(true);
            }
            'D' => {
                ax.hist(&s1).bins(12).color(Color::MEDIUM_PURPLE);
                ax.title("D");
            }
            _ => {}
        })
        .save(out.join("58_mosaic_suptitle.png"))?;

    // 59 sharex / sharey
    Figure::new()
        .size(6.5, 4.5)
        .dpi(150.0)
        .suptitle("59 ShareX / ShareY")
        .subplots(2, 2, |g| {
            g.sharex(true).sharey(true).hspace(0.25).wspace(0.2);
            g.at(0, 0, |ax| {
                ax.line(&xm58, &s1).color(Color::CRIMSON);
                ax.title("TL");
            });
            g.at(0, 1, |ax| {
                ax.line(&xm58, &s2).color(Color::STEEL_BLUE);
                ax.title("TR");
            });
            g.at(1, 0, |ax| {
                ax.scatter(&xm58, &s1).marker(MarkerStyle::Square).size(3.0);
                ax.title("BL").grid(true);
            });
            g.at(1, 1, |ax| {
                ax.line(
                    &xm58,
                    xm58.iter().map(|t| 0.5 * t.sin()).collect::<Vec<_>>(),
                )
                .linestyle(LineStyle::DashDot);
                ax.title("BR");
            });
        })
        .save(out.join("59_share_axes.png"))?;

    // 60 tricontourf (auto Delaunay) + colorbar_label
    let mut trx = Vec::new();
    let mut try_ = Vec::new();
    let mut trz = Vec::new();
    for i in 0..8 {
        for j in 0..8 {
            let x = i as f64 / 7.0 * 2.0 - 1.0;
            let y = j as f64 / 7.0 * 2.0 - 1.0;
            trx.push(x);
            try_.push(y);
            trz.push((-(x * x + y * y) * 2.0).exp());
        }
    }
    Figure::new()
        .size(6.0, 4.5)
        .dpi(150.0)
        .axes(|ax| {
            ax.tricontourf(&trx, &try_, &trz)
                .levels(10)
                .cmap(Colormap::Turbo)
                .colorbar(true);
            ax.colorbar_label("intensity");
            ax.title("60 Tricontourf + Colorbar Label (auto Delaunay)");
        })
        .save(out.join("60_tricontourf.png"))?;

    // 61 legend ncol + multi series
    Figure::new()
        .size(6.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            for i in 0..6 {
                let phase = i as f64 * 0.4;
                let y: Vec<f64> = xl
                    .iter()
                    .map(|t| (t + phase).sin() + i as f64 * 0.15)
                    .collect();
                ax.line(&xl, &y).width(1.6).label(format!("s{i}"));
            }
            ax.title("61 Legend NCol")
                .legend(Legend::UpperCenter)
                .legend_ncol(3)
                .grid(true);
        })
        .save(out.join("61_legend_ncol.png"))?;

    // 62 mathtext accents
    Figure::new()
        .size(6.0, 3.8)
        .dpi(150.0)
        .theme(Theme::paper())
        .axes(|ax| {
            ax.line(&xl, xl.iter().map(|t| t.sin()).collect::<Vec<_>>())
                .width(2.0)
                .color(Color::STEEL_BLUE);
            ax.title(r"62 Accents: $\hat{x},\bar{y},\vec{v},\tilde{n},\dot{a},\ddot{b}$")
                .x_label(r"$\overline{AB}$")
                .y_label(r"$\underline{f}$")
                .despine()
                .grid(true);
            ax.text(3.5, 0.6, r"$\hat{\alpha}+\vec{\beta}$").size(14.0);
        })
        .save(out.join("62_accents.png"))?;

    // 63 colormap sampler (named maps via FromStr)
    let names = [
        "viridis", "plasma", "inferno", "turbo", "coolwarm", "seismic", "jet", "spectral",
    ];
    Figure::new()
        .size(7.0, 5.5)
        .dpi(150.0)
        .suptitle("63 Colormaps (FromStr)")
        .subplots(2, 4, |g| {
            g.hspace(0.35).wspace(0.25);
            for (i, name) in names.iter().enumerate() {
                let row = i / 4;
                let col = i % 4;
                let cmap: Colormap = name.parse().unwrap_or(Colormap::Viridis);
                g.at(row, col, |ax| {
                    let n = 20;
                    let mut z = Vec::with_capacity(n * n);
                    for r in 0..n {
                        for c in 0..n {
                            z.push((r + c) as f64);
                        }
                    }
                    ax.heatmap(n, n, &z).cmap(cmap).colorbar(false);
                    ax.title(*name);
                });
            }
        })
        .save(out.join("63_colormaps.png"))?;

    // 64 SVG + PDF of style showcase
    Figure::new()
        .size(5.5, 3.8)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&xl, xl.iter().map(|t| t.cos()).collect::<Vec<_>>())
                .linestyle(LineStyle::Dashed)
                .width(2.0)
                .label("cos");
            ax.scatter(
                xl.iter().step_by(8).copied().collect::<Vec<_>>(),
                xl.iter().step_by(8).map(|t| t.cos()).collect::<Vec<_>>(),
            )
            .marker(MarkerStyle::Star)
            .size(9.0)
            .label("samples");
            ax.despine()
                .minor_ticks(true)
                .title("64 Style → SVG/PDF")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save(out.join("64_style.svg"))?;
    Figure::new()
        .size(5.5, 3.8)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&xl, xl.iter().map(|t| t.cos()).collect::<Vec<_>>())
                .linestyle(LineStyle::DashDot)
                .width(2.0);
            ax.despine().minor_ticks(true).title("64 Style PDF");
        })
        .save(out.join("64_style.pdf"))?;

    // 65 hatch + Legend::Best
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.bar([0.0, 1.0, 2.0, 3.0], [3.0, 5.0, 2.5, 4.0])
                .color(Color::STEEL_BLUE)
                .hatch(Hatch::Diagonal)
                .label("hatched");
            ax.line([0.0, 1.0, 2.0, 3.0], [1.0, 4.5, 2.0, 3.5])
                .color(Color::CRIMSON)
                .width(2.0)
                .label("series");
            ax.title("65 Hatch + Legend::Best")
                .legend(Legend::Best)
                .grid(true)
                .grid_axis(GridAxis::Y);
        })
        .save(out.join("65_hatch_legend_best.png"))?;

    // 66 grid linestyle + fontsize + outside legend
    Figure::new()
        .size(6.2, 3.8)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&xl, xl.iter().map(|t| t.sin()).collect::<Vec<_>>())
                .width(2.0)
                .color(Color::STEEL_BLUE)
                .label("sin");
            ax.line(&xl, xl.iter().map(|t| (t * 0.7).cos()).collect::<Vec<_>>())
                .width(1.8)
                .linestyle(LineStyle::Dashed)
                .color(Color::CRIMSON)
                .label("cos");
            ax.title("66 Style chrome")
                .title_fontsize(14.0)
                .x_label("x")
                .x_label_fontsize(11.0)
                .y_label("y")
                .y_label_fontsize(11.0)
                .grid(true)
                .grid_linestyle(LineStyle::Dashed)
                .legend(Legend::OutsideUpperRight);
        })
        .save(out.join("66_grid_fontsize_legend_out.png"))?;

    // 67 tick formatter + legend linestyle
    Figure::new()
        .size(6.0, 3.8)
        .dpi(150.0)
        .axes(|ax| {
            ax.line(&xl, xl.iter().map(|t| t.sin().abs()).collect::<Vec<_>>())
                .width(2.0)
                .color(Color::STEEL_BLUE)
                .label("abs sin");
            ax.line(
                &xl,
                xl.iter()
                    .map(|t| 0.5 * (t * 0.5).cos() + 0.5)
                    .collect::<Vec<_>>(),
            )
            .width(1.8)
            .linestyle(LineStyle::DashDot)
            .color(Color::CRIMSON)
            .label("shifted");
            ax.y_tick_formatter(TickFormatter::percent(0));
            ax.title("67 Formatter + legend dash")
                .legend(Legend::OutsideUpperRight)
                .grid(true)
                .grid_linestyle(LineStyle::Dotted);
        })
        .save(out.join("67_tick_formatter.png"))?;

    // 68 patches: rectangle / circle / ellipse
    Figure::new()
        .size(5.5, 4.0)
        .dpi(150.0)
        .axes(|ax| {
            ax.rectangle(0.5, 0.5, 2.0, 1.2)
                .color(Color::STEEL_BLUE)
                .alpha(0.35)
                .hatch(Hatch::Diagonal)
                .label("rect");
            ax.circle(4.0, 2.0, 0.9)
                .color(Color::CRIMSON)
                .alpha(0.35)
                .label("circle");
            ax.ellipse(2.5, 3.2, 2.4, 1.2)
                .color(Color::FOREST_GREEN)
                .alpha(0.35)
                .label("ellipse");
            ax.title("68 Patches")
                .x_range(0.0, 6.0)
                .y_range(0.0, 4.5)
                .legend(Legend::OutsideUpperRight)
                .grid(true)
                .grid_linestyle(LineStyle::Dotted);
        })
        .save(out.join("68_patches.png"))?;

    // 69 geographic map (PlateCarree + Mercator)
    Figure::new()
        .size(7.0, 3.6)
        .dpi(150.0)
        .subplots(1, 2, |g| {
            g.wspace(0.2);
            g.at(0, 0, |ax| {
                ax.projection(GeoProjection::PlateCarree);
                ax.coastline()
                    .color(Color::rgb(0x4a, 0x55, 0x64))
                    .width(0.7);
                ax.scatter([0.0, 116.4, -74.0, 139.7], [51.5, 39.9, 40.7, 35.7])
                    .color(Color::CRIMSON)
                    .size(4.5)
                    .label("cities");
                ax.title("69a PlateCarree")
                    .legend(Legend::BottomLeft)
                    .grid(true)
                    .grid_linestyle(LineStyle::Dotted);
            });
            g.at(0, 1, |ax| {
                ax.projection(GeoProjection::Mercator);
                ax.coastline()
                    .color(Color::rgb(0x4a, 0x55, 0x64))
                    .width(0.7);
                let track_lon = linspace(-120.0, 20.0, 40);
                let track_lat: Vec<f64> = track_lon
                    .iter()
                    .enumerate()
                    .map(|(i, _)| 10.0 + (i as f64) * 0.8)
                    .collect();
                ax.line(&track_lon, &track_lat)
                    .color(Color::STEEL_BLUE)
                    .width(1.8)
                    .label("track");
                ax.title("69b Mercator")
                    .legend(Legend::BottomLeft)
                    .grid(true)
                    .grid_linestyle(LineStyle::Dotted);
            });
        })
        .save(out.join("69_geo_map.png"))?;

    // 70 external LaTeX (optional: feature + system TeX)
    #[cfg(feature = "latex")]
    if plotine::latex::tools_available() {
        Figure::new()
            .size(6.0, 3.8)
            .dpi(150.0)
            .usetex(true)
            .axes(|ax| {
                ax.line(&xl, xl.iter().map(|t| t.sin()).collect::<Vec<_>>())
                    .color(Color::STEEL_BLUE)
                    .width(2.0)
                    .label(r"$\sin x$");
                ax.title(r"70 usetex: $\displaystyle\frac{1}{2\pi}\int e^{-x^2}\,dx$")
                    .x_label(r"$x$")
                    .y_label(r"$y$")
                    .legend(Legend::TopRight)
                    .grid(true);
            })
            .save(out.join("70_usetex.png"))?;
    } else {
        println!("skip 70_usetex.png (latex/dvipng not on PATH)");
    }

    // 71 correlation heatmap (seaborn-thin)
    {
        let a: Vec<f64> = (0..40).map(|i| i as f64).collect();
        let b: Vec<f64> = a.iter().map(|v| v * 0.5 + 3.0).collect();
        let c: Vec<f64> = a.iter().map(|v| (40.0 - v) + (v % 7.0)).collect();
        plotine::stats::corr_heatmap(&["a", "b", "c"], &[&a, &b, &c])?
            .size(5.0, 4.2)
            .dpi(150.0)
            .save(out.join("71_corr_heatmap.png"))?;
    }

    // 72 GeoJSON polygon overlay
    {
        let js = br#"{
          "type":"FeatureCollection",
          "features":[{
            "type":"Feature",
            "properties":{},
            "geometry":{
              "type":"Polygon",
              "coordinates":[[[-30,-20],[40,-15],[20,35],[-40,25],[-30,-20]]]
            }
          },{
            "type":"Feature",
            "properties":{},
            "geometry":{
              "type":"LineString",
              "coordinates":[[-80,0],[-40,10],[0,5],[40,-5],[80,0]]
            }
          }]
        }"#;
        Figure::new()
            .size(7.0, 3.8)
            .dpi(150.0)
            .axes(|ax| {
                ax.projection(GeoProjection::PlateCarree);
                ax.coastline()
                    .color(Color::rgb(0x4a, 0x55, 0x64))
                    .width(0.6);
                ax.geojson(js).ok();
                ax.title("72 GeoJSON + coastline");
            })
            .save(out.join("72_geojson.png"))?;
    }

    write_gallery_index(out)?;
    println!(
        "wrote gallery figures (+ SVG/PDF) under {} — open gallery/index.html",
        out.display()
    );
    Ok(())
}

fn write_gallery_index(out: &Path) -> Result<()> {
    use std::io::Write;
    let mut entries: Vec<String> = fs::read_dir(out)
        .map_err(|e| PlotError::io(e.to_string()))?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with(".png") || name.ends_with(".svg") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    entries.sort();
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>plotine gallery</title>
<style>
  body{margin:0;font-family:"Segoe UI","PingFang SC",sans-serif;background:#f4f1ec;color:#1c1917}
  header{padding:1.2rem 1.5rem;border-bottom:1px solid #d6d3d1;position:sticky;top:0;background:#f4f1ecee;backdrop-filter:blur(8px)}
  h1{margin:0 0 .35rem;font-size:1.35rem}
  p{margin:0;color:#78716c}
  main{display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:1rem;padding:1.25rem}
  figure{margin:0;background:#fffdf9;border:1px solid #d6d3d1;border-radius:8px;overflow:hidden}
  figcaption{padding:.55rem .75rem;font-size:.85rem;border-top:1px solid #e7e5e4}
  img{display:block;width:100%;height:auto;background:#fff}
</style></head><body>
<header><h1>plotine gallery</h1>
<p>All chart / style / layout demos — regenerate with <code>cargo run -p plotine --example gallery</code></p></header>
<main>
"#,
    );
    for name in &entries {
        let caption = name.trim_end_matches(".png").trim_end_matches(".svg");
        html.push_str(&format!(
            r#"<figure><img src="{name}" alt="{caption}" loading="lazy"/><figcaption>{caption}</figcaption></figure>
"#
        ));
    }
    html.push_str("</main></body></html>\n");
    let mut f =
        fs::File::create(out.join("index.html")).map_err(|e| PlotError::io(e.to_string()))?;
    f.write_all(html.as_bytes())
        .map_err(|e| PlotError::io(e.to_string()))?;
    Ok(())
}
