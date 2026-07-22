//! 3D rendering: project, depth-sort, and draw through the 2D renderer.
//!
//! Chrome defaults (pane / grid / camera) follow matplotlib mplot3d via
//! [`crate::mpl_policy::axes3d`].

#![allow(clippy::items_after_test_module)]

use kurbo::BezPath;

use plotine_core::color::DEFAULT_CYCLE;
use plotine_core::{Color, Colormap, LinearScale, Norm, Point, Rect, Tick, TickLocator};
use plotine_render::{
    FillStyle, LineCap, LineJoin, Renderer, StrokeStyle, TextAlign, TextBaseline, TextStyle,
};

use crate::axes3d::{
    mesh_xy, Axes3D, Bar3D, Contour3D, Line3D, PlotElement3D, Quiver3D, Scatter3D, Surface3D,
    Wireframe3D,
};
use crate::legend::Legend;
use crate::mpl_policy::axes3d as ax3_policy;
use crate::projection::{cube_corners, Point3, Projected, Projection, CUBE_EDGES};
use crate::recipes::{contour_level_segments, nice_levels};
use crate::theme::{points_to_px, points_to_px_f32, Theme};

type Result<T> = plotine_core::Result<T>;

/// Matplotlib default `Axes3D` subplot box inside the figure (y-down pixels).
fn axes3d_content_rect(figure: Rect) -> Rect {
    let w = figure.width();
    let h = figure.height();
    let x0 = figure.x0 + w * ax3_policy::AXES_LEFT;
    let x1 = x0 + w * ax3_policy::AXES_WIDTH;
    // mpl `bottom` / `height` are y-up figure fractions.
    let y0 = figure.y0 + h * (1.0 - ax3_policy::AXES_BOTTOM - ax3_policy::AXES_HEIGHT);
    let y1 = figure.y0 + h * (1.0 - ax3_policy::AXES_BOTTOM);
    Rect::new(x0, y0, x1, y1)
}

/// Fits the projected unit cube into the Axes3D content box.
///
/// Computes the projected AABB of the cube corners and uniformly scales it
/// into matplotlib's default Axes3D subplot rectangle.
#[derive(Debug, Clone, Copy)]
struct View3D {
    scale: f64,
    cx: f64,
    cy: f64,
    mid_x: f64,
    mid_y: f64,
    /// Axes3D subplot box (for title/legend anchoring).
    content: Rect,
}

impl View3D {
    fn from_projection(proj: &Projection, figure: Rect, _dpi: f64) -> Self {
        let content = axes3d_content_rect(figure);
        let corners = cube_corners();
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for c in corners {
            let p = proj.project(c);
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
        let span_x = (max_x - min_x).max(1e-9);
        let span_y = (max_y - min_y).max(1e-9);

        let avail_w = content.width().max(1.0);
        let avail_h = content.height().max(1.0);
        let scale = (avail_w / span_x).min(avail_h / span_y) * ax3_policy::FIT_SHRINK;

        let mid_x = 0.5 * (min_x + max_x);
        let mid_y = 0.5 * (min_y + max_y);
        let cx = content.center().x;
        let cy = content.center().y;

        Self {
            scale,
            cx,
            cy,
            mid_x,
            mid_y,
            content,
        }
    }

    fn map(self, p: Projected) -> Point {
        Point::new(
            self.cx + (p.x - self.mid_x) * self.scale,
            self.cy - (p.y - self.mid_y) * self.scale,
        )
    }
}

/// Full 3D→pixel helper.
fn project_to_pixel(
    proj: &Projection,
    view: View3D,
    data: Point3,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
) -> (Point, f64) {
    let p = proj.project_data(data, ranges.0, ranges.1, ranges.2);
    (view.map(p), p.depth)
}

/// Draw the complete 3D panel.
pub(crate) fn draw_axes3d(
    renderer: &mut dyn Renderer,
    axes: &Axes3D,
    rect: Rect,
    theme: &Theme,
    dpi: f64,
) -> Result<()> {
    let proj = Projection::from_camera(axes.camera);
    let view = View3D::from_projection(&proj, rect, dpi);
    let ranges = axes.ranges();
    let px = points_to_px(1.0, dpi);

    // Matplotlib figure face is white; panes carry the grey fill.
    renderer.fill_rect(rect, &FillStyle::solid(theme.background))?;

    // Draw panes + grid + cube edges + ticks (mplot3d chrome).
    draw_frame(renderer, &proj, view, ranges, axes.show_grid, theme, dpi)?;

    // Draw artists (no depth sorting between artists for simplicity — draw in order).
    renderer.push_clip_rect(rect)?;
    for el in &axes.elements {
        match el {
            PlotElement3D::Line(line) => {
                draw_line3d(renderer, line, &proj, view, ranges, px)?;
            }
            PlotElement3D::Scatter(sc) => {
                draw_scatter3d(renderer, sc, &proj, view, ranges, px)?;
            }
            PlotElement3D::Surface(surf) => {
                draw_surface3d(renderer, surf, &proj, view, ranges, px)?;
            }
            PlotElement3D::Wireframe(wf) => {
                draw_wireframe3d(renderer, wf, &proj, view, ranges, px)?;
            }
            PlotElement3D::Bar(bar) => {
                draw_bar3d(renderer, bar, &proj, view, ranges, px)?;
            }
            PlotElement3D::Contour(c) => {
                draw_contour3d(renderer, c, &proj, view, ranges, px)?;
            }
            PlotElement3D::Quiver(q) => {
                draw_quiver3d(renderer, q, &proj, view, ranges, px)?;
            }
        }
    }
    renderer.pop_clip()?;

    // Title — centered above the Axes3D box (mpl title sits in the top margin).
    if let Some(title) = &axes.title {
        let style = TextStyle::new(theme.title, points_to_px_f32(theme.title_size, dpi))
            .align(TextAlign::Center)
            .baseline(TextBaseline::Bottom);
        let pos = Point::new(
            view.content.center().x,
            view.content.y0 - points_to_px(6.0, dpi),
        );
        renderer.draw_text(title, pos, &style)?;
    }

    // Legend — anchored to the Axes3D box (not the full figure).
    if let Some(loc) = axes.legend {
        draw_legend3d(renderer, axes, view.content, theme, dpi, loc)?;
    }

    Ok(())
}

/// Cube faces as corner-index quads. Order: −z, +z, −y, +y, −x, +x.
const CUBE_FACES: [[usize; 4]; 6] = [
    [0, 1, 2, 3], // z = −0.5 (bottom)
    [4, 5, 6, 7], // z = +0.5 (top)
    [0, 1, 5, 4], // y = −0.5
    [3, 2, 6, 7], // y = +0.5
    [0, 3, 7, 4], // x = −0.5
    [1, 2, 6, 5], // x = +0.5
];

/// Which data axis is fixed on each face (`0=x, 1=y, 2=z`).
const FACE_FIXED_AXIS: [usize; 6] = [2, 2, 1, 1, 0, 0];

fn face_avg_depth(proj: &Projection, face: &[usize; 4]) -> f64 {
    let corners = cube_corners();
    face.iter()
        .map(|&i| proj.project(corners[i]).depth)
        .sum::<f64>()
        / 4.0
}

/// Indices of the three back-facing panes (farthest from camera), sorted back→front.
fn back_face_indices(proj: &Projection) -> [usize; 3] {
    let mut order: Vec<(usize, f64)> = (0..6)
        .map(|i| (i, face_avg_depth(proj, &CUBE_FACES[i])))
        .collect();
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    [order[0].0, order[1].0, order[2].0]
}

fn major_ticks(min: f64, max: f64) -> Vec<Tick> {
    let scale = LinearScale::new(min, max)
        .unwrap_or_else(|_| LinearScale::new(0.0, 1.0).expect("unit scale"));
    // Shared axis-wide decimal alignment (`format_aligned_ticks` via TickLocator).
    TickLocator::new(ax3_policy::TICK_TARGETS).ticks_linear(scale)
}

#[cfg(test)]
mod format_tests {
    use plotine_core::format_aligned_ticks;

    use super::{axis_line_edge_points, coord_info_3d};
    use crate::projection::{Camera, Projection};

    #[test]
    fn default_view_highs_and_z_edge_match_mplot3d() {
        // elev=30, azim=-60, ranges like compare/surface_3d.
        let proj = Projection::from_camera(Camera {
            elev: 30.0,
            azim: -60.0,
        });
        let ranges = ((-4.0, 4.0), (-4.0, 4.0), (-1.0, 1.0));
        let info = coord_info_3d(&proj, ranges);
        assert_eq!(info.highs, [false, true, false]);
        let (z0, z1) = axis_line_edge_points(&info, 2);
        assert!((z0[0] - 4.0).abs() < 1e-9 && (z0[1] - 4.0).abs() < 1e-9);
        assert!((z1[0] - 4.0).abs() < 1e-9 && (z1[1] - 4.0).abs() < 1e-9);
        assert!((z0[2] - (-1.0)).abs() < 1e-9 && (z1[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn axis_decimals_align() {
        let labels = format_aligned_ticks(&[-1.0, -0.5, 0.75], 0.25);
        assert_eq!(labels[0], "\u{2212}1.00");
        assert_eq!(labels[1], "\u{2212}0.50");
        assert_eq!(labels[2], "0.75");
        assert_eq!(format_aligned_ticks(&[2.0], 2.0), vec!["2".to_string()]);
    }
}

fn draw_frame(
    renderer: &mut dyn Renderer,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    show_grid: bool,
    theme: &Theme,
    dpi: f64,
) -> Result<()> {
    let corners = cube_corners();
    let x_ticks = major_ticks(ranges.0 .0, ranges.0 .1);
    let y_ticks = major_ticks(ranges.1 .0, ranges.1 .1);
    let z_ticks = major_ticks(ranges.2 .0, ranges.2 .1);

    // 1) Filled panes (matplotlib per-axis pane facecolor) — back faces only.
    let back = back_face_indices(proj);
    // Painter: farthest pane first.
    for &fi in &back {
        let face = &CUBE_FACES[fi];
        let pane_fill = FillStyle::solid(ax3_policy::pane_face(FACE_FIXED_AXIS[fi]));
        let mut path = BezPath::new();
        for (k, &ci) in face.iter().enumerate() {
            let p = view.map(proj.project(corners[ci]));
            if k == 0 {
                path.move_to(kurbo::Point::new(p.x, p.y));
            } else {
                path.line_to(kurbo::Point::new(p.x, p.y));
            }
        }
        path.close_path();
        renderer.fill_path(&path, &pane_fill)?;
    }

    // 2) Grid lines on panes (matplotlib axes3d grid).
    if show_grid {
        let grid = StrokeStyle::new(
            ax3_policy::grid_color(),
            points_to_px(ax3_policy::GRID_WIDTH_PT, dpi),
        );
        for &fi in &back {
            draw_pane_grid(
                renderer, proj, view, ranges, fi, &x_ticks, &y_ticks, &z_ticks, &grid,
            )?;
        }
    }

    // 3) Axis lines only (matplotlib `Axis.line`, lw≈0.8, black).
    //
    // mplot3d does **not** draw a dark full cage: pane `edgecolor` is ~0.9 gray
    // @ α=0.5 (effectively invisible on white), and the three edges that meet at
    // the near corner stay open. Visible frame = tick-bearing axis spines.
    let axis_edge = StrokeStyle::new(theme.spine, points_to_px(ax3_policy::EDGE_WIDTH_PT, dpi));
    let axis_eis = axis_line_cube_edges(proj, view, ranges, &corners);
    for ei in axis_eis {
        let (a, b) = CUBE_EDGES[ei];
        let pa = view.map(proj.project(corners[a]));
        let pb = view.map(proj.project(corners[b]));
        renderer.draw_line(pa, pb, &axis_edge)?;
    }

    // 4) Tick labels outside the box (mplot3d style).
    draw_axis_ticks(
        renderer, proj, view, ranges, &x_ticks, &y_ticks, &z_ticks, theme, dpi,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_pane_grid(
    renderer: &mut dyn Renderer,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    face_idx: usize,
    x_ticks: &[Tick],
    y_ticks: &[Tick],
    z_ticks: &[Tick],
    style: &StrokeStyle,
) -> Result<()> {
    let fixed = FACE_FIXED_AXIS[face_idx];
    // Fixed coordinate at the face (data space).
    let fixed_val = match (fixed, face_idx % 2) {
        (0, 0) => ranges.0 .0,
        (0, _) => ranges.0 .1,
        (1, 0) => ranges.1 .0,
        (1, _) => ranges.1 .1,
        (2, 0) => ranges.2 .0,
        _ => ranges.2 .1,
    };

    let mut line = |a: Point3, b: Point3| -> Result<()> {
        let pa = view.map(proj.project_data(a, ranges.0, ranges.1, ranges.2));
        let pb = view.map(proj.project_data(b, ranges.0, ranges.1, ranges.2));
        renderer.draw_line(pa, pb, style)
    };

    match fixed {
        2 => {
            // Floor/ceiling: lines of constant x and constant y.
            for t in x_ticks {
                line(
                    Point3::new(t.value, ranges.1 .0, fixed_val),
                    Point3::new(t.value, ranges.1 .1, fixed_val),
                )?;
            }
            for t in y_ticks {
                line(
                    Point3::new(ranges.0 .0, t.value, fixed_val),
                    Point3::new(ranges.0 .1, t.value, fixed_val),
                )?;
            }
        }
        1 => {
            // Wall of constant y.
            for t in x_ticks {
                line(
                    Point3::new(t.value, fixed_val, ranges.2 .0),
                    Point3::new(t.value, fixed_val, ranges.2 .1),
                )?;
            }
            for t in z_ticks {
                line(
                    Point3::new(ranges.0 .0, fixed_val, t.value),
                    Point3::new(ranges.0 .1, fixed_val, t.value),
                )?;
            }
        }
        _ => {
            // Wall of constant x.
            for t in y_ticks {
                line(
                    Point3::new(fixed_val, t.value, ranges.2 .0),
                    Point3::new(fixed_val, t.value, ranges.2 .1),
                )?;
            }
            for t in z_ticks {
                line(
                    Point3::new(fixed_val, ranges.1 .0, t.value),
                    Point3::new(fixed_val, ranges.1 .1, t.value),
                )?;
            }
        }
    }
    Ok(())
}

/// mplot3d `Axis._PLANES` corner indices into [`cube_corners`] / data cube.
const MPL_PLANES: [[usize; 4]; 6] = [
    [0, 3, 7, 4],
    [1, 2, 6, 5], // yz @ x=min / x=max
    [0, 1, 5, 4],
    [3, 2, 6, 7], // xz @ y=min / y=max
    [0, 1, 2, 3],
    [4, 5, 6, 7], // xy @ z=min / z=max
];

/// Per-axis juggled indices (`_AXINFO[*]['juggled']`) for vertical_axis = z.
const AXIS_JUGGLED: [[usize; 3]; 3] = [
    [1, 0, 2], // x
    [0, 1, 2], // y
    [0, 2, 1], // z
];

/// Default tickdir for vertical_axis=z (`_get_tickdir('default')`).
const AXIS_TICKDIR: [usize; 3] = [1, 0, 0];

#[derive(Debug, Clone, Copy)]
struct CoordInfo3D {
    mins: [f64; 3],
    maxs: [f64; 3],
    /// Which bound plane is "high" in the projected sense (mplot3d `highs`).
    highs: [bool; 3],
    deltas: [f64; 3],
    centers: [f64; 3],
}

/// mplot3d `Axis._get_coord_info` + `_calc_centers_deltas`.
fn coord_info_3d(proj: &Projection, ranges: ((f64, f64), (f64, f64), (f64, f64))) -> CoordInfo3D {
    let mins = [
        ranges.0 .0.min(ranges.0 .1),
        ranges.1 .0.min(ranges.1 .1),
        ranges.2 .0.min(ranges.2 .1),
    ];
    let maxs = [
        ranges.0 .0.max(ranges.0 .1),
        ranges.1 .0.max(ranges.1 .1),
        ranges.2 .0.max(ranges.2 .1),
    ];

    // 8 data-space corners in the same order as [`cube_corners`].
    let corners = [
        Point3::new(mins[0], mins[1], mins[2]),
        Point3::new(maxs[0], mins[1], mins[2]),
        Point3::new(maxs[0], maxs[1], mins[2]),
        Point3::new(mins[0], maxs[1], mins[2]),
        Point3::new(mins[0], mins[1], maxs[2]),
        Point3::new(maxs[0], mins[1], maxs[2]),
        Point3::new(maxs[0], maxs[1], maxs[2]),
        Point3::new(mins[0], maxs[1], maxs[2]),
    ];
    let depths: [f64; 8] = std::array::from_fn(|i| {
        proj.project_data(corners[i], ranges.0, ranges.1, ranges.2)
            .depth
    });

    // Empirically our `Projected.depth` (larger = farther) ranks opposite to
    // mpl's projected-z inequality for the same planes, so use the same
    // relational form as mplot3d: `highs = mean0 < mean1`.
    let mut highs = [false; 3];
    for axis in 0..3 {
        let mean = |plane: &[usize; 4]| plane.iter().map(|&i| depths[i]).sum::<f64>() / 4.0;
        let d0 = mean(&MPL_PLANES[2 * axis]);
        let d1 = mean(&MPL_PLANES[2 * axis + 1]);
        highs[axis] = d0 < d1;
    }

    let deltas = [
        (maxs[0] - mins[0]) * ax3_policy::DELTA_SCALE,
        (maxs[1] - mins[1]) * ax3_policy::DELTA_SCALE,
        (maxs[2] - mins[2]) * ax3_policy::DELTA_SCALE,
    ];
    let centers = [
        0.5 * (mins[0] + maxs[0]),
        0.5 * (mins[1] + maxs[1]),
        0.5 * (mins[2] + maxs[2]),
    ];

    CoordInfo3D {
        mins,
        maxs,
        highs,
        deltas,
        centers,
    }
}

/// mplot3d `_move_from_center`: nudge *coord* away from *centers* on masked axes.
fn move_from_center(
    coord: [f64; 3],
    centers: [f64; 3],
    deltas: [f64; 3],
    axmask: [bool; 3],
) -> [f64; 3] {
    let mut out = coord;
    for i in 0..3 {
        if axmask[i] {
            let sign = if coord[i] - centers[i] >= 0.0 {
                1.0
            } else {
                -1.0
            };
            out[i] += sign * deltas[i];
        }
    }
    out
}

/// mplot3d `deltas_per_point = 48 / sum(72 * ax_inches)`.
fn deltas_per_point(view: View3D, dpi: f64) -> f64 {
    let ax_w_in = view.content.width() / dpi.max(1e-9);
    let ax_h_in = view.content.height() / dpi.max(1e-9);
    let ax_points = 72.0 * (ax_w_in + ax_h_in);
    ax3_policy::DELTAS_PER_POINT_NUM / ax_points.max(1e-9)
}

/// Axis line endpoints in data space (`_get_axis_line_edge_points`, vertical_axis=z).
fn axis_line_edge_points(info: &CoordInfo3D, axis: usize) -> ([f64; 3], [f64; 3]) {
    let minmax = [
        if info.highs[0] {
            info.maxs[0]
        } else {
            info.mins[0]
        },
        if info.highs[1] {
            info.maxs[1]
        } else {
            info.mins[1]
        },
        if info.highs[2] {
            info.maxs[2]
        } else {
            info.mins[2]
        },
    ];
    let maxmin = [
        if info.highs[0] {
            info.mins[0]
        } else {
            info.maxs[0]
        },
        if info.highs[1] {
            info.mins[1]
        } else {
            info.maxs[1]
        },
        if info.highs[2] {
            info.mins[2]
        } else {
            info.maxs[2]
        },
    ];
    let juggled = AXIS_JUGGLED[axis];
    let mut edge0 = minmax;
    // default position branch: move juggled[0] to the near-camera corner.
    edge0[juggled[0]] = maxmin[juggled[0]];
    let mut edge1 = edge0;
    edge1[juggled[1]] = maxmin[juggled[1]];
    (edge0, edge1)
}

/// Cube-edge indices that carry the three axis spines for this view.
fn axis_line_cube_edges(
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    corners: &[Point3; 8],
) -> [usize; 3] {
    let info = coord_info_3d(proj, ranges);
    let seg_mid = |p0: Point, p1: Point| Point::new(0.5 * (p0.x + p1.x), 0.5 * (p0.y + p1.y));
    let nearest = |target: Point| -> usize {
        CUBE_EDGES
            .iter()
            .enumerate()
            .min_by(|(_, &(a, b)), (_, &(c, d))| {
                let ma = seg_mid(
                    view.map(proj.project(corners[a])),
                    view.map(proj.project(corners[b])),
                );
                let mb = seg_mid(
                    view.map(proj.project(corners[c])),
                    view.map(proj.project(corners[d])),
                );
                let da = (ma.x - target.x).hypot(ma.y - target.y);
                let db = (mb.x - target.x).hypot(mb.y - target.y);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    };

    let mut out = [0usize; 3];
    for (axis, slot) in out.iter_mut().enumerate() {
        let (e0, e1) = axis_line_edge_points(&info, axis);
        let p0 = view.map(proj.project_data(
            Point3::new(e0[0], e0[1], e0[2]),
            ranges.0,
            ranges.1,
            ranges.2,
        ));
        let p1 = view.map(proj.project_data(
            Point3::new(e1[0], e1[1], e1[2]),
            ranges.0,
            ranges.1,
            ranges.2,
        ));
        *slot = nearest(seg_mid(p0, p1));
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn draw_axis_ticks(
    renderer: &mut dyn Renderer,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    x_ticks: &[Tick],
    y_ticks: &[Tick],
    z_ticks: &[Tick],
    theme: &Theme,
    dpi: f64,
) -> Result<()> {
    let tick_font = points_to_px_f32(theme.tick_label_size, dpi);
    let info = coord_info_3d(proj, ranges);
    // mplot3d `_draw_ticks`:
    //   labeldeltas = (tick.get_pad() + 8) * deltas_per_point * deltas
    //   pos = _move_from_center(pos, centers, labeldeltas, axmask)
    // Tick labels are always ha=center, va=top (verified on mpl 3.10).
    let dpp = deltas_per_point(view, dpi);
    let label_scale = (ax3_policy::TICK_LABEL_PAD_PT + ax3_policy::TICK_LABEL_OFFSET_PT) * dpp;
    let labeldeltas = [
        label_scale * info.deltas[0],
        label_scale * info.deltas[1],
        label_scale * info.deltas[2],
    ];
    let stub_style = StrokeStyle::new(theme.spine, points_to_px(ax3_policy::EDGE_WIDTH_PT, dpi));
    let style = TextStyle::new(theme.label, tick_font)
        .align(TextAlign::Center)
        .baseline(TextBaseline::Top);

    let in_range = |v: f64, (a, b): (f64, f64)| {
        let lo = a.min(b);
        let hi = a.max(b);
        v >= lo - 1e-9 && v <= hi + 1e-9
    };

    let axis_ticks = [x_ticks, y_ticks, z_ticks];

    for axis in 0..3 {
        let (edge0, _edge1) = axis_line_edge_points(&info, axis);
        let tickdir = AXIS_TICKDIR[axis];
        let tickdelta = if info.highs[tickdir] {
            info.deltas[tickdir]
        } else {
            -info.deltas[tickdir]
        };
        let tick_out = ax3_policy::TICK_OUTWARD_FACTOR * tickdelta;
        let tick_in = ax3_policy::TICK_INWARD_FACTOR * tickdelta;
        let edge_tick = edge0[tickdir];
        let out_tick = edge_tick + tick_out;
        let in_tick = edge_tick - tick_in;

        // axmask: do not move along the axis being labeled.
        let mut axmask = [true, true, true];
        axmask[axis] = false;

        for t in axis_ticks[axis] {
            let range = match axis {
                0 => ranges.0,
                1 => ranges.1,
                _ => ranges.2,
            };
            if !in_range(t.value, range) {
                continue;
            }
            let mut pos = edge0;
            pos[axis] = t.value;

            // Tick stub (mplot3d inward / outward along tickdir).
            let mut p_out = pos;
            p_out[tickdir] = out_tick;
            let mut p_in = pos;
            p_in[tickdir] = in_tick;
            let s_out = view.map(proj.project_data(
                Point3::new(p_out[0], p_out[1], p_out[2]),
                ranges.0,
                ranges.1,
                ranges.2,
            ));
            let s_in = view.map(proj.project_data(
                Point3::new(p_in[0], p_in[1], p_in[2]),
                ranges.0,
                ranges.1,
                ranges.2,
            ));
            renderer.draw_line(s_out, s_in, &stub_style)?;

            // Label: data-space move_from_center, then project (mplot3d).
            let mut label_pos = pos;
            label_pos[tickdir] = edge_tick;
            let moved = move_from_center(label_pos, info.centers, labeldeltas, axmask);
            let screen = view.map(proj.project_data(
                Point3::new(moved[0], moved[1], moved[2]),
                ranges.0,
                ranges.1,
                ranges.2,
            ));
            renderer.draw_text(&t.label, screen, &style)?;
        }
    }

    Ok(())
}

// ─── Line3D ──────────────────────────────────────────────────────────────────

fn draw_line3d(
    renderer: &mut dyn Renderer,
    line: &Line3D,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    px: f64,
) -> Result<()> {
    let color = line
        .color
        .unwrap_or_else(|| DEFAULT_CYCLE[line.color_index % DEFAULT_CYCLE.len()]);

    let n = line.x.len().min(line.y.len()).min(line.z.len());
    if n < 2 {
        return Ok(());
    }

    // Painter's algorithm per segment so self-crossing curves (helix) occlude
    // like mplot3d Line3DCollection (zsort='average').
    struct Seg {
        p0: Point,
        p1: Point,
        depth: f64,
    }
    let mut segs: Vec<Seg> = Vec::with_capacity(n - 1);
    let mut prev: Option<(Point, f64)> = None;
    for i in 0..n {
        let (pt, d) = project_to_pixel(
            proj,
            view,
            Point3::new(
                line.x.as_slice()[i],
                line.y.as_slice()[i],
                line.z.as_slice()[i],
            ),
            ranges,
        );
        if let Some((p0, d0)) = prev {
            segs.push(Seg {
                p0,
                p1: pt,
                depth: 0.5 * (d0 + d),
            });
        }
        prev = Some((pt, d));
    }
    segs.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut style = StrokeStyle::new(color, line.width * px);
    style.cap = LineCap::Round;
    style.join = LineJoin::Round;
    for s in &segs {
        let mut path = BezPath::new();
        path.move_to(kurbo::Point::new(s.p0.x, s.p0.y));
        path.line_to(kurbo::Point::new(s.p1.x, s.p1.y));
        renderer.stroke_path(&path, &style)?;
    }
    Ok(())
}

// ─── Scatter3D ───────────────────────────────────────────────────────────────

fn draw_scatter3d(
    renderer: &mut dyn Renderer,
    sc: &Scatter3D,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    px: f64,
) -> Result<()> {
    let color = sc
        .color
        .unwrap_or_else(|| DEFAULT_CYCLE[sc.color_index % DEFAULT_CYCLE.len()]);

    let n = sc.x.len().min(sc.y.len()).min(sc.z.len());
    let radius = sc.size * px * 0.5;

    // Sort by depth (back to front) for correct occlusion.
    let mut pts: Vec<(Point, f64)> = (0..n)
        .map(|i| {
            project_to_pixel(
                proj,
                view,
                Point3::new(sc.x.as_slice()[i], sc.y.as_slice()[i], sc.z.as_slice()[i]),
                ranges,
            )
        })
        .collect();
    pts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let (d_near, d_far) = if sc.depthshade && !pts.is_empty() {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for &(_, d) in &pts {
            lo = lo.min(d);
            hi = hi.max(d);
        }
        (lo, hi)
    } else {
        (0.0, 1.0)
    };
    let d_span = (d_far - d_near).max(1e-9);

    let base_alpha = f64::from(color.a) / 255.0;
    for (pt, depth) in pts {
        let c = if sc.depthshade {
            // Matplotlib `art3d._zalpha`: fade **alpha** with depth, keep RGB.
            let t = ((depth - d_near) / d_span).clamp(0.0, 1.0);
            let sat = 1.0 - t * ax3_policy::DEPTHSHADE_ALPHA_RANGE;
            color.with_alpha(base_alpha * sat)
        } else {
            color
        };
        let marker = crate::recipes::marker_path(
            crate::recipes::Marker { center: pt, radius },
            crate::style::MarkerStyle::Circle,
        );
        renderer.fill_path(&marker, &FillStyle::solid(c))?;
    }
    Ok(())
}

// ─── Surface3D ───────────────────────────────────────────────────────────────

fn draw_surface3d(
    renderer: &mut dyn Renderer,
    surf: &Surface3D,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    _px: f64,
) -> Result<()> {
    let nx = surf.nx;
    let ny = surf.ny;
    let zs = surf.z.as_slice();
    if zs.len() < nx * ny {
        return Ok(());
    }

    let z_min = zs.iter().copied().fold(f64::INFINITY, f64::min);
    let z_max = zs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (xs, ys) = mesh_xy(nx, ny, &surf.x, &surf.y);

    // Build quad faces with depth for sorting.
    struct Face {
        corners: [Point; 4],
        depth: f64,
        z_val: f64,
    }

    let mut faces: Vec<Face> = Vec::with_capacity((nx - 1) * (ny - 1));
    for j in 0..(ny - 1) {
        for i in 0..(nx - 1) {
            let idx = |ii: usize, jj: usize| jj * nx + ii;
            let corners_3d = [
                Point3::new(xs[i], ys[j], zs[idx(i, j)]),
                Point3::new(xs[i + 1], ys[j], zs[idx(i + 1, j)]),
                Point3::new(xs[i + 1], ys[j + 1], zs[idx(i + 1, j + 1)]),
                Point3::new(xs[i], ys[j + 1], zs[idx(i, j + 1)]),
            ];
            let mut total_depth = 0.0;
            let mut corners_2d = [Point::new(0.0, 0.0); 4];
            for (k, &c3) in corners_3d.iter().enumerate() {
                let (pt, d) = project_to_pixel(proj, view, c3, ranges);
                corners_2d[k] = pt;
                total_depth += d;
            }
            let avg_z =
                (zs[idx(i, j)] + zs[idx(i + 1, j)] + zs[idx(i + 1, j + 1)] + zs[idx(i, j + 1)])
                    / 4.0;
            faces.push(Face {
                corners: corners_2d,
                depth: total_depth / 4.0,
                z_val: avg_z,
            });
        }
    }

    // Painter's algorithm: draw back faces first.
    faces.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // No per-face edge stroke: mpl `plot_surface` defaults omit mesh edges, and
    // stroking every quad doubles raster cost on dense grids.
    for face in &faces {
        let color = if let Some(c) = surf.color {
            c.with_alpha(surf.alpha)
        } else {
            surf.cmap
                .map_norm(face.z_val, z_min, z_max, Norm::Linear)
                .with_alpha(surf.alpha)
        };

        let mut path = BezPath::new();
        path.move_to(kurbo::Point::new(face.corners[0].x, face.corners[0].y));
        for &c in &face.corners[1..] {
            path.line_to(kurbo::Point::new(c.x, c.y));
        }
        path.close_path();

        renderer.fill_path(&path, &FillStyle::solid(color))?;
    }
    Ok(())
}

// ─── Wireframe3D ─────────────────────────────────────────────────────────────

fn draw_wireframe3d(
    renderer: &mut dyn Renderer,
    wf: &Wireframe3D,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    px: f64,
) -> Result<()> {
    let nx = wf.nx;
    let ny = wf.ny;
    let zs = wf.z.as_slice();
    if zs.len() < nx * ny {
        return Ok(());
    }

    let color = wf
        .color
        .unwrap_or_else(|| DEFAULT_CYCLE[wf.color_index % DEFAULT_CYCLE.len()]);
    let mut style = StrokeStyle::new(color, wf.width * px);
    style.cap = LineCap::Round;
    style.join = LineJoin::Round;

    let idx = |i: usize, j: usize| j * nx + i;
    let (xs, ys) = mesh_xy(nx, ny, &wf.x, &wf.y);

    // Same-color opaque strokes: depth sort is a no-op visually; one multi-contour
    // stroke beats O(edges) backend calls on dense grids.
    let mut path = BezPath::new();
    let mut push_edge = |i0: usize, j0: usize, i1: usize, j1: usize| {
        let (p0, _) = project_to_pixel(
            proj,
            view,
            Point3::new(xs[i0], ys[j0], zs[idx(i0, j0)]),
            ranges,
        );
        let (p1, _) = project_to_pixel(
            proj,
            view,
            Point3::new(xs[i1], ys[j1], zs[idx(i1, j1)]),
            ranges,
        );
        path.move_to(kurbo::Point::new(p0.x, p0.y));
        path.line_to(kurbo::Point::new(p1.x, p1.y));
    };
    for j in 0..ny {
        for i in 0..(nx - 1) {
            push_edge(i, j, i + 1, j);
        }
    }
    for i in 0..nx {
        for j in 0..(ny - 1) {
            push_edge(i, j, i, j + 1);
        }
    }
    if !path.elements().is_empty() {
        renderer.stroke_path(&path, &style)?;
    }
    Ok(())
}

// ─── Bar3D ───────────────────────────────────────────────────────────────────

fn draw_bar3d(
    renderer: &mut dyn Renderer,
    bar: &Bar3D,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    px: f64,
) -> Result<()> {
    let color = bar
        .color
        .unwrap_or_else(|| DEFAULT_CYCLE[bar.color_index % DEFAULT_CYCLE.len()]);

    let n = bar.x.len().min(bar.y.len()).min(bar.z.len());

    // Each bar is a rectangular prism. We draw the 3 visible faces.
    struct BarFace {
        corners: [Point; 4],
        depth: f64,
        shade: f64,
    }

    let mut all_faces: Vec<BarFace> = Vec::new();

    // Face order matches matplotlib bar3d: -Z, +Z, -Y, +Y, -X, +X.
    let shades = ax3_policy::BAR_FACE_SHADE;

    for i in 0..n {
        // Matplotlib `bar3d`: (x,y,z) is the anchor corner; bar spans +dx,+dy,+dz.
        let x0 = bar.x.as_slice()[i];
        let y0 = bar.y.as_slice()[i];
        let h = bar.z.as_slice()[i];
        let x1 = x0 + bar.dx.abs();
        let y1 = y0 + bar.dy.abs();
        let z0 = 0.0_f64;
        let z1 = h;

        let faces_3d: [[Point3; 4]; 6] = [
            // -Z (bottom)
            [
                Point3::new(x0, y0, z0),
                Point3::new(x0, y1, z0),
                Point3::new(x1, y1, z0),
                Point3::new(x1, y0, z0),
            ],
            // +Z (top)
            [
                Point3::new(x0, y0, z1),
                Point3::new(x1, y0, z1),
                Point3::new(x1, y1, z1),
                Point3::new(x0, y1, z1),
            ],
            // -Y
            [
                Point3::new(x0, y0, z0),
                Point3::new(x1, y0, z0),
                Point3::new(x1, y0, z1),
                Point3::new(x0, y0, z1),
            ],
            // +Y
            [
                Point3::new(x0, y1, z0),
                Point3::new(x0, y1, z1),
                Point3::new(x1, y1, z1),
                Point3::new(x1, y1, z0),
            ],
            // -X
            [
                Point3::new(x0, y0, z0),
                Point3::new(x0, y0, z1),
                Point3::new(x0, y1, z1),
                Point3::new(x0, y1, z0),
            ],
            // +X
            [
                Point3::new(x1, y0, z0),
                Point3::new(x1, y1, z0),
                Point3::new(x1, y1, z1),
                Point3::new(x1, y0, z1),
            ],
        ];

        for (face_i, corners_3d) in faces_3d.iter().enumerate() {
            let mut total_depth = 0.0;
            let mut corners_2d = [Point::new(0.0, 0.0); 4];
            for (k, &c3) in corners_3d.iter().enumerate() {
                let (pt, d) = project_to_pixel(proj, view, c3, ranges);
                corners_2d[k] = pt;
                total_depth += d;
            }
            all_faces.push(BarFace {
                corners: corners_2d,
                depth: total_depth / 4.0,
                shade: shades[face_i],
            });
        }
    }

    // Sort back-to-front (matplotlib Poly3DCollection zsort='average').
    all_faces.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Matplotlib `bar3d` default has no edge strokes (`edgecolors` unset).
    let _ = px;
    for face in &all_faces {
        let shaded = shade_color(color, face.shade).with_alpha(bar.alpha);

        let mut path = BezPath::new();
        path.move_to(kurbo::Point::new(face.corners[0].x, face.corners[0].y));
        for &c in &face.corners[1..] {
            path.line_to(kurbo::Point::new(c.x, c.y));
        }
        path.close_path();

        renderer.fill_path(&path, &FillStyle::solid(shaded))?;
    }
    Ok(())
}

// ─── Contour3D ───────────────────────────────────────────────────────────────

fn draw_contour3d(
    renderer: &mut dyn Renderer,
    c: &Contour3D,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    px: f64,
) -> Result<()> {
    let nx = c.nx;
    let ny = c.ny;
    let zs = c.z.as_slice();
    if zs.len() < nx * ny || nx < 2 || ny < 2 {
        return Ok(());
    }

    let (xs, ys) = mesh_xy(nx, ny, &c.x, &c.y);
    let mut zmin = f64::INFINITY;
    let mut zmax = f64::NEG_INFINITY;
    for &v in &zs[..nx * ny] {
        if v.is_finite() {
            zmin = zmin.min(v);
            zmax = zmax.max(v);
        }
    }
    if !zmin.is_finite() || !zmax.is_finite() {
        return Ok(());
    }

    let levels = nice_levels(zmin, zmax, c.n_levels);

    struct Seg {
        p0: Point,
        p1: Point,
        depth: f64,
        color: Color,
    }

    let mut segs: Vec<Seg> = Vec::new();
    for &level in &levels {
        let color = c
            .color
            .unwrap_or_else(|| Colormap::Viridis.map_norm(level, zmin, zmax, Norm::Linear));
        let level_segs =
            contour_level_segments(zs, ny, nx, Some(xs.as_slice()), Some(ys.as_slice()), level);
        for s in level_segs {
            let (p0, d0) = project_to_pixel(proj, view, Point3::new(s.x0, s.y0, level), ranges);
            let (p1, d1) = project_to_pixel(proj, view, Point3::new(s.x1, s.y1, level), ranges);
            segs.push(Seg {
                p0,
                p1,
                depth: 0.5 * (d0 + d1),
                color,
            });
        }
    }

    segs.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for s in &segs {
        let mut style = StrokeStyle::new(s.color, c.width * px);
        style.cap = LineCap::Round;
        style.join = LineJoin::Round;
        let mut path = BezPath::new();
        path.move_to(kurbo::Point::new(s.p0.x, s.p0.y));
        path.line_to(kurbo::Point::new(s.p1.x, s.p1.y));
        renderer.stroke_path(&path, &style)?;
    }
    Ok(())
}

// ─── Quiver3D ────────────────────────────────────────────────────────────────

fn draw_quiver3d(
    renderer: &mut dyn Renderer,
    q: &Quiver3D,
    proj: &Projection,
    view: View3D,
    ranges: ((f64, f64), (f64, f64), (f64, f64)),
    px: f64,
) -> Result<()> {
    let color = q
        .color
        .unwrap_or_else(|| DEFAULT_CYCLE[q.color_index % DEFAULT_CYCLE.len()]);
    let scale = q.scale.max(1e-12);
    let n =
        q.x.len()
            .min(q.y.len())
            .min(q.z.len())
            .min(q.u.len())
            .min(q.v.len())
            .min(q.w.len());

    struct Arrow {
        shaft0: Point,
        shaft1: Point,
        head: [Point; 3],
        depth: f64,
    }

    let mut arrows: Vec<Arrow> = Vec::with_capacity(n);
    for i in 0..n {
        let x = q.x.as_slice()[i];
        let y = q.y.as_slice()[i];
        let z = q.z.as_slice()[i];
        let u = q.u.as_slice()[i] / scale;
        let v = q.v.as_slice()[i] / scale;
        let w = q.w.as_slice()[i] / scale;
        if !(x.is_finite()
            && y.is_finite()
            && z.is_finite()
            && u.is_finite()
            && v.is_finite()
            && w.is_finite())
        {
            continue;
        }
        if u * u + v * v + w * w < 1e-24 {
            continue;
        }
        let (p0, d0) = project_to_pixel(proj, view, Point3::new(x, y, z), ranges);
        let (p1, d1) = project_to_pixel(proj, view, Point3::new(x + u, y + v, z + w), ranges);
        let sx = p1.x - p0.x;
        let sy = p1.y - p0.y;
        let len = (sx * sx + sy * sy).sqrt().max(1e-9);
        let ux = sx / len;
        let uy = sy / len;
        let px_dir = -uy;
        let py_dir = ux;
        let hl_hi = len * 0.9;
        let hw_hi = len * 0.5;
        let hl = (len * 0.35).clamp(2.0_f64.min(hl_hi), hl_hi);
        let hw = (len * 0.22).clamp(1.0_f64.min(hw_hi), hw_hi);
        let tip = p1;
        let base = Point::new(tip.x - ux * hl, tip.y - uy * hl);
        let left = Point::new(base.x + px_dir * hw, base.y + py_dir * hw);
        let right = Point::new(base.x - px_dir * hw, base.y - py_dir * hw);
        arrows.push(Arrow {
            shaft0: p0,
            shaft1: base, // stop shaft at head base
            head: [tip, left, right],
            depth: 0.5 * (d0 + d1),
        });
    }

    arrows.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut stroke = StrokeStyle::new(color, q.width * px);
    stroke.cap = LineCap::Round;
    stroke.join = LineJoin::Round;
    for a in &arrows {
        let mut shaft = BezPath::new();
        shaft.move_to(kurbo::Point::new(a.shaft0.x, a.shaft0.y));
        shaft.line_to(kurbo::Point::new(a.shaft1.x, a.shaft1.y));
        renderer.stroke_path(&shaft, &stroke)?;

        let mut head = BezPath::new();
        head.move_to(kurbo::Point::new(a.head[0].x, a.head[0].y));
        head.line_to(kurbo::Point::new(a.head[1].x, a.head[1].y));
        head.line_to(kurbo::Point::new(a.head[2].x, a.head[2].y));
        head.close_path();
        renderer.fill_path(&head, &FillStyle::solid(color))?;
    }
    Ok(())
}

/// Matplotlib `_shade_colors`: multiply RGB by shade factor in `[0.3, 1]`.
fn shade_color(c: Color, shade: f64) -> Color {
    let [r, g, b, a] = c.to_rgba_u8();
    let factor = shade.clamp(0.3, 1.0);
    Color::rgba(
        (r as f64 * factor).round() as u8,
        (g as f64 * factor).round() as u8,
        (b as f64 * factor).round() as u8,
        a,
    )
}

// ─── Legend ──────────────────────────────────────────────────────────────────

fn draw_legend3d(
    renderer: &mut dyn Renderer,
    axes: &Axes3D,
    rect: Rect,
    theme: &Theme,
    dpi: f64,
    loc: Legend,
) -> Result<()> {
    let entries: Vec<(&str, Color)> = axes
        .elements
        .iter()
        .filter_map(|el| match el {
            PlotElement3D::Line(p) => p.label.as_deref().map(|l| {
                (
                    l,
                    p.color
                        .unwrap_or(DEFAULT_CYCLE[p.color_index % DEFAULT_CYCLE.len()]),
                )
            }),
            PlotElement3D::Scatter(p) => p.label.as_deref().map(|l| {
                (
                    l,
                    p.color
                        .unwrap_or(DEFAULT_CYCLE[p.color_index % DEFAULT_CYCLE.len()]),
                )
            }),
            PlotElement3D::Wireframe(p) => p.label.as_deref().map(|l| {
                (
                    l,
                    p.color
                        .unwrap_or(DEFAULT_CYCLE[p.color_index % DEFAULT_CYCLE.len()]),
                )
            }),
            PlotElement3D::Bar(p) => p.label.as_deref().map(|l| {
                (
                    l,
                    p.color
                        .unwrap_or(DEFAULT_CYCLE[p.color_index % DEFAULT_CYCLE.len()]),
                )
            }),
            PlotElement3D::Surface(p) => p.label.as_deref().map(|l| (l, Color::STEEL_BLUE)),
            PlotElement3D::Contour(p) => p.label.as_deref().map(|l| {
                (
                    l,
                    p.color
                        .unwrap_or(DEFAULT_CYCLE[p.color_index % DEFAULT_CYCLE.len()]),
                )
            }),
            PlotElement3D::Quiver(p) => p.label.as_deref().map(|l| {
                (
                    l,
                    p.color
                        .unwrap_or(DEFAULT_CYCLE[p.color_index % DEFAULT_CYCLE.len()]),
                )
            }),
        })
        .collect();

    if entries.is_empty() {
        return Ok(());
    }

    let font = points_to_px_f32(theme.tick_label_size, dpi);
    let fs = f64::from(theme.tick_label_size);
    let borderpad = points_to_px(0.4 * fs, dpi);
    let labelspacing = points_to_px(0.5 * fs, dpi);
    let handlelength = points_to_px(2.0 * fs, dpi);
    let handleheight = points_to_px(0.7 * fs, dpi);
    let inset = points_to_px(0.5 * fs, dpi);
    let text_gap = points_to_px(0.8 * fs, dpi);
    let row_h = (font as f64).max(handleheight) + labelspacing;
    let mut text_w = 0.0_f64;
    for (label, _) in &entries {
        let (w, _) = renderer.measure_text(label, font)?;
        text_w = text_w.max(w);
    }
    let box_w = borderpad * 2.0 + handlelength + text_gap + text_w;
    let box_h = borderpad * 2.0 + row_h * entries.len() as f64 - labelspacing;

    // 3D Best has no cheap data samples here — fall back to TopRight.
    let loc = loc.resolve_best(rect, box_w, box_h, inset, &[]);
    let (x0, y0) = loc.anchor(rect, box_w, box_h, inset);

    let box_rect = Rect::new(x0, y0, x0 + box_w, y0 + box_h);
    renderer.fill_rect(box_rect, &FillStyle::solid(Color::WHITE.with_alpha(0.8)))?;
    renderer.stroke_rect(
        box_rect,
        &StrokeStyle::new(theme.spine.with_alpha(0.7), points_to_px(0.8, dpi)),
    )?;

    let text_style = TextStyle::new(theme.label, font)
        .align(TextAlign::Left)
        .baseline(TextBaseline::Middle);

    for (i, (label, color)) in entries.iter().enumerate() {
        let cy = y0 + borderpad + row_h * i as f64 + (row_h - labelspacing) * 0.5;
        let sx0 = x0 + borderpad;
        let sx1 = sx0 + handlelength;
        let style = StrokeStyle::new(*color, points_to_px(1.5, dpi));
        renderer.draw_line(Point::new(sx0, cy), Point::new(sx1, cy), &style)?;
        renderer.draw_text(label, Point::new(sx1 + text_gap, cy), &text_style)?;
    }
    Ok(())
}
