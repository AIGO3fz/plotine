//! Matplotlib-aligned geometry and style policy.
//!
//! These values are **not** per-figure compare hacks. They encode stock
//! matplotlib rc / Axes behaviour so every chart type inherits the same rules:
//!
//! - [`figure`](crate::mpl_policy::figure) — stock `figsize` / DPI (plotine keeps 150 for sharper rasters)
//! - [`font`](crate::mpl_policy::font) — stock rcParams text sizes (`axes.titlesize` / `labelsize` / ticks)
//! - [`subplot`](crate::mpl_policy::subplot) — `figure.subplot.{left,right,bottom,top,hspace,wspace}`
//! - [`colorbar`](crate::mpl_policy::colorbar) — default `fig.colorbar(ax)` shrink/pad geometry
//! - [`chrome`](crate::mpl_policy::chrome) — title / axis-label gaps in points
//! - [`datetime`](crate::mpl_policy::datetime) — `autofmt_xdate` tick rotation
//! - [`polar`](crate::mpl_policy::polar) — auto `rmax` margin, ring span, title clearance
//! - [`quiver`](crate::mpl_policy::quiver) — default shaft width / head fractions / auto-scale density
//! - [`pie`](crate::mpl_policy::pie) — default view box and label distance
//! - [`barbs`](crate::mpl_policy::barbs) — matplotlib `PolyCollection` staff length scaling
//! - [`annotate`](crate::mpl_policy::annotate) — FancyArrowPatch / `arrowstyle` mutation scale
//! - [`inset`](crate::mpl_policy::inset) — inset_axes chrome pad inside the fraction box
//! - [`margin`](crate::mpl_policy::margin) — cartesian data-limit padding (`ax.margins`)
//! - [`ticks`](crate::mpl_policy::ticks) — linear major-tick density (`MaxNLocator`-ish)
//! - [`violin`](crate::mpl_policy::violin) — `violinplot` median / extrema stem fractions
//! - [`hexbin`](crate::mpl_policy::hexbin) — hex polygon scale vs lattice spacing
//! - [`axes3d`](crate::mpl_policy::axes3d) — mplot3d camera / pane / grid defaults
//! - [`geo`](crate::mpl_policy::geo) — Mercator lat clamp / default coastline stroke
//! - [`mathtext`](crate::mpl_policy::mathtext) — large-op (∫ ∑) scale / side-script nestle
//!
//! When aligning a new chart, extend this module rather than sprinkling
//! literals (`0.255`, `1.026`, …) into recipes.

#![allow(missing_docs)]

/// Stock figure size / resolution.
pub mod figure {
    /// Matplotlib default `figsize` width (inches).
    pub const WIDTH_IN: f64 = 6.4;
    /// Matplotlib default `figsize` height (inches).
    pub const HEIGHT_IN: f64 = 4.8;
    /// Plotine default DPI (sharper than matplotlib stock 100).
    pub const DPI: f64 = 150.0;
}

/// Stock rcParams text sizes (points).
pub mod font {
    /// `axes.titlesize` → `large` at `font.size=10`.
    pub const TITLE_PT: f32 = 12.0;
    /// `axes.labelsize` → `medium`.
    pub const LABEL_PT: f32 = 10.0;
    /// Tick label size → `medium`.
    pub const TICK_PT: f32 = 10.0;
}

/// Outside-legend gutter (matplotlib `bbox_to_anchor` ≈ 1.02 style).
pub mod legend {
    /// Gap from axes right edge to the legend box (points).
    pub const OUTSIDE_PAD_PT: f64 = 4.0;
    /// Estimated legend width as a multiple of tick fontsize (em) when
    /// reserving layout chrome before measuring text.
    pub const OUTSIDE_WIDTH_EM: f64 = 6.0;
}

/// Hatch stroke geometry (matplotlib `Patch` hatch density is dpi-dependent;
/// these are screen-space defaults tuned for plotine's 150 DPI default).
pub mod hatch {
    /// Spacing between hatch strokes, in CSS px at 1× `px` scale.
    pub const SPACING_PX: f64 = 8.0;
    /// Hatch stroke width multiplier × `px`.
    pub const STROKE_WIDTH: f64 = 0.85;
    /// Dot hatch marker radius multiplier × `px`.
    pub const DOT_RADIUS: f64 = 1.1;
}

/// Stock `figure.subplot.*` fractions (matplotlib defaults).
pub mod subplot {
    pub const LEFT: f64 = 0.125;
    pub const RIGHT: f64 = 0.9;
    pub const BOTTOM: f64 = 0.11;
    pub const TOP: f64 = 0.88;
    /// `figure.subplot.hspace` — fraction of **average cell height** (mpl GridSpec).
    pub const HSPACE: f64 = 0.2;
    /// `figure.subplot.wspace` — fraction of **average cell width** (mpl GridSpec).
    pub const WSPACE: f64 = 0.2;

    /// Axes width as a fraction of figure width: `RIGHT - LEFT`.
    pub const AXES_WIDTH: f64 = RIGHT - LEFT;
    /// Axes height as a fraction of figure height: `TOP - BOTTOM`.
    pub const AXES_HEIGHT: f64 = TOP - BOTTOM;

    /// Multi-panel outer pad ≈ matplotlib `tight_layout(pad=1.08)` × medium (10 pt).
    pub const TIGHT_PAD_PT: f64 = 10.8;
    /// Fraction of [`TIGHT_PAD_PT`] added below measured tick bands for the
    /// figure bottom margin (title clearance is already in the band).
    pub const TIGHT_PAD_BOTTOM_FACTOR: f64 = 0.5;
    /// Fraction of [`TIGHT_PAD_PT`] added above measured title bands for the
    /// figure top margin. Higher than bottom — titles sit closer to the figure
    /// edge after mpl `tight_layout` than a half-pad implies.
    pub const TIGHT_PAD_TOP_FACTOR: f64 = 0.85;
    /// Extra fraction of [`TIGHT_PAD_PT`] between facing interior chrome when
    /// sizing wspace/hspace (mpl subplot pad between tightbboxes).
    pub const TIGHT_INTERIOR_PAD_FACTOR: f64 = 0.95;
    /// Interior spine pad when tick/label chrome is painted into wspace/hspace.
    /// Keep small so GridSpec cells ≈ mpl axes boxes after `tight_layout`.
    pub const INTERIOR_PAD_PT: f64 = 2.5;
}

/// Default `fig.colorbar(ax)` geometry (`fraction=0.15`, `pad=0.05`).
///
/// On stock subplots this compresses axes so `x1 ≈ 0.745`.
pub mod colorbar {
    /// Axes right edge after a default colorbar (figure fraction).
    pub const AXES_X1: f64 = 0.745;
    /// Right gutter as a fraction of figure/cell width: `1 - AXES_X1`.
    pub const RIGHT_GUTTER: f64 = 1.0 - AXES_X1;
    /// Multi-panel colorbar gutter as a fraction of the available cell width.
    pub const GUTTER_FRAC: f64 = 0.20;

    /// Colorbar strip width in points (≈ mpl cax @ 150 dpi).
    pub const BAR_WIDTH_PT: f64 = 11.0;
    /// Gap from axes spine to colorbar in points.
    pub const GAP_PT: f64 = 14.0;

    /// Target count for linear colorbar ticks (dense for small integer ranges).
    ///
    /// Matches matplotlib colorbars on integer count scales (hist2d / hexbin):
    /// span 0–15 → step 2 (~9 ticks), not a sparse 5-tick Wilkinson pass.
    pub fn linear_tick_targets(vmin: f64, vmax: f64) -> usize {
        let span = (vmax - vmin).abs();
        let intish = (vmin - vmin.round()).abs() < 1e-9 && (vmax - vmax.round()).abs() < 1e-9;
        if span > 0.0 && intish && span <= 24.0 {
            // Prefer ~step 2 for mid-size integer spans (0..15 → 9 targets).
            let step2_targets = (span / 2.0).round() as usize + 1;
            step2_targets.clamp(5, 13)
        } else if span > 0.0 && span <= 5.0 {
            // Continuous imshow-like colorbars: denser ticks (mpl AutoLocator).
            7
        } else {
            5
        }
    }
}

/// Label / title clearance in typographic points (layout chrome).
pub mod chrome {
    /// Gap from axes top to title baseline (`axes.titlepad`, mpl default 6).
    pub const TITLE_GAP_PT: f64 = 6.0;
    /// When twin/secondary top x chrome is present, pin the axes title this far
    /// below the cell top (baseline ≈ Bottom/alphabetic), so it clears the
    /// twin x label inside the stock `1 - subplot.top` margin (~30 pt @ 3.5in).
    pub const TITLE_CELL_INSET_PT: f64 = 2.0;
    /// Title baseline offset as a multiple of title size from the cell top
    /// (plus [`TITLE_CELL_INSET_PT`]). Tuned so "Twin X" centroid matches mpl
    /// (~row 39 @ 150 dpi) while leaving room for the twin x label below.
    pub const TITLE_TWIN_BASELINE_EM: f64 = 0.90;
    /// Extra top inset when both an axes title and a twin/secondary top-x label
    /// are present (only used when chrome expands; twins keep stock box).
    pub const TITLE_TWIN_STACK_GAP_PT: f64 = 8.0;
    /// Clear air between tick-label band and x-axis title.
    pub const X_LABEL_GAP_PT: f64 = 14.0;
    /// Minimum inset from cell bottom for the x-axis title.
    /// Slightly larger than early drafts so autofmt "date" sits nearer mpl.
    pub const X_LABEL_EDGE_PT: f64 = 15.0;
    /// Inset from cell left for the primary y-axis title.
    pub const Y_LABEL_INSET_PT: f64 = 17.0;
    /// Tighter left inset when a right-hand twin/secondary y-axis is present
    /// (twin_y compare scene; stock left margin is shared with denser right chrome).
    pub const Y_LABEL_INSET_TWIN_Y_PT: f64 = 15.0;
    /// Inset from cell right for twin/secondary y-axis titles (stock right
    /// margin is tighter than the left; keep closer to the figure edge).
    pub const Y_LABEL_INSET_RIGHT_PT: f64 = 11.0;
    /// Gap from axes top spine to twin (top) x-axis title baseline.
    /// Must clear top tick labels (~tick_len + pad + em) but stay below the
    /// figure title inside the stock top margin — 18 pt was colliding with
    /// the title when the axes box stopped expanding for twins.
    pub const TWIN_X_LABEL_GAP_PT: f64 = 12.5;
    /// Pad from tick tip to tick label.
    pub const TICK_LABEL_PAD_PT: f64 = 3.5;
    /// Clear air between tick glyphs and the near edge of a rotated y-axis title.
    /// Total center offset = this + ~0.65·label_em (half the rotated glyph width).
    /// Slightly above early drafts so multi-panel / `at_span` labels clear dense
    /// tick strings (`-1.00`) without looking glued to the spine.
    pub const Y_LABEL_AIR_PT: f64 = 10.0;
    /// Soft right-side pad when twin/secondary y titles extend past the cell.
    pub const Y_LABEL_EDGE_PT: f64 = 2.0;
    /// Minimum x for primary y titles that paint into the figure margin
    /// (keeps `y` on-canvas for full-bleed / `at_span` panels).
    pub const Y_LABEL_FIGURE_FLOOR_PT: f64 = 9.0;
}

/// Datetime tick formatting (`ConciseDateFormatter` + `autofmt_xdate`).
pub mod datetime {
    /// Concise day/month labels are short; keep a mild lean for dense axes.
    /// Screen y-down ⇒ negative for the same visual lean as mpl `rotation=30`.
    pub const TICK_ROTATION_DEG: f64 = -30.0;
    /// Vertical extent factor × tick font size for measuring rotated labels.
    pub const ROTATED_BAND_EM: f64 = 0.9;
    /// Final bottom subplot edge after `autofmt_xdate(bottom=0.2)` (figure fraction).
    /// Tick labels and the x-axis title share this band — do not add more below it.
    pub const AUTOFMT_BOTTOM: f64 = 0.20;
    /// When placing the x-axis title under rotated ticks, use this fraction of the
    /// measured rotated band (full band +
    /// [`chrome::X_LABEL_GAP_PT`](crate::mpl_policy::chrome::X_LABEL_GAP_PT)
    /// overshoots mpl).
    pub const X_LABEL_BAND_FRAC: f64 = 0.72;
}

/// Polar Axes auto limits (matplotlib `PolarAxes`).
pub mod polar {
    /// `rmax / data_rmax` after mpl's auto radial margin (~2.6%).
    pub const R_MARGIN: f64 = 1.026;
    /// Span passed to nice ring levels (slightly under `rmax` so steps stay at 0.2).
    pub const RING_LEVEL_FACTOR: f64 = 1.02;
    /// MaxNLocator-style nbins hint for radial rings (~mpl polar near unit `r`).
    pub const RING_N_HINT: usize = 5;
    /// Minimum clearance between polar title bottom and axes top (points).
    /// Used when measuring the polar top inset (draw stacks via `top_x_chrome_anchors`).
    pub const TITLE_CLEAR_PT: f64 = 28.0;
    /// Extra top inset (points) reserved for the `90°` angle label outside the ring.
    pub const ANGLE_LABEL_TOP_PT: f64 = 14.0;

    #[inline]
    pub fn rmax_from_data(data_rmax: f64) -> f64 {
        data_rmax.abs().max(1e-9) * R_MARGIN
    }

    #[inline]
    pub fn ring_level_span(data_rmax: f64) -> f64 {
        data_rmax.abs().max(1e-9) * RING_LEVEL_FACTOR
    }
}

/// Scatter marker defaults (matplotlib `Axes.scatter`).
pub mod scatter {
    use std::f64::consts::PI;

    /// Matplotlib default `scatter(..., s=36)` as marker **diameter** in points
    /// (`diameter = 2·√(s/π)`).
    pub const DEFAULT_DIAMETER_PT: f64 = 6.770_275_002_573_076;

    /// Convert matplotlib `s` (marker area in pt²) to plotine diameter in points.
    #[inline]
    pub fn diameter_from_area_pt2(s: f64) -> f64 {
        2.0 * (s.max(0.0) / PI).sqrt()
    }
}

/// Quiver arrow defaults (matplotlib `FancyArrow` / `quiver` stock look).
pub mod quiver {
    /// Shaft stroke width in points (~mpl auto `width≈0.0075` → ~1.4 pt at
    /// figsize 5×4 / dpi 150).
    pub const WIDTH_PT: f64 = 1.4;
    /// Head length as a multiple of shaft width (matplotlib `headlength=5`).
    pub const HEAD_LENGTH_WIDTH: f64 = 5.0;
    /// Head full-width as a multiple of shaft width (matplotlib `headwidth=3`).
    pub const HEAD_WIDTH_WIDTH: f64 = 3.0;
    /// Auto-scale density: aim for arrows spanning `1/SPAN_DIV` of the axes.
    /// Tuned so max arrow length ≈ stock mpl auto-scale on the compare grid.
    pub const SPAN_DIV: f64 = 18.0;
}

/// Pie defaults matching `ax.pie(..., startangle=90, counterclock=False)`.
pub mod pie {
    /// View half-extent in data units (`xlim = ylim = ±VIEW`).
    pub const VIEW: f64 = 1.25;
    /// Wedge radius in data units (matplotlib default `radius=1`).
    pub const RADIUS: f64 = 1.0;
    /// Wedge radius as a fraction of the plot-rect side.
    ///
    /// Data span is `2 * VIEW`, so `radius_px = side * RADIUS / (2 * VIEW)`.
    pub const RADIUS_FRAC: f64 = RADIUS / (2.0 * VIEW);
    /// Label distance as a multiple of radius (`labeldistance=1.1`).
    pub const LABEL_DISTANCE: f64 = 1.1;
}

/// Inset axes geometry (matplotlib `inset_axes`, axes-fraction).
pub mod inset {
    /// Matplotlib: the fraction box *is* the axes bbox; tick/title chrome paints
    /// outside into the parent. Keep 0 — an inner pad shrinks nested content
    /// until `TickLocator` falls back to raw padded endpoints (`0.114`…).
    pub const CHROME_PAD_PT: f64 = 0.0;
}

/// Annotate arrow sizing (matplotlib `FancyArrowPatch` / `arrowstyle`).
pub mod annotate {
    /// Stock `Annotation.arrow_patch.get_mutation_scale()` with `arrowstyle=…`.
    pub const MUTATION_SCALE_PT: f64 = 10.0;
    /// Default `FancyArrowPatch` / annotate arrow linewidth (points).
    pub const LINEWIDTH_PT: f64 = 1.0;
    /// `FancyArrowPatch.shrinkA` / `shrinkB` in points (after patchA clip).
    pub const SHRINK_PT: f64 = 2.0;
    /// Padding around the annotation text box used as `patchA` (matplotlib: 4 pt).
    pub const TEXT_PAD_PT: f64 = 4.0;
    /// `ArrowStyle` head length × mutation_scale (`-|>` / `->` / `<->`).
    pub const HEAD_LENGTH: f64 = 0.4;
    /// `ArrowStyle` head width × mutation_scale.
    pub const HEAD_WIDTH: f64 = 0.2;
    /// `-[` BracketB `widthB` × mutation_scale — half-extent of the crossbar
    /// (mpl `get_normal_points` distance from center; full bar ≈ `2·widthB·ms`).
    pub const BRACKET_WIDTH: f64 = 1.0;
    /// `-[` BracketB `lengthB` × mutation_scale (arm length past the tip).
    pub const BRACKET_LENGTH: f64 = 0.2;
}

/// Barb staff sizing (matplotlib `Barbs` → `PolyCollection`).
pub mod barbs {
    /// After unit verts of height `length`, `sizes=(length²/4,)` scales by
    /// `length/2`, so the drawn staff is `length²/2` points tall.
    pub const STAFF_PT_SQUARE_SCALE: f64 = 0.5;

    #[inline]
    pub fn staff_length_px(length_pt: f64, px_per_pt: f64) -> f64 {
        length_pt * length_pt * STAFF_PT_SQUARE_SCALE * px_per_pt
    }
}

/// Cartesian `ax.margins` style padding on open data limits.
pub mod margin {
    pub const PAD: f64 = 0.05;
}

/// Matplotlib `Axes.violinplot` stem geometry (`cbook.violin_stats` / PolyCollection).
pub mod violin {
    /// Half-width of the median bar and extrema end-caps, as a fraction of
    /// `widths` (mpl `line_ends = ±0.25 * widths`).
    pub const STEM_HALF_WIDTH_FRAC: f64 = 0.25;
    /// Default KDE sample count along each density (`points=100`).
    pub const POINTS: usize = 100;
    /// Default body width fraction of unit category spacing.
    pub const WIDTHS: f64 = 0.5;
}

/// Matplotlib `Axes.hexbin` lattice / polygon sizing.
pub mod hexbin {
    /// Polygon scale vs lattice spacing. Stock mpl uses `1.0`; larger values
    /// seal AA gaps but inflate cells relative to Agg.
    pub const POLYGON_SCALE: f64 = 1.0;
}

/// Linear major-tick density (matplotlib `MaxNLocator` / `AutoLocator`).
pub mod ticks {
    /// Fallback / large-axes major tick target (`AutoLocator` clips `nbins` ≤ 9,
    /// so at most ~10 ticks). Used when axis pixel length is unknown.
    pub const LINEAR_TARGETS: usize = 9;
    /// Cap matching `MaxNLocator` `nbins='auto'` clip upper bound.
    pub const AUTO_NBINS_MAX: usize = 9;
    /// `XAxis.get_tick_space`: label size × this aspect.
    pub const TICK_SPACE_ASPECT_X: f64 = 3.0;
    /// `YAxis.get_tick_space`: label size × this aspect.
    pub const TICK_SPACE_ASPECT_Y: f64 = 2.0;

    /// Matplotlib `Axis.get_tick_space` → `nbins` (`clip(..., 1, 9)`).
    pub fn auto_nbins(axis_len_px: f64, dpi: f64, tick_label_size_pt: f64, is_x: bool) -> usize {
        let length_pt = axis_len_px * 72.0 / dpi.max(1e-6);
        let aspect = if is_x {
            TICK_SPACE_ASPECT_X
        } else {
            TICK_SPACE_ASPECT_Y
        };
        let size = tick_label_size_pt * aspect;
        let space = if size > 0.0 {
            (length_pt / size).floor().max(0.0) as usize
        } else {
            AUTO_NBINS_MAX
        };
        space.clamp(1, AUTO_NBINS_MAX)
    }

    /// Target count for [`plotine_core::TickLocator`] from axis pixel length.
    ///
    /// `TickLocator` divides the span by `targets - 1`, matching MaxNLocator's
    /// `raw_step = span / nbins` when `targets = nbins + 1`.
    pub fn auto_targets(axis_len_px: f64, dpi: f64, tick_label_size_pt: f64, is_x: bool) -> usize {
        auto_nbins(axis_len_px, dpi, tick_label_size_pt, is_x) + 1
    }
}

/// Matplotlib `Axes3D` / mplot3d stock defaults.
pub mod axes3d {
    use plotine_core::Color;

    /// Default elevation (degrees).
    pub const ELEV: f64 = 30.0;
    /// Default azimuth (degrees).
    pub const AZIM: f64 = -60.0;
    /// Default Axes3D box aspect ratio `x:y:z = 4:4:3` (`set_box_aspect(None)`).
    pub const BOX_ASPECT_RATIO: [f64; 3] = [4.0, 4.0, 3.0];
    /// Matplotlib scales `(4,4,3)` by `1.8294640721620434 * 25/24 / ‖aspect‖`.
    pub fn box_aspect() -> [f64; 3] {
        let [ax, ay, az] = BOX_ASPECT_RATIO;
        let norm = (ax * ax + ay * ay + az * az).sqrt();
        let scale = 1.829_464_072_162_043_4 * (25.0 / 24.0) / norm;
        [ax * scale, ay * scale, az * scale]
    }
    /// Camera distance used by `Axes3D.get_proj` (matplotlib default).
    pub const DIST: f64 = 10.0;
    /// Perspective focal length (matplotlib default `1`; `inf` would be ortho).
    pub const FOCAL_LENGTH: f64 = 1.0;
    /// `bar3d(shade=True)` light: `LightSource(azdeg=225, altdeg=19.4712)`.
    /// Shade factors for faces `(-Z,+Z,-Y,+Y,-X,+X)` after mpl `_shade_colors`.
    pub const BAR_FACE_SHADE: [f64; 6] = [0.5333, 0.7667, 0.8833, 0.4167, 0.8833, 0.4167];
    /// Matplotlib `art3d._zalpha`: `sats = 1 - norm(z) * DEPTHSHADE_ALPHA_RANGE`
    /// then multiply marker **alpha** (RGB unchanged). Far points → α × 0.3.
    pub const DEPTHSHADE_ALPHA_RANGE: f64 = 0.7;
    /// Default 3D scatter diameter in points for matplotlib `s=16`
    /// (`diameter = 2·√(s/π)` ≈ 4.513).
    pub const SCATTER_DIAMETER_PT: f64 = 4.513_519_491_406_296;
    /// Pane face for a fixed axis (`0=x, 1=y, 2=z`).
    ///
    /// Matplotlib defaults are `(0.95 / 0.90 / 0.925, α=0.5)` over white. We use
    /// the **premultiplied** opaque RGB so overlapping back-faces do not stack
    /// alpha and go muddy gray.
    pub fn pane_face(axis: usize) -> Color {
        match axis {
            1 => Color::rgb(243, 243, 243), // 0.90×0.5 + 1×0.5
            2 => Color::rgb(245, 245, 245), // 0.925×0.5 + 1×0.5
            _ => Color::rgb(248, 248, 248), // 0.95×0.5 + 1×0.5
        }
    }
    /// Pane grid: matplotlib `axes3d` grid color `#b0b0b0`, linewidth 0.8.
    pub fn grid_color() -> Color {
        Color::rgb(0xb0, 0xb0, 0xb0)
    }
    /// Grid stroke width in points.
    pub const GRID_WIDTH_PT: f64 = 0.8;
    /// Cube edge / spine width in points.
    pub const EDGE_WIDTH_PT: f64 = 0.8;
    /// Target major tick count per axis.
    ///
    /// Stock `Axes3D` on ~6×5in @150dpi lands near `MaxNLocator(nbins=10)`
    /// (`AutoLocator` with `nbins='auto'`), which needs `targets = nbins + 1`
    /// for our locator's `span / (targets - 1)` step. Using 2D's fallback of 9
    /// under-ticks x/y (step 0.5 vs mpl 0.25 on the helix).
    pub const TICK_TARGETS: usize = 11;
    /// Matplotlib `Axes3D._view_margin` (`1/48`) applied in `_set_lim3d` after
    /// cartesian margins (when `axes3d.automargin` is on — the default).
    pub const VIEW_MARGIN: f64 = 1.0 / 48.0;
    /// Default `Axes3D` subplot position (figure fraction) — mpl 3.10 on a
    /// lone `add_subplot(111, projection='3d')`: `(0.1917, 0.11, 0.6417×0.77)`.
    pub const AXES_LEFT: f64 = 0.191_666_666_666_666_65;
    pub const AXES_BOTTOM: f64 = 0.11;
    pub const AXES_WIDTH: f64 = 0.641_666_666_666_666_7;
    pub const AXES_HEIGHT: f64 = 0.77;
    /// Extra shrink after fitting the projected cube into the Axes3D box
    /// (room for tick labels just outside the cube).
    ///
    /// Tuned against `compare/plotine_*_3d.png` vs `mpl_*_3d.png` (2026-07-19):
    /// 0.90 under-filled (content AABB ~4% small); 0.93 helped bar/gaussian but
    /// raised scatter/surface. 0.92 remains the mean-MSE sweet spot after
    /// α-depthshade + segment z-sort on helix/wireframe.
    pub const FIT_SHRINK: f64 = 0.92;
    /// mplot3d `Axis._calc_centers_deltas` scale (`0.08` since mpl 3.9).
    pub const DELTA_SCALE: f64 = 0.08;
    /// mplot3d `_axinfo['tick']['inward_factor']`.
    pub const TICK_INWARD_FACTOR: f64 = 0.2;
    /// mplot3d `_axinfo['tick']['outward_factor']`.
    pub const TICK_OUTWARD_FACTOR: f64 = 0.1;
    /// Stock major tick pad in points (`xtick.major.pad` / `ytick.major.pad` = 3.5).
    pub const TICK_LABEL_PAD_PT: f64 = 3.5;
    /// mplot3d `_draw_ticks` `default_label_offset` (points).
    pub const TICK_LABEL_OFFSET_PT: f64 = 8.0;
    /// Numerator of mplot3d `deltas_per_point = 48 / ax_points_estimate`.
    pub const DELTAS_PER_POINT_NUM: f64 = 48.0;
}

/// Built-in mathtext metrics — copied from matplotlib `_mathtext.FontConstantsBase`
/// (stock `mathtext.fontset = 'dejavusans'`; DejaVu Sans/Serif subclasses share
/// these numbers). Glyphs come from embedded DejaVu Sans + Oblique.
///
/// See `matplotlib/_mathtext.py` `FontConstantsBase` and `Parser.subsuper`
/// dropsub branch for slanted integrals.
pub mod mathtext {
    /// Display-style large ops (`\displaystyle` / `\limits`): scale vs surrounding size.
    pub const LARGE_OP_DISPLAY_SCALE: f32 = 1.35;
    /// Textstyle large ops match surrounding size (mpl does not upscale ∫ in titles).
    pub const LARGE_OP_TEXTSTYLE_SCALE: f32 = 1.0;
    /// Gap above/below large op for stacked `\limits` / displaystyle (× size).
    pub const LARGE_OP_LIMIT_GAP: f64 = 0.08;

    // --- FontConstantsBase (DejaVu*) — × x-height unless noted ---
    /// Extra horiz. space after ordinary sub/superscripts (`script_space`).
    pub const SCRIPT_SPACE: f64 = 0.05;
    /// Sub/superscript drop for dropsub nuclei (`subdrop`).
    pub const SUBDROP: f64 = 0.4;
    /// Superscript raise for ordinary nuclei (`sup1`).
    pub const SUP1: f64 = 0.7;
    /// Subscript drop when alone (`sub1`).
    pub const SUB1: f64 = 0.3;
    /// Subscript drop when a superscript is present (`sub2`).
    pub const SUB2: f64 = 0.5;
    /// Nucleus-edge kern (`delta`).
    pub const DELTA: f64 = 0.025;
    /// Extra slanted-nucleus kern (`delta_slanted`).
    pub const DELTA_SLANTED: f64 = 0.2;
    /// Integral-specific kern (`delta_integral`).
    pub const DELTA_INTEGRAL: f64 = 0.1;
    /// Approximate x-height as a fraction of surrounding size (DejaVu).
    pub const X_HEIGHT_FRAC: f64 = 0.44;

    // --- Parser.sqrt (`matplotlib/_mathtext.py`) ---
    /// Root index vertical shift as a fraction of radical height (`-height * 0.6`).
    pub const SQRT_ROOT_SHIFT: f64 = 0.6;
    /// Negative kern as a fraction of check (radical stem) width (`-0.5 * check.width`).
    pub const SQRT_ROOT_KERN: f64 = 0.5;
    /// Empty root placeholder width when `\sqrt` has no index (`0.5 * check.width`).
    pub const SQRT_EMPTY_ROOT_FRAC: f64 = 0.5;
    /// Extra clearance above the body before sizing the radical (`5 * thickness`).
    pub const SQRT_BODY_CLEARANCE_THICKNESS: f64 = 5.0;
    /// Horizontal pad on each side of the radicand (`2 * thickness`).
    pub const SQRT_BODY_SIDE_PAD_THICKNESS: f64 = 2.0;
}

/// Geographic projection defaults (cartopy-thin).
pub mod geo {
    /// Mercator latitude clamp (Web Mercator / EPSG:3857 convention).
    pub const MERCATOR_MAX_LAT: f64 = 85.051_128_78;
    /// Default coastline stroke width in points.
    pub const COASTLINE_WIDTH_PT: f64 = 0.6;
}

/// Whether a panel's chrome needs more room than stock subplot fractions.
///
/// Used by tight-layout: keep the mpl axes box for plain cartesian plots, but
/// expand when colorbar / polar θ labels / secondary axes need gutters.
/// **Datetime does not expand via this flag**: `autofmt_xdate(bottom=0.2)` only
/// raises the bottom edge (see [`tight_layout_for_grid`](crate::layout::tight_layout_for_grid));
/// left/right/top stay
/// stock so the axes width matches mpl `0.775`.
/// **Twins do not expand**: matplotlib `twinx`/`twiny` keep the host subplot box
/// (`0.125,0.11,0.775×0.77`) and paint twin ticks/labels into the existing
/// figure margins — expanding would shrink the axes and under-tick the host
/// (e.g. twin_x y-step 1.0 vs mpl 0.5).
pub fn chrome_expands_stock_insets(axes: &crate::axes::Axes) -> bool {
    axes.needs_colorbar()
        || axes.secondary_x.is_some()
        || axes.secondary_y.is_some()
        || axes.polar
        || axes.legend.is_some_and(|l| l.is_outside())
        // Display-style mathtext titles (∫ limits, fractions, …) can exceed the
        // stock 12% top margin; expand via measured insets (see layout::measure_insets).
        || axes
            .title
            .as_deref()
            .is_some_and(crate::mathtext::needs_mathtext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subplot_box_matches_stock_mpl() {
        assert!((subplot::AXES_WIDTH - 0.775).abs() < 1e-12);
        assert!((subplot::AXES_HEIGHT - 0.77).abs() < 1e-12);
    }

    #[test]
    fn colorbar_gutter_left_of_stock_right() {
        const { assert!(colorbar::AXES_X1 < subplot::RIGHT) };
        assert!((colorbar::RIGHT_GUTTER - 0.255).abs() < 1e-12);
    }

    #[test]
    fn pie_radius_frac_is_half_view_ratio() {
        assert!((pie::RADIUS_FRAC - 0.4).abs() < 1e-12);
        assert!((pie::RADIUS / (2.0 * pie::VIEW) - pie::RADIUS_FRAC).abs() < 1e-12);
    }

    #[test]
    fn polar_rmax_scales_data() {
        assert!((polar::rmax_from_data(1.35) - 1.35 * 1.026).abs() < 1e-12);
    }

    #[test]
    fn barbs_staff_matches_polycollection() {
        // length=7 pt @ 150 dpi → px_per_pt = 150/72; staff = 7²/2 × px/pt
        let px = 150.0 / 72.0;
        let got = barbs::staff_length_px(7.0, px);
        assert!((got - 7.0 * 7.0 * 0.5 * px).abs() < 1e-12);
    }

    #[test]
    fn twins_keep_stock_subplot_box() {
        // twinx/twiny paint into figure margins; expanding shrinks axes and
        // under-ticks the host (compare twin_x y-step).
        let mut ax = crate::axes::Axes::new();
        assert!(!chrome_expands_stock_insets(&ax));
        ax.twin_y(|t| {
            t.y_label("r");
        });
        assert!(!chrome_expands_stock_insets(&ax));
        let mut ax = crate::axes::Axes::new();
        ax.twin_x(|t| {
            t.x_label("top");
        });
        assert!(!chrome_expands_stock_insets(&ax));
    }

    #[test]
    fn datetime_does_not_expand_via_chrome_flag() {
        // autofmt only raises bottom; width stays stock 0.775.
        let mut ax = crate::axes::Axes::new();
        ax.x_datetime(true);
        assert!(!chrome_expands_stock_insets(&ax));
    }

    #[test]
    fn outside_legend_expands_stock_box() {
        let mut ax = crate::axes::Axes::new();
        assert!(!chrome_expands_stock_insets(&ax));
        ax.legend(crate::legend::Legend::OutsideUpperRight);
        assert!(chrome_expands_stock_insets(&ax));
    }

    #[test]
    fn colorbar_dense_ticks_for_small_int_span() {
        // span/2 + 1, clamped to [5, 13] — e.g. 0..15 → 9, 0..9 → 6.
        assert_eq!(colorbar::linear_tick_targets(0.0, 15.0), 9);
        assert_eq!(colorbar::linear_tick_targets(0.0, 9.0), 6);
        assert_eq!(colorbar::linear_tick_targets(0.0, 1.0), 5);
        assert_eq!(colorbar::linear_tick_targets(0.0, 100.0), 5);
        assert_eq!(colorbar::linear_tick_targets(-0.9, 2.0), 7);
    }

    #[test]
    fn violin_stem_matches_mpl_line_ends() {
        assert!((violin::STEM_HALF_WIDTH_FRAC - 0.25).abs() < 1e-12);
        assert_eq!(violin::POINTS, 100);
        assert!((violin::WIDTHS - 0.5).abs() < 1e-12);
        assert!((hexbin::POLYGON_SCALE - 1.0).abs() < 1e-12);
    }
}
