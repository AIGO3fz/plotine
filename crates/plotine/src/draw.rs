//! Per-artist rendering dispatch, extracted from figure to keep it lean.

use kurbo::{BezPath, Point as KurboPoint};
use plotine_core::color::DEFAULT_CYCLE;

use crate::mpl_policy::{
    annotate as ann_policy, barbs as barbs_policy, hatch as hatch_policy, pie as pie_policy,
    polar as polar_policy,
};
use plotine_core::{Color, DataToPixel, Point, Rect};
use plotine_render::{
    FillStyle, LineCap, LineJoin, Renderer, StrokeStyle, TextAlign, TextBaseline, TextStyle,
};

use crate::artist::{
    AnnotatePlot, AreaPlot, AxHLinePlot, AxHSpanPlot, AxLinePlot, AxVLinePlot, AxVSpanPlot,
    BarHPlot, BarPlot, BarbsPlot, BoxPlot, BrokenBarHPlot, CirclePlot, ContourPlot, ContourfPlot,
    EllipsePlot, ErrorBarPlot, EventPlot, FillBetweenPlot, FillBetweenXPlot, HLinesPlot,
    HeatmapPlot, HexbinPlot, Hist2dPlot, HistPlot, LinePlot, PcolorMeshPlot, PiePlot, PlotElement,
    PolarFramePlot, PolygonPlot, QuiverPlot, RectanglePlot, ScatterPlot, SpyPlot, StackPlot,
    StairsPlot, StemPlot, StepPlot, StreamPlot, TablePlot, TextPlot, TricontourPlot,
    TricontourfPlot, TripcolorPlot, VLinesPlot, ViolinPlot,
};
use crate::recipes::{
    annotation_arrow_styled, area_path, axhspan_rect, axvspan_rect, bar_rects, barb_geoms,
    barh_rects, boxplot_stats, broken_barh_rects, circle_path, contour_labels, contour_paths,
    contourf_fills, data_to_pixel, delaunay, ellipse_path, errorbar_geoms_asym,
    errorbar_x_geoms_asym, fill_between_path, fill_betweenx_path, format_contour_level,
    heatmap_cells, hexbin_cells, hist2d_bins, hist2d_cells, hist2d_limits, hline_segments,
    infer_bar_width, infer_barh_height, infer_quiver_scale, line_path, marker_path, nice_levels,
    pcolormesh_cells, pie_wedges, polar_angle_labels, polar_frame_paths, polar_radial_labels,
    polygon_path, quiver_arrows, rectangle_pixel_rect, segment_in_label_gap, spy_markers,
    stackplot_paths, stairs_path, stem_geoms, step_path, streamlines, table_cell_geoms,
    tricontour_paths, tricontourf_fills, tripcolor_fills, violin_geoms, violin_path,
    vline_segments, BarRect, HeatmapOrigin, Marker,
};
use crate::style::{Hatch, LineStyle, MarkerStyle};

type Result<T> = plotine_core::Result<T>;

fn stroke_with_linestyle(color: Color, width_px: f64, linestyle: LineStyle) -> StrokeStyle {
    let mut style = StrokeStyle::new(color, width_px);
    if let Some(dash) = linestyle.dash_pattern(width_px) {
        style.dash = Some(dash);
        if matches!(linestyle, LineStyle::Dotted) {
            style.cap = LineCap::Round;
        }
    }
    style
}

/// Stroke a hatch pattern inside `rect` (clipped). `color` is typically the edge color.
fn draw_hatch_rect(
    renderer: &mut dyn Renderer,
    rect: Rect,
    hatch: Hatch,
    color: Color,
    px: f64,
) -> Result<()> {
    if !hatch.is_drawn() {
        return Ok(());
    }
    let x0 = rect.x0.min(rect.x1);
    let x1 = rect.x0.max(rect.x1);
    let y0 = rect.y0.min(rect.y1);
    let y1 = rect.y0.max(rect.y1);
    let w = x1 - x0;
    let h = y1 - y0;
    if w < 1.0 || h < 1.0 {
        return Ok(());
    }
    let clip = Rect::new(x0, y0, x1, y1);
    renderer.push_clip_rect(clip)?;
    let spacing = hatch_policy::SPACING_PX * px.max(0.75);
    let stroke = StrokeStyle::new(color, hatch_policy::STROKE_WIDTH * px.max(0.75));
    match hatch {
        Hatch::None => {}
        Hatch::Diagonal => hatch_diag(renderer, clip, spacing, &stroke)?,
        Hatch::DiagonalBack => hatch_diag_back(renderer, clip, spacing, &stroke)?,
        Hatch::Cross => {
            hatch_diag(renderer, clip, spacing, &stroke)?;
            hatch_diag_back(renderer, clip, spacing, &stroke)?;
        }
        Hatch::Horizontal => {
            let mut y = y0;
            while y <= y1 + 0.5 {
                renderer.draw_line(Point::new(x0, y), Point::new(x1, y), &stroke)?;
                y += spacing;
            }
        }
        Hatch::Vertical => {
            let mut x = x0;
            while x <= x1 + 0.5 {
                renderer.draw_line(Point::new(x, y0), Point::new(x, y1), &stroke)?;
                x += spacing;
            }
        }
        Hatch::Grid => {
            let mut y = y0;
            while y <= y1 + 0.5 {
                renderer.draw_line(Point::new(x0, y), Point::new(x1, y), &stroke)?;
                y += spacing;
            }
            let mut x = x0;
            while x <= x1 + 0.5 {
                renderer.draw_line(Point::new(x, y0), Point::new(x, y1), &stroke)?;
                x += spacing;
            }
        }
        Hatch::Dots => {
            let r = hatch_policy::DOT_RADIUS * px.max(0.75);
            let fill = FillStyle::solid(color);
            let mut y = y0 + spacing * 0.5;
            while y <= y1 {
                let mut x = x0 + spacing * 0.5;
                while x <= x1 {
                    renderer.fill_rect(Rect::new(x - r, y - r, x + r, y + r), &fill)?;
                    x += spacing;
                }
                y += spacing;
            }
        }
    }
    renderer.pop_clip()?;
    Ok(())
}

/// `/` hatch family in screen space (y down): slope −1 → `x + y = c`.
fn hatch_diag(
    renderer: &mut dyn Renderer,
    rect: Rect,
    spacing: f64,
    stroke: &StrokeStyle,
) -> Result<()> {
    let (x0, y0, x1, y1) = (rect.x0, rect.y0, rect.x1, rect.y1);
    let span = rect.width() + rect.height();
    let mut c = x0 + y0 - span;
    let c_end = x1 + y1 + span;
    while c <= c_end {
        renderer.draw_line(Point::new(x0, c - x0), Point::new(x1, c - x1), stroke)?;
        c += spacing;
    }
    Ok(())
}

/// `\` hatch family: slope +1 → `x − y = c`.
fn hatch_diag_back(
    renderer: &mut dyn Renderer,
    rect: Rect,
    spacing: f64,
    stroke: &StrokeStyle,
) -> Result<()> {
    let (x0, y0, x1, y1) = (rect.x0, rect.y0, rect.x1, rect.y1);
    let span = rect.width() + rect.height();
    let mut c = x0 - y1 - span;
    let c_end = x1 - y0 + span;
    while c <= c_end {
        renderer.draw_line(Point::new(x0, x0 - c), Point::new(x1, x1 - c), stroke)?;
        c += spacing;
    }
    Ok(())
}

/// Render a single plot element into the given axes transform.
///
/// `px` is the points→pixels scale factor (`dpi / 72`). Artist widths/sizes are
/// specified in points (matplotlib convention) and multiplied by `px` here.
pub(crate) fn draw_element(
    renderer: &mut dyn Renderer,
    el: &PlotElement,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    match el {
        PlotElement::Line(line) => draw_line(renderer, line, color, transform, px),
        PlotElement::Scatter(scatter) => draw_scatter(renderer, scatter, color, transform, px),
        PlotElement::Bar(bar) => draw_bar(renderer, bar, color, transform, px),
        PlotElement::BarH(bar) => draw_barh(renderer, bar, color, transform, px),
        PlotElement::Hist(hist) => draw_hist(renderer, hist, color, transform, px),
        PlotElement::Area(area) => draw_area(renderer, area, color, transform, px),
        PlotElement::FillBetween(fb) => draw_fill_between(renderer, fb, color, transform),
        PlotElement::FillBetweenX(fb) => draw_fill_betweenx(renderer, fb, color, transform),
        PlotElement::Step(step) => draw_step(renderer, step, color, transform, px),
        PlotElement::Stairs(stairs) => draw_stairs(renderer, stairs, color, transform, px),
        PlotElement::Stem(stem) => draw_stem(renderer, stem, color, transform, px),
        PlotElement::HLines(h) => draw_hlines(renderer, h, color, transform, px),
        PlotElement::VLines(v) => draw_vlines(renderer, v, color, transform, px),
        PlotElement::AxHLine(h) => draw_axhline(renderer, h, color, transform, px),
        PlotElement::AxVLine(v) => draw_axvline(renderer, v, color, transform, px),
        PlotElement::AxHSpan(h) => draw_axhspan(renderer, h, color, transform),
        PlotElement::AxVSpan(v) => draw_axvspan(renderer, v, color, transform),
        PlotElement::Polygon(p) => draw_polygon(renderer, p, color, transform, px),
        PlotElement::Rectangle(p) => draw_rectangle(renderer, p, color, transform, px),
        PlotElement::Circle(p) => draw_circle(renderer, p, color, transform, px),
        PlotElement::Ellipse(p) => draw_ellipse(renderer, p, color, transform, px),
        PlotElement::Pie(p) => draw_pie(renderer, p, color, transform, px),
        PlotElement::StackPlot(p) => draw_stackplot(renderer, p, color, transform),
        PlotElement::EventPlot(p) => draw_eventplot(renderer, p, color, transform, px),
        PlotElement::BrokenBarH(p) => draw_broken_barh(renderer, p, color, transform, px),
        PlotElement::ErrorBar(eb) => draw_errorbar(renderer, eb, color, transform, px),
        PlotElement::Heatmap(hm) => draw_heatmap(renderer, hm, transform),
        PlotElement::Hist2d(h) => draw_hist2d(renderer, h, transform),
        PlotElement::Hexbin(h) => draw_hexbin(renderer, h, transform),
        PlotElement::Contour(c) => draw_contour(renderer, c, color, transform, px),
        PlotElement::Contourf(c) => draw_contourf(renderer, c, transform),
        PlotElement::Tripcolor(p) => draw_tripcolor(renderer, p, transform),
        PlotElement::Tricontour(p) => draw_tricontour(renderer, p, color, transform, px),
        PlotElement::Tricontourf(p) => draw_tricontourf(renderer, p, transform),
        PlotElement::PcolorMesh(p) => draw_pcolormesh(renderer, p, transform),
        PlotElement::Spy(s) => draw_spy(renderer, s, color, transform, px),
        PlotElement::Quiver(q) => draw_quiver(renderer, q, color, transform, px),
        PlotElement::Barbs(b) => draw_barbs(renderer, b, color, transform, px),
        PlotElement::StreamPlot(s) => draw_streamplot(renderer, s, color, transform, px),
        PlotElement::PolarFrame(p) => draw_polar_frame(renderer, p, color, transform, px),
        PlotElement::BoxPlot(bp) => draw_boxplot(renderer, bp, color, transform, px),
        PlotElement::Violin(vp) => draw_violin(renderer, vp, color, transform, px),
        PlotElement::Text(t) => draw_text_plot(renderer, t, color, transform, px),
        PlotElement::Annotate(a) => draw_annotate(renderer, a, color, transform, px),
        PlotElement::Table(t) => draw_table(renderer, t, transform, px),
        PlotElement::AxLine(l) => draw_axline(renderer, l, color, transform, px),
    }
}

fn draw_line(
    renderer: &mut dyn Renderer,
    line: &LinePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let path = line_path(line.x.as_slice(), line.y.as_slice(), transform);
    let mut style = stroke_with_linestyle(color, line.width * px, line.linestyle);
    style.cap = LineCap::Round;
    style.join = LineJoin::Round;
    renderer.stroke_path(&path, &style)
}

fn draw_scatter(
    renderer: &mut dyn Renderer,
    scatter: &ScatterPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let radius = scatter.size * 0.5 * px;
    let stroke_style = scatter
        .marker
        .is_stroke()
        .then(|| StrokeStyle::new(color, (radius * 0.35).max(0.75)));
    // Batch filled markers (chunked so one huge path does not thrash the
    // rasterizer). Circles use a cheap regular octagon instead of dense cubics.
    const CHUNK: usize = 512;
    let mut batch = BezPath::new();
    let mut n_in_batch = 0usize;
    let flush = |renderer: &mut dyn Renderer,
                 batch: &mut BezPath,
                 n: &mut usize,
                 stroke: &Option<StrokeStyle>|
     -> Result<()> {
        if *n == 0 {
            return Ok(());
        }
        if let Some(style) = stroke {
            renderer.stroke_path(batch, style)?;
        } else {
            renderer.fill_path(batch, &FillStyle::solid(color))?;
        }
        *batch = BezPath::new();
        *n = 0;
        Ok(())
    };

    for (&xi, &yi) in scatter.x.as_slice().iter().zip(scatter.y.as_slice().iter()) {
        if !(xi.is_finite() && yi.is_finite()) {
            continue;
        }
        let marker = Marker {
            center: transform.map(Point::new(xi, yi)),
            radius,
        };
        match scatter.marker {
            MarkerStyle::Circle | MarkerStyle::Point => {
                let r = if scatter.marker == MarkerStyle::Point {
                    (radius * 0.35).max(0.5)
                } else {
                    radius.max(0.5)
                };
                append_regular_ngon(&mut batch, marker.center, r, 8);
            }
            other => {
                batch.extend(marker_path(marker, other).elements().iter().copied());
            }
        }
        n_in_batch += 1;
        if n_in_batch >= CHUNK {
            flush(renderer, &mut batch, &mut n_in_batch, &stroke_style)?;
        }
    }
    flush(renderer, &mut batch, &mut n_in_batch, &stroke_style)?;
    Ok(())
}

fn append_regular_ngon(path: &mut BezPath, center: Point, radius: f64, n: usize) {
    let n = n.max(3);
    let r = radius.max(0.5);
    for i in 0..n {
        let a = std::f64::consts::TAU * (i as f64) / (n as f64) - std::f64::consts::FRAC_PI_2;
        let p = kurbo::Point::new(center.x + r * a.cos(), center.y + r * a.sin());
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    path.close_path();
}

fn draw_bar(
    renderer: &mut dyn Renderer,
    bar: &BarPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let width = infer_bar_width(bar.x.as_slice(), bar.width);
    let rects = bar_rects(
        bar.x.as_slice(),
        bar.heights.as_slice(),
        width,
        bar.baseline,
    );
    let fill = FillStyle::solid(color);
    let hatch_color = bar.edgecolor.unwrap_or(Color::SPINE.with_alpha(0.65));
    let edge = StrokeStyle::new(
        bar.edgecolor.unwrap_or(Color::SPINE.with_alpha(0.55)),
        0.9 * px,
    );
    for r in rects {
        let pixel = r.to_pixel_rect(transform);
        renderer.fill_rect(pixel, &fill)?;
        draw_hatch_rect(renderer, pixel, bar.hatch, hatch_color, px)?;
        renderer.stroke_rect(pixel, &edge)?;
    }
    Ok(())
}

fn draw_hist(
    renderer: &mut dyn Renderer,
    hist: &HistPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let bins = hist.compute_bins();
    let fill = FillStyle::solid(color);
    // Matplotlib default `hist` edgecolor is fully transparent — only stroke when set.
    let edge = hist.edgecolor.map(|c| StrokeStyle::new(c, 0.9 * px));
    let hatch_color = hist.edgecolor.unwrap_or(Color::SPINE.with_alpha(0.65));
    for r in bins.bars() {
        let pixel = r.to_pixel_rect(transform);
        renderer.fill_rect(pixel, &fill)?;
        draw_hatch_rect(renderer, pixel, hist.hatch, hatch_color, px)?;
        if let Some(ref edge) = edge {
            renderer.stroke_rect(pixel, edge)?;
        }
    }
    Ok(())
}

fn draw_area(
    renderer: &mut dyn Renderer,
    area: &AreaPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let fill_color = color.with_alpha(area.alpha);
    let path = area_path(
        area.x.as_slice(),
        area.y.as_slice(),
        area.baseline,
        transform,
    );
    renderer.fill_path(&path, &FillStyle::solid(fill_color))?;
    let mut stroke = StrokeStyle::new(color, 1.5 * px);
    stroke.cap = LineCap::Round;
    stroke.join = LineJoin::Round;
    let outline = line_path(area.x.as_slice(), area.y.as_slice(), transform);
    renderer.stroke_path(&outline, &stroke)
}

fn draw_barh(
    renderer: &mut dyn Renderer,
    bar: &BarHPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let height = infer_barh_height(bar.y.as_slice(), bar.height);
    let rects = barh_rects(
        bar.y.as_slice(),
        bar.widths.as_slice(),
        height,
        bar.baseline,
    );
    let fill = FillStyle::solid(color);
    let hatch_color = bar.edgecolor.unwrap_or(Color::SPINE.with_alpha(0.65));
    let edge = StrokeStyle::new(
        bar.edgecolor.unwrap_or(Color::SPINE.with_alpha(0.55)),
        0.9 * px,
    );
    for r in rects {
        let pixel = r.to_pixel_rect(transform);
        renderer.fill_rect(pixel, &fill)?;
        draw_hatch_rect(renderer, pixel, bar.hatch, hatch_color, px)?;
        renderer.stroke_rect(pixel, &edge)?;
    }
    Ok(())
}

fn draw_fill_between(
    renderer: &mut dyn Renderer,
    fb: &FillBetweenPlot,
    color: Color,
    transform: &DataToPixel,
) -> Result<()> {
    let path = fill_between_path(
        fb.x.as_slice(),
        fb.y1.as_slice(),
        fb.y2.as_slice(),
        transform,
    );
    renderer.fill_path(&path, &FillStyle::solid(color.with_alpha(fb.alpha)))
}

fn draw_fill_betweenx(
    renderer: &mut dyn Renderer,
    fb: &FillBetweenXPlot,
    color: Color,
    transform: &DataToPixel,
) -> Result<()> {
    let path = fill_betweenx_path(
        fb.y.as_slice(),
        fb.x1.as_slice(),
        fb.x2.as_slice(),
        transform,
    );
    renderer.fill_path(&path, &FillStyle::solid(color.with_alpha(fb.alpha)))
}

fn draw_step(
    renderer: &mut dyn Renderer,
    step: &StepPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let path = step_path(step.x.as_slice(), step.y.as_slice(), step.mode, transform);
    let mut style = stroke_with_linestyle(color, step.width * px, step.linestyle);
    if !matches!(step.linestyle, LineStyle::Dotted) {
        style.cap = LineCap::Butt;
    }
    style.join = LineJoin::Miter;
    renderer.stroke_path(&path, &style)
}

fn draw_stairs(
    renderer: &mut dyn Renderer,
    stairs: &StairsPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let path = stairs_path(
        stairs.edges.as_slice(),
        stairs.values.as_slice(),
        stairs.baseline,
        transform,
    );
    let mut style = StrokeStyle::new(color, stairs.width * px);
    style.cap = LineCap::Butt;
    style.join = LineJoin::Miter;
    renderer.stroke_path(&path, &style)
}

fn draw_stem(
    renderer: &mut dyn Renderer,
    stem: &StemPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let geoms = stem_geoms(
        stem.x.as_slice(),
        stem.y.as_slice(),
        stem.baseline,
        transform,
    );
    let style = StrokeStyle::new(color, stem.width * px);
    let fill = FillStyle::solid(color);
    let radius = stem.marker_size * 0.5 * px;
    for g in geoms {
        renderer.draw_line(Point::new(g.x0, g.y0), Point::new(g.x1, g.y1), &style)?;
        let marker = Marker {
            center: g.head,
            radius,
        };
        renderer.fill_path(&marker_path(marker, MarkerStyle::Circle), &fill)?;
    }
    Ok(())
}

fn draw_hlines(
    renderer: &mut dyn Renderer,
    h: &HLinesPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let Some(segs) = hline_segments(
        h.y.as_slice(),
        h.xmin.as_slice(),
        h.xmax.as_slice(),
        transform,
    ) else {
        return Ok(());
    };
    let style = StrokeStyle::new(color, h.width * px);
    for s in segs {
        renderer.draw_line(Point::new(s.x0, s.y0), Point::new(s.x1, s.y1), &style)?;
    }
    Ok(())
}

fn draw_vlines(
    renderer: &mut dyn Renderer,
    v: &VLinesPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let Some(segs) = vline_segments(
        v.x.as_slice(),
        v.ymin.as_slice(),
        v.ymax.as_slice(),
        transform,
    ) else {
        return Ok(());
    };
    let style = StrokeStyle::new(color, v.width * px);
    for s in segs {
        renderer.draw_line(Point::new(s.x0, s.y0), Point::new(s.x1, s.y1), &style)?;
    }
    Ok(())
}

fn draw_axhline(
    renderer: &mut dyn Renderer,
    h: &AxHLinePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let (x0, x1) = transform.x_scale().domain();
    let p0 = transform.map(Point::new(x0, h.y));
    let p1 = transform.map(Point::new(x1, h.y));
    let style = stroke_with_linestyle(color, h.width * px, h.linestyle);
    renderer.draw_line(Point::new(p0.x, p0.y), Point::new(p1.x, p1.y), &style)
}

fn draw_axvline(
    renderer: &mut dyn Renderer,
    v: &AxVLinePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let (y0, y1) = transform.y_scale().domain();
    let p0 = transform.map(Point::new(v.x, y0));
    let p1 = transform.map(Point::new(v.x, y1));
    let style = stroke_with_linestyle(color, v.width * px, v.linestyle);
    renderer.draw_line(Point::new(p0.x, p0.y), Point::new(p1.x, p1.y), &style)
}

fn draw_axhspan(
    renderer: &mut dyn Renderer,
    h: &AxHSpanPlot,
    color: Color,
    transform: &DataToPixel,
) -> Result<()> {
    let Some(rect) = axhspan_rect(h.ymin, h.ymax, transform) else {
        return Ok(());
    };
    renderer.fill_rect(rect, &FillStyle::solid(color.with_alpha(h.alpha)))
}

fn draw_axvspan(
    renderer: &mut dyn Renderer,
    v: &AxVSpanPlot,
    color: Color,
    transform: &DataToPixel,
) -> Result<()> {
    let Some(rect) = axvspan_rect(v.xmin, v.xmax, transform) else {
        return Ok(());
    };
    renderer.fill_rect(rect, &FillStyle::solid(color.with_alpha(v.alpha)))
}

fn draw_rectangle(
    renderer: &mut dyn Renderer,
    r: &RectanglePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let Some(pixel) = rectangle_pixel_rect(r.x, r.y, r.width, r.height, transform) else {
        return Ok(());
    };
    renderer.fill_rect(pixel, &FillStyle::solid(color.with_alpha(r.alpha)))?;
    let hatch_color = r.edgecolor.unwrap_or(Color::SPINE.with_alpha(0.65));
    draw_hatch_rect(renderer, pixel, r.hatch, hatch_color, px)?;
    if r.linewidth > 0.0 {
        let edge = StrokeStyle::new(r.edgecolor.unwrap_or(color), r.linewidth * px);
        renderer.stroke_rect(pixel, &edge)?;
    }
    Ok(())
}

fn draw_circle(
    renderer: &mut dyn Renderer,
    c: &CirclePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let path = circle_path(c.x, c.y, c.radius, transform, 64);
    renderer.fill_path(&path, &FillStyle::solid(color.with_alpha(c.alpha)))?;
    if c.linewidth > 0.0 {
        let edge = StrokeStyle::new(c.edgecolor.unwrap_or(color), c.linewidth * px);
        renderer.stroke_path(&path, &edge)?;
    }
    Ok(())
}

fn draw_ellipse(
    renderer: &mut dyn Renderer,
    e: &EllipsePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let path = ellipse_path(e.x, e.y, e.width, e.height, transform, 64);
    renderer.fill_path(&path, &FillStyle::solid(color.with_alpha(e.alpha)))?;
    if e.linewidth > 0.0 {
        let edge = StrokeStyle::new(e.edgecolor.unwrap_or(color), e.linewidth * px);
        renderer.stroke_path(&path, &edge)?;
    }
    Ok(())
}

fn draw_polygon(
    renderer: &mut dyn Renderer,
    poly: &PolygonPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let path = polygon_path(poly.x.as_slice(), poly.y.as_slice(), transform);
    let fill = color.with_alpha(poly.alpha);
    renderer.fill_path(&path, &FillStyle::solid(fill))?;
    // Matplotlib `Patch(alpha=…)` applies alpha to face *and* edge unless the
    // edge color carries its own alpha; match that so poly outlines are not
    // opaque forest-green rings against a translucent fill.
    let edge_color = poly
        .edgecolor
        .map(|c| {
            if c.a == 255 {
                c.with_alpha(poly.alpha)
            } else {
                c
            }
        })
        .unwrap_or(fill);
    let edge = StrokeStyle::new(edge_color, 1.0 * px);
    renderer.stroke_path(&path, &edge)
}

fn draw_pie(
    renderer: &mut dyn Renderer,
    pie: &PiePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let rect = transform.pixel_rect();
    let center = Point::new((rect.x0 + rect.x1) * 0.5, (rect.y0 + rect.y1) * 0.5);
    // Policy: `pie::RADIUS` in view `±pie::VIEW` → `RADIUS_FRAC` of the plot side.
    let radius = rect.width().min(rect.height()) * pie_policy::RADIUS_FRAC;
    let wedges = pie_wedges(
        pie.sizes.as_slice(),
        center,
        radius,
        pie.start_angle,
        pie.counterclock,
        64,
    );
    // Default: no wedge edge (mpl edgecolor fully transparent).
    let edge = pie.edgecolor.map(|c| StrokeStyle::new(c, 1.0 * px));
    for (i, wedge) in wedges.iter().enumerate() {
        let fill = pie
            .color
            .unwrap_or_else(|| DEFAULT_CYCLE[(pie.color_index + i) % DEFAULT_CYCLE.len()]);
        // If a single override color was set via builder, use the passed `color`
        // (already resolved) for all wedges.
        let fill = if pie.color.is_some() { color } else { fill };
        renderer.fill_path(&wedge.path, &FillStyle::solid(fill.with_alpha(pie.alpha)))?;
        if let Some(ref edge) = edge {
            renderer.stroke_path(&wedge.path, edge)?;
        }
        if let Some(label) = pie.labels.get(i) {
            if !label.is_empty() {
                let lr = radius * pie_policy::LABEL_DISTANCE;
                let pos = Point::new(
                    center.x + lr * wedge.mid_angle.cos(),
                    center.y - lr * wedge.mid_angle.sin(),
                );
                // Matplotlib wedges: ha leans toward the outward side.
                let cos_a = wedge.mid_angle.cos();
                let align = if cos_a > 0.25 {
                    plotine_render::TextAlign::Left
                } else if cos_a < -0.25 {
                    plotine_render::TextAlign::Right
                } else {
                    plotine_render::TextAlign::Center
                };
                let style = TextStyle::new(Color::BLACK, (10.0 * px) as f32)
                    .align(align)
                    .baseline(plotine_render::TextBaseline::Middle);
                renderer.draw_text(label, pos, &style)?;
            }
        }
    }
    Ok(())
}

fn draw_stackplot(
    renderer: &mut dyn Renderer,
    stack: &StackPlot,
    color: Color,
    transform: &DataToPixel,
) -> Result<()> {
    let refs: Vec<&[f64]> = stack.ys.iter().map(|s| s.as_slice()).collect();
    let paths = stackplot_paths(stack.x.as_slice(), &refs, transform);
    for (i, path) in paths.iter().enumerate() {
        let fill = if stack.color.is_some() {
            color
        } else {
            DEFAULT_CYCLE[(stack.color_index + i) % DEFAULT_CYCLE.len()]
        };
        renderer.fill_path(path, &FillStyle::solid(fill.with_alpha(stack.alpha)))?;
    }
    Ok(())
}

fn draw_eventplot(
    renderer: &mut dyn Renderer,
    ev: &EventPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    for (row, xs) in ev.positions.iter().enumerate() {
        let row_color = if ev.color.is_some() {
            color
        } else {
            DEFAULT_CYCLE[(ev.color_index + row) % DEFAULT_CYCLE.len()]
        };
        let style = StrokeStyle::new(row_color, ev.linewidth * px);
        let y = (row + 1) as f64;
        let half = ev.lineoffset * 0.5;
        for &x in xs.as_slice() {
            if !x.is_finite() {
                continue;
            }
            let a = transform.map(Point::new(x, y - half));
            let b = transform.map(Point::new(x, y + half));
            renderer.draw_line(Point::new(a.x, a.y), Point::new(b.x, b.y), &style)?;
        }
    }
    Ok(())
}

fn draw_broken_barh(
    renderer: &mut dyn Renderer,
    bb: &BrokenBarHPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let rects = broken_barh_rects(&bb.xranges, bb.y, bb.height);
    let fill = FillStyle::solid(color);
    let edge = bb.edgecolor.map(|c| StrokeStyle::new(c, 0.9 * px));
    for r in rects {
        let pixel = r.to_pixel_rect(transform);
        renderer.fill_rect(pixel, &fill)?;
        if let Some(edge) = &edge {
            renderer.stroke_rect(pixel, edge)?;
        }
    }
    Ok(())
}

fn draw_errorbar(
    renderer: &mut dyn Renderer,
    eb: &ErrorBarPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let (ylo, yhi) = eb.yerr.arms();
    let geoms = errorbar_geoms_asym(eb.x.as_slice(), eb.y.as_slice(), ylo, yhi, transform);
    let x_geoms = eb.xerr.as_ref().map(|xerr| {
        let (xlo, xhi) = xerr.arms();
        errorbar_x_geoms_asym(eb.x.as_slice(), eb.y.as_slice(), xlo, xhi, transform)
    });
    let style = StrokeStyle::new(color, eb.width * px);
    let fill = FillStyle::solid(color);
    let cap = eb.capsize * px;

    // Connecting line through central points (matplotlib `fmt='o-'`).
    if eb.connect && geoms.len() >= 2 {
        let mut path = BezPath::new();
        path.move_to(KurboPoint::new(geoms[0].x, geoms[0].y_mid));
        for g in geoms.iter().skip(1) {
            path.line_to(KurboPoint::new(g.x, g.y_mid));
        }
        let mut line = StrokeStyle::new(color, eb.width * px);
        line.cap = LineCap::Round;
        line.join = LineJoin::Round;
        renderer.stroke_path(&path, &line)?;
    }

    if let Some(x_geoms) = &x_geoms {
        for g in x_geoms {
            renderer.draw_line(Point::new(g.x_lo, g.y), Point::new(g.x_hi, g.y), &style)?;
            if cap > 0.0 {
                renderer.draw_line(
                    Point::new(g.x_lo, g.y - cap),
                    Point::new(g.x_lo, g.y + cap),
                    &style,
                )?;
                renderer.draw_line(
                    Point::new(g.x_hi, g.y - cap),
                    Point::new(g.x_hi, g.y + cap),
                    &style,
                )?;
            }
        }
    }

    for g in &geoms {
        renderer.draw_line(Point::new(g.x, g.y_lo), Point::new(g.x, g.y_hi), &style)?;
        if cap > 0.0 {
            renderer.draw_line(
                Point::new(g.x - cap, g.y_lo),
                Point::new(g.x + cap, g.y_lo),
                &style,
            )?;
            renderer.draw_line(
                Point::new(g.x - cap, g.y_hi),
                Point::new(g.x + cap, g.y_hi),
                &style,
            )?;
        }
        renderer.fill_path(
            &marker_path(
                Marker {
                    center: Point::new(g.x, g.y_mid),
                    radius: 3.0 * px,
                },
                MarkerStyle::Circle,
            ),
            &fill,
        )?;
    }
    Ok(())
}

/// Slightly expand mesh cells so shared edges overlap (hides hairlines in SVG viewers too).
fn seal_mesh_rect(r: plotine_core::Rect) -> plotine_core::Rect {
    r.inset(-0.75, -0.75, -0.75, -0.75)
}

fn fill_sealed_rect(
    renderer: &mut dyn Renderer,
    rect: plotine_core::Rect,
    color: Color,
) -> Result<()> {
    // Crisp (no AA) + overlap: adjacent bins must not leak the axes face.
    renderer.fill_rect(seal_mesh_rect(rect), &FillStyle::solid_crisp(color))
}

fn draw_heatmap(
    renderer: &mut dyn Renderer,
    hm: &HeatmapPlot,
    transform: &DataToPixel,
) -> Result<()> {
    let ncells = hm.nrows.saturating_mul(hm.ncols);
    // Dense grids: one RGBA blit beats O(cells) fill_rect calls (512² stress).
    if ncells >= 256 {
        return draw_heatmap_blit(renderer, hm, transform);
    }
    let (vmin, vmax) = hm.value_limits();
    let cells = heatmap_cells(
        hm.nrows,
        hm.ncols,
        hm.values.as_slice(),
        &hm.cmap,
        vmin,
        vmax,
        hm.norm,
        hm.origin,
        hm.extent,
    );
    for cell in cells {
        let mut color = cell.color;
        if hm.alpha < 1.0 {
            color = color.with_alpha((f64::from(color.a) / 255.0) * hm.alpha);
        }
        if color.a == 0 {
            continue;
        }
        let pixel = cell.rect.to_pixel_rect(transform);
        fill_sealed_rect(renderer, pixel, color)?;
    }
    Ok(())
}

fn draw_heatmap_blit(
    renderer: &mut dyn Renderer,
    hm: &HeatmapPlot,
    transform: &DataToPixel,
) -> Result<()> {
    let nrows = hm.nrows.max(1);
    let ncols = hm.ncols.max(1);
    let (vmin, vmax) = hm.value_limits();
    let explicit_extent = hm.extent.is_some();
    let (x0, x1, y0, y1) = match hm.extent {
        Some([l, r, b, t]) if l.is_finite() && r.is_finite() && b.is_finite() && t.is_finite() => {
            (l, r, b, t)
        }
        _ => (-0.5, (ncols as f64) - 0.5, -0.5, (nrows as f64) - 0.5),
    };
    let dx = (x1 - x0) / ncols as f64;
    let dy = (y1 - y0) / nrows as f64;

    // Pixel bbox from data extent corners (+ seal pad).
    let corners = [
        transform.map(Point::new(x0, y0)),
        transform.map(Point::new(x1, y0)),
        transform.map(Point::new(x0, y1)),
        transform.map(Point::new(x1, y1)),
    ];
    let mut min_x = corners[0].x;
    let mut min_y = corners[0].y;
    let mut max_x = corners[0].x;
    let mut max_y = corners[0].y;
    for p in &corners[1..] {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    min_x -= 0.75;
    min_y -= 0.75;
    max_x += 0.75;
    max_y += 0.75;
    let ox = min_x.floor();
    let oy = min_y.floor();
    let tw = ((max_x.ceil() - ox).max(1.0)) as u32;
    let th = ((max_y.ceil() - oy).max(1.0)) as u32;
    if tw.saturating_mul(th) > 16_000_000 {
        // Degenerate transform — fall back to sealed rects.
        let cells = heatmap_cells(
            hm.nrows,
            hm.ncols,
            hm.values.as_slice(),
            &hm.cmap,
            vmin,
            vmax,
            hm.norm,
            hm.origin,
            hm.extent,
        );
        for cell in cells {
            let mut color = cell.color;
            if hm.alpha < 1.0 {
                color = color.with_alpha((f64::from(color.a) / 255.0) * hm.alpha);
            }
            if color.a == 0 {
                continue;
            }
            fill_sealed_rect(renderer, cell.rect.to_pixel_rect(transform), color)?;
        }
        return Ok(());
    }

    let mut rgba = vec![0u8; (tw as usize) * (th as usize) * 4];
    let values = hm.values.as_slice();
    for row in 0..nrows {
        let ry = match hm.origin {
            HeatmapOrigin::Upper if explicit_extent => (nrows - 1 - row) as f64,
            HeatmapOrigin::Upper | HeatmapOrigin::Lower => row as f64,
        };
        let cy0 = y0 + ry * dy;
        let cy1 = cy0 + dy;
        for col in 0..ncols {
            let value = values.get(row * ncols + col).copied().unwrap_or(f64::NAN);
            let mut color = hm.cmap.map_norm(value, vmin, vmax, hm.norm);
            if hm.alpha < 1.0 {
                color = color.with_alpha((f64::from(color.a) / 255.0) * hm.alpha);
            }
            if color.a == 0 {
                continue;
            }
            let cx0 = x0 + col as f64 * dx;
            let pr = seal_mesh_rect(
                BarRect {
                    x0: cx0,
                    x1: cx0 + dx,
                    y0: cy0.min(cy1),
                    y1: cy0.max(cy1),
                }
                .to_pixel_rect(transform),
            );
            fill_rgba_rect(
                &mut rgba,
                tw,
                th,
                Rect::new(pr.x0 - ox, pr.y0 - oy, pr.x1 - ox, pr.y1 - oy),
                color,
            );
        }
    }
    renderer.draw_rgba_image(&rgba, tw, th, Point::new(ox, oy))
}

fn fill_rgba_rect(buf: &mut [u8], tw: u32, th: u32, rect: Rect, color: Color) {
    let x0 = rect.x0.floor().max(0.0) as i32;
    let y0 = rect.y0.floor().max(0.0) as i32;
    let x1 = rect.x1.ceil().min(f64::from(tw)) as i32;
    let y1 = rect.y1.ceil().min(f64::from(th)) as i32;
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let px = color.to_rgba_u8();
    let tw = tw as usize;
    for y in y0..y1 {
        let row = (y as usize) * tw;
        for x in x0..x1 {
            let i = (row + x as usize) * 4;
            buf[i] = px[0];
            buf[i + 1] = px[1];
            buf[i + 2] = px[2];
            buf[i + 3] = px[3];
        }
    }
}

fn draw_hist2d(renderer: &mut dyn Renderer, h: &Hist2dPlot, transform: &DataToPixel) -> Result<()> {
    let bins = hist2d_bins(h.x.as_slice(), h.y.as_slice(), h.bins_x, h.bins_y);
    let (vmin, vmax) = hist2d_limits(&bins.counts, h.vmin, h.vmax);
    let cells = hist2d_cells(&bins, &h.cmap, vmin, vmax, h.norm);
    for cell in cells {
        if cell.color.a == 0 {
            continue;
        }
        let pixel = cell.rect.to_pixel_rect(transform);
        fill_sealed_rect(renderer, pixel, cell.color)?;
    }
    Ok(())
}

fn draw_hexbin(renderer: &mut dyn Renderer, h: &HexbinPlot, transform: &DataToPixel) -> Result<()> {
    let (cells, _, _) = hexbin_cells(
        h.x.as_slice(),
        h.y.as_slice(),
        h.gridsize,
        &h.cmap,
        h.vmin,
        h.vmax,
        h.norm,
        transform,
    );
    for cell in cells {
        if cell.color.a == 0 {
            continue;
        }
        // Crisp fill only (hex mesh shares edges; hairline seal was ~2× paint).
        renderer.fill_path(&cell.path, &FillStyle::solid_crisp(cell.color))?;
    }
    Ok(())
}

fn draw_contour(
    renderer: &mut dyn Renderer,
    c: &ContourPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let levels = c.resolved_levels();
    let paths = contour_paths(
        c.values.as_slice(),
        c.nrows,
        c.ncols,
        None,
        None,
        &levels,
        transform,
    );
    let labels = if c.clabel {
        contour_labels(
            c.values.as_slice(),
            c.nrows,
            c.ncols,
            None,
            None,
            &levels,
            transform,
            (6.0 * px).max(8.0),
        )
    } else {
        Vec::new()
    };

    // Precompute inline gap boxes (matplotlib `inline=True`).
    // Sizes are in px; ~0.55em per glyph width, ~0.7em half-height — close to mpl padding.
    let gaps: Vec<_> = labels
        .iter()
        .map(|label| {
            let text = format_contour_level(label.level);
            let em = c.clabel_size * px;
            let half_w = (text.len() as f64 * 0.55 * em).max(6.0 * px);
            let half_h = (0.7 * em).max(4.0 * px);
            (label, half_w, half_h)
        })
        .collect();

    let style = StrokeStyle::new(color, c.width * px);
    for path in &paths {
        if c.clabel {
            // ContourPath is a 2-point segment; skip those under a same-level label.
            let els = path.path.elements();
            let blocked = match (els.first(), els.get(1)) {
                (Some(kurbo::PathEl::MoveTo(a)), Some(kurbo::PathEl::LineTo(b))) => {
                    let q0 = Point::new(a.x, a.y);
                    let q1 = Point::new(b.x, b.y);
                    gaps.iter().any(|(lab, hw, hh)| {
                        (lab.level - path.level).abs() < 1e-12
                            && segment_in_label_gap(q0, q1, lab, *hw, *hh)
                    })
                }
                _ => false,
            };
            if blocked {
                continue;
            }
        }
        renderer.stroke_path(&path.path, &style)?;
    }

    if c.clabel {
        let ink = c.clabel_color.unwrap_or(Color::LABEL);
        let text_style = TextStyle::new(ink, (c.clabel_size * px) as f32)
            .align(TextAlign::Center)
            .baseline(TextBaseline::Middle)
            .bold(true);
        for label in &labels {
            let text = format_contour_level(label.level);
            if text.is_empty() {
                continue;
            }
            let style = text_style.clone().rotation(label.rotation_deg);
            renderer.draw_text(&text, label.pos, &style)?;
        }
    }
    Ok(())
}

fn draw_contourf(
    renderer: &mut dyn Renderer,
    c: &ContourfPlot,
    transform: &DataToPixel,
) -> Result<()> {
    let levels = c.resolved_levels();
    let fills = contourf_fills(
        c.values.as_slice(),
        c.nrows,
        c.ncols,
        None,
        None,
        &levels,
        &c.cmap,
        c.norm,
        transform,
    );
    // Crisp fill only: after per-band path merge, a same-color hairline seal
    // strokes every cell edge and dominates raster cost without fixing seams
    // (solid_crisp already disables AA on shared edges).
    for fill in fills {
        renderer.fill_path(&fill.path, &FillStyle::solid_crisp(fill.color))?;
    }
    Ok(())
}

fn draw_tripcolor(
    renderer: &mut dyn Renderer,
    p: &TripcolorPlot,
    transform: &DataToPixel,
) -> Result<()> {
    let triangles = if p.triangles.is_empty() {
        delaunay(p.x.as_slice(), p.y.as_slice()).map_err(plotine_core::PlotError::render)?
    } else {
        p.triangles.clone()
    };
    let (vmin, vmax) = {
        if let (Some(lo), Some(hi)) = (p.vmin, p.vmax) {
            (lo.min(hi), lo.max(hi))
        } else {
            let (zmin, zmax) = crate::recipes::tripcolor_face_limits(p.z.as_slice(), &triangles)
                .unwrap_or((0.0, 1.0));
            let lo = p.vmin.unwrap_or(zmin);
            let hi = p.vmax.unwrap_or(zmax);
            (lo.min(hi), lo.max(hi))
        }
    };
    let fills = tripcolor_fills(
        p.x.as_slice(),
        p.y.as_slice(),
        p.z.as_slice(),
        &triangles,
        vmin,
        vmax,
        &p.cmap,
        p.norm,
        transform,
    );
    // Crisp fill only — same-color hairline seal doubles paint cost on dense
    // meshes without fixing AA seams (`solid_crisp` already disables AA).
    for fill in fills {
        renderer.fill_path(&fill.path, &FillStyle::solid_crisp(fill.color))?;
    }
    Ok(())
}

fn draw_tricontour(
    renderer: &mut dyn Renderer,
    p: &TricontourPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let triangles = if p.triangles.is_empty() {
        delaunay(p.x.as_slice(), p.y.as_slice()).map_err(plotine_core::PlotError::render)?
    } else {
        p.triangles.clone()
    };
    let levels = crate::recipes::resolve_tri_levels(
        p.z.as_slice(),
        &triangles,
        p.levels.as_deref(),
        p.level_count,
    );
    let paths = tricontour_paths(
        p.x.as_slice(),
        p.y.as_slice(),
        p.z.as_slice(),
        &triangles,
        &levels,
        transform,
    );
    let stroke = StrokeStyle::new(color, p.width * px);
    for path in paths {
        renderer.stroke_path(&path.path, &stroke)?;
    }
    Ok(())
}

fn draw_pcolormesh(
    renderer: &mut dyn Renderer,
    p: &PcolorMeshPlot,
    transform: &DataToPixel,
) -> Result<()> {
    let (vmin, vmax) = p.value_limits();
    let cells = pcolormesh_cells(
        p.x_edges.as_slice(),
        p.y_edges.as_slice(),
        p.values.as_slice(),
        &p.cmap,
        vmin,
        vmax,
        p.norm,
    );
    for cell in cells {
        if cell.color.a == 0 {
            continue;
        }
        let pixel = cell.rect.to_pixel_rect(transform);
        fill_sealed_rect(renderer, pixel, cell.color)?;
    }
    Ok(())
}

fn draw_spy(
    renderer: &mut dyn Renderer,
    s: &SpyPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let markers = spy_markers(
        s.nrows,
        s.ncols,
        s.values.as_slice(),
        s.precision,
        s.marker_size * 0.5 * px,
        transform,
    );
    let fill = FillStyle::solid(color);
    for marker in markers {
        // Matplotlib `spy` defaults to square markers (`marker='s'`).
        renderer.fill_path(&marker_path(marker, MarkerStyle::Square), &fill)?;
    }
    Ok(())
}

fn draw_quiver(
    renderer: &mut dyn Renderer,
    q: &QuiverPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let (xmin, xmax) = q.x.min_max().unwrap_or((0.0, 1.0));
    let (ymin, ymax) = q.y.min_max().unwrap_or((0.0, 1.0));
    let scale = q.scale.unwrap_or_else(|| {
        infer_quiver_scale(q.u.as_slice(), q.v.as_slice(), xmax - xmin, ymax - ymin)
    });
    let shaft_w = q.width * px;
    let arrows = quiver_arrows(
        q.x.as_slice(),
        q.y.as_slice(),
        q.u.as_slice(),
        q.v.as_slice(),
        scale,
        shaft_w,
        crate::mpl_policy::quiver::HEAD_LENGTH_WIDTH,
        crate::mpl_policy::quiver::HEAD_WIDTH_WIDTH,
        transform,
    );
    let fill = FillStyle::solid(color);
    for a in &arrows {
        // Shaft + head are filled polygons (matplotlib `Quiver` / `FancyArrow`).
        renderer.fill_path(&a.shaft, &fill)?;
        renderer.fill_path(&a.head, &fill)?;
    }
    if let Some(key_len) = q.key_length {
        // Place reference arrow at axes fraction (0.85, 0.90) like matplotlib's
        // quiverkey(X=0.85, Y=0.9) — independent of data range.
        let ar = transform.pixel_rect();
        let kpx = ar.x0 + 0.85 * ar.width();
        let kpy = ar.y0 + (1.0 - 0.90) * ar.height();
        let (dx0, dx1) = transform.x_scale().domain();
        let (dy0, dy1) = transform.y_scale().domain();
        let kx = dx0 + (kpx - ar.x0) / ar.width() * (dx1 - dx0);
        let ky = dy0 + (1.0 - (kpy - ar.y0) / ar.height()) * (dy1 - dy0);
        let key = quiver_arrows(
            &[kx],
            &[ky],
            &[key_len],
            &[0.0],
            scale,
            shaft_w,
            crate::mpl_policy::quiver::HEAD_LENGTH_WIDTH,
            crate::mpl_policy::quiver::HEAD_WIDTH_WIDTH,
            transform,
        );
        for a in &key {
            renderer.fill_path(&a.shaft, &fill)?;
            renderer.fill_path(&a.head, &fill)?;
        }
        if let Some(label) = q.key_label.as_deref() {
            let tip = transform.map(Point::new(kx + key_len / scale, ky));
            let style = TextStyle::new(color, (9.0 * px) as f32)
                .align(plotine_render::TextAlign::Left)
                .baseline(plotine_render::TextBaseline::Middle);
            renderer.draw_text(label, Point::new(tip.x + 4.0 * px, tip.y), &style)?;
        }
    }
    Ok(())
}

fn draw_text_plot(
    renderer: &mut dyn Renderer,
    t: &TextPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let Some(pos) = data_to_pixel(t.x, t.y, transform) else {
        return Ok(());
    };
    if t.text.is_empty() {
        return Ok(());
    }
    let style = TextStyle::new(color, (t.size * px) as f32)
        .align(t.align)
        .baseline(t.baseline)
        .rotation(t.rotation_deg);
    crate::mathtext::draw_text(renderer, &t.text, pos, &style)
}

fn draw_table(
    renderer: &mut dyn Renderer,
    t: &TablePlot,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let (grid, has_col, has_row) = build_table_grid(t);
    if grid.is_empty() {
        return Ok(());
    }
    let font = (t.fontsize * px) as f32;
    let pad = t.cell_pad * px;
    let ncols = grid[0].len();
    let nrows = grid.len();
    let mut col_w = vec![0.0_f64; ncols];
    let mut row_h = vec![0.0_f64; nrows];
    for (r, row) in grid.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let (w, h) = crate::mathtext::measure_text(renderer, cell, font)?;
            col_w[c] = col_w[c].max(w + pad * 2.0);
            row_h[r] = row_h[r].max(h + pad * 2.0);
        }
    }
    // Minimum readable cell size.
    for w in &mut col_w {
        *w = w.max(font as f64 * 1.6);
    }
    for h in &mut row_h {
        *h = h.max(font as f64 * 1.35);
    }
    let geoms = table_cell_geoms(
        transform.pixel_rect(),
        t.loc,
        &col_w,
        &row_h,
        &grid,
        has_col,
        has_row,
    );
    let edge = StrokeStyle::new(t.edgecolor, 0.8 * px);
    for cell in &geoms {
        let fill = if cell.header {
            t.header_facecolor
        } else {
            t.facecolor
        };
        renderer.fill_rect(cell.rect, &FillStyle::solid(fill))?;
        renderer.stroke_rect(cell.rect, &edge)?;
        let style = TextStyle::new(Color::LABEL, font)
            .align(TextAlign::Center)
            .baseline(TextBaseline::Middle);
        let cx = (cell.rect.x0 + cell.rect.x1) * 0.5;
        let cy = (cell.rect.y0 + cell.rect.y1) * 0.5;
        crate::mathtext::draw_text(renderer, &cell.text, Point::new(cx, cy), &style)?;
    }
    Ok(())
}

fn build_table_grid(t: &TablePlot) -> (Vec<Vec<String>>, bool, bool) {
    let has_col = !t.col_labels.is_empty();
    let has_row = !t.row_labels.is_empty();
    let body_cols = t
        .cells
        .iter()
        .map(|r| r.len())
        .max()
        .unwrap_or(0)
        .max(t.col_labels.len());
    if body_cols == 0 && !has_row {
        return (Vec::new(), false, false);
    }
    let mut grid = Vec::new();
    if has_col {
        let mut header = Vec::new();
        if has_row {
            header.push(String::new());
        }
        for i in 0..body_cols {
            header.push(t.col_labels.get(i).cloned().unwrap_or_default());
        }
        grid.push(header);
    }
    for (ri, row) in t.cells.iter().enumerate() {
        let mut out = Vec::new();
        if has_row {
            out.push(t.row_labels.get(ri).cloned().unwrap_or_default());
        }
        for i in 0..body_cols {
            out.push(row.get(i).cloned().unwrap_or_default());
        }
        grid.push(out);
    }
    (grid, has_col, has_row)
}

fn draw_annotate(
    renderer: &mut dyn Renderer,
    a: &AnnotatePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let size_px = a.size * px;
    let text_box = if a.arrow && !a.text.is_empty() {
        let (tw, th) = crate::mathtext::measure_text(renderer, &a.text, size_px as f32)?;
        Some(crate::recipes::AnnotateTextBox {
            width: tw,
            height: th,
            size_px,
            align: a.align,
            baseline: a.baseline,
            pad_px: ann_policy::TEXT_PAD_PT * px,
        })
    } else {
        None
    };
    if a.arrow {
        let mutation_px = ann_policy::MUTATION_SCALE_PT * px;
        let linewidth_px = a.arrow_width * px;
        if let Some(arrow) = annotation_arrow_styled(
            a.xy,
            a.xytext,
            transform,
            mutation_px,
            a.arrow_style,
            linewidth_px,
            text_box,
        ) {
            let acolor = a.arrow_color.unwrap_or(color);
            // FancyArrowPatch defaults: capstyle=round, joinstyle=round.
            let mut stroke = StrokeStyle::new(acolor, a.arrow_width * px);
            stroke.cap = LineCap::Round;
            stroke.join = LineJoin::Round;
            renderer.stroke_path(&arrow.shaft, &stroke)?;
            match a.arrow_style {
                // `-|>` filled tip (also stroke edge so AA matches FancyArrowPatch).
                crate::artist::ArrowStyle::Triangle => {
                    renderer.fill_path(&arrow.head, &FillStyle::solid(acolor))?;
                    renderer.stroke_path(&arrow.head, &stroke)?;
                }
                // `->` / `<->` / `-[` are stroke-only.
                crate::artist::ArrowStyle::Simple
                | crate::artist::ArrowStyle::Bracket
                | crate::artist::ArrowStyle::BothEnds => {
                    renderer.stroke_path(&arrow.head, &stroke)?;
                }
            }
        }
    }
    let Some(pos) = data_to_pixel(a.xytext.0, a.xytext.1, transform) else {
        return Ok(());
    };
    if a.text.is_empty() {
        return Ok(());
    }
    let style = TextStyle::new(color, size_px as f32)
        .align(a.align)
        .baseline(a.baseline)
        .rotation(a.rotation_deg);
    crate::mathtext::draw_text(renderer, &a.text, pos, &style)
}

fn draw_barbs(
    renderer: &mut dyn Renderer,
    b: &BarbsPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    // Policy: [`barbs_policy::staff_length_px`] (mpl PolyCollection scaling).
    let length_px = barbs_policy::staff_length_px(b.length, px);
    let geoms = barb_geoms(
        b.x.as_slice(),
        b.y.as_slice(),
        b.u.as_slice(),
        b.v.as_slice(),
        length_px,
        b.half,
        b.full,
        b.flag,
        b.flip,
        transform,
    );
    let stroke = StrokeStyle::new(color, b.width * px);
    let fill = FillStyle::solid(color);
    for g in &geoms {
        if let Some(empty) = &g.empty {
            renderer.stroke_path(empty, &stroke)?;
            continue;
        }
        if let Some(shaft) = &g.shaft {
            renderer.stroke_path(shaft, &stroke)?;
        }
        for feather in &g.feathers {
            renderer.stroke_path(feather, &stroke)?;
        }
        for flag in &g.flags {
            renderer.fill_path(flag, &fill)?;
            renderer.stroke_path(flag, &stroke)?;
        }
    }
    Ok(())
}

fn draw_streamplot(
    renderer: &mut dyn Renderer,
    s: &StreamPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let lines = streamlines(
        s.u.as_slice(),
        s.v.as_slice(),
        s.nrows,
        s.ncols,
        s.density,
        s.arrow_size,
        transform,
        px,
    );
    let style = StrokeStyle::new(color, s.width * px);
    let fill = FillStyle::solid(color);
    for line in &lines {
        renderer.stroke_path(&line.path, &style)?;
        for arrow in &line.arrows {
            renderer.fill_path(arrow, &fill)?;
        }
    }
    Ok(())
}

fn draw_polar_frame(
    renderer: &mut dyn Renderer,
    p: &PolarFramePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let rmax = p.rmax.abs().max(1e-9);
    // Interior rings: same span policy as `polar_rings`.
    let data_est = rmax / polar_policy::R_MARGIN;
    let mut ring_radii: Vec<f64> =
        nice_levels(0.0, polar_policy::ring_level_span(data_est), p.rings)
            .into_iter()
            .filter(|&r| r > 1e-12 && r < rmax * 0.999)
            .collect();
    if ring_radii.is_empty() {
        ring_radii = nice_levels(0.0, rmax, p.rings)
            .into_iter()
            .filter(|&r| r > 1e-12 && r < rmax * 0.999)
            .collect();
    }
    // Outer spine path (unlabeled).
    ring_radii.push(rmax);
    let (rings, spokes) = polar_frame_paths(rmax, &ring_radii, p.spokes, transform);
    // Matplotlib polar defaults: grid `#b0b0b0`, spine black @ 0.8 pt, labels black @ 10 pt.
    let grid = if color == Color::GRID {
        Color::rgb(0xb0, 0xb0, 0xb0)
    } else {
        color
    };
    let grid_style = StrokeStyle::new(grid, 0.8 * px);
    let spine_style = StrokeStyle::new(Color::rgb(0x00, 0x00, 0x00), 0.8 * px);

    for path in &spokes {
        renderer.stroke_path(path, &grid_style)?;
    }
    // Inner rings as grid; outermost as circular spine (matplotlib polar).
    if let Some((last, inner)) = rings.split_last() {
        for path in inner {
            renderer.stroke_path(path, &grid_style)?;
        }
        renderer.stroke_path(last, &spine_style)?;
    }

    let tick_px = (10.0 * px) as f32;
    // Labels sit just outside the spine (axes-filling disk); clip is the cell.
    for label in polar_angle_labels(rmax, p.spokes, transform, 10.0 * px) {
        let align = match label.align {
            crate::recipes::PolarLabelAlign::Left => TextAlign::Left,
            crate::recipes::PolarLabelAlign::Center => TextAlign::Center,
            crate::recipes::PolarLabelAlign::Right => TextAlign::Right,
        };
        let baseline = match label.baseline {
            crate::recipes::PolarLabelBaseline::Top => TextBaseline::Top,
            crate::recipes::PolarLabelBaseline::Middle => TextBaseline::Middle,
            crate::recipes::PolarLabelBaseline::Bottom => TextBaseline::Bottom,
        };
        let style = TextStyle::new(Color::BLACK, tick_px)
            .align(align)
            .baseline(baseline);
        renderer.draw_text(&label.text, label.pos, &style)?;
    }
    // Matplotlib polar r-tick labels stay horizontal (rotation = 0).
    // Do not label the outer spine (mpl clips the next nice tick past rmax).
    let r_style = TextStyle::new(Color::BLACK, tick_px)
        .align(TextAlign::Center)
        .baseline(TextBaseline::Middle);
    let label_rings: Vec<f64> = ring_radii
        .iter()
        .copied()
        .filter(|&r| r < rmax * 0.995)
        .collect();
    for label in polar_radial_labels(&label_rings, p.spokes, transform) {
        renderer.draw_text(&label.text, label.pos, &r_style)?;
    }
    Ok(())
}

fn draw_boxplot(
    renderer: &mut dyn Renderer,
    bp: &BoxPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let refs: Vec<&[f64]> = bp.groups.iter().map(|s| s.as_slice()).collect();
    let items = boxplot_stats(&refs, bp.widths);
    let edge = bp.edgecolor.unwrap_or(Color::SPINE.with_alpha(0.85));
    // matplotlib patch_artist face alpha ~0.7; median uses tableau C1.
    let fill = FillStyle::solid(color.with_alpha(0.7));
    let stroke = StrokeStyle::new(edge, 1.25 * px);
    let median_stroke = StrokeStyle::new(Color::TAB_ORANGE, 1.6 * px);
    let flier_stroke = StrokeStyle::new(edge, 1.0 * px);
    for (stats, box_rect) in items {
        let pixel_box = box_rect.to_pixel_rect(transform);
        renderer.fill_rect(pixel_box, &fill)?;
        renderer.stroke_rect(pixel_box, &stroke)?;

        let x_mid = transform.map_x(stats.x);
        let med_y = transform.map_y(stats.median);
        renderer.draw_line(
            Point::new(pixel_box.x0, med_y),
            Point::new(pixel_box.x1, med_y),
            &median_stroke,
        )?;

        let y_lo = transform.map_y(stats.whisker_lo);
        let y_hi = transform.map_y(stats.whisker_hi);
        let y_q1 = transform.map_y(stats.q1);
        let y_q3 = transform.map_y(stats.q3);
        renderer.draw_line(Point::new(x_mid, y_lo), Point::new(x_mid, y_q1), &stroke)?;
        renderer.draw_line(Point::new(x_mid, y_q3), Point::new(x_mid, y_hi), &stroke)?;
        let cap = (pixel_box.x1 - pixel_box.x0) * 0.35;
        renderer.draw_line(
            Point::new(x_mid - cap, y_lo),
            Point::new(x_mid + cap, y_lo),
            &stroke,
        )?;
        renderer.draw_line(
            Point::new(x_mid - cap, y_hi),
            Point::new(x_mid + cap, y_hi),
            &stroke,
        )?;

        if bp.show_fliers {
            // matplotlib: marker='o', markerfacecolor='none' (open circles)
            for &fy in &stats.fliers {
                let py = transform.map_y(fy);
                renderer.stroke_path(
                    &marker_path(
                        Marker {
                            center: Point::new(x_mid, py),
                            radius: 2.5 * px,
                        },
                        MarkerStyle::Circle,
                    ),
                    &flier_stroke,
                )?;
            }
        }
    }
    Ok(())
}

fn draw_violin(
    renderer: &mut dyn Renderer,
    vp: &ViolinPlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let refs: Vec<&[f64]> = vp.groups.iter().map(|s| s.as_slice()).collect();
    let geoms = violin_geoms(&refs, vp.points);
    let fill = FillStyle::solid(color.with_alpha(vp.alpha));
    let stroke = vp.edgecolor.map(|edge| StrokeStyle::new(edge, 1.15 * px));
    // matplotlib cmaxes/cmins/cbars — clear blue stem (not the body edge)
    let stem = StrokeStyle::new(Color::rgb(0x1f, 0x77, 0xb4), 1.4 * px);
    for geom in &geoms {
        let path = violin_path(geom, vp.widths, transform);
        renderer.fill_path(&path, &fill)?;
        if let Some(stroke) = &stroke {
            renderer.stroke_path(&path, stroke)?;
        }

        let x_mid = transform.map_x(geom.center);
        if vp.show_extrema {
            let y_lo = transform.map_y(geom.ymin);
            let y_hi = transform.map_y(geom.ymax);
            renderer.draw_line(Point::new(x_mid, y_lo), Point::new(x_mid, y_hi), &stem)?;
            let cap = points_like_cap(vp.widths, transform, geom.center);
            renderer.draw_line(
                Point::new(x_mid - cap, y_lo),
                Point::new(x_mid + cap, y_lo),
                &stem,
            )?;
            renderer.draw_line(
                Point::new(x_mid - cap, y_hi),
                Point::new(x_mid + cap, y_hi),
                &stem,
            )?;
        }
        if vp.show_median {
            // mpl: line_ends = ±0.25 * widths
            let half = vp.widths * crate::mpl_policy::violin::STEM_HALF_WIDTH_FRAC;
            let y = transform.map_y(geom.median);
            let x0 = transform.map_x(geom.center - half);
            let x1 = transform.map_x(geom.center + half);
            renderer.draw_line(Point::new(x0, y), Point::new(x1, y), &stem)?;
        }
    }
    Ok(())
}

/// Half-width of extrema end-caps in pixels (mpl `±0.25 * widths`).
fn points_like_cap(widths: f64, transform: &DataToPixel, center: f64) -> f64 {
    let half = widths.clamp(0.1, 0.95) * crate::mpl_policy::violin::STEM_HALF_WIDTH_FRAC;
    let x0 = transform.map_x(center - half);
    let x1 = transform.map_x(center + half);
    ((x1 - x0) * 0.5).abs().max(1.0)
}

fn draw_tricontourf(
    renderer: &mut dyn Renderer,
    p: &TricontourfPlot,
    transform: &DataToPixel,
) -> Result<()> {
    let triangles = if p.triangles.is_empty() {
        delaunay(p.x.as_slice(), p.y.as_slice()).map_err(plotine_core::PlotError::render)?
    } else {
        p.triangles.clone()
    };
    let levels = crate::recipes::resolve_tri_levels(
        p.z.as_slice(),
        &triangles,
        p.levels.as_deref(),
        p.level_count,
    );
    let fills = tricontourf_fills(
        p.x.as_slice(),
        p.y.as_slice(),
        p.z.as_slice(),
        &triangles,
        &levels,
        &p.cmap,
        p.norm,
        transform,
    );
    for fill in fills {
        renderer.fill_path(&fill.path, &FillStyle::solid_crisp(fill.color))?;
    }
    Ok(())
}

fn draw_axline(
    renderer: &mut dyn Renderer,
    l: &AxLinePlot,
    color: Color,
    transform: &DataToPixel,
    px: f64,
) -> Result<()> {
    let (x1, y1) = l.xy1;
    let (x2, y2) = l.xy2;
    if !x1.is_finite() || !y1.is_finite() || !x2.is_finite() || !y2.is_finite() {
        return Ok(());
    }
    let (xmin, xmax) = transform.x_scale().domain();
    let (ymin, ymax) = transform.y_scale().domain();
    let dx = x2 - x1;
    let dy = y2 - y1;

    if dx.abs() < 1e-15 && dy.abs() < 1e-15 {
        return Ok(());
    }

    // Collect candidate intersection points with the four domain edges.
    let mut candidates: Vec<(f64, f64)> = Vec::with_capacity(4);

    if dx.abs() > 1e-15 {
        // Intersection with x = xmin
        let t = (xmin - x1) / dx;
        let yy = y1 + t * dy;
        if yy >= ymin && yy <= ymax {
            candidates.push((xmin, yy));
        }
        // Intersection with x = xmax
        let t = (xmax - x1) / dx;
        let yy = y1 + t * dy;
        if yy >= ymin && yy <= ymax {
            candidates.push((xmax, yy));
        }
    }
    if dy.abs() > 1e-15 {
        // Intersection with y = ymin
        let t = (ymin - y1) / dy;
        let xx = x1 + t * dx;
        if xx >= xmin && xx <= xmax {
            candidates.push((xx, ymin));
        }
        // Intersection with y = ymax
        let t = (ymax - y1) / dy;
        let xx = x1 + t * dx;
        if xx >= xmin && xx <= xmax {
            candidates.push((xx, ymax));
        }
    }

    // Deduplicate near-identical intersection points (corner hits).
    candidates.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12);

    if candidates.len() < 2 {
        return Ok(());
    }

    let p0 = transform.map(Point::new(candidates[0].0, candidates[0].1));
    let p1 = transform.map(Point::new(candidates[1].0, candidates[1].1));
    let style = stroke_with_linestyle(color, l.width * px, l.linestyle);
    renderer.draw_line(Point::new(p0.x, p0.y), Point::new(p1.x, p1.y), &style)
}
