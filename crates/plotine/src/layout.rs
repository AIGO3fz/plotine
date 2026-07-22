//! Figure / subplot layout: grid cells + tight-layout margin solving.

use plotine_core::{Point, Rect, Size};
use plotine_render::Renderer;

use crate::axes::Axes;
use crate::mpl_policy::{
    self, chrome as chrome_policy, colorbar as cbar_policy, datetime as datetime_policy,
    legend as legend_policy, polar as polar_policy, subplot as subplot_policy,
    ticks as ticks_policy,
};
use crate::theme::{points_to_px, points_to_px_f32, Theme};

/// Grid geometry for one or more subplot panels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridSpec {
    /// Number of subplot rows.
    pub nrows: usize,
    /// Number of subplot columns.
    pub ncols: usize,
    /// Vertical gap between rows as a fraction of average cell height.
    pub hspace: f64,
    /// Horizontal gap between columns as a fraction of average cell width.
    pub wspace: f64,
    /// Left outer margin as a fraction of figure width.
    pub left: f64,
    /// Right outer margin as a fraction of figure width.
    pub right: f64,
    /// Bottom outer margin as a fraction of figure height.
    pub bottom: f64,
    /// Top outer margin as a fraction of figure height.
    pub top: f64,
}

impl Default for GridSpec {
    fn default() -> Self {
        Self {
            nrows: 1,
            ncols: 1,
            hspace: subplot_policy::HSPACE,
            wspace: subplot_policy::WSPACE,
            // Full figure; axes box uses matplotlib `figure.subplot.*` insets.
            left: 0.0,
            right: 1.0,
            bottom: 0.0,
            top: 1.0,
        }
    }
}

impl GridSpec {
    /// Create an `nrows × ncols` grid with default margins and spacing.
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Self {
            nrows: nrows.max(1),
            ncols: ncols.max(1),
            ..Self::default()
        }
    }

    /// Vertical gap between rows as a fraction of average cell height.
    pub fn hspace(mut self, v: f64) -> Self {
        self.hspace = v.clamp(0.0, 1.0);
        self
    }

    /// Horizontal gap between columns as a fraction of average cell width.
    pub fn wspace(mut self, v: f64) -> Self {
        self.wspace = v.clamp(0.0, 1.0);
        self
    }

    /// Cell rectangle for `(row, col)` in pixel coordinates (y grows downward).
    ///
    /// `wspace` / `hspace` follow matplotlib `GridSpec`: gap as a fraction of the
    /// **average cell** size, i.e.
    /// `cell = area / (n + (n-1)·space)`, `gap = cell · space`.
    pub fn cell_rect(self, figure: Size, row: usize, col: usize) -> Rect {
        let nrows = self.nrows.max(1) as f64;
        let ncols = self.ncols.max(1) as f64;
        let x0 = figure.width * self.left;
        let x1 = figure.width * self.right;
        let y0 = figure.height * (1.0 - self.top);
        let y1 = figure.height * (1.0 - self.bottom);
        let area_w = (x1 - x0).max(1.0);
        let area_h = (y1 - y0).max(1.0);

        let cell_w = area_w / (ncols + (ncols - 1.0) * self.wspace);
        let cell_h = area_h / (nrows + (nrows - 1.0) * self.hspace);
        let gap_w = if ncols > 1.0 {
            cell_w * self.wspace
        } else {
            0.0
        };
        let gap_h = if nrows > 1.0 {
            cell_h * self.hspace
        } else {
            0.0
        };

        let col = col.min(self.ncols.saturating_sub(1)) as f64;
        let row = row.min(self.nrows.saturating_sub(1)) as f64;
        let cx0 = x0 + col * (cell_w + gap_w);
        let cy0 = y0 + row * (cell_h + gap_h);
        Rect::new(cx0, cy0, cx0 + cell_w, cy0 + cell_h)
    }

    /// Rectangle covering the cells from `(row, col)` spanning `rowspan × colspan`.
    pub fn span_rect(
        self,
        figure: Size,
        row: usize,
        col: usize,
        rowspan: usize,
        colspan: usize,
    ) -> Rect {
        let rowspan = rowspan.max(1);
        let colspan = colspan.max(1);
        let r1 = (row + rowspan - 1).min(self.nrows.saturating_sub(1));
        let c1 = (col + colspan - 1).min(self.ncols.saturating_sub(1));
        let a = self.cell_rect(figure, row, col);
        let b = self.cell_rect(figure, r1, c1);
        Rect::new(
            a.x0.min(b.x0),
            a.y0.min(b.y0),
            a.x1.max(b.x1),
            a.y1.max(b.y1),
        )
    }
}

/// Pixel insets from a cell edge to the axes box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insets {
    /// Left margin in pixels.
    pub left: f64,
    /// Right margin in pixels.
    pub right: f64,
    /// Top margin in pixels.
    pub top: f64,
    /// Bottom margin in pixels.
    pub bottom: f64,
}

impl Insets {
    /// Zero insets on all sides.
    pub fn zero() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        }
    }

    /// Component-wise maximum of two inset sets.
    pub fn max_with(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            right: self.right.max(other.right),
            top: self.top.max(other.top),
            bottom: self.bottom.max(other.bottom),
        }
    }

    /// Clamp insets so the axes box remains a usable fraction of `cell`.
    pub fn clamp_to_cell(self, cell: Rect) -> Self {
        Self {
            left: self.left.min(cell.width() * 0.35),
            // Allow a wider right gutter for colorbars.
            right: self.right.min(cell.width() * 0.28),
            top: self.top.min(cell.height() * 0.30),
            bottom: self.bottom.min(cell.height() * 0.35),
        }
    }
}

/// Layout for a single axes panel inside a cell.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Full subplot cell rectangle in figure pixels.
    pub cell: Rect,
    /// Inner axes box where data is drawn.
    pub axes: Rect,
}

impl Layout {
    /// Backward-compatible single-panel layout over the whole figure.
    pub fn compute(
        figure: Size,
        axes: &Axes,
        theme: &Theme,
        renderer: &dyn Renderer,
        dpi: f64,
    ) -> Self {
        let cell = Rect::new(0.0, 0.0, figure.width, figure.height);
        Self::compute_in_cell(cell, axes, theme, renderer, dpi)
    }

    /// Compute a publication-friendly axes box inside `cell`.
    pub fn compute_in_cell(
        cell: Rect,
        axes: &Axes,
        theme: &Theme,
        renderer: &dyn Renderer,
        dpi: f64,
    ) -> Self {
        let insets = measure_insets(cell, axes, theme, renderer, dpi).clamp_to_cell(cell);
        Self::from_insets(cell, insets)
    }

    /// Build an axes box from explicit insets (used by tight-layout).
    pub fn from_insets(cell: Rect, insets: Insets) -> Self {
        let insets = insets.clamp_to_cell(cell);
        let axes = Rect::new(
            cell.x0 + insets.left,
            cell.y0 + insets.top,
            cell.x1 - insets.right,
            cell.y1 - insets.bottom,
        );
        Self { cell, axes }
    }

    /// Anchor point for the axes title text.
    pub fn title_anchor(&self, dpi: f64) -> Point {
        Point::new(
            self.axes.center().x,
            self.axes.y0 - points_to_px(chrome_policy::TITLE_GAP_PT, dpi),
        )
    }

    /// Adaptive anchors for figure title + optional twin/secondary top-x label.
    ///
    /// Stacks from the spine upward: tick band → top-x label → title, then
    /// compresses toward the cell top if the stock margin is tight. Both use
    /// [`TextBaseline::Bottom`](crate::TextBaseline::Bottom).
    pub fn top_x_chrome_anchors(
        &self,
        dpi: f64,
        title_size_pt: Option<f64>,
        top_x_label_size_pt: Option<f64>,
        tick_band: f64,
    ) -> (Option<Point>, Option<Point>) {
        let gap = points_to_px(chrome_policy::TWIN_X_LABEL_GAP_PT, dpi);
        let stack = points_to_px(chrome_policy::TITLE_TWIN_STACK_GAP_PT, dpi);
        let edge = points_to_px(chrome_policy::TITLE_CELL_INSET_PT, dpi);
        let tick_outer = self.axes.y0 - tick_band;

        let label_h = top_x_label_size_pt
            .map(|pt| points_to_px(pt * 0.95, dpi))
            .unwrap_or(0.0);
        let title_h = title_size_pt
            .map(|pt| points_to_px(pt * 0.90, dpi))
            .unwrap_or(0.0);

        // Prefer label just above the tick band.
        let mut label_y = if top_x_label_size_pt.is_some() {
            Some(tick_outer - gap)
        } else {
            None
        };

        // Title sits above the label (or above the top tick band / spine).
        let mut title_y = title_size_pt.map(|_| {
            if let Some(ly) = label_y {
                ly - stack - label_h
            } else {
                // Spy / polar θ / any top tick band: clear the outer tick edge,
                // not the spine (ignoring `tick_band` put titles on the axes).
                tick_outer - points_to_px(chrome_policy::TITLE_GAP_PT, dpi)
            }
        });

        // Bottom baseline: glyph body sits mostly above the anchor.
        // Floor at the *figure* top (not `cell.y0`) so multi-panel full-bleed
        // titles can use the outer margin that tight_layout already reserved
        // (polar `90°` + title). Clamping to `cell.y0` was pushing "Polar"
        // down onto the ring.
        if let Some(ty) = title_y.as_mut() {
            let min_ty = edge + title_h * 0.85;
            if *ty < min_ty {
                *ty = min_ty;
            }
            if let Some(ly) = label_y.as_mut() {
                let below_title = *ty + stack;
                let above_ticks = tick_outer - gap;
                *ly = below_title.max(*ly).min(above_ticks).max(below_title);
            }
        }

        let cx = self.axes.center().x;
        (
            title_y.map(|y| Point::new(cx, y)),
            label_y.map(|y| Point::new(cx, y)),
        )
    }

    /// Anchor point for the x-axis label text.
    ///
    /// `below_ticks` is the pixel depth of the tick-label band under the spine
    /// (larger for rotated datetime labels) so the axis title stays clear of it.
    pub fn x_label_anchor(&self, dpi: f64, below_ticks: f64) -> Point {
        let gap = points_to_px(chrome_policy::X_LABEL_GAP_PT, dpi);
        let min_gap = points_to_px(2.0, dpi);
        // Clear the tick band; keep the Top-baseline glyph inside the cell.
        let tick_bottom = self.axes.y1 + below_ticks;
        let ideal = tick_bottom + gap;
        let max_y = self.cell.y1 - min_gap;
        let y = ideal.min(max_y).max(tick_bottom + min_gap);
        Point::new(self.axes.center().x, y)
    }

    /// Anchor point for the y-axis label text.
    ///
    /// Sits just outside the measured y tick-label band. `label_size_pt` accounts
    /// for the horizontal half-width of ±90° rotated text so long words like
    /// "value" / "category" do not collide with tick digits.
    ///
    /// Placement may extend left of [`Self::cell`] into the figure margin —
    /// multi-panel `tight_layout` / full-bleed keeps only a spine pad inside the
    /// cell and expects axis titles to paint in the outer chrome (mpl-like).
    pub fn y_label_anchor(&self, dpi: f64, tick_band: f64, label_size_pt: f64) -> Point {
        let gap = y_label_center_gap_px(dpi, label_size_pt);
        let floor = points_to_px(chrome_policy::Y_LABEL_FIGURE_FLOOR_PT, dpi);
        let tick_outer = self.axes.x0 - tick_band;
        let hi = self.axes.x0 - gap;
        let ideal = tick_outer - gap;
        let x = ideal.min(hi).max(floor);
        Point::new(x, self.axes.center().y)
    }

    /// Anchor point for a twin/secondary (right-hand) y-axis label.
    ///
    /// May extend right of [`Self::cell`] into the figure margin (see
    /// [`Self::y_label_anchor`]).
    pub fn y_label_anchor_right(&self, dpi: f64, tick_band: f64, label_size_pt: f64) -> Point {
        let gap = y_label_center_gap_px(dpi, label_size_pt);
        let edge = points_to_px(chrome_policy::Y_LABEL_EDGE_PT, dpi);
        // Soft ceiling: stay on-canvas when the figure is known via cell parent;
        // prefer ideal even if past `cell.x1` (full-bleed right chrome).
        let tick_outer = self.axes.x1 + tick_band;
        let lo = self.axes.x1 + gap;
        let ideal = tick_outer + gap;
        let x = ideal.max(lo);
        // Avoid running past a generous right bound when cell is the figure.
        let ceiling = self.cell.x1.max(self.axes.x1) + tick_band + gap + edge;
        Point::new(x.min(ceiling), self.axes.center().y)
    }

    /// Anchor point for a twin/secondary (top) x-axis label (Bottom baseline).
    ///
    /// Prefer [`Self::top_x_chrome_anchors`] when a figure title is also present so
    /// the two lines stack without overlap.
    pub fn x_label_anchor_top(&self, dpi: f64, tick_band: f64) -> Point {
        let (_, label) = self.top_x_chrome_anchors(dpi, None, Some(10.0), tick_band);
        label.unwrap_or_else(|| {
            let gap = points_to_px(chrome_policy::TWIN_X_LABEL_GAP_PT, dpi);
            Point::new(self.axes.center().x, self.axes.y0 - tick_band - gap)
        })
    }
}

/// Distance from tick-band outer edge to the center of a ±90° y-axis title.
#[inline]
fn y_label_half_em_px(dpi: f64, label_size_pt: f64) -> f64 {
    // Rotated ±90°: string length is vertical; horizontal extent ≈ em.
    points_to_px(label_size_pt.max(1.0) * 0.65, dpi)
}

fn y_label_center_gap_px(dpi: f64, label_size_pt: f64) -> f64 {
    let air = points_to_px(chrome_policy::Y_LABEL_AIR_PT, dpi);
    air + y_label_half_em_px(dpi, label_size_pt)
}

/// Space left of the y tick-label band for a rotated y-axis title (center gap +
/// half glyph past the center toward the figure edge).
fn y_label_outer_reserve_px(dpi: f64, label_size_pt: f64) -> f64 {
    y_label_center_gap_px(dpi, label_size_pt) + y_label_half_em_px(dpi, label_size_pt)
}

/// Pixel depth of the x tick-label band below the axes spine (excluding axis title).
pub fn x_tick_label_band(axes: &Axes, theme: &Theme, renderer: &dyn Renderer, dpi: f64) -> f64 {
    // Prefer size-aware targets when the axes box is already known; fall back to
    // the large-axes default during early measure (no layout yet).
    x_tick_label_band_targeted(axes, theme, renderer, dpi, ticks_policy::LINEAR_TARGETS)
}

/// Pixel width of the y tick-label band left/right of the axes spine.
pub fn y_tick_label_band_targeted(
    axes: &Axes,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
    y_targets: usize,
) -> f64 {
    let tick_len = points_to_px(theme.tick_length, dpi);
    let label_pad = points_to_px(chrome_policy::TICK_LABEL_PAD_PT, dpi);
    let tick_px = points_to_px(f64::from(theme.tick_label_size), dpi);
    let tick_font = points_to_px_f32(theme.tick_label_size, dpi);
    let mut tick_w = 0.0_f64;
    for tick in axes.major_ticks_y_targeted(y_targets) {
        if let Ok((w, _)) = crate::mathtext::measure_text(renderer, &tick.label, tick_font) {
            tick_w = tick_w.max(w);
        }
    }
    if tick_w <= 0.0 {
        tick_w = tick_px * 2.2;
    }
    tick_len + label_pad + tick_w
}

/// Like [`x_tick_label_band`] with an explicit locator target count.
pub fn x_tick_label_band_targeted(
    axes: &Axes,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
    x_targets: usize,
) -> f64 {
    let tick_len = points_to_px(theme.tick_length, dpi);
    let label_pad = points_to_px(chrome_policy::TICK_LABEL_PAD_PT, dpi);
    let tick_px = points_to_px(f64::from(theme.tick_label_size), dpi);
    let tick_font = points_to_px_f32(theme.tick_label_size, dpi);
    let mut x_tick_w = 0.0_f64;
    for tick in axes.major_ticks_x_targeted(x_targets) {
        if let Ok((w, _)) = crate::mathtext::measure_text(renderer, &tick.label, tick_font) {
            x_tick_w = x_tick_w.max(w);
        }
    }
    if x_tick_w <= 0.0 {
        x_tick_w = tick_px * 2.2;
    }
    if axes.x_datetime {
        // −30°: extent below spine ≈ w·sin(30°) + h·cos(30°) (+ small fudge).
        // Use em height (not line_height) — matches Skia Top-baseline draw.
        tick_len + label_pad + x_tick_w * 0.60 + tick_px * 1.0
    } else {
        // Upright Top-baseline ticks: visual depth ≈ one em, not line_height 1.2em.
        tick_len + label_pad + tick_px
    }
}

/// Matplotlib stock `figure.subplot.{left,right,bottom,top}` as cell insets.
///
/// See [`crate::mpl_policy::subplot`].
pub fn matplotlib_subplot_insets(figure: Size, cell: Rect) -> Insets {
    let ax_x0 = figure.width * subplot_policy::LEFT;
    let ax_x1 = figure.width * subplot_policy::RIGHT;
    let ax_y0 = figure.height * (1.0 - subplot_policy::TOP);
    let ax_y1 = figure.height * (1.0 - subplot_policy::BOTTOM);
    Insets {
        left: (ax_x0 - cell.x0).max(0.0),
        right: (cell.x1 - ax_x1).max(0.0),
        top: (ax_y0 - cell.y0).max(0.0),
        bottom: (cell.y1 - ax_y1).max(0.0),
    }
}

/// Measure the pixel margins a panel wants inside its cell.
///
/// Left (y-axis) and bottom (x-axis) use the same recipe:
/// `edge + tick_len + label_pad + tick_label_extent [+ axis label]`.
pub fn measure_insets(
    cell: Rect,
    axes: &Axes,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
) -> Insets {
    measure_insets_ctx(cell, axes, theme, renderer, dpi, false)
}

/// Like [`measure_insets`] but `multi_panel` selects fraction-based colorbar
/// gutter sizing (0.20 of available width, matching mpl `make_axes`). Single-
/// panel uses the calibrated `AXES_X1 = 0.745` gutter.
fn measure_insets_ctx(
    cell: Rect,
    axes: &Axes,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
    multi_panel: bool,
) -> Insets {
    let px = points_to_px(1.0, dpi);
    let ref_w = points_to_px(420.0, dpi);
    let scale = (cell.width() / ref_w).clamp(0.65, 1.0);

    let title_px = points_to_px(f64::from(axes.title_size_pt(theme)), dpi);
    let x_label_px = points_to_px(f64::from(axes.x_label_size_pt(theme)), dpi);
    let label_px = points_to_px(f64::from(theme.label_size), dpi);
    let tick_px = points_to_px(f64::from(theme.tick_label_size), dpi);
    let tick_len = points_to_px(theme.tick_length, dpi);
    let label_pad = points_to_px(3.5, dpi);
    let edge = 4.0 * px * scale;
    let axis_label_gap = 3.0 * px * scale;

    let tick_font = points_to_px_f32(theme.tick_label_size, dpi) * scale as f32;
    let est_w = (cell.width() - 100.0 * scale).max(40.0);
    let est_h = (cell.height() - 80.0 * scale).max(40.0);
    let tick_pt = f64::from(theme.tick_label_size);
    let x_targets = ticks_policy::auto_targets(est_w, dpi, tick_pt, true);
    let y_targets = ticks_policy::auto_targets(est_h, dpi, tick_pt, false);
    let mut tick_w = 0.0_f64;
    for tick in axes.major_ticks_y_targeted(y_targets) {
        if let Ok((w, _)) = crate::mathtext::measure_text(renderer, &tick.label, tick_font) {
            tick_w = tick_w.max(w);
        }
    }
    if tick_w <= 0.0 {
        tick_w = tick_px * 2.2;
    }

    // Tick-label band under / beside the spines (not including axis titles).
    let y_tick_band = tick_len + label_pad + tick_w;
    let x_tick_band = x_tick_label_band_targeted(axes, theme, renderer, dpi, x_targets);
    let x_ticks_top = axes.x_ticks_top && !axes.x_datetime;

    let mut left = edge + y_tick_band;
    let mut bottom = if x_ticks_top {
        edge
    } else {
        edge + x_tick_band
    };
    let mut right = 8.0 * px * scale;
    let mut top = 8.0 * px * scale;

    let title_h = if let Some(title) = axes.title.as_deref() {
        let title_font = points_to_px_f32(axes.title_size_pt(theme), dpi) * scale as f32;
        let measured_h = crate::mathtext::measure_text(renderer, title, title_font)
            .map(|(_, h)| h)
            .unwrap_or(title_px * scale);
        measured_h.max(title_px * scale)
    } else {
        0.0
    };
    let stack_gap = points_to_px(chrome_policy::TITLE_TWIN_STACK_GAP_PT, dpi) * scale;
    let x_label_gap = points_to_px(chrome_policy::X_LABEL_GAP_PT, dpi) * scale;
    let twin_x_gap = points_to_px(chrome_policy::TWIN_X_LABEL_GAP_PT, dpi) * scale;

    let top_x_tick_band = if let Some(twin) = axes.twin_x.as_deref() {
        Some(x_tick_label_band_targeted(
            twin, theme, renderer, dpi, x_targets,
        ))
    } else if let Some(sec) = axes.secondary_x.as_ref() {
        let mut sec_tick_h = 0.0_f64;
        for (_, tick) in sec.mapped_ticks(axes.x_min, axes.x_max) {
            if let Ok((_, h)) = crate::mathtext::measure_text(renderer, &tick.label, tick_font) {
                sec_tick_h = sec_tick_h.max(h);
            }
        }
        if sec_tick_h <= 0.0 {
            sec_tick_h = tick_px;
        }
        Some(tick_len + label_pad + sec_tick_h.min(tick_px))
    } else if x_ticks_top {
        Some(x_tick_band)
    } else {
        None
    };
    let top_x_label = axes.twin_x.as_deref().is_some_and(|t| t.x_label.is_some())
        || axes.secondary_x.as_ref().is_some_and(|s| s.label.is_some());

    // Top chrome is one stack: [title] → [top-x label] → [top tick band].
    // Avoid adding title + twin band independently (was ~2× too tall).
    if let Some(band) = top_x_tick_band {
        top = edge + band;
        if top_x_label {
            top += twin_x_gap + label_px * scale;
        }
        if title_h > 0.0 {
            top += stack_gap + title_h;
        }
    } else if title_h > 0.0 {
        top += title_h + 4.0 * px * scale;
    }

    if axes.polar {
        // θ labels (esp. `90°`) sit outside the ring into the top margin.
        let angle_top = points_to_px(polar_policy::ANGLE_LABEL_TOP_PT, dpi) * scale;
        let clear = points_to_px(polar_policy::TITLE_CLEAR_PT, dpi) * scale;
        let polar_top = edge
            + angle_top.max(clear)
            + if title_h > 0.0 {
                stack_gap + title_h
            } else {
                0.0
            };
        top = top.max(polar_top);
    }

    if axes.x_label.is_some() && !axes.x_datetime {
        // xlabel below tick band (matches draw: band + X_LABEL_GAP + em).
        bottom = x_tick_band + x_label_gap + x_label_px * scale + edge * 0.5;
    }
    // `autofmt_xdate(bottom=0.2)` sets the *final* subplot edge; ticks + xlabel
    // share that band (do not stack extra label clearance on top of 0.20).
    if axes.x_datetime {
        bottom = cell.height() * datetime_policy::AUTOFMT_BOTTOM;
    }
    if axes.y_label.is_some() {
        // Match [`Layout::y_label_anchor`]: reserve tick→center gap plus the
        // half-em that extends past the center into the figure margin.
        let label_pt = f64::from(axes.y_label_size_pt(theme));
        left += y_label_outer_reserve_px(dpi, label_pt) * scale;
    }
    if let Some(twin) = axes.twin_y.as_deref() {
        let mut twin_tick_w = 0.0_f64;
        for tick in twin.major_ticks_y_targeted(y_targets) {
            if let Ok((w, _)) = crate::mathtext::measure_text(renderer, &tick.label, tick_font) {
                twin_tick_w = twin_tick_w.max(w);
            }
        }
        if twin_tick_w <= 0.0 {
            twin_tick_w = tick_px * 2.2;
        }
        let twin_band = tick_len + label_pad + twin_tick_w;
        right = edge + twin_band;
        if twin.y_label.is_some() {
            let twin_label_pt = f64::from(twin.y_label_size_pt(theme));
            right += y_label_outer_reserve_px(dpi, twin_label_pt) * scale;
        }
    } else if let Some(sec) = axes.secondary_y.as_ref() {
        let mut sec_tick_w = 0.0_f64;
        for (_, tick) in sec.mapped_ticks(axes.y_min, axes.y_max) {
            if let Ok((w, _)) = crate::mathtext::measure_text(renderer, &tick.label, tick_font) {
                sec_tick_w = sec_tick_w.max(w);
            }
        }
        if sec_tick_w <= 0.0 {
            sec_tick_w = tick_px * 2.2;
        }
        let sec_band = tick_len + label_pad + sec_tick_w;
        right = edge + sec_band;
        if sec.label.is_some() {
            right += axis_label_gap + label_px * scale;
        }
    }
    if axes.needs_colorbar() {
        if multi_panel {
            let available = (cell.width() - left - right).max(1.0);
            right += available * cbar_policy::GUTTER_FRAC;
        } else {
            right = right.max(cell.width() * (1.0 - cbar_policy::AXES_X1));
        }
    }
    if axes.legend.is_some_and(|l| l.is_outside()) {
        // Reserve a right gutter so Outside* legends sit in figure chrome.
        let est = points_to_px(
            legend_policy::OUTSIDE_WIDTH_EM * f64::from(theme.tick_label_size)
                + legend_policy::OUTSIDE_PAD_PT,
            dpi,
        ) * scale;
        right = right.max(est);
    }

    Insets {
        left,
        right,
        top,
        bottom,
    }
}

/// Align insets across a subplot grid: shared left/right per column, top/bottom per row.
///
/// Single-panel figures use matplotlib stock `figure.subplot.*` as a floor so the
/// axes box matches mpl (~0.775×0.77), expanding only when labels/colorbars need more.
pub fn tight_insets_for_grid(
    grid: GridSpec,
    panels: &[(usize, usize, usize, usize, &Axes)],
    figure: Size,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
) -> Vec<Insets> {
    tight_layout_for_grid(grid, panels, figure, theme, renderer, dpi).1
}

/// Matplotlib-like tight layout for a subplot grid.
///
/// Multi-panel + full-bleed `GridSpec`: move outer label/title chrome into figure
/// margins and give every panel the same spine pad so axes boxes are equal-sized
/// (tick labels paint into `wspace` / `hspace`, matching mpl `tight_layout`).
pub fn tight_layout_for_grid(
    grid: GridSpec,
    panels: &[(usize, usize, usize, usize, &Axes)],
    figure: Size,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
) -> (GridSpec, Vec<Insets>) {
    let single = grid.nrows.max(1) == 1 && grid.ncols.max(1) == 1;
    let full_bleed = grid.left == 0.0 && grid.right == 1.0 && grid.bottom == 0.0 && grid.top == 1.0;

    // Probe chrome with a full-figure grid so outer margins can be derived.
    let probe = if !single && full_bleed {
        GridSpec {
            left: 0.0,
            right: 1.0,
            bottom: 0.0,
            top: 1.0,
            ..grid
        }
    } else {
        grid
    };

    // Single-pass layout (the iterative hint is wired but currently unused
    // to avoid disturbing colorbar charts whose gutter is fraction-based).
    tight_layout_pass(
        grid, &probe, panels, figure, theme, renderer, dpi, single, full_bleed,
    )
}

#[allow(clippy::too_many_arguments)]
fn tight_layout_pass(
    mut grid: GridSpec,
    probe: &GridSpec,
    panels: &[(usize, usize, usize, usize, &Axes)],
    figure: Size,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
    single: bool,
    full_bleed: bool,
) -> (GridSpec, Vec<Insets>) {
    let mut col_left = vec![0.0_f64; grid.ncols.max(1)];
    let mut col_right = vec![0.0_f64; grid.ncols.max(1)];
    let mut row_top = vec![0.0_f64; grid.nrows.max(1)];
    let mut row_bottom = vec![0.0_f64; grid.nrows.max(1)];

    for &(row, col, rowspan, colspan, axes) in panels {
        let rowspan = rowspan.max(1);
        let colspan = colspan.max(1);
        let cell = probe.span_rect(figure, row, col, rowspan, colspan);
        let needed = measure_insets_ctx(cell, axes, theme, renderer, dpi, !single);
        let insets = if single {
            let mpl = matplotlib_subplot_insets(figure, cell);
            if axes.x_datetime || axes.y_datetime {
                // `autofmt_xdate(bottom=0.2)` / y analogue: keep stock L/R/T (or
                // L/B/R for y-datetime) and only raise the autofmt edge.
                let mut inv = mpl;
                if axes.x_datetime {
                    inv.bottom = cell.height() * datetime_policy::AUTOFMT_BOTTOM;
                }
                if axes.y_datetime {
                    inv.left = cell.width() * datetime_policy::AUTOFMT_BOTTOM;
                }
                inv
            } else if axes.aspect_equal {
                // Geo / `aspect_equal`: keep stock subplot box like mpl
                // `set_aspect('equal', adjustable='box')`. Expanding for
                // lon/lat titles then letterboxing under-fills the map.
                mpl
            } else if mpl_policy::chrome_expands_stock_insets(axes) {
                needed.max_with(mpl)
            } else {
                // Keep stock right (colorbar / twin-y density), but grow
                // top/bottom/left when axis titles need more room than subplot.*
                // — otherwise xlabel / ylabel / twin-x chrome clips or overlaps.
                let mut inv = mpl;
                inv.bottom = inv.bottom.max(needed.bottom);
                inv.top = inv.top.max(needed.top);
                if axes.y_label.is_some() {
                    inv.left = inv.left.max(needed.left);
                }
                inv
            }
        } else {
            needed
        };
        let col_end = (col + colspan - 1).min(grid.ncols.saturating_sub(1));
        let row_end = (row + rowspan - 1).min(grid.nrows.saturating_sub(1));
        col_left[col] = col_left[col].max(insets.left);
        col_right[col_end] = col_right[col_end].max(insets.right);
        row_top[row] = row_top[row].max(insets.top);
        row_bottom[row_end] = row_bottom[row_end].max(insets.bottom);
    }

    let edge = points_to_px(subplot_policy::INTERIOR_PAD_PT, dpi);
    let pad = points_to_px(subplot_policy::TIGHT_PAD_PT, dpi);
    let ncols = grid.ncols.max(1);
    let nrows = grid.nrows.max(1);

    if !single && full_bleed {
        // Outer chrome → figure margins; axes boxes ≈ GridSpec cells (small spine
        // pad). Tick/title chrome paints into the outer margins and into
        // wspace/hspace — so those gaps must be large enough (mpl `tight_layout`).
        let outer_left = col_left.first().copied().unwrap_or(0.0) + pad;
        // Right panels rarely have y-labels; half-pad avoids over-shrink vs mpl.
        let outer_right = col_right.last().copied().unwrap_or(0.0)
            + pad * subplot_policy::TIGHT_PAD_BOTTOM_FACTOR;
        let outer_top =
            row_top.first().copied().unwrap_or(0.0) + pad * subplot_policy::TIGHT_PAD_TOP_FACTOR;
        let outer_bottom = row_bottom.last().copied().unwrap_or(0.0)
            + pad * subplot_policy::TIGHT_PAD_BOTTOM_FACTOR;
        grid.left = (outer_left / figure.width).clamp(0.0, 0.4);
        grid.right = (1.0 - outer_right / figure.width).clamp(0.6, 1.0);
        grid.bottom = (outer_bottom / figure.height).clamp(0.0, 0.4);
        grid.top = (1.0 - outer_top / figure.height).clamp(0.6, 1.0);
        if grid.right <= grid.left + 0.1 {
            grid.left = 0.05;
            grid.right = 0.95;
        }
        if grid.top <= grid.bottom + 0.1 {
            grid.bottom = 0.05;
            grid.top = 0.95;
        }

        // Interior gaps: facing chrome of adjacent panels + subplot pad.
        // Each cell keeps `edge` as spine inset, so that much of the measured
        // band sits inside the cell; the rest must fit in the GridSpec gap.
        let interior_pad = pad * subplot_policy::TIGHT_INTERIOR_PAD_FACTOR;
        if ncols > 1 {
            let mut need_gap_w = 0.0_f64;
            for c in 0..ncols - 1 {
                let facing = (col_right[c] - edge).max(0.0) + (col_left[c + 1] - edge).max(0.0);
                need_gap_w = need_gap_w.max(facing + interior_pad);
            }
            let area_w = figure.width * (grid.right - grid.left).max(0.1);
            let max_gap = area_w * 0.45;
            let gap_w = need_gap_w.min(max_gap);
            let cell_w = (area_w - (ncols as f64 - 1.0) * gap_w) / ncols as f64;
            if cell_w > 1.0 {
                let space = (gap_w / cell_w).clamp(0.0, 1.0);
                grid.wspace = grid.wspace.max(space);
            }
        }
        if nrows > 1 {
            let mut need_gap_h = 0.0_f64;
            for r in 0..nrows - 1 {
                let facing = (row_bottom[r] - edge).max(0.0) + (row_top[r + 1] - edge).max(0.0);
                need_gap_h = need_gap_h.max(facing + interior_pad);
            }
            let area_h = figure.height * (grid.top - grid.bottom).max(0.1);
            let max_gap = area_h * 0.45;
            let gap_h = need_gap_h.min(max_gap);
            let cell_h = (area_h - (nrows as f64 - 1.0) * gap_h) / nrows as f64;
            if cell_h > 1.0 {
                let space = (gap_h / cell_h).clamp(0.0, 1.0);
                grid.hspace = grid.hspace.max(space);
            }
        }

        let insets: Vec<Insets> = panels
            .iter()
            .map(|&(row, col, rowspan, colspan, axes)| {
                let cell = grid.span_rect(figure, row, col, rowspan.max(1), colspan.max(1));
                let right_inset = if axes.needs_colorbar() {
                    let available = (cell.width() - edge - edge).max(1.0);
                    edge + available * cbar_policy::GUTTER_FRAC
                } else {
                    edge
                };
                // Polar: keep only θ-label pad in-cell. Title clearance is already
                // in `row_top` → `grid.top` (outer margin). Using full `needed.top`
                // here double-counted chrome and shrunk the polar disk vs mpl.
                let top = if axes.polar {
                    let angle = points_to_px(polar_policy::ANGLE_LABEL_TOP_PT, dpi);
                    (edge + angle).min(cell.height() * 0.18)
                } else {
                    edge
                };
                Insets {
                    left: edge,
                    right: right_inset,
                    top,
                    bottom: edge,
                }
                .clamp_to_cell(cell)
            })
            .collect();
        return (grid, insets);
    }

    // Single-panel (or custom GridSpec margins): column/row shared insets.
    if !single {
        for c in 0..ncols {
            if c + 1 < ncols {
                col_right[c] = col_right[c].min(edge);
            }
            if c > 0 {
                col_left[c] = col_left[c].min(edge);
            }
        }
        for r in 0..nrows {
            if r + 1 < nrows {
                row_bottom[r] = row_bottom[r].min(edge);
            }
            if r > 0 {
                row_top[r] = row_top[r].min(edge);
            }
        }
    }

    let insets = panels
        .iter()
        .map(|&(row, col, rowspan, colspan, _)| {
            let colspan = colspan.max(1);
            let rowspan = rowspan.max(1);
            let col_end = (col + colspan - 1).min(ncols.saturating_sub(1));
            let row_end = (row + rowspan - 1).min(nrows.saturating_sub(1));
            let cell = grid.span_rect(figure, row, col, rowspan, colspan);
            Insets {
                left: col_left[col],
                right: col_right[col_end],
                top: row_top[row],
                bottom: row_bottom[row_end],
            }
            .clamp_to_cell(cell)
        })
        .collect();
    (grid, insets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_cells_cover_interior() {
        let g = GridSpec::new(2, 2);
        let fig = Size::new(800.0, 600.0);
        let c00 = g.cell_rect(fig, 0, 0);
        let c11 = g.cell_rect(fig, 1, 1);
        assert!(c00.x0 < c11.x0);
        assert!(c00.y0 < c11.y0);
        assert!(c00.width() > 100.0);
        assert!(c00.height() > 100.0);
    }

    #[test]
    fn grid_wspace_matches_matplotlib_formula() {
        // cell = area / (n + (n-1)·wspace), gap = cell · wspace
        let g = GridSpec::new(1, 2).wspace(0.2);
        let fig = Size::new(1000.0, 500.0);
        let c0 = g.cell_rect(fig, 0, 0);
        let c1 = g.cell_rect(fig, 0, 1);
        let cell = 1000.0 / (2.0 + 0.2);
        let gap = cell * 0.2;
        assert!((c0.width() - cell).abs() < 1e-6, "w={}", c0.width());
        assert!((c1.x0 - c0.x1 - gap).abs() < 1e-6, "gap={}", c1.x0 - c0.x1);
    }

    #[test]
    fn span_rect_covers_two_rows() {
        let g = GridSpec::new(2, 2);
        let fig = Size::new(800.0, 600.0);
        let c00 = g.cell_rect(fig, 0, 0);
        let c10 = g.cell_rect(fig, 1, 0);
        let span = g.span_rect(fig, 0, 0, 2, 1);
        assert!((span.x0 - c00.x0).abs() < 1e-9);
        assert!((span.x1 - c00.x1).abs() < 1e-9);
        assert!((span.y0 - c00.y0).abs() < 1e-9);
        assert!((span.y1 - c10.y1).abs() < 1e-9);
        assert!(span.height() > c00.height() + c10.height() * 0.5);
    }

    #[test]
    fn insets_max_with_takes_componentwise_max() {
        let a = Insets {
            left: 10.0,
            right: 5.0,
            top: 20.0,
            bottom: 8.0,
        };
        let b = Insets {
            left: 12.0,
            right: 3.0,
            top: 15.0,
            bottom: 11.0,
        };
        let m = a.max_with(b);
        assert_eq!(m.left, 12.0);
        assert_eq!(m.right, 5.0);
        assert_eq!(m.top, 20.0);
        assert_eq!(m.bottom, 11.0);
    }

    #[test]
    fn y_label_outer_reserve_covers_anchor_geometry() {
        let dpi = 150.0;
        let label_pt = 10.0;
        let reserve = y_label_outer_reserve_px(dpi, label_pt);
        let gap = y_label_center_gap_px(dpi, label_pt);
        let half = y_label_half_em_px(dpi, label_pt);
        assert!((reserve - (gap + half)).abs() < 1e-9);
        // Must exceed the old under-reserve (~3px gap + 1em) used in measure.
        let legacy = points_to_px(3.0, dpi) + points_to_px(label_pt, dpi);
        assert!(
            reserve > legacy + points_to_px(2.0, dpi),
            "reserve={reserve} legacy={legacy}"
        );
    }
}
