//! Visual regression snapshots (PNG binary + SVG text).
//!
//! Coverage goal (pragmatic matrix, not full cartesian product):
//! - every chart type at least once (PNG), including hexbin / streamplot / 3D
//! - light / dark / paper themes (representative figures)
//! - SVG for line + bar (+ heatmap)
//!
//! Review intentional changes with: `cargo insta review` (Linux baselines)

use plotine::prelude::*;

fn empty_axes_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.title("Empty Axes")
            .x_label("time (s)")
            .y_label("amplitude")
            .x_range(0.0, 10.0)
            .y_range(-1.0, 1.0);
    })
}

fn sine_figure() -> Figure {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
        ax.title("Sine").x_label("x").y_label("y");
    })
}

fn scatter_figure() -> Figure {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [0.2, 1.1, 0.8, 1.6, 1.3];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.scatter(x, y)
            .size(5.0)
            .color(Color::STEEL_BLUE)
            .label("pts");
        ax.title("Scatter")
            .x_label("x")
            .y_label("y")
            .legend(Legend::TopLeft);
    })
}

fn bar_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.bar([1.0, 2.0, 3.0, 4.0], [3.0, 7.0, 2.0, 5.0])
            .color(Color::STEEL_BLUE)
            .label("counts");
        ax.title("Bars")
            .x_label("x")
            .y_label("y")
            .legend(Legend::TopRight);
    })
}

fn hist_figure() -> Figure {
    let data = [
        0.1, 0.2, 0.25, 0.4, 0.5, 0.55, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.4, 1.5,
    ];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.hist(data).bins(6).color(Color::FOREST_GREEN).label("n");
        ax.title("Histogram")
            .x_label("value")
            .y_label("count")
            .legend(Legend::TopRight);
    })
}

fn area_figure() -> Figure {
    let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.15).collect();
    let y: Vec<f64> = x.iter().map(|v| (v * 0.8).sin().abs() + 0.2).collect();
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.area(&x, &y)
            .color(Color::STEEL_BLUE)
            .alpha(0.45)
            .label("area");
        ax.title("Area")
            .x_label("x")
            .y_label("y")
            .legend(Legend::TopRight);
    })
}

fn errorbar_figure() -> Figure {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [1.0, 1.5, 1.2, 2.0, 1.8];
    let e = [0.2, 0.25, 0.15, 0.3, 0.2];
    let xe = [0.12, 0.1, 0.15, 0.1, 0.12];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.errorbar(x, y, e)
            .xerr(xe)
            .color(Color::STEEL_BLUE)
            .label("data");
        ax.title("Errorbar")
            .x_label("x")
            .y_label("y")
            .legend(Legend::TopLeft);
    })
}

fn loglog_figure() -> Figure {
    let x: Vec<f64> = (0..40)
        .map(|i| 10f64.powf(-1.0 + i as f64 * 0.08))
        .collect();
    let y: Vec<f64> = x.iter().map(|v| 2.0 * v.powf(1.5)).collect();
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
        ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
        ax.title("LogLog");
    })
}

fn subplots_figure() -> Figure {
    let x = [0.0, 1.0, 2.0, 3.0];
    let y = [0.0, 1.0, 0.5, 1.2];
    Figure::new().size(6.0, 4.5).dpi(72.0).subplots(2, 2, |g| {
        g.hspace(0.3).wspace(0.25);
        g.at(0, 0, |ax| {
            ax.line(x, y).color(Color::CRIMSON).width(1.5);
            ax.title("A");
        });
        g.at(0, 1, |ax| {
            ax.scatter(x, y).size(4.0).color(Color::STEEL_BLUE);
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
}

fn datetime_figure() -> Figure {
    let start = 1_577_836_800_f64;
    let x: Vec<f64> = (0..12).map(|i| start + i as f64 * 86_400.0).collect();
    let y: Vec<f64> = (0..12).map(|i| (i as f64 * 0.5).sin() + 1.0).collect();
    Figure::new().size(5.0, 3.0).dpi(72.0).axes(|ax| {
        ax.line(&x, &y).color(Color::STEEL_BLUE).width(2.0);
        ax.x_datetime(true).title("Datetime").x_label("date");
    })
}

fn heatmap_figure() -> Figure {
    let values: Vec<f64> = (0..16)
        .map(|i| {
            let r = (i / 4) as f64;
            let c = (i % 4) as f64;
            (r * 0.7).sin() + (c * 0.9).cos()
        })
        .collect();
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.heatmap(4, 4, &values)
            .cmap(Colormap::Viridis)
            .colorbar(true);
        ax.title("Heatmap");
    })
}

fn boxplot_figure() -> Figure {
    let a = [1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 7.0];
    let b = [2.0, 2.5, 3.0, 3.2, 3.8, 4.5];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.boxplot([&a[..], &b[..]])
            .color(Color::STEEL_BLUE)
            .widths(0.5);
        ax.title("Boxplot").grid(true);
    })
}

fn violin_figure() -> Figure {
    let a = [1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
    let b = [2.0, 2.5, 3.0, 3.2, 3.8, 4.5];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.violin([&a[..], &b[..]])
            .color(Color::MEDIUM_PURPLE)
            .alpha(0.55);
        ax.title("Violin").grid(true);
    })
}

fn fill_between_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.fill_between(
            [0.0, 1.0, 2.0, 3.0],
            [1.0, 2.0, 1.5, 2.5],
            [0.0, 0.5, 0.2, 0.8],
        )
        .color(Color::STEEL_BLUE)
        .alpha(0.4);
        ax.title("Fill Between");
    })
}

fn step_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.step([0.0, 1.0, 2.0, 3.0], [1.0, 2.5, 1.5, 3.0])
            .mode(StepMode::Pre)
            .color(Color::CRIMSON)
            .width(2.0);
        ax.title("Step");
    })
}

fn stem_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.stem([0.0, 1.0, 2.0, 3.0], [1.2, -0.8, 1.5, 0.3])
            .color(Color::MEDIUM_PURPLE);
        ax.axhline(0.0).color(Color::SPINE).width(0.8);
        ax.title("Stem");
    })
}

fn barh_spans_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.barh([1.0, 2.0, 3.0], [4.0, 7.0, 2.5])
            .color(Color::STEEL_BLUE);
        ax.vlines(2.0, 0.5, 3.5).color(Color::CRIMSON);
        ax.title("BarH / Spans");
    })
}

fn pie_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.pie([35.0, 25.0, 20.0, 20.0])
            .labels(["A", "B", "C", "D"]);
        ax.title("Pie").legend(Legend::TopRight);
    })
}

fn stackplot_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.stackplot(
            [0.0, 1.0, 2.0, 3.0],
            [
                [1.0, 1.2, 1.0, 1.3],
                [0.5, 0.6, 0.8, 0.5],
                [0.3, 0.4, 0.2, 0.5],
            ],
        )
        .labels(["a", "b", "c"]);
        ax.title("Stackplot").legend(Legend::TopLeft);
    })
}

fn eventplot_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.eventplot([[1.0, 2.0, 5.0], [0.5, 3.0, 4.0]]);
        ax.title("Eventplot");
    })
}

fn polygon_spans_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.axvspan(1.0, 2.0).color(Color::STEEL_BLUE).alpha(0.25);
        ax.polygon([0.5, 2.5, 1.5], [0.2, 0.2, 1.2])
            .color(Color::FOREST_GREEN)
            .alpha(0.5);
        ax.title("Polygon / Spans");
    })
}

fn hist2d_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.hist2d(
            [0.1, 0.2, 0.8, 0.9, 0.5, 0.55],
            [0.1, 0.9, 0.2, 0.8, 0.5, 0.45],
        )
        .bins(4)
        .colorbar(true);
        ax.title("Hist2D");
    })
}

fn contour_figure() -> Figure {
    let z = [
        0.0, 0.2, 0.4, 0.2, 0.0, 0.2, 0.6, 0.8, 0.6, 0.2, 0.4, 0.8, 1.0, 0.8, 0.4, 0.2, 0.6, 0.8,
        0.6, 0.2, 0.0, 0.2, 0.4, 0.2, 0.0,
    ];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.contourf(5, 5, z).levels(6).cmap(Colormap::Viridis);
        ax.contour(5, 5, z).levels(6).color(Color::SPINE).width(0.8);
        ax.title("Contour");
    })
}

fn spy_figure() -> Figure {
    let z = [0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.spy(3, 3, z).color(Color::STEEL_BLUE);
        ax.title("Spy");
    })
}

fn quiver_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.quiver(
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 0.0, 0.5, -0.5],
            [0.0, 1.0, 0.5, 0.5],
        )
        .color(Color::STEEL_BLUE);
        ax.title("Quiver");
    })
}

fn polar_figure() -> Figure {
    use std::f64::consts::PI;
    let th: Vec<f64> = (0..40).map(|i| i as f64 * PI / 20.0).collect();
    let r: Vec<f64> = th.iter().map(|t| 1.0 + 0.3 * t.cos()).collect();
    Figure::new().size(4.0, 4.0).dpi(72.0).axes(|ax| {
        ax.polar_line(&th, &r).color(Color::CRIMSON).width(2.0);
        ax.title("Polar");
    })
}

fn annotate_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.line([0.0, 1.0, 2.0, 3.0], [0.2, 1.0, 0.4, 0.8])
            .color(Color::STEEL_BLUE)
            .width(2.0);
        ax.text(0.2, 0.9, "start").color(Color::LABEL).size(10.0);
        ax.annotate("peak", (1.0, 1.0), (1.6, 1.25))
            .arrow(true)
            .color(Color::CRIMSON)
            .ha(TextAlign::Left)
            .va(TextBaseline::Bottom);
        ax.title("Annotate");
    })
}

fn twin_y_figure() -> Figure {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.line(x, [1.0, 2.0, 1.5, 2.5, 2.0])
            .color(Color::STEEL_BLUE)
            .width(2.0)
            .label("left");
        ax.y_label("left");
        ax.twin_y(|ax2| {
            ax2.line(x, [10.0, 30.0, 20.0, 45.0, 35.0])
                .color(Color::CRIMSON)
                .width(2.0)
                .label("right");
            ax2.y_label("right");
        });
        ax.title("Twin Y").legend(Legend::TopLeft);
    })
}

fn math_labels_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.line([0.0, 1.0, 2.0, 3.0], [0.0, 1.0, 0.5, 0.8])
            .color(Color::STEEL_BLUE)
            .width(2.0)
            .label(math::unicode(r"x^2"));
        ax.title(math::unicode(r"$\alpha$ spectrum"))
            .x_label(math::unicode(r"$t$ (s)"))
            .y_label(math::unicode(r"$\theta$ (rad)"))
            .legend(Legend::TopRight);
    })
}

fn categories_figure() -> Figure {
    let cats = ["A", "B", "C"];
    let x = category_indices(cats.len());
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.x_categories(cats);
        ax.bar(&x, [2.0, 4.0, 3.0]).color(Color::STEEL_BLUE);
        ax.title("Categories");
    })
}

fn lognorm_figure() -> Figure {
    let z = [1.0, 10.0, 100.0, 3.0, 30.0, 300.0, 5.0, 50.0, 500.0];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.heatmap(3, 3, z)
            .cmap(Colormap::Viridis)
            .norm(Norm::Log)
            .colorbar(true);
        ax.title("LogNorm");
    })
}

fn twin_x_figure() -> Figure {
    let y = [0.0, 1.0, 2.0, 3.0, 4.0];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.line([1.0, 2.0, 3.0, 4.0, 5.0], y)
            .color(Color::STEEL_BLUE)
            .width(2.0)
            .label("bottom");
        ax.x_label("bottom");
        ax.twin_x(|ax2| {
            ax2.line([10.0, 20.0, 30.0, 40.0, 50.0], y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("top");
            ax2.x_label("top");
        });
        ax.title("Twin X").legend(Legend::BottomRight);
    })
}

fn clabel_figure() -> Figure {
    let z = [
        0.0, 0.2, 0.4, 0.2, 0.0, 0.2, 0.6, 0.8, 0.6, 0.2, 0.4, 0.8, 1.0, 0.8, 0.4, 0.2, 0.6, 0.8,
        0.6, 0.2, 0.0, 0.2, 0.4, 0.2, 0.0,
    ];
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.contour(5, 5, z)
            .levels(5)
            .color(Color::STEEL_BLUE)
            .width(1.0)
            .clabel(true)
            .clabel_size(8.0);
        ax.title("Clabel");
    })
}

fn barbs_figure() -> Figure {
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.barbs(
            [0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
            [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            [15.0, 25.0, 55.0, 5.0, 35.0, 75.0],
            [0.0, 10.0, 0.0, 5.0, -10.0, 15.0],
        )
        .length(12.0)
        .color(Color::STEEL_BLUE);
        ax.title("Barbs");
    })
}

fn hexbin_figure() -> Figure {
    let mut x = Vec::with_capacity(200);
    let mut y = Vec::with_capacity(200);
    for i in 0..200 {
        let t = i as f64 * 0.07;
        x.push(t.cos() + 0.15 * (i as f64 * 0.31).sin());
        y.push(t.sin() + 0.15 * (i as f64 * 0.27).cos());
    }
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.hexbin(&x, &y)
            .gridsize(12)
            .cmap(Colormap::Plasma)
            .colorbar(true);
        ax.title("Hexbin");
    })
}

fn streamplot_figure() -> Figure {
    let n = 12;
    let mut u = Vec::with_capacity(n * n);
    let mut v = Vec::with_capacity(n * n);
    for r in 0..n {
        for c in 0..n {
            let x = c as f64 - (n - 1) as f64 * 0.5;
            let y = r as f64 - (n - 1) as f64 * 0.5;
            u.push(-y);
            v.push(x);
        }
    }
    Figure::new().size(4.0, 3.0).dpi(72.0).axes(|ax| {
        ax.streamplot(n, n, &u, &v)
            .density(1.0)
            .color(Color::STEEL_BLUE);
        ax.title("Streamplot");
    })
}

fn helix_3d_figure() -> Figure {
    let t: Vec<f64> = (0..80).map(|i| i as f64 * 0.1).collect();
    let x: Vec<f64> = t.iter().map(|v| v.cos()).collect();
    let y: Vec<f64> = t.iter().map(|v| v.sin()).collect();
    let z = t;
    Figure::new().size(4.0, 3.5).dpi(72.0).axes3d(|ax| {
        ax.plot3d(&x, &y, &z)
            .color(Color::CRIMSON)
            .width(2.0)
            .label("helix");
        ax.title("3D Helix")
            .elev(30.0)
            .azim(-60.0)
            .legend(Legend::TopRight);
    })
}

fn surface_3d_figure() -> Figure {
    let n = 16;
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
    Figure::new().size(4.0, 3.5).dpi(72.0).axes3d(|ax| {
        ax.surface(n, n, &z)
            .x(&xs)
            .y(&ys)
            .cmap(Colormap::Viridis)
            .alpha(0.9);
        ax.title("3D Surface").elev(35.0).azim(-50.0);
    })
}

fn scatter_3d_figure() -> Figure {
    let n = 40;
    let x: Vec<f64> = (0..n).map(|i| (i as f64 * 0.3).cos()).collect();
    let y: Vec<f64> = (0..n).map(|i| (i as f64 * 0.3).sin()).collect();
    let z: Vec<f64> = (0..n).map(|i| i as f64 * 0.05).collect();
    Figure::new().size(4.0, 3.5).dpi(72.0).axes3d(|ax| {
        ax.scatter3d(&x, &y, &z).color(Color::STEEL_BLUE);
        ax.title("3D Scatter").elev(30.0).azim(-60.0);
    })
}

fn wireframe_3d_figure() -> Figure {
    let n = 12;
    let xs: Vec<f64> = (0..n)
        .map(|i| (i as f64 / (n - 1) as f64) * 4.0 - 2.0)
        .collect();
    let ys = xs.clone();
    let mut z = Vec::with_capacity(n * n);
    for &yv in &ys {
        for &xv in &xs {
            z.push(xv * xv - yv * yv);
        }
    }
    Figure::new().size(4.0, 3.5).dpi(72.0).axes3d(|ax| {
        ax.wireframe(n, n, &z)
            .x(&xs)
            .y(&ys)
            .color(Color::STEEL_BLUE);
        ax.title("3D Wireframe").elev(30.0).azim(-60.0);
    })
}

fn bar_3d_figure() -> Figure {
    Figure::new().size(4.0, 3.5).dpi(72.0).axes3d(|ax| {
        ax.bar3d([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [3.0, 5.0, 2.0])
            .color(Color::STEEL_BLUE);
        ax.title("3D Bars").elev(30.0).azim(-60.0);
    })
}

fn contour_3d_figure() -> Figure {
    let n = 16;
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
    Figure::new().size(4.0, 3.5).dpi(72.0).axes3d(|ax| {
        ax.contour3d(n, n, &z).x(&xs).y(&ys).levels(6).width(1.0);
        ax.title("3D Contour").elev(30.0).azim(-60.0);
    })
}

fn quiver_3d_figure() -> Figure {
    Figure::new().size(4.0, 3.5).dpi(72.0).axes3d(|ax| {
        ax.quiver3d(
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [0.0, 0.0, 0.0, 0.5],
            [0.5, 0.0, 0.3, -0.2],
            [0.0, 0.5, -0.2, 0.3],
            [0.2, 0.2, 0.4, 0.1],
        )
        .scale(1.0)
        .color(Color::STEEL_BLUE);
        ax.title("3D Quiver").elev(30.0).azim(-60.0);
    })
}

fn themed_sine(theme: Theme, title: &str) -> Figure {
    let x: Vec<f64> = (0..80).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    Figure::new()
        .size(4.0, 3.0)
        .dpi(72.0)
        .theme(theme)
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin");
            ax.title(title)
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight);
        })
}

// PNG rasterization can produce platform-specific results due to floating-point
// differences in font shaping (cosmic-text) and anti-aliasing (tiny-skia).
// Reference snapshots are generated on Linux; skip binary comparisons elsewhere.

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_empty_axes() {
    let png = empty_axes_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("empty_axes.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_sine_line() {
    let png = sine_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("sine_line.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_scatter() {
    let png = scatter_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("scatter.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_bars() {
    let png = bar_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("bars.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_hist() {
    let png = hist_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("hist.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_area() {
    let png = area_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("area.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_errorbar() {
    let png = errorbar_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("errorbar.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_loglog() {
    let png = loglog_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("loglog.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_subplots() {
    let png = subplots_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("subplots_2x2.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_datetime() {
    let png = datetime_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("datetime_axis.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_heatmap() {
    let png = heatmap_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("heatmap.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_boxplot() {
    let png = boxplot_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("boxplot.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_violin() {
    let png = violin_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("violin.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_fill_between() {
    let png = fill_between_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("fill_between.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_step() {
    let png = step_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("step.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_stem() {
    let png = stem_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("stem.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_barh_spans() {
    let png = barh_spans_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("barh_spans.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_pie() {
    let png = pie_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("pie.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_stackplot() {
    let png = stackplot_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("stackplot.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_eventplot() {
    let png = eventplot_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("eventplot.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_polygon_spans() {
    let png = polygon_spans_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("polygon_spans.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_hist2d() {
    let png = hist2d_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("hist2d.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_contour() {
    let png = contour_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("contour.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_spy() {
    let png = spy_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("spy.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_quiver() {
    let png = quiver_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("quiver.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_polar() {
    let png = polar_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("polar.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_annotate() {
    let png = annotate_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("annotate.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_twin_y() {
    let png = twin_y_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("twin_y.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_math_labels() {
    let png = math_labels_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("math_labels.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_categories() {
    let png = categories_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("categories.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_lognorm() {
    let png = lognorm_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("lognorm.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_twin_x() {
    let png = twin_x_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("twin_x.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_clabel() {
    let png = clabel_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("clabel.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_barbs() {
    let png = barbs_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("barbs.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_hexbin() {
    let png = hexbin_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("hexbin.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_streamplot() {
    let png = streamplot_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("streamplot.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_helix_3d() {
    let png = helix_3d_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("helix_3d.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_surface_3d() {
    let png = surface_3d_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("surface_3d.png", png);
}

/// Smoke-render remaining 3D types on all platforms.
#[test]
fn render_extra_3d_figures_smoke() {
    for fig in [
        scatter_3d_figure(),
        wireframe_3d_figure(),
        bar_3d_figure(),
        contour_3d_figure(),
        quiver_3d_figure(),
    ] {
        let png = fig.render_png().expect("png");
        assert!(png.len() > 100, "expected non-trivial PNG");
    }
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_scatter_3d() {
    let png = scatter_3d_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("scatter_3d.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_wireframe_3d() {
    let png = wireframe_3d_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("wireframe_3d.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_bar_3d() {
    let png = bar_3d_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("bar_3d.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_contour_3d() {
    let png = contour_3d_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("contour_3d.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_quiver_3d() {
    let png = quiver_3d_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("quiver_3d.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_theme_dark() {
    let png = themed_sine(Theme::dark(), "Dark Theme")
        .render_png()
        .expect("png");
    insta::assert_binary_snapshot!("theme_dark.png", png);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_theme_paper() {
    let png = themed_sine(Theme::paper(), "Paper Theme")
        .render_png()
        .expect("png");
    insta::assert_binary_snapshot!("theme_paper.png", png);
}

// SVG layout depends on text measurement (cosmic-text) which can produce
// sub-pixel differences across platforms. Gate to Linux for consistency.
#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_svg_line() {
    let svg = Figure::new()
        .size(3.0, 2.0)
        .dpi(72.0)
        .axes(|ax| {
            ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.25])
                .color(Color::STEEL_BLUE)
                .width(2.0);
            ax.title("SVG");
        })
        .render_svg()
        .expect("svg");
    insta::assert_snapshot!("line_svg", svg);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_svg_bar() {
    let svg = bar_figure().render_svg().expect("svg");
    insta::assert_snapshot!("bar_svg", svg);
}

#[test]
#[cfg_attr(not(target_os = "linux"), ignore)]
fn snapshot_svg_heatmap() {
    let svg = heatmap_figure().render_svg().expect("svg");
    insta::assert_snapshot!("heatmap_svg", svg);
}

// --- M9–M17 smoke (all platforms) + Linux visual slots ----------------------

fn geo_mercator_figure() -> Figure {
    Figure::new().size(5.0, 3.0).dpi(72.0).axes(|ax| {
        ax.projection(GeoProjection::Mercator);
        ax.coastline().color(Color::rgb(0x55, 0x55, 0x55));
        ax.scatter([0.0, 116.4, -74.0], [51.5, 39.9, 40.7])
            .color(Color::CRIMSON)
            .size(4.0);
        ax.title("Mercator");
    })
}

fn stats_corr_figure() -> Figure {
    let a = [1.0, 2.0, 3.0, 4.0, 5.0];
    let b = [2.0, 1.0, 4.0, 3.0, 6.0];
    let c = [5.0, 4.0, 3.0, 2.0, 1.0];
    plotine::stats::corr_heatmap(&["a", "b", "c"], &[&a, &b, &c])
        .unwrap()
        .size(4.0, 3.5)
        .dpi(72.0)
}

fn anim_frame0_figure() -> Figure {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.1).collect();
    let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    Figure::new().size(3.5, 2.5).dpi(72.0).axes(|ax| {
        ax.line(&x, &y0).color(Color::CRIMSON).width(2.0);
        ax.y_range(-1.2, 1.2);
        ax.title("Anim frame 0");
    })
}

/// Smoke-render M11 / M16 / M10 paths on every platform (no insta baseline).
#[test]
fn render_m9_m17_figures_smoke() {
    // Also exercise animate → distinct frames (content contract).
    let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.1).collect();
    let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let fig = Figure::new().size(3.0, 2.0).dpi(48.0).axes(|ax| {
        ax.line(&x, &y0);
        ax.y_range(-1.2, 1.2);
    });
    let anim = fig
        .animate(0..2, |fig, i| {
            let t = i as f64 * 0.5;
            let y: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
            fig.axes_at_mut(0)
                .unwrap()
                .line_at_mut(0)
                .unwrap()
                .set_y(&y)?;
            Ok(())
        })
        .expect("animate");
    assert_ne!(anim.frames()[0].rgba, anim.frames()[1].rgba);

    for (name, png) in [
        (
            "geo_mercator",
            geo_mercator_figure().render_png().expect("png"),
        ),
        ("stats_corr", stats_corr_figure().render_png().expect("png")),
        (
            "anim_frame0",
            anim_frame0_figure().render_png().expect("png"),
        ),
    ] {
        assert!(
            png.len() > 200 && png.starts_with(&[0x89, b'P', b'N', b'G']),
            "{name}: expected non-trivial PNG"
        );
    }
}

/// Linux visual baselines — un-ignore on Linux, then `cargo insta review`.
#[test]
#[ignore = "pending Linux baseline: cargo insta review"]
fn snapshot_geo_mercator() {
    let png = geo_mercator_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("geo_mercator.png", png);
}

#[test]
#[ignore = "pending Linux baseline: cargo insta review"]
fn snapshot_stats_corr() {
    let png = stats_corr_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("stats_corr.png", png);
}

#[test]
#[ignore = "pending Linux baseline: cargo insta review"]
fn snapshot_anim_frame0() {
    let png = anim_frame0_figure().render_png().expect("png");
    insta::assert_binary_snapshot!("anim_frame0.png", png);
}
