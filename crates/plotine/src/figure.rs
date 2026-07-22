use std::path::Path;

use plotine_core::{
    color::DEFAULT_CYCLE, Color, DataToPixel, Point, Rect, Result, Size, TickLocator,
};
use plotine_render::{
    FillStyle, LineCap, Renderer, StrokeStyle, TextAlign, TextBaseline, TextStyle,
};

use crate::artist::LegendKind;
use crate::axes::{Axes, GridAxis};
use crate::axes3d::Axes3D;
use crate::draw;
use crate::draw3d;
use crate::layout::{GridSpec, Layout};
use crate::mpl_policy::{
    chrome as chrome_policy, colorbar as cbar_policy, datetime as datetime_policy,
    figure as fig_policy, inset as inset_policy, legend as legend_policy, polar as polar_policy,
    ticks as ticks_policy,
};
use crate::recipes::{marker_path, Marker};
use crate::style::{LineStyle, MarkerStyle};
use crate::subplots::{Panel, SubplotGrid};
use crate::theme::{points_to_px, points_to_px_f32, Theme};

/// Top-level figure owning one or more axes panels.
///
/// A `Figure` is the entry point for all plotting operations. It owns the
/// canvas dimensions, theme, and a grid of [`Axes`] panels. Build one with
/// [`Figure::new()`], configure axes via [`axes()`](Self::axes) or
/// [`subplots()`](Self::subplots), then render with [`save()`](Self::save)
/// (or `show()` with `feature = "gui"`).
///
/// # Example
///
/// ```
/// use plotine::prelude::*;
///
/// let png = Figure::new()
///     .size(4.0, 3.0)
///     .dpi(72.0)
///     .axes(|ax| {
///         ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5])
///             .color(Color::CRIMSON);
///     })
///     .render_png()
///     .unwrap();
/// assert!(!png.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Figure {
    width: f64,
    height: f64,
    pub(crate) dpi: f64,
    pub(crate) theme: Theme,
    pub(crate) grid: GridSpec,
    pub(crate) panels: Vec<Panel>,
    /// Optional 3D axes (mutually exclusive with 2D panels when set).
    pub(crate) axes3d: Option<Axes3D>,
    /// Centered figure title above all subplots (matplotlib `fig.suptitle`).
    suptitle: Option<String>,
    /// Route `$...$` (and TeX strings) through system LaTeX when `feature = "latex"`.
    usetex: bool,
}

impl Default for Figure {
    fn default() -> Self {
        Self::new()
    }
}

impl Figure {
    /// Create a figure with matplotlib-aligned size (`6.4?4.8` in) @ 150 DPI.
    ///
    /// Size / DPI come from [`crate::mpl_policy::figure`]. Theme font/stroke
    /// sizes are in points and scale with DPI (`px = pt ? dpi/72`).
    ///
    /// Use [`size()`](Self::size) and [`dpi()`](Self::dpi) to override.
    pub fn new() -> Self {
        Self {
            width: fig_policy::WIDTH_IN,
            height: fig_policy::HEIGHT_IN,
            dpi: fig_policy::DPI,
            theme: Theme::default(),
            grid: GridSpec::default(),
            panels: Vec::new(),
            axes3d: None,
            suptitle: None,
            usetex: false,
        }
    }

    /// Set the figure size in inches (width, height).
    pub fn size(mut self, width_in: f64, height_in: f64) -> Self {
        self.width = width_in;
        self.height = height_in;
        self
    }

    /// Use system LaTeX (`latex` + `dvipng`) for math / `$...$` labels.
    ///
    /// Requires Cargo `features = ["latex"]` and TeX tools on `PATH`. Default
    /// remains built-in [`crate::mathtext`] (no external binary).
    ///
    /// Equivalent in spirit to matplotlib `rcParams['text.usetex'] = True`.
    pub fn usetex(mut self, enabled: bool) -> Self {
        self.usetex = enabled;
        self
    }

    /// Set the output resolution in dots per inch.
    pub fn dpi(mut self, dpi: f64) -> Self {
        self.dpi = dpi;
        self
    }

    /// Apply a visual [`Theme`] (colors, font sizes, spine widths).
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set a centered title above all subplots (matplotlib `fig.suptitle`).
    pub fn suptitle(mut self, title: impl Into<String>) -> Self {
        self.suptitle = Some(title.into());
        self
    }

    /// Create a figure with one empty axes panel (for imperative / pyplot-style use).
    pub fn with_empty_axes() -> Self {
        Self::new().axes(|_| {})
    }

    /// Configure a single full-figure axes panel (1×1 grid).
    pub fn axes<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Axes),
    {
        let mut ax = Axes::new();
        f(&mut ax);
        ax.finalize_artist_limits();
        self.grid = GridSpec::new(1, 1);
        self.panels = vec![Panel {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 1,
            axes: ax,
        }];
        self.axes3d = None;
        self
    }

    /// Configure a 3D axes panel (replaces any 2D panels).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes3d(|ax| {
    ///     ax.plot3d([0.0, 1.0, 2.0], [0.0, 1.0, 0.5], [0.0, 0.5, 1.0])
    ///         .color(Color::CRIMSON);
    ///     ax.title("3D");
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn axes3d<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Axes3D),
    {
        let mut ax = Axes3D::new();
        f(&mut ax);
        self.axes3d = Some(ax);
        self.panels.clear();
        self
    }

    /// Configure an `nrows ? ncols` subplot grid.
    ///
    /// ```
    /// # use plotine::prelude::*;
    /// let fig = Figure::new().size(4.0, 3.0).dpi(72.0).subplots(2, 1, |g| {
    ///     g.at(0, 0, |ax| { ax.title("Top").x_range(0.0, 1.0).y_range(0.0, 1.0); });
    ///     g.at(1, 0, |ax| { ax.title("Bottom").x_range(0.0, 1.0).y_range(0.0, 1.0); });
    /// });
    /// assert!(!fig.render_png().unwrap().is_empty());
    /// ```
    pub fn subplots<F>(mut self, nrows: usize, ncols: usize, f: F) -> Self
    where
        F: FnOnce(&mut SubplotGrid<'_>),
    {
        self.grid = GridSpec::new(nrows, ncols);
        self.panels.clear();
        let (do_sharex, do_sharey);
        {
            let mut grid = SubplotGrid {
                spec: self.grid,
                panels: &mut self.panels,
                sharex: false,
                sharey: false,
            };
            f(&mut grid);
            self.grid = grid.spec;
            do_sharex = grid.sharex;
            do_sharey = grid.sharey;
        }
        if do_sharex {
            sync_shared_x(&mut self.panels, self.grid.nrows);
        }
        if do_sharey {
            sync_shared_y(&mut self.panels, self.grid.ncols);
        }
        self
    }

    /// Like [`subplots`](Self::subplots) but starts from a custom [`GridSpec`].
    pub fn subplots_with<F>(mut self, spec: GridSpec, f: F) -> Self
    where
        F: FnOnce(&mut SubplotGrid<'_>),
    {
        self.grid = spec;
        self.panels.clear();
        let mut grid = SubplotGrid {
            spec: self.grid,
            panels: &mut self.panels,
            sharex: false,
            sharey: false,
        };
        f(&mut grid);
        self.grid = grid.spec;
        self
    }

    /// Configure subplots using a string layout (like matplotlib's `subplot_mosaic`).
    ///
    /// Each unique letter defines a named panel; `.` marks empty cells.
    /// Rows are separated by newlines or `;`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(6.0, 4.0).dpi(72.0).subplot_mosaic(
    ///     "AB;CC",
    ///     |name, ax| match name {
    ///         'A' => { ax.line([0.0, 1.0], [0.0, 1.0]); ax.title("A"); },
    ///         'B' => { ax.scatter([0.0, 1.0], [1.0, 0.0]); ax.title("B"); },
    ///         'C' => { ax.bar([1.0, 2.0, 3.0], [3.0, 1.0, 2.0]); ax.title("C"); },
    ///         _ => {},
    ///     },
    /// ).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn subplot_mosaic<F>(mut self, layout: &str, f: F) -> Self
    where
        F: Fn(char, &mut Axes),
    {
        let regions = crate::subplots::parse_mosaic(layout);
        let nrows = regions.iter().map(|r| r.1 + r.3).max().unwrap_or(1);
        let ncols = regions.iter().map(|r| r.2 + r.4).max().unwrap_or(1);
        self.grid = GridSpec::new(nrows, ncols);
        self.panels.clear();
        for (name, row, col, rowspan, colspan) in regions {
            let mut ax = Axes::new();
            f(name, &mut ax);
            ax.finalize_artist_limits();
            self.panels.push(Panel {
                row,
                col,
                rowspan,
                colspan,
                axes: ax,
            });
        }
        self.axes3d = None;
        self
    }

    /// Pixel dimensions `(width, height)` after applying size ? DPI.
    pub fn pixel_size(&self) -> (u32, u32) {
        let w = (self.width * self.dpi).round().max(1.0) as u32;
        let h = (self.height * self.dpi).round().max(1.0) as u32;
        (w, h)
    }

    /// Render into an arbitrary [`Renderer`].
    pub fn draw(&self, renderer: &mut dyn Renderer) -> Result<()> {
        if self.usetex {
            #[cfg(not(feature = "latex"))]
            {
                return Err(plotine_core::PlotError::latex_unavailable(
                    "Cargo feature \"latex\" is not enabled",
                ));
            }
        }
        #[cfg(feature = "latex")]
        let _usetex_guard = crate::latex::UsetexGuard::enter(self.usetex, self.dpi);

        // 3D path: delegate to draw3d module.
        if let Some(axes3d) = &self.axes3d {
            let (pw, ph) = self.pixel_size();
            let figure_size = Size::new(pw as f64, ph as f64);
            renderer.clear(self.theme.background)?;
            let rect = Rect::new(0.0, 0.0, figure_size.width, figure_size.height);
            draw3d::draw_axes3d(renderer, axes3d, rect, &self.theme, self.dpi)?;
            if let Some(ref title) = self.suptitle {
                let style = TextStyle::new(
                    self.theme.title,
                    points_to_px_f32(self.theme.title_size * 1.2, self.dpi),
                )
                .align(TextAlign::Center)
                .baseline(TextBaseline::Top);
                let pos = Point::new(figure_size.width * 0.5, points_to_px(4.0, self.dpi));
                crate::mathtext::draw_text(renderer, title, pos, &style)?;
            }
            return Ok(());
        }

        if self.panels.is_empty() {
            return Err(plotine_core::PlotError::empty_figure());
        }
        for panel in &self.panels {
            panel.axes.validate()?;
        }

        let (pw, ph) = self.pixel_size();
        let figure_size = Size::new(pw as f64, ph as f64);
        let theme = &self.theme;

        renderer.clear(theme.background)?;

        // Tight-layout: align left/right within each column and top/bottom within each row.
        // Multi-panel may rewrite GridSpec outer margins (mpl-like) when still full-bleed.
        let panel_refs: Vec<(usize, usize, usize, usize, &Axes)> = self
            .panels
            .iter()
            .map(|p| (p.row, p.col, p.rowspan, p.colspan, &p.axes))
            .collect();
        let (grid, insets) = crate::layout::tight_layout_for_grid(
            self.grid,
            &panel_refs,
            figure_size,
            theme,
            renderer,
            self.dpi,
        );

        for (panel, panel_insets) in self.panels.iter().zip(insets) {
            let cell = grid.span_rect(
                figure_size,
                panel.row,
                panel.col,
                panel.rowspan,
                panel.colspan,
            );
            let layout = Layout::from_insets(cell, panel_insets);
            self.draw_panel(renderer, &panel.axes, &layout, theme)?;
        }
        if let Some(ref title) = self.suptitle {
            let style = TextStyle::new(
                theme.title,
                points_to_px_f32(theme.title_size * 1.2, self.dpi),
            )
            .align(TextAlign::Center)
            .baseline(TextBaseline::Top);
            let pos = Point::new(figure_size.width * 0.5, points_to_px(4.0, self.dpi));
            crate::mathtext::draw_text(renderer, title, pos, &style)?;
        }
        Ok(())
    }

    fn draw_panel(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        // `aspect_equal` (geo / set_aspect): shrink the axes box like mpl
        // `adjustable='box'` so spines/ticks/grid share the artist plot rect.
        let mut layout = *layout;
        if axes.aspect_equal && !axes.polar && axes.frame_on {
            layout.axes =
                aspect_fit_rect(layout.axes, axes.x_min, axes.x_max, axes.y_min, axes.y_max);
        }
        // Polar matches matplotlib: white face (not the grey cartesian axes patch).
        let face = if axes.polar {
            theme.background
        } else {
            theme.axes_face
        };
        renderer.fill_rect(layout.axes, &FillStyle::solid(face))?;
        if axes.polar || !axes.frame_on {
            // Polar / pie: no cartesian grid/spines/ticks.
            self.draw_artists(renderer, axes, &layout)?;
            self.draw_titles(renderer, axes, &layout, theme)?;
            self.draw_legend(renderer, axes, &layout, theme)?;
            self.draw_insets(renderer, axes, &layout, theme)?;
            return Ok(());
        }
        self.draw_grid(renderer, axes, &layout, theme)?;
        self.draw_artists(renderer, axes, &layout)?;
        if let Some(twin) = axes.twin_y.as_deref() {
            self.draw_twin_y_artists(renderer, axes, twin, &layout)?;
        }
        if let Some(twin) = axes.twin_x.as_deref() {
            self.draw_twin_x_artists(renderer, axes, twin, &layout)?;
        }
        self.draw_spines(renderer, axes, &layout, theme)?;
        self.draw_ticks_and_labels(renderer, axes, &layout, theme)?;
        if let Some(twin) = axes.twin_y.as_deref() {
            self.draw_twin_y_ticks(renderer, axes, twin, &layout, theme)?;
        }
        if let Some(twin) = axes.twin_x.as_deref() {
            self.draw_twin_x_ticks(renderer, axes, twin, &layout, theme)?;
        }
        if let Some(sec) = axes.secondary_y.as_ref() {
            self.draw_secondary_y_ticks(renderer, axes, sec, &layout, theme)?;
        }
        if let Some(sec) = axes.secondary_x.as_ref() {
            self.draw_secondary_x_ticks(renderer, axes, sec, &layout, theme)?;
        }
        self.draw_titles(renderer, axes, &layout, theme)?;
        if let Some(twin) = axes.twin_y.as_deref() {
            self.draw_twin_y_label(renderer, twin, &layout, theme)?;
        }
        if let Some(twin) = axes.twin_x.as_deref() {
            self.draw_twin_x_label(renderer, axes, twin, &layout, theme)?;
        }
        if let Some(sec) = axes.secondary_y.as_ref() {
            self.draw_secondary_y_label(renderer, axes, sec, &layout, theme)?;
        }
        if let Some(sec) = axes.secondary_x.as_ref() {
            self.draw_secondary_x_label(renderer, axes, sec, &layout, theme)?;
        }
        self.draw_legend(renderer, axes, &layout, theme)?;
        self.draw_colorbar(renderer, axes, &layout, theme)?;
        self.draw_insets(renderer, axes, &layout, theme)?;
        Ok(())
    }

    fn draw_insets(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        for inset in &axes.insets {
            let inset_layout = layout_from_axes_fraction(layout.axes, inset.rect, self.dpi);
            self.draw_panel(renderer, &inset.axes, &inset_layout, theme)?;
        }
        Ok(())
    }

    fn draw_grid(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let show = axes.show_grid.unwrap_or(theme.show_grid);
        if !show {
            return Ok(());
        }
        let x_scale = axes.x_scale_kind()?;
        let y_scale = axes.y_scale_kind()?;
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(axes.y_inverted);
        let width_px = points_to_px(theme.grid_width, self.dpi);
        let mut style = StrokeStyle::new(theme.grid, width_px);
        if let Some(dash) = axes.grid_linestyle.dash_pattern(width_px) {
            style.dash = Some(dash);
            if matches!(axes.grid_linestyle, crate::style::LineStyle::Dotted) {
                style.cap = LineCap::Round;
            }
        }
        let draw_x = matches!(axes.grid_axis, GridAxis::Both | GridAxis::X);
        let draw_y = matches!(axes.grid_axis, GridAxis::Both | GridAxis::Y);

        let (x_targets, y_targets) = self.tick_targets(axes, layout, theme);
        if draw_x {
            for tick in axes.major_ticks_x_targeted(x_targets) {
                let x = transform.map_x(tick.value);
                renderer.draw_line(
                    Point::new(x, layout.axes.y0),
                    Point::new(x, layout.axes.y1),
                    &style,
                )?;
            }
        }
        if draw_y {
            for tick in axes.major_ticks_y_targeted(y_targets) {
                let y = transform.map_y(tick.value);
                renderer.draw_line(
                    Point::new(layout.axes.x0, y),
                    Point::new(layout.axes.x1, y),
                    &style,
                )?;
            }
        }
        Ok(())
    }

    fn draw_artists(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
    ) -> Result<()> {
        let x_scale = axes.x_scale_kind()?;
        let y_scale = axes.y_scale_kind()?;
        // `aspect_equal` already applied to `layout.axes` in `draw_panel`.
        let plot_rect = if axes.polar {
            // Matplotlib polar: circle fills the axes; θ labels sit in the cell margins.
            inscribed_square(layout.axes)
        } else if !axes.frame_on {
            // Pie: square disk in the axes (no θ-label pad).
            inscribed_square(layout.axes)
        } else {
            layout.axes
        };
        let transform =
            DataToPixel::new(x_scale, y_scale, plot_rect).with_invert_y(axes.y_inverted);
        let px = points_to_px(1.0, self.dpi);
        // Polar: clip to the cell so ?? labels outside the axes remain visible.
        // Pie: clip to axes (labels stay near the disk).
        let clip = if axes.polar {
            layout.cell
        } else if !axes.frame_on {
            layout.axes
        } else {
            plot_rect
        };
        renderer.push_clip_rect(clip)?;

        for el in &axes.elements {
            let color = el.resolved_color(&DEFAULT_CYCLE);
            draw::draw_element(renderer, el, color, &transform, px)?;
        }

        renderer.pop_clip()?;
        Ok(())
    }

    fn draw_twin_y_artists(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        twin: &Axes,
        layout: &Layout,
    ) -> Result<()> {
        let x_scale = host.x_scale_kind()?;
        let y_scale = twin.y_scale_kind()?;
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(twin.y_inverted);
        let px = points_to_px(1.0, self.dpi);
        renderer.push_clip_rect(layout.axes)?;
        for el in &twin.elements {
            let color = el.resolved_color(&DEFAULT_CYCLE);
            draw::draw_element(renderer, el, color, &transform, px)?;
        }
        renderer.pop_clip()?;
        Ok(())
    }

    fn draw_twin_x_artists(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        twin: &Axes,
        layout: &Layout,
    ) -> Result<()> {
        let x_scale = twin.x_scale_kind()?;
        let y_scale = host.y_scale_kind()?;
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(host.y_inverted);
        let px = points_to_px(1.0, self.dpi);
        renderer.push_clip_rect(layout.axes)?;
        for el in &twin.elements {
            let color = el.resolved_color(&DEFAULT_CYCLE);
            draw::draw_element(renderer, el, color, &transform, px)?;
        }
        renderer.pop_clip()?;
        Ok(())
    }

    fn draw_spines(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let style = StrokeStyle::new(theme.spine, points_to_px(theme.spine_width, self.dpi));
        let r = layout.axes;
        if axes.spines.left {
            renderer.draw_line(Point::new(r.x0, r.y0), Point::new(r.x0, r.y1), &style)?;
        }
        if axes.spines.right {
            renderer.draw_line(Point::new(r.x1, r.y0), Point::new(r.x1, r.y1), &style)?;
        }
        if axes.spines.top {
            renderer.draw_line(Point::new(r.x0, r.y0), Point::new(r.x1, r.y0), &style)?;
        }
        if axes.spines.bottom {
            renderer.draw_line(Point::new(r.x0, r.y1), Point::new(r.x1, r.y1), &style)?;
        }
        Ok(())
    }

    /// Size-aware tick targets (`matplotlib.ticker.AutoLocator` / `get_tick_space`).
    ///
    /// Tick count follows axis pixel length (`MaxNLocator` / `get_tick_space`).
    /// Colorbar shrinks the axes width, so `auto_targets` already yields fewer
    /// x ticks — do **not** floor upward (that densified coolwarm/hist2d/hexbin
    /// to step `2.5` with `0.0` labels while stock mpl uses `5` / bare integers).
    fn tick_targets(&self, _axes: &Axes, layout: &Layout, theme: &Theme) -> (usize, usize) {
        let tick_pt = f64::from(theme.tick_label_size);
        let x = ticks_policy::auto_targets(layout.axes.width(), self.dpi, tick_pt, true);
        let y = ticks_policy::auto_targets(layout.axes.height(), self.dpi, tick_pt, false);
        (x, y)
    }

    fn draw_ticks_and_labels(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let x_scale = axes.x_scale_kind()?;
        let y_scale = axes.y_scale_kind()?;
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(axes.y_inverted);
        let tick_len = points_to_px(theme.tick_length, self.dpi);
        let tick_stroke = StrokeStyle::new(theme.tick, points_to_px(theme.tick_width, self.dpi));
        let tick_font = points_to_px_f32(theme.tick_label_size, self.dpi);
        // Shared pad from tick tip ??label for both axes (keeps x/y chrome aligned).
        let label_pad = points_to_px(crate::mpl_policy::chrome::TICK_LABEL_PAD_PT, self.dpi);
        // Datetime: [`datetime_policy`] (`autofmt_xdate`).
        let x_label_style = if axes.x_datetime {
            TextStyle::new(theme.label, tick_font)
                .align(TextAlign::Right)
                .baseline(TextBaseline::Top)
                .rotation(datetime_policy::TICK_ROTATION_DEG)
        } else {
            TextStyle::new(theme.label, tick_font)
                .align(TextAlign::Center)
                .baseline(TextBaseline::Top)
        };
        // Datetime uses the same tip?label pad as linear ticks (mpl `xtick.major.pad`);
        // `ROTATED_BAND_EM` is only for measuring the label band, not draw offset.
        let x_gap = label_pad;

        let (x_targets, y_targets) = self.tick_targets(axes, layout, theme);
        let x_majors = axes.major_ticks_x_targeted(x_targets);
        let x_on_top = axes.x_ticks_top && !axes.x_datetime;
        let x_label_style = if x_on_top {
            TextStyle::new(theme.label, tick_font)
                .align(TextAlign::Center)
                .baseline(TextBaseline::Bottom)
        } else {
            x_label_style
        };
        for tick in &x_majors {
            let x = transform.map_x(tick.value);
            if x_on_top {
                renderer.draw_line(
                    Point::new(x, layout.axes.y0),
                    Point::new(x, layout.axes.y0 - tick_len),
                    &tick_stroke,
                )?;
                crate::mathtext::draw_text(
                    renderer,
                    &tick.label,
                    Point::new(x, layout.axes.y0 - tick_len - x_gap),
                    &x_label_style,
                )?;
            } else {
                renderer.draw_line(
                    Point::new(x, layout.axes.y1),
                    Point::new(x, layout.axes.y1 + tick_len),
                    &tick_stroke,
                )?;
                crate::mathtext::draw_text(
                    renderer,
                    &tick.label,
                    Point::new(x, layout.axes.y1 + tick_len + x_gap),
                    &x_label_style,
                )?;
            }
        }
        // ConciseDateFormatter offset — upright, below the rotated tick band
        // (matplotlib puts this in `Axis.offsetText`, not as an extra tick).
        if axes.x_datetime && !x_on_top {
            let vals: Vec<f64> = x_majors.iter().map(|t| t.value).collect();
            if let Some(offset) = plotine_core::format_concise_datetime_ticks(&vals).1 {
                let off_style = TextStyle::new(theme.label, tick_font * 0.85)
                    .align(TextAlign::Right)
                    .baseline(TextBaseline::Top);
                let off_y = layout.axes.y1
                    + tick_len
                    + x_gap
                    + points_to_px(f64::from(theme.tick_label_size) * 1.35, self.dpi);
                crate::mathtext::draw_text(
                    renderer,
                    &offset,
                    Point::new(
                        layout.axes.x1,
                        off_y.min(layout.cell.y1 - points_to_px(2.0, self.dpi)),
                    ),
                    &off_style,
                )?;
            }
        }
        if axes.x_minor_ticks {
            let minor_len = tick_len * 0.5;
            let minor_stroke =
                StrokeStyle::new(theme.tick, points_to_px(theme.tick_width, self.dpi) * 0.8);
            for v in axes.minor_tick_values_x(&x_majors) {
                let x = transform.map_x(v);
                if x_on_top {
                    renderer.draw_line(
                        Point::new(x, layout.axes.y0),
                        Point::new(x, layout.axes.y0 - minor_len),
                        &minor_stroke,
                    )?;
                } else {
                    renderer.draw_line(
                        Point::new(x, layout.axes.y1),
                        Point::new(x, layout.axes.y1 + minor_len),
                        &minor_stroke,
                    )?;
                }
            }
        }

        let y_label_style = TextStyle::new(theme.label, tick_font)
            .align(TextAlign::Right)
            .baseline(TextBaseline::Middle);
        let y_majors = axes.major_ticks_y_targeted(y_targets);
        for tick in &y_majors {
            let y = transform.map_y(tick.value);
            renderer.draw_line(
                Point::new(layout.axes.x0 - tick_len, y),
                Point::new(layout.axes.x0, y),
                &tick_stroke,
            )?;
            crate::mathtext::draw_text(
                renderer,
                &tick.label,
                Point::new(layout.axes.x0 - tick_len - label_pad, y),
                &y_label_style,
            )?;
        }
        if axes.y_datetime {
            let vals: Vec<f64> = y_majors.iter().map(|t| t.value).collect();
            if let Some(offset) = plotine_core::format_concise_datetime_ticks(&vals).1 {
                let off_style = TextStyle::new(theme.label, tick_font * 0.9)
                    .align(TextAlign::Right)
                    .baseline(TextBaseline::Bottom);
                crate::mathtext::draw_text(
                    renderer,
                    &offset,
                    Point::new(layout.axes.x0 - tick_len - label_pad, layout.axes.y0),
                    &off_style,
                )?;
            }
        }
        if axes.y_minor_ticks {
            let minor_len = tick_len * 0.5;
            let minor_stroke =
                StrokeStyle::new(theme.tick, points_to_px(theme.tick_width, self.dpi) * 0.8);
            for v in axes.minor_tick_values_y(&y_majors) {
                let y = transform.map_y(v);
                renderer.draw_line(
                    Point::new(layout.axes.x0 - minor_len, y),
                    Point::new(layout.axes.x0, y),
                    &minor_stroke,
                )?;
            }
        }
        Ok(())
    }

    fn draw_twin_y_ticks(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        twin: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let x_scale = host.x_scale_kind()?;
        let y_scale = twin.y_scale_kind()?;
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(twin.y_inverted);
        let tick_len = points_to_px(theme.tick_length, self.dpi);
        let tick_stroke = StrokeStyle::new(theme.tick, points_to_px(theme.tick_width, self.dpi));
        let tick_font = points_to_px_f32(theme.tick_label_size, self.dpi);
        let label_pad = points_to_px(3.5, self.dpi);
        let y_label_style = TextStyle::new(theme.label, tick_font)
            .align(TextAlign::Left)
            .baseline(TextBaseline::Middle);
        let (_, y_targets) = self.tick_targets(twin, layout, theme);
        for tick in twin.major_ticks_y_targeted(y_targets) {
            let y = transform.map_y(tick.value);
            renderer.draw_line(
                Point::new(layout.axes.x1, y),
                Point::new(layout.axes.x1 + tick_len, y),
                &tick_stroke,
            )?;
            crate::mathtext::draw_text(
                renderer,
                &tick.label,
                Point::new(layout.axes.x1 + tick_len + label_pad, y),
                &y_label_style,
            )?;
        }
        Ok(())
    }

    fn draw_twin_y_label(
        &self,
        renderer: &mut dyn Renderer,
        twin: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let Some(ylabel) = &twin.y_label else {
            return Ok(());
        };
        let style = TextStyle::new(theme.label, points_to_px_f32(theme.label_size, self.dpi))
            .align(TextAlign::Center)
            .baseline(TextBaseline::Middle)
            .rotation(90.0);
        let (_, y_targets) = self.tick_targets(twin, layout, theme);
        let band =
            crate::layout::y_tick_label_band_targeted(twin, theme, renderer, self.dpi, y_targets);
        let label_pt = f64::from(theme.label_size);
        renderer.draw_text(
            ylabel,
            layout.y_label_anchor_right(self.dpi, band, label_pt),
            &style,
        )?;
        Ok(())
    }

    fn draw_twin_x_ticks(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        twin: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let x_scale = twin.x_scale_kind()?;
        let y_scale = host.y_scale_kind()?;
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(host.y_inverted);
        let tick_len = points_to_px(theme.tick_length, self.dpi);
        let tick_stroke = StrokeStyle::new(theme.tick, points_to_px(theme.tick_width, self.dpi));
        let tick_font = points_to_px_f32(theme.tick_label_size, self.dpi);
        let label_pad = points_to_px(3.5, self.dpi);
        let x_label_style = TextStyle::new(theme.label, tick_font)
            .align(TextAlign::Center)
            .baseline(TextBaseline::Bottom);
        let (x_targets, _) = self.tick_targets(twin, layout, theme);
        for tick in twin.major_ticks_x_targeted(x_targets) {
            let x = transform.map_x(tick.value);
            renderer.draw_line(
                Point::new(x, layout.axes.y0),
                Point::new(x, layout.axes.y0 - tick_len),
                &tick_stroke,
            )?;
            crate::mathtext::draw_text(
                renderer,
                &tick.label,
                Point::new(x, layout.axes.y0 - tick_len - label_pad),
                &x_label_style,
            )?;
        }
        Ok(())
    }

    fn draw_twin_x_label(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        twin: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let Some(xlabel) = &twin.x_label else {
            return Ok(());
        };
        let style = TextStyle::new(theme.label, points_to_px_f32(theme.label_size, self.dpi))
            .align(TextAlign::Center)
            .baseline(TextBaseline::Bottom);
        let (x_targets, _) = self.tick_targets(twin, layout, theme);
        let band =
            crate::layout::x_tick_label_band_targeted(twin, theme, renderer, self.dpi, x_targets);
        let title_pt = host
            .title
            .as_ref()
            .map(|_| f64::from(host.title_size_pt(theme)));
        let (_, label_pos) = layout.top_x_chrome_anchors(
            self.dpi,
            title_pt,
            Some(f64::from(theme.label_size)),
            band,
        );
        let pos = label_pos.unwrap_or_else(|| layout.x_label_anchor_top(self.dpi, band));
        renderer.draw_text(xlabel, pos, &style)?;
        Ok(())
    }

    fn draw_secondary_y_ticks(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        sec: &crate::secondary::SecondaryAxis,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let x_scale = host.x_scale_kind()?;
        let y_scale = host.y_scale_kind()?;
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(host.y_inverted);
        let tick_len = points_to_px(theme.tick_length, self.dpi);
        let tick_stroke = StrokeStyle::new(theme.tick, points_to_px(theme.tick_width, self.dpi));
        let tick_font = points_to_px_f32(theme.tick_label_size, self.dpi);
        let label_pad = points_to_px(crate::mpl_policy::chrome::TICK_LABEL_PAD_PT, self.dpi);
        let y_label_style = TextStyle::new(theme.label, tick_font)
            .align(TextAlign::Left)
            .baseline(TextBaseline::Middle);
        for (primary, tick) in sec.mapped_ticks(host.y_min, host.y_max) {
            let y = transform.map_y(primary);
            renderer.draw_line(
                Point::new(layout.axes.x1, y),
                Point::new(layout.axes.x1 + tick_len, y),
                &tick_stroke,
            )?;
            crate::mathtext::draw_text(
                renderer,
                &tick.label,
                Point::new(layout.axes.x1 + tick_len + label_pad, y),
                &y_label_style,
            )?;
        }
        Ok(())
    }

    fn draw_secondary_y_label(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        sec: &crate::secondary::SecondaryAxis,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let Some(ylabel) = &sec.label else {
            return Ok(());
        };
        let style = TextStyle::new(theme.label, points_to_px_f32(theme.label_size, self.dpi))
            .align(TextAlign::Center)
            .baseline(TextBaseline::Middle)
            .rotation(90.0);
        let band = secondary_y_tick_band(host, sec, theme, renderer, self.dpi);
        let label_pt = f64::from(theme.label_size);
        renderer.draw_text(
            ylabel,
            layout.y_label_anchor_right(self.dpi, band, label_pt),
            &style,
        )?;
        Ok(())
    }

    fn draw_secondary_x_ticks(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        sec: &crate::secondary::SecondaryAxis,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let x_scale = host.x_scale_kind()?;
        let y_scale = host.y_scale_kind()?;
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(host.y_inverted);
        let tick_len = points_to_px(theme.tick_length, self.dpi);
        let tick_stroke = StrokeStyle::new(theme.tick, points_to_px(theme.tick_width, self.dpi));
        let tick_font = points_to_px_f32(theme.tick_label_size, self.dpi);
        let label_pad = points_to_px(crate::mpl_policy::chrome::TICK_LABEL_PAD_PT, self.dpi);
        let x_label_style = TextStyle::new(theme.label, tick_font)
            .align(TextAlign::Center)
            .baseline(TextBaseline::Bottom);
        for (primary, tick) in sec.mapped_ticks(host.x_min, host.x_max) {
            let x = transform.map_x(primary);
            renderer.draw_line(
                Point::new(x, layout.axes.y0),
                Point::new(x, layout.axes.y0 - tick_len),
                &tick_stroke,
            )?;
            crate::mathtext::draw_text(
                renderer,
                &tick.label,
                Point::new(x, layout.axes.y0 - tick_len - label_pad),
                &x_label_style,
            )?;
        }
        Ok(())
    }

    fn draw_secondary_x_label(
        &self,
        renderer: &mut dyn Renderer,
        host: &Axes,
        sec: &crate::secondary::SecondaryAxis,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let Some(xlabel) = &sec.label else {
            return Ok(());
        };
        let style = TextStyle::new(theme.label, points_to_px_f32(theme.label_size, self.dpi))
            .align(TextAlign::Center)
            .baseline(TextBaseline::Bottom);
        let band = secondary_x_tick_band(host, sec, theme, renderer, self.dpi);
        let title_pt = host
            .title
            .as_ref()
            .map(|_| f64::from(host.title_size_pt(theme)));
        let (_, label_pos) = layout.top_x_chrome_anchors(
            self.dpi,
            title_pt,
            Some(f64::from(theme.label_size)),
            band,
        );
        let pos = label_pos.unwrap_or_else(|| layout.x_label_anchor_top(self.dpi, band));
        renderer.draw_text(xlabel, pos, &style)?;
        Ok(())
    }

    fn draw_titles(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let title_pt = axes.title_size_pt(theme);
        if let Some(title) = &axes.title {
            let style = TextStyle::new(theme.title, points_to_px_f32(title_pt, self.dpi))
                .align(TextAlign::Center)
                .baseline(TextBaseline::Bottom);
            // Polar / top-x chrome: same stack as twin/spy (cell clamp + tick band).
            let pos = if axes.polar {
                // Sit in the outer margin above the cell when available, but
                // always clear the outward `90°` θ label.
                let floor = points_to_px(2.0, self.dpi)
                    + points_to_px(f64::from(title_pt) * 0.85, self.dpi);
                let clear = layout.axes.y0 - points_to_px(polar_policy::TITLE_CLEAR_PT, self.dpi);
                let in_margin = layout.cell.y0 - points_to_px(3.0, self.dpi);
                let y = in_margin.min(clear).max(floor);
                Point::new(layout.axes.center().x, y)
            } else if axes.twin_x.is_some()
                || axes.secondary_x.is_some()
                || (axes.x_ticks_top && !axes.x_datetime)
            {
                let (x_targets, _) = self.tick_targets(axes, layout, theme);
                let band = if let Some(twin) = axes.twin_x.as_deref() {
                    crate::layout::x_tick_label_band_targeted(
                        twin, theme, renderer, self.dpi, x_targets,
                    )
                } else if let Some(sec) = axes.secondary_x.as_ref() {
                    secondary_x_tick_band(axes, sec, theme, renderer, self.dpi)
                } else if axes.x_ticks_top {
                    crate::layout::x_tick_label_band_targeted(
                        axes, theme, renderer, self.dpi, x_targets,
                    )
                } else {
                    0.0
                };
                let has_top_label = axes.twin_x.as_deref().is_some_and(|t| t.x_label.is_some())
                    || axes.secondary_x.as_ref().is_some_and(|s| s.label.is_some());
                let label_pt = has_top_label.then_some(f64::from(theme.label_size));
                let (title_pos, _) = layout.top_x_chrome_anchors(
                    self.dpi,
                    Some(f64::from(title_pt)),
                    label_pt,
                    band,
                );
                title_pos.unwrap_or_else(|| layout.title_anchor(self.dpi))
            } else {
                layout.title_anchor(self.dpi)
            };
            crate::mathtext::draw_text(renderer, title, pos, &style)?;
        }
        // Polar axes use ?? / r tick labels on the disk; skip cartesian axis titles.
        if axes.polar {
            return Ok(());
        }
        if let Some(xlabel) = &axes.x_label {
            let style = TextStyle::new(
                theme.label,
                points_to_px_f32(axes.x_label_size_pt(theme), self.dpi),
            )
            .align(TextAlign::Center)
            .baseline(TextBaseline::Top);
            let (x_targets, _) = self.tick_targets(axes, layout, theme);
            let mut below_ticks = crate::layout::x_tick_label_band_targeted(
                axes, theme, renderer, self.dpi, x_targets,
            );
            if axes.x_datetime {
                // Rotated autofmt ticks already consume most of the bottom band;
                // using the full measured depth parks "date" below mpl.
                below_ticks *= datetime_policy::X_LABEL_BAND_FRAC;
            }
            crate::mathtext::draw_text(
                renderer,
                xlabel,
                layout.x_label_anchor(self.dpi, below_ticks),
                &style,
            )?;
        }
        if let Some(ylabel) = &axes.y_label {
            let style = TextStyle::new(
                theme.label,
                points_to_px_f32(axes.y_label_size_pt(theme), self.dpi),
            )
            .align(TextAlign::Center)
            .baseline(TextBaseline::Middle)
            .rotation(-90.0);
            let (_, y_targets) = self.tick_targets(axes, layout, theme);
            let band = crate::layout::y_tick_label_band_targeted(
                axes, theme, renderer, self.dpi, y_targets,
            );
            let label_pt = f64::from(axes.y_label_size_pt(theme));
            crate::mathtext::draw_text(
                renderer,
                ylabel,
                layout.y_label_anchor(self.dpi, band, label_pt),
                &style,
            )?;
        }
        Ok(())
    }

    fn draw_colorbar(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let Some(spec) = axes.colorbar_spec() else {
            return Ok(());
        };
        let (vmin, vmax) = (spec.vmin, spec.vmax);
        let bar_w = points_to_px(cbar_policy::BAR_WIDTH_PT, self.dpi);
        let gap = points_to_px(cbar_policy::GAP_PT, self.dpi);
        let x0 = layout.axes.x1 + gap;
        let x1 = (x0 + bar_w).min(layout.cell.x1 - points_to_px(4.0, self.dpi));
        if x1 <= x0 + points_to_px(2.0, self.dpi) {
            return Ok(());
        }
        let y0 = layout.axes.y0;
        let y1 = layout.axes.y1;
        if let Some(bounds) = spec.boundaries.as_ref().filter(|b| b.len() >= 2) {
            // Discrete BoundaryNorm colorbar (matplotlib contourf).
            let span = (vmax - vmin).abs().max(1e-15);
            for i in 0..bounds.len() - 1 {
                let lo = bounds[i];
                let hi = bounds[i + 1];
                let mid = 0.5 * (lo + hi);
                let color = spec.cmap.map_norm(mid, vmin, vmax, spec.norm);
                // vmin at colorbar bottom (pixel y1), vmax at top (pixel y0).
                let t0 = ((lo - vmin) / span).clamp(0.0, 1.0);
                let t1 = ((hi - vmin) / span).clamp(0.0, 1.0);
                let yy_lo = y1 - t0 * (y1 - y0); // higher pixel y
                let yy_hi = y1 - t1 * (y1 - y0);
                renderer.fill_rect(
                    Rect::new(x0, yy_hi.min(yy_lo), x1, yy_hi.max(yy_lo)),
                    &FillStyle::solid_crisp(color),
                )?;
            }
        } else {
            // Dense strips ??mpl's continuous ScalarMappable colorbar.
            let n = ((y1 - y0).ceil() as usize).clamp(64, 512);
            let h = ((y1 - y0) / n as f64).max(0.5);
            for i in 0..n {
                // Bottom = vmin, top = vmax (data y grows up; pixel y grows down).
                let t = i as f64 / (n - 1).max(1) as f64;
                let color =
                    spec.cmap
                        .map_norm(vmin + (1.0 - t) * (vmax - vmin), vmin, vmax, spec.norm);
                let yy0 = y0 + i as f64 * h;
                let yy1 = (yy0 + h + 0.5).min(y1);
                renderer.fill_rect(Rect::new(x0, yy0, x1, yy1), &FillStyle::solid_crisp(color))?;
            }
        }
        renderer.stroke_rect(
            Rect::new(x0, y0, x1, y1),
            &StrokeStyle::new(theme.spine, points_to_px(0.8, self.dpi)),
        )?;

        let ticks = if let Some(bounds) = spec.boundaries.as_ref().filter(|b| b.len() >= 2) {
            if spec.cmap.listed_len().is_some() {
                // ListedColormap colorbar: nice linear ticks (mpl tab10 ??0,2,4,??.
                let linear_targets = cbar_policy::linear_tick_targets(vmin, vmax);
                TickLocator::new(linear_targets).ticks_linear(
                    plotine_core::LinearScale::new(vmin, vmax).unwrap_or_else(|_| {
                        plotine_core::LinearScale::new(0.0, 1.0).expect("unit scale")
                    }),
                )
            } else {
                // Contourf BoundaryNorm: label boundaries (skip when dense).
                let step = if bounds.len() > 10 { 2 } else { 1 };
                bounds
                    .iter()
                    .step_by(step)
                    .copied()
                    .map(plotine_core::Tick::from_value)
                    .collect::<Vec<_>>()
            }
        } else {
            let linear_targets = cbar_policy::linear_tick_targets(vmin, vmax);
            match spec.norm {
                plotine_core::Norm::Log => {
                    match plotine_core::LogScale::new(
                        vmin.max(1e-300),
                        vmax.max(vmin.max(1e-300) * 10.0),
                    ) {
                        Ok(scale) => TickLocator::new(5).ticks_log(scale),
                        Err(_) => TickLocator::new(5).ticks_linear(
                            plotine_core::LinearScale::new(0.0, 1.0).expect("unit scale"),
                        ),
                    }
                }
                plotine_core::Norm::Linear | _ => TickLocator::new(linear_targets).ticks_linear(
                    plotine_core::LinearScale::new(vmin, vmax).unwrap_or_else(|_| {
                        plotine_core::LinearScale::new(0.0, 1.0).expect("unit scale")
                    }),
                ),
            }
        };
        let label_style = TextStyle::new(
            theme.label,
            points_to_px_f32(theme.tick_label_size * 0.9, self.dpi),
        )
        .align(TextAlign::Left)
        .baseline(TextBaseline::Middle);
        let tick_stub = points_to_px(3.0, self.dpi);
        let label_gap = points_to_px(5.0, self.dpi);
        for tick in ticks {
            let t = spec.norm.normalize(tick.value, vmin, vmax);
            let y = y1 - t * (y1 - y0);
            renderer.draw_line(
                Point::new(x1, y),
                Point::new(x1 + tick_stub, y),
                &StrokeStyle::new(theme.tick, points_to_px(0.9, self.dpi)),
            )?;
            crate::mathtext::draw_text(
                renderer,
                &tick.label,
                Point::new(x1 + label_gap, y),
                &label_style,
            )?;
        }
        if let Some(ref label) = axes.colorbar_label {
            let cbar_label_style =
                TextStyle::new(theme.label, points_to_px_f32(theme.label_size, self.dpi))
                    .align(TextAlign::Center)
                    .baseline(TextBaseline::Middle)
                    .rotation(90.0);
            let label_x = x1 + points_to_px(30.0, self.dpi);
            let label_y = (y0 + y1) * 0.5;
            crate::mathtext::draw_text(
                renderer,
                label,
                Point::new(label_x, label_y),
                &cbar_label_style,
            )?;
        }
        Ok(())
    }

    /// Pixel samples used by [`Legend::Best`] to avoid covering data.
    fn legend_collision_samples(&self, axes: &Axes, layout: &Layout) -> Vec<(f64, f64)> {
        let Ok(x_scale) = axes.x_scale_kind() else {
            return Vec::new();
        };
        let Ok(y_scale) = axes.y_scale_kind() else {
            return Vec::new();
        };
        let transform =
            DataToPixel::new(x_scale, y_scale, layout.axes).with_invert_y(axes.y_inverted);
        let mut out = Vec::new();
        let mut push_xy = |xs: &[f64], ys: &[f64]| {
            let n = xs.len().min(ys.len());
            let step = (n / 64).max(1);
            for i in (0..n).step_by(step) {
                if xs[i].is_finite() && ys[i].is_finite() {
                    let p = transform.map(Point::new(xs[i], ys[i]));
                    out.push((p.x, p.y));
                }
            }
        };
        for el in &axes.elements {
            match el {
                crate::artist::PlotElement::Line(p) => push_xy(p.x.as_slice(), p.y.as_slice()),
                crate::artist::PlotElement::Scatter(p) => push_xy(p.x.as_slice(), p.y.as_slice()),
                crate::artist::PlotElement::Step(p) => push_xy(p.x.as_slice(), p.y.as_slice()),
                crate::artist::PlotElement::Stem(p) => push_xy(p.x.as_slice(), p.y.as_slice()),
                crate::artist::PlotElement::ErrorBar(p) => push_xy(p.x.as_slice(), p.y.as_slice()),
                crate::artist::PlotElement::Bar(p) => push_xy(p.x.as_slice(), p.heights.as_slice()),
                crate::artist::PlotElement::Area(p) => push_xy(p.x.as_slice(), p.y.as_slice()),
                _ => {}
            }
        }
        if let Some(twin) = axes.twin_y.as_deref() {
            if let (Ok(tx), Ok(ty)) = (twin.x_scale_kind(), twin.y_scale_kind()) {
                let tform = DataToPixel::new(tx, ty, layout.axes).with_invert_y(twin.y_inverted);
                for el in &twin.elements {
                    if let crate::artist::PlotElement::Line(p) = el {
                        let n = p.x.len().min(p.y.len());
                        let step = (n / 64).max(1);
                        for i in (0..n).step_by(step) {
                            let xs = p.x.as_slice();
                            let ys = p.y.as_slice();
                            if xs[i].is_finite() && ys[i].is_finite() {
                                let pt = tform.map(Point::new(xs[i], ys[i]));
                                out.push((pt.x, pt.y));
                            }
                        }
                    }
                }
            }
        }
        if let Some(twin) = axes.twin_x.as_deref() {
            if let (Ok(tx), Ok(ty)) = (twin.x_scale_kind(), twin.y_scale_kind()) {
                let tform = DataToPixel::new(tx, ty, layout.axes).with_invert_y(axes.y_inverted);
                for el in &twin.elements {
                    if let crate::artist::PlotElement::Line(p) = el {
                        let n = p.x.len().min(p.y.len());
                        let step = (n / 64).max(1);
                        for i in (0..n).step_by(step) {
                            let xs = p.x.as_slice();
                            let ys = p.y.as_slice();
                            if xs[i].is_finite() && ys[i].is_finite() {
                                let pt = tform.map(Point::new(xs[i], ys[i]));
                                out.push((pt.x, pt.y));
                            }
                        }
                    }
                }
            }
        }
        out
    }

    fn draw_legend(
        &self,
        renderer: &mut dyn Renderer,
        axes: &Axes,
        layout: &Layout,
        theme: &Theme,
    ) -> Result<()> {
        let Some(loc) = axes.legend else {
            return Ok(());
        };

        let mut entries: Vec<_> = axes
            .elements
            .iter()
            .flat_map(|el| el.legend_items(&DEFAULT_CYCLE))
            .collect();
        if let Some(twin) = axes.twin_y.as_deref() {
            entries.extend(
                twin.elements
                    .iter()
                    .flat_map(|el| el.legend_items(&DEFAULT_CYCLE)),
            );
        }
        if let Some(twin) = axes.twin_x.as_deref() {
            entries.extend(
                twin.elements
                    .iter()
                    .flat_map(|el| el.legend_items(&DEFAULT_CYCLE)),
            );
        }
        if entries.is_empty() {
            return Ok(());
        }

        // Matplotlib legend defaults (fontsize = tick label size):
        // borderpad=0.4, labelspacing=0.5, handlelength=2.0, handleheight=0.7,
        // borderaxespad=0.5, framealpha=0.8.
        let font = points_to_px_f32(theme.tick_label_size, self.dpi);
        let fs = f64::from(theme.tick_label_size);
        let borderpad = points_to_px(0.4 * fs, self.dpi);
        let labelspacing = points_to_px(0.5 * fs, self.dpi);
        let handlelength = points_to_px(2.0 * fs, self.dpi);
        let handleheight = points_to_px(0.7 * fs, self.dpi);
        let inset = if loc.is_outside() {
            points_to_px(legend_policy::OUTSIDE_PAD_PT, self.dpi)
        } else {
            points_to_px(0.5 * fs, self.dpi)
        };
        let text_gap = points_to_px(0.8 * fs, self.dpi); // handletextpad ≈0.8
        let col_gap = points_to_px(1.0 * fs, self.dpi);
        let row_h = (font as f64).max(handleheight) + labelspacing;

        let ncol = axes.legend_ncol.max(1).min(entries.len().max(1));
        let nrows_legend = entries.len().div_ceil(ncol);

        let mut col_text_w = vec![0.0_f64; ncol];
        for (i, (label, _, _)) in entries.iter().enumerate() {
            let col = i / nrows_legend;
            let (w, _) = crate::mathtext::measure_text(renderer, label, font)?;
            col_text_w[col] = col_text_w[col].max(w);
        }

        let entry_w = |c: usize| handlelength + text_gap + col_text_w[c];
        let box_w = borderpad * 2.0
            + (0..ncol).map(entry_w).sum::<f64>()
            + col_gap * (ncol.saturating_sub(1)) as f64;
        let box_h = borderpad * 2.0 + row_h * nrows_legend as f64 - labelspacing;

        let samples = self.legend_collision_samples(axes, layout);
        let loc = loc.resolve_best(layout.axes, box_w, box_h, inset, &samples);
        let (x0, y0) = loc.anchor(layout.axes, box_w, box_h, inset);

        let box_rect = Rect::new(x0, y0, x0 + box_w, y0 + box_h);
        renderer.fill_rect(box_rect, &FillStyle::solid(Color::WHITE.with_alpha(0.8)))?;
        renderer.stroke_rect(
            box_rect,
            &StrokeStyle::new(theme.spine.with_alpha(0.7), points_to_px(0.8, self.dpi)),
        )?;

        let text_style = TextStyle::new(theme.label, font)
            .align(TextAlign::Left)
            .baseline(TextBaseline::Middle);

        let mut col_x = vec![0.0_f64; ncol];
        col_x[0] = x0 + borderpad;
        for c in 1..ncol {
            col_x[c] = col_x[c - 1] + entry_w(c - 1) + col_gap;
        }

        for (i, (label, color, kind)) in entries.iter().enumerate() {
            let col = i / nrows_legend;
            let row = i % nrows_legend;
            let cy = y0 + borderpad + row_h * row as f64 + (row_h - labelspacing) * 0.5;
            let sx0 = col_x[col];
            let sx1 = sx0 + handlelength;
            match kind {
                LegendKind::Line(ls) => {
                    let width = points_to_px(1.5, self.dpi);
                    let mut style = StrokeStyle::new(*color, width);
                    if let Some(dash) = ls.dash_pattern(width) {
                        style.dash = Some(dash);
                        if matches!(ls, LineStyle::Dotted) {
                            style.cap = LineCap::Round;
                        }
                    }
                    renderer.draw_line(Point::new(sx0, cy), Point::new(sx1, cy), &style)?;
                }
                LegendKind::Marker => {
                    let mx = (sx0 + sx1) * 0.5;
                    renderer.fill_path(
                        &marker_path(
                            Marker {
                                center: Point::new(mx, cy),
                                radius: points_to_px(3.0, self.dpi),
                            },
                            MarkerStyle::Circle,
                        ),
                        &FillStyle::solid(*color),
                    )?;
                }
                LegendKind::ErrorBar => {
                    // Matplotlib ErrorbarContainer: line + marker + ±y/±x caps.
                    let width = points_to_px(1.5, self.dpi);
                    let style = StrokeStyle::new(*color, width);
                    let mx = (sx0 + sx1) * 0.5;
                    let half_y = handleheight * 0.45;
                    let half_x = handlelength * 0.22;
                    let cap = points_to_px(2.0, self.dpi);
                    renderer.draw_line(Point::new(sx0, cy), Point::new(sx1, cy), &style)?;
                    renderer.draw_line(
                        Point::new(mx, cy - half_y),
                        Point::new(mx, cy + half_y),
                        &style,
                    )?;
                    renderer.draw_line(
                        Point::new(mx - cap, cy - half_y),
                        Point::new(mx + cap, cy - half_y),
                        &style,
                    )?;
                    renderer.draw_line(
                        Point::new(mx - cap, cy + half_y),
                        Point::new(mx + cap, cy + half_y),
                        &style,
                    )?;
                    renderer.draw_line(
                        Point::new(mx - half_x, cy),
                        Point::new(mx + half_x, cy),
                        &style,
                    )?;
                    renderer.draw_line(
                        Point::new(mx - half_x, cy - cap),
                        Point::new(mx - half_x, cy + cap),
                        &style,
                    )?;
                    renderer.draw_line(
                        Point::new(mx + half_x, cy - cap),
                        Point::new(mx + half_x, cy + cap),
                        &style,
                    )?;
                    renderer.fill_path(
                        &marker_path(
                            Marker {
                                center: Point::new(mx, cy),
                                radius: points_to_px(2.5, self.dpi),
                            },
                            MarkerStyle::Circle,
                        ),
                        &FillStyle::solid(*color),
                    )?;
                }
                LegendKind::Patch => {
                    let half_h = handleheight * 0.5;
                    let r = Rect::new(sx0, cy - half_h, sx1, cy + half_h);
                    renderer.fill_rect(r, &FillStyle::solid(*color))?;
                }
            }
            crate::mathtext::draw_text(
                renderer,
                label,
                Point::new(sx1 + text_gap, cy),
                &text_style,
            )?;
        }
        Ok(())
    }

    /// Save the figure. Format is inferred from the file extension
    /// (`.png` / `.svg` / `.pdf` / `.pgf` / `.eps`).
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            #[cfg(feature = "png")]
            "png" => self.save_png(path),
            #[cfg(feature = "svg")]
            "svg" => self.save_svg(path),
            #[cfg(feature = "pdf")]
            "pdf" => self.save_pdf(path),
            #[cfg(feature = "pgf")]
            "pgf" => self.save_pgf(path),
            #[cfg(feature = "eps")]
            "eps" => self.save_eps(path),
            _ => Err(plotine_core::PlotError::unsupported_format(
                path.display().to_string(),
            )),
        }
    }

    /// Render and write a PNG file (requires feature `png`).
    #[cfg(feature = "png")]
    pub fn save_png(&self, path: impl AsRef<Path>) -> Result<()> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_skia::SkiaRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        renderer.save_png(path)
    }

    /// Render and write an SVG file (requires feature `svg`).
    #[cfg(feature = "svg")]
    pub fn save_svg(&self, path: impl AsRef<Path>) -> Result<()> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_svg::SvgRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        renderer.save_svg(path)
    }

    /// Encode the figure as a deterministic SVG string.
    #[cfg(feature = "svg")]
    pub fn render_svg(&self) -> Result<String> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_svg::SvgRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        Ok(renderer.into_svg())
    }

    /// Render and write a PDF file (requires feature `pdf`).
    ///
    /// Vector output suitable for LaTeX `\includegraphics{...pdf}`.
    #[cfg(feature = "pdf")]
    pub fn save_pdf(&self, path: impl AsRef<Path>) -> Result<()> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_pdf::PdfRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        renderer.save_pdf(path)
    }

    /// Encode the figure as PDF bytes (requires feature `pdf`).
    #[cfg(feature = "pdf")]
    pub fn render_pdf(&self) -> Result<Vec<u8>> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_pdf::PdfRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        renderer.into_pdf()
    }

    /// Render and write a PGF/TikZ fragment (requires feature `pgf`).
    ///
    /// Suitable for `\input{figure.pgf}` after `\usepackage{pgf}` / `tikz`.
    #[cfg(feature = "pgf")]
    pub fn save_pgf(&self, path: impl AsRef<Path>) -> Result<()> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_pgf::PgfRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        renderer.save_pgf(path)
    }

    /// Encode the figure as a PGF string (requires feature `pgf`).
    #[cfg(feature = "pgf")]
    pub fn render_pgf(&self) -> Result<String> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_pgf::PgfRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        Ok(renderer.into_pgf())
    }

    /// Render and write an EPS file via Ghostscript (`feature = "eps"`).
    ///
    /// Pipeline: PDF → `gs -sDEVICE=eps2write`. Requires Ghostscript on `PATH`.
    #[cfg(feature = "eps")]
    pub fn save_eps(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let dir = tempfile::tempdir().map_err(|e| {
            plotine_core::PlotError::external_tool_failed("gs", format!("temp directory: {e}"))
        })?;
        let pdf_path = dir.path().join("figure.pdf");
        self.save_pdf(&pdf_path)?;
        crate::ext_tools::pdf_to_eps(&pdf_path, path)
    }

    /// Encode the figure as PNG bytes (for tests / notebooks).
    #[cfg(feature = "png")]
    pub fn render_png(&self) -> Result<Vec<u8>> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_skia::SkiaRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        renderer.encode_png()
    }

    /// Display this figure inline in an [evcxr](https://github.com/evcxr/evcxr) Jupyter notebook.
    ///
    /// Requires `feature = "evcxr"` (implies `png`). Call as the last expression of a cell:
    ///
    /// ```ignore
    /// Figure::new().axes(|ax| { ax.line(&x, &y); }).evcxr_display()?;
    /// ```
    #[cfg(feature = "evcxr")]
    pub fn evcxr_display(&self) -> Result<()> {
        use base64::Engine;
        let png = self.render_png()?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(png);
        println!("EVCXR_BEGIN_CONTENT image/png\n{b64}\nEVCXR_END_CONTENT");
        Ok(())
    }

    /// Render to raw RGBA8 bytes (width * height * 4).
    #[cfg(feature = "png")]
    pub fn render_rgba(&self) -> Result<(u32, u32, Vec<u8>)> {
        let (w, h) = self.pixel_size();
        let mut renderer = plotine_backend_skia::SkiaRenderer::new(w, h)?;
        self.draw(&mut renderer)?;
        let pixmap = renderer.into_pixmap();
        Ok((w, h, pixmap.data().to_vec()))
    }
}

/// Largest axis-aligned square centered inside `rect` (polar / pie disk).
fn inscribed_square(rect: Rect) -> Rect {
    let side = rect.width().min(rect.height()).max(1.0);
    let c = rect.center();
    let half = side * 0.5;
    Rect::new(c.x - half, c.y - half, c.x + half, c.y + half)
}

/// Synchronize x-axis ranges across panels in the same column (`sharex`).
fn sync_shared_x(panels: &mut [Panel], nrows: usize) {
    let max_col = panels.iter().map(|p| p.col).max().unwrap_or(0);
    for col in 0..=max_col {
        let (mut xmin, mut xmax) = (f64::INFINITY, f64::NEG_INFINITY);
        for p in panels.iter().filter(|p| p.col == col) {
            xmin = xmin.min(p.axes.x_min);
            xmax = xmax.max(p.axes.x_max);
        }
        if !xmin.is_finite() || !xmax.is_finite() {
            continue;
        }
        let bottom_row = panels
            .iter()
            .filter(|p| p.col == col)
            .map(|p| p.row + p.rowspan - 1)
            .max()
            .unwrap_or(nrows.saturating_sub(1));
        for p in panels.iter_mut().filter(|p| p.col == col) {
            p.axes.x_min = xmin;
            p.axes.x_max = xmax;
            let is_bottom = p.row + p.rowspan - 1 == bottom_row;
            if !is_bottom {
                p.axes.x_categories = None;
            }
        }
    }
}

/// Synchronize y-axis ranges across panels in the same row (`sharey`).
fn sync_shared_y(panels: &mut [Panel], _ncols: usize) {
    let max_row = panels.iter().map(|p| p.row).max().unwrap_or(0);
    for row in 0..=max_row {
        let (mut ymin, mut ymax) = (f64::INFINITY, f64::NEG_INFINITY);
        for p in panels.iter().filter(|p| p.row == row) {
            ymin = ymin.min(p.axes.y_min);
            ymax = ymax.max(p.axes.y_max);
        }
        if !ymin.is_finite() || !ymax.is_finite() {
            continue;
        }
        let left_col = panels
            .iter()
            .filter(|p| p.row == row)
            .map(|p| p.col)
            .min()
            .unwrap_or(0);
        for p in panels.iter_mut().filter(|p| p.row == row) {
            p.axes.y_min = ymin;
            p.axes.y_max = ymax;
            if p.col != left_col {
                p.axes.y_categories = None;
            }
        }
    }
}

/// Map matplotlib-style axes fractions `[x0, y0, w, h]` (y up) onto a pixel `Layout`.
///
/// Matches `Axes.inset_axes`: the fraction box *is* the axes bbox (no inner chrome
/// pad). Tick/title labels paint outside into the parent, like stock mpl.
fn layout_from_axes_fraction(parent_axes: Rect, rect: [f64; 4], dpi: f64) -> Layout {
    let [fx, fy, fw, fh] = rect;
    let pw = parent_axes.width().max(1.0);
    let ph = parent_axes.height().max(1.0);
    let x0 = parent_axes.x0 + fx.clamp(0.0, 1.0) * pw;
    let width = fw.clamp(1e-3, 1.0) * pw;
    let height = fh.clamp(1e-3, 1.0) * ph;
    // Fraction y is from the bottom; pixel y increases downward.
    let y1 = parent_axes.y1 - fy.clamp(0.0, 1.0) * ph;
    let y0 = y1 - height;
    let cell = Rect::new(x0, y0, x0 + width, y0 + height);
    // Optional policy pad (stock mpl = 0). Kept for experiments / docs.
    let pad = points_to_px(inset_policy::CHROME_PAD_PT, dpi);
    let axes = if pad > 0.0 {
        Rect::new(
            cell.x0 + pad.min(cell.width() * 0.28),
            cell.y0 + (pad * 0.55).min(cell.height() * 0.22),
            cell.x1 - (pad * 0.45).min(cell.width() * 0.18),
            cell.y1 - pad.min(cell.height() * 0.28),
        )
    } else {
        cell
    };
    Layout { cell, axes }
}

/// Shrink `rect` to the data aspect ratio, centered (mpl `aspect='equal', adjustable='box'`).
fn aspect_fit_rect(rect: Rect, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Rect {
    let dx = (x_max - x_min).abs().max(1e-12);
    let dy = (y_max - y_min).abs().max(1e-12);
    let data_aspect = dx / dy;
    let rect_aspect = rect.width() / rect.height().max(1e-12);
    if data_aspect > rect_aspect {
        let new_h = rect.width() / data_aspect;
        let cy = rect.center().y;
        Rect::new(rect.x0, cy - new_h / 2.0, rect.x1, cy + new_h / 2.0)
    } else {
        let new_w = rect.height() * data_aspect;
        let cx = rect.center().x;
        Rect::new(cx - new_w / 2.0, rect.y0, cx + new_w / 2.0, rect.y1)
    }
}

fn secondary_y_tick_band(
    host: &Axes,
    sec: &crate::secondary::SecondaryAxis,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
) -> f64 {
    let tick_len = points_to_px(theme.tick_length, dpi);
    let label_pad = points_to_px(chrome_policy::TICK_LABEL_PAD_PT, dpi);
    let tick_px = points_to_px(f64::from(theme.tick_label_size), dpi);
    let tick_font = points_to_px_f32(theme.tick_label_size, dpi);
    let mut tick_w = 0.0_f64;
    for (_, tick) in sec.mapped_ticks(host.y_min, host.y_max) {
        if let Ok((w, _)) = crate::mathtext::measure_text(renderer, &tick.label, tick_font) {
            tick_w = tick_w.max(w);
        }
    }
    if tick_w <= 0.0 {
        tick_w = tick_px * 2.2;
    }
    tick_len + label_pad + tick_w
}

fn secondary_x_tick_band(
    host: &Axes,
    sec: &crate::secondary::SecondaryAxis,
    theme: &Theme,
    renderer: &dyn Renderer,
    dpi: f64,
) -> f64 {
    let tick_len = points_to_px(theme.tick_length, dpi);
    let label_pad = points_to_px(chrome_policy::TICK_LABEL_PAD_PT, dpi);
    let tick_px = points_to_px(f64::from(theme.tick_label_size), dpi);
    let tick_font = points_to_px_f32(theme.tick_label_size, dpi);
    let mut tick_h = 0.0_f64;
    for (_, tick) in sec.mapped_ticks(host.x_min, host.x_max) {
        if let Ok((_, h)) = crate::mathtext::measure_text(renderer, &tick.label, tick_font) {
            tick_h = tick_h.max(h);
        }
    }
    if tick_h <= 0.0 {
        tick_h = tick_px;
    }
    tick_len + label_pad + tick_h
}
