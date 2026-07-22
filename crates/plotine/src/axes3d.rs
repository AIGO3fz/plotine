//! 3D axes panel: camera, data ranges, and 3D artist methods.
//!
//! API mirrors matplotlib's `Axes3D`:
//! ```ignore
//! Figure::new().axes3d(|ax| {
//!     ax.plot3d(&x, &y, &z).color(Color::CRIMSON);
//!     ax.scatter3d(&x, &y, &z).size(4.0);
//!     ax.surface(nx, ny, &z).cmap(Colormap::Viridis);
//! }).save("3d.png")?;
//! ```

use plotine_core::{Cmap, Color, Colormap};

use crate::legend::Legend;
use crate::mpl_policy::axes3d as ax3_policy;
use crate::projection::Camera;
use crate::series::{IntoSeries, Series};

// ─── Artist structs ──────────────────────────────────────────────────────────

/// A 3D line through `(x[i], y[i], z[i])`.
#[derive(Debug, Clone)]
pub struct Line3D {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) z: Series,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl Line3D {
    /// Set the line color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = Some(c);
        self
    }
    /// Set the line width in points.
    pub fn width(&mut self, w: f64) -> &mut Self {
        self.width = w;
        self
    }
    /// Set a legend label.
    pub fn label(&mut self, s: impl Into<String>) -> &mut Self {
        self.label = Some(s.into());
        self
    }
}

/// 3D scatter markers at `(x[i], y[i], z[i])`.
#[derive(Debug, Clone)]
pub struct Scatter3D {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) z: Series,
    pub(crate) color: Option<Color>,
    pub(crate) size: f64,
    pub(crate) depthshade: bool,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl Scatter3D {
    /// Set the marker color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = Some(c);
        self
    }
    /// Set the marker diameter in points (matplotlib `s` ≈ diameter²·π/4;
    /// default matches `s=16`).
    pub fn size(&mut self, s: f64) -> &mut Self {
        self.size = s;
        self
    }
    /// Fade far markers via alpha (matplotlib `depthshade=True` / `_zalpha`; default on).
    pub fn depthshade(&mut self, on: bool) -> &mut Self {
        self.depthshade = on;
        self
    }
    /// Set a legend label.
    pub fn label(&mut self, s: impl Into<String>) -> &mut Self {
        self.label = Some(s.into());
        self
    }
}

/// 3D surface plot (row-major Z grid over X×Y).
#[derive(Debug, Clone)]
pub struct Surface3D {
    pub(crate) nx: usize,
    pub(crate) ny: usize,
    /// Optional length-`nx` sample coordinates (default: `0 .. nx-1`).
    pub(crate) x: Option<Series>,
    /// Optional length-`ny` sample coordinates (default: `0 .. ny-1`).
    pub(crate) y: Option<Series>,
    pub(crate) z: Series,
    pub(crate) cmap: Cmap,
    pub(crate) alpha: f64,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
}

impl Surface3D {
    /// Set X sample coordinates (length `nx`), like matplotlib `plot_surface(X, Y, Z)`.
    pub fn x<S: IntoSeries>(&mut self, x: S) -> &mut Self {
        self.x = Some(x.into_series());
        self
    }
    /// Set Y sample coordinates (length `ny`).
    pub fn y<S: IntoSeries>(&mut self, y: S) -> &mut Self {
        self.y = Some(y.into_series());
        self
    }
    /// Set the colormap.
    pub fn cmap(&mut self, cmap: impl Into<Cmap>) -> &mut Self {
        self.cmap = cmap.into();
        self
    }
    /// Set surface alpha (0.0–1.0).
    pub fn alpha(&mut self, a: f64) -> &mut Self {
        self.alpha = a;
        self
    }
    /// Use a single flat color instead of a colormap.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = Some(c);
        self
    }
    /// Set a legend label.
    pub fn label(&mut self, s: impl Into<String>) -> &mut Self {
        self.label = Some(s.into());
        self
    }
}

/// 3D wireframe plot (surface without filled faces).
#[derive(Debug, Clone)]
pub struct Wireframe3D {
    pub(crate) nx: usize,
    pub(crate) ny: usize,
    /// Optional length-`nx` sample coordinates (default: `0 .. nx-1`).
    pub(crate) x: Option<Series>,
    /// Optional length-`ny` sample coordinates (default: `0 .. ny-1`).
    pub(crate) y: Option<Series>,
    pub(crate) z: Series,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl Wireframe3D {
    /// Set X sample coordinates (length `nx`).
    pub fn x<S: IntoSeries>(&mut self, x: S) -> &mut Self {
        self.x = Some(x.into_series());
        self
    }
    /// Set Y sample coordinates (length `ny`).
    pub fn y<S: IntoSeries>(&mut self, y: S) -> &mut Self {
        self.y = Some(y.into_series());
        self
    }
    /// Set the wireframe color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = Some(c);
        self
    }
    /// Set the line width in points.
    pub fn width(&mut self, w: f64) -> &mut Self {
        self.width = w;
        self
    }
    /// Set a legend label.
    pub fn label(&mut self, s: impl Into<String>) -> &mut Self {
        self.label = Some(s.into());
        self
    }
}

/// 3D bar chart with bars extending along z from baseline.
#[derive(Debug, Clone)]
pub struct Bar3D {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) z: Series,
    pub(crate) dx: f64,
    pub(crate) dy: f64,
    pub(crate) color: Option<Color>,
    pub(crate) alpha: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl Bar3D {
    /// Set x-extent of each bar from the anchor (default 0.8; matplotlib `dx`).
    pub fn dx(&mut self, v: f64) -> &mut Self {
        self.dx = v;
        self
    }
    /// Set y-extent of each bar from the anchor (default 0.8; matplotlib `dy`).
    pub fn dy(&mut self, v: f64) -> &mut Self {
        self.dy = v;
        self
    }
    /// Set the bar color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = Some(c);
        self
    }
    /// Set bar alpha (0.0–1.0).
    pub fn alpha(&mut self, a: f64) -> &mut Self {
        self.alpha = a;
        self
    }
    /// Set a legend label.
    pub fn label(&mut self, s: impl Into<String>) -> &mut Self {
        self.label = Some(s.into());
        self
    }
}

/// Static 3D contour lines (iso-z curves on a grid; matplotlib `contour` on Axes3D).
#[derive(Debug, Clone)]
pub struct Contour3D {
    pub(crate) nx: usize,
    pub(crate) ny: usize,
    pub(crate) x: Option<Series>,
    pub(crate) y: Option<Series>,
    pub(crate) z: Series,
    pub(crate) n_levels: usize,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl Contour3D {
    /// Set X sample coordinates (length `nx`).
    pub fn x<S: IntoSeries>(&mut self, x: S) -> &mut Self {
        self.x = Some(x.into_series());
        self
    }
    /// Set Y sample coordinates (length `ny`).
    pub fn y<S: IntoSeries>(&mut self, y: S) -> &mut Self {
        self.y = Some(y.into_series());
        self
    }
    /// Number of automatically spaced contour levels (default 8).
    pub fn levels(&mut self, n: usize) -> &mut Self {
        self.n_levels = n.max(1);
        self
    }
    /// Line color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = Some(c);
        self
    }
    /// Stroke width in points.
    pub fn width(&mut self, w: f64) -> &mut Self {
        self.width = w;
        self
    }
    /// Legend label.
    pub fn label(&mut self, s: impl Into<String>) -> &mut Self {
        self.label = Some(s.into());
        self
    }
}

/// Static 3D quiver arrows at `(x,y,z)` with components `(u,v,w)`.
#[derive(Debug, Clone)]
pub struct Quiver3D {
    pub(crate) x: Series,
    pub(crate) y: Series,
    pub(crate) z: Series,
    pub(crate) u: Series,
    pub(crate) v: Series,
    pub(crate) w: Series,
    pub(crate) scale: f64,
    pub(crate) color: Option<Color>,
    pub(crate) width: f64,
    pub(crate) label: Option<String>,
    pub(crate) color_index: usize,
}

impl Quiver3D {
    /// Divide vector components by this factor (matplotlib `length` / scale; larger → shorter).
    pub fn scale(&mut self, s: f64) -> &mut Self {
        self.scale = s.max(1e-12);
        self
    }
    /// Arrow color.
    pub fn color(&mut self, c: Color) -> &mut Self {
        self.color = Some(c);
        self
    }
    /// Shaft width in points.
    pub fn width(&mut self, w: f64) -> &mut Self {
        self.width = w;
        self
    }
    /// Legend label.
    pub fn label(&mut self, s: impl Into<String>) -> &mut Self {
        self.label = Some(s.into());
        self
    }
}

// ─── PlotElement3D enum ──────────────────────────────────────────────────────

/// All possible 3D artist types.
#[derive(Debug, Clone)]
pub(crate) enum PlotElement3D {
    Line(Line3D),
    Scatter(Scatter3D),
    Surface(Surface3D),
    Wireframe(Wireframe3D),
    Bar(Bar3D),
    Contour(Contour3D),
    Quiver(Quiver3D),
}

// ─── Axes3D ──────────────────────────────────────────────────────────────────

/// A 3D axes panel.
///
/// You receive `&mut Axes3D` inside the closure passed to
/// [`Figure::axes3d()`](crate::Figure::axes3d).
/// Call methods like [`plot3d()`](Self::plot3d), [`scatter3d()`](Self::scatter3d),
/// or [`surface()`](Self::surface) to add 3D artists.
///
/// # Example
///
/// ```
/// use plotine::prelude::*;
/// use std::f64::consts::PI;
///
/// let t: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
/// let x: Vec<f64> = t.iter().map(|v| v.cos()).collect();
/// let y: Vec<f64> = t.iter().map(|v| v.sin()).collect();
/// let z: Vec<f64> = t.clone();
///
/// let png = Figure::new()
///     .size(6.4, 4.8)
///     .dpi(100.0)
///     .axes3d(|ax| {
///         ax.plot3d(&x, &y, &z).color(Color::CRIMSON).width(2.0);
///         ax.title("3D Helix");
///     })
///     .render_png()
///     .unwrap();
/// assert!(!png.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Axes3D {
    pub(crate) title: Option<String>,
    pub(crate) x_label: Option<String>,
    pub(crate) y_label: Option<String>,
    pub(crate) z_label: Option<String>,
    pub(crate) camera: Camera,
    /// View limits (data extent + margin).
    pub(crate) x_min: f64,
    pub(crate) x_max: f64,
    pub(crate) y_min: f64,
    pub(crate) y_max: f64,
    pub(crate) z_min: f64,
    pub(crate) z_max: f64,
    /// Unpadded data extents (padding is recomputed from these — never compound).
    pub(crate) x_data_min: f64,
    pub(crate) x_data_max: f64,
    pub(crate) y_data_min: f64,
    pub(crate) y_data_max: f64,
    pub(crate) z_data_min: f64,
    pub(crate) z_data_max: f64,
    pub(crate) x_lim_manual: bool,
    pub(crate) y_lim_manual: bool,
    pub(crate) z_lim_manual: bool,
    pub(crate) x_seeded: bool,
    pub(crate) y_seeded: bool,
    pub(crate) z_seeded: bool,
    pub(crate) legend: Option<Legend>,
    pub(crate) show_grid: bool,
    pub(crate) elements: Vec<PlotElement3D>,
    pub(crate) next_color_index: usize,
}

impl Default for Axes3D {
    fn default() -> Self {
        Self {
            title: None,
            x_label: None,
            y_label: None,
            z_label: None,
            camera: Camera::default(),
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
            z_min: 0.0,
            z_max: 1.0,
            x_data_min: 0.0,
            x_data_max: 1.0,
            y_data_min: 0.0,
            y_data_max: 1.0,
            z_data_min: 0.0,
            z_data_max: 1.0,
            x_lim_manual: false,
            y_lim_manual: false,
            z_lim_manual: false,
            x_seeded: false,
            y_seeded: false,
            z_seeded: false,
            legend: None,
            show_grid: true,
            elements: Vec::new(),
            next_color_index: 0,
        }
    }
}

impl Axes3D {
    /// Create an empty 3D axes panel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the plot title.
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = Some(title.into());
        self
    }

    /// Set the x-axis label.
    pub fn x_label(&mut self, label: impl Into<String>) -> &mut Self {
        self.x_label = Some(label.into());
        self
    }

    /// Set the y-axis label.
    pub fn y_label(&mut self, label: impl Into<String>) -> &mut Self {
        self.y_label = Some(label.into());
        self
    }

    /// Set the z-axis label.
    pub fn z_label(&mut self, label: impl Into<String>) -> &mut Self {
        self.z_label = Some(label.into());
        self
    }

    /// Set camera elevation angle in degrees (default 30°).
    pub fn elev(&mut self, deg: f64) -> &mut Self {
        self.camera.elev = deg;
        self
    }

    /// Set camera azimuth angle in degrees (default -60°).
    pub fn azim(&mut self, deg: f64) -> &mut Self {
        self.camera.azim = deg;
        self
    }

    /// Manually set the x-axis range.
    pub fn x_range(&mut self, min: f64, max: f64) -> &mut Self {
        let (lo, hi) = ordered(min, max);
        self.x_min = lo;
        self.x_max = hi;
        self.x_data_min = lo;
        self.x_data_max = hi;
        self.x_lim_manual = true;
        self.x_seeded = true;
        self
    }

    /// Manually set the y-axis range.
    pub fn y_range(&mut self, min: f64, max: f64) -> &mut Self {
        let (lo, hi) = ordered(min, max);
        self.y_min = lo;
        self.y_max = hi;
        self.y_data_min = lo;
        self.y_data_max = hi;
        self.y_lim_manual = true;
        self.y_seeded = true;
        self
    }

    /// Manually set the z-axis range.
    pub fn z_range(&mut self, min: f64, max: f64) -> &mut Self {
        let (lo, hi) = ordered(min, max);
        self.z_min = lo;
        self.z_max = hi;
        self.z_data_min = lo;
        self.z_data_max = hi;
        self.z_lim_manual = true;
        self.z_seeded = true;
        self
    }

    /// Show/hide the 3D grid (default true).
    pub fn grid(&mut self, show: bool) -> &mut Self {
        self.show_grid = show;
        self
    }

    /// Show a legend.
    pub fn legend(&mut self, loc: Legend) -> &mut Self {
        self.legend = Some(loc);
        self
    }

    /// 3D line plot through `(x[i], y[i], z[i])`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes3d(|ax| {
    ///     ax.plot3d([0.0, 1.0, 2.0], [0.0, 1.0, 0.5], [0.0, 0.5, 1.0])
    ///         .color(Color::CRIMSON);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn plot3d<X, Y, Z>(&mut self, x: X, y: Y, z: Z) -> &mut Line3D
    where
        X: IntoSeries,
        Y: IntoSeries,
        Z: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let z = z.into_series();
        self.expand_xyz(&x, &y, &z);
        let ci = self.alloc_color();
        self.elements.push(PlotElement3D::Line(Line3D {
            x,
            y,
            z,
            color: None,
            width: 1.75,
            label: None,
            color_index: ci,
        }));
        match self.elements.last_mut() {
            Some(PlotElement3D::Line(p)) => p,
            _ => unreachable!(),
        }
    }

    /// 3D scatter plot at `(x[i], y[i], z[i])`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes3d(|ax| {
    ///     ax.scatter3d([0.0, 1.0, 2.0], [0.0, 1.0, 0.5], [0.0, 0.5, 1.0])
    ///         .size(5.0);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn scatter3d<X, Y, Z>(&mut self, x: X, y: Y, z: Z) -> &mut Scatter3D
    where
        X: IntoSeries,
        Y: IntoSeries,
        Z: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let z = z.into_series();
        self.expand_xyz(&x, &y, &z);
        let ci = self.alloc_color();
        self.elements.push(PlotElement3D::Scatter(Scatter3D {
            x,
            y,
            z,
            color: None,
            size: ax3_policy::SCATTER_DIAMETER_PT,
            depthshade: true,
            label: None,
            color_index: ci,
        }));
        match self.elements.last_mut() {
            Some(PlotElement3D::Scatter(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Surface plot of a row-major `ny × nx` grid of Z values.
    ///
    /// X spans `[0, nx-1]` and Y spans `[0, ny-1]` by default (override with
    /// custom x/y arrays not yet exposed; use `x_range`/`y_range`).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let nx = 10;
    /// let ny = 10;
    /// let z: Vec<f64> = (0..ny).flat_map(|j| {
    ///     (0..nx).map(move |i| {
    ///         let x = i as f64 / nx as f64 * 4.0 - 2.0;
    ///         let y = j as f64 / ny as f64 * 4.0 - 2.0;
    ///         (-(x * x + y * y)).exp()
    ///     })
    /// }).collect();
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes3d(|ax| {
    ///     ax.surface(nx, ny, &z).cmap(Colormap::Viridis);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn surface<V>(&mut self, nx: usize, ny: usize, z: V) -> &mut Surface3D
    where
        V: IntoSeries,
    {
        let z = z.into_series();
        let nx = nx.max(2);
        let ny = ny.max(2);
        self.expand_range_x(0.0, (nx - 1) as f64);
        self.expand_range_y(0.0, (ny - 1) as f64);
        self.expand_z_series(&z);
        self.elements.push(PlotElement3D::Surface(Surface3D {
            nx,
            ny,
            x: None,
            y: None,
            z,
            cmap: Colormap::Viridis.into(),
            alpha: 0.85,
            color: None,
            label: None,
        }));
        match self.elements.last_mut() {
            Some(PlotElement3D::Surface(p)) => p,
            _ => unreachable!(),
        }
    }

    /// Wireframe plot of a row-major `ny × nx` grid of Z values.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let nx = 8;
    /// let ny = 8;
    /// let z: Vec<f64> = (0..ny).flat_map(|j| {
    ///     (0..nx).map(move |i| (i as f64 - 3.5).powi(2) + (j as f64 - 3.5).powi(2))
    /// }).collect();
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes3d(|ax| {
    ///     ax.wireframe(nx, ny, &z).color(Color::STEEL_BLUE);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn wireframe<V>(&mut self, nx: usize, ny: usize, z: V) -> &mut Wireframe3D
    where
        V: IntoSeries,
    {
        let z = z.into_series();
        let nx = nx.max(2);
        let ny = ny.max(2);
        self.expand_range_x(0.0, (nx - 1) as f64);
        self.expand_range_y(0.0, (ny - 1) as f64);
        self.expand_z_series(&z);
        let ci = self.alloc_color();
        self.elements.push(PlotElement3D::Wireframe(Wireframe3D {
            nx,
            ny,
            x: None,
            y: None,
            z,
            color: None,
            width: 0.8,
            label: None,
            color_index: ci,
        }));
        match self.elements.last_mut() {
            Some(PlotElement3D::Wireframe(p)) => p,
            _ => unreachable!(),
        }
    }

    /// 3D bar chart: bars anchored at `(x[i], y[i], 0)` with height `z[i]`.
    ///
    /// Matches matplotlib `bar3d`: each bar spans `[x, x+dx] × [y, y+dy] × [0, z]`.
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes3d(|ax| {
    ///     ax.bar3d([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [3.0, 5.0, 2.0])
    ///         .color(Color::STEEL_BLUE);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn bar3d<X, Y, Z>(&mut self, x: X, y: Y, z: Z) -> &mut Bar3D
    where
        X: IntoSeries,
        Y: IntoSeries,
        Z: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let z = z.into_series();
        // Default dx=dy=0.8: include full footprint in axis limits.
        let dx = 0.8;
        let dy = 0.8;
        if let (Some((x0, x1)), Some((y0, y1)), Some((z0, z1))) =
            (x.min_max(), y.min_max(), z.min_max())
        {
            self.expand_range_x(x0, x1 + dx);
            self.expand_range_y(y0, y1 + dy);
            self.expand_range_z(z0.min(0.0), z1.max(0.0));
        }
        let ci = self.alloc_color();
        self.elements.push(PlotElement3D::Bar(Bar3D {
            x,
            y,
            z,
            dx: 0.8,
            dy: 0.8,
            color: None,
            alpha: 0.85,
            label: None,
            color_index: ci,
        }));
        match self.elements.last_mut() {
            Some(PlotElement3D::Bar(p)) => p,
            _ => unreachable!(),
        }
    }

    /// 3D contour lines of a row-major `ny × nx` Z grid (static; no interactive).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let n = 12usize;
    /// let xs: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64 * 4.0 - 2.0).collect();
    /// let ys = xs.clone();
    /// let mut z = Vec::with_capacity(n * n);
    /// for &yv in &ys {
    ///     for &xv in &xs {
    ///         z.push((-(xv * xv + yv * yv) * 0.5).exp());
    ///     }
    /// }
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes3d(|ax| {
    ///     ax.contour3d(n, n, &z).x(&xs).y(&ys).levels(6).color(Color::CRIMSON);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn contour3d<V>(&mut self, nx: usize, ny: usize, z: V) -> &mut Contour3D
    where
        V: IntoSeries,
    {
        let z = z.into_series();
        let nx = nx.max(2);
        let ny = ny.max(2);
        self.expand_range_x(0.0, (nx - 1) as f64);
        self.expand_range_y(0.0, (ny - 1) as f64);
        self.expand_z_series(&z);
        let ci = self.alloc_color();
        self.elements.push(PlotElement3D::Contour(Contour3D {
            nx,
            ny,
            x: None,
            y: None,
            z,
            n_levels: 8,
            color: None,
            width: 1.0,
            label: None,
            color_index: ci,
        }));
        match self.elements.last_mut() {
            Some(PlotElement3D::Contour(p)) => p,
            _ => unreachable!(),
        }
    }

    /// 3D quiver: arrows at `(x,y,z)` with components `(u,v,w)` (static).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(4.0, 3.0).dpi(72.0).axes3d(|ax| {
    ///     ax.quiver3d(
    ///         [0.0, 1.0], [0.0, 0.0], [0.0, 0.0],
    ///         [1.0, 0.0], [0.0, 1.0], [0.5, 0.5],
    ///     )
    ///     .scale(1.0)
    ///     .color(Color::STEEL_BLUE);
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn quiver3d<X, Y, Z, U, V, W>(
        &mut self,
        x: X,
        y: Y,
        z: Z,
        u: U,
        v: V,
        w: W,
    ) -> &mut Quiver3D
    where
        X: IntoSeries,
        Y: IntoSeries,
        Z: IntoSeries,
        U: IntoSeries,
        V: IntoSeries,
        W: IntoSeries,
    {
        let x = x.into_series();
        let y = y.into_series();
        let z = z.into_series();
        let u = u.into_series();
        let v = v.into_series();
        let w = w.into_series();
        self.expand_xyz(&x, &y, &z);
        // Expand by a unit step so arrow tips stay inside the box.
        if let (Some((x0, x1)), Some((y0, y1)), Some((z0, z1))) =
            (x.min_max(), y.min_max(), z.min_max())
        {
            self.expand_range_x(x0 - 0.5, x1 + 0.5);
            self.expand_range_y(y0 - 0.5, y1 + 0.5);
            self.expand_range_z(z0 - 0.5, z1 + 0.5);
        }
        let ci = self.alloc_color();
        self.elements.push(PlotElement3D::Quiver(Quiver3D {
            x,
            y,
            z,
            u,
            v,
            w,
            scale: 1.0,
            color: None,
            width: 1.0,
            label: None,
            color_index: ci,
        }));
        match self.elements.last_mut() {
            Some(PlotElement3D::Quiver(p)) => p,
            _ => unreachable!(),
        }
    }

    // ─── Internal helpers ────────────────────────────────────────────────────

    fn alloc_color(&mut self) -> usize {
        let idx = self.next_color_index;
        self.next_color_index += 1;
        idx
    }

    fn expand_xyz(&mut self, x: &Series, y: &Series, z: &Series) {
        if let Some((a, b)) = x.min_max() {
            self.expand_range_x(a, b);
        }
        if let Some((a, b)) = y.min_max() {
            self.expand_range_y(a, b);
        }
        if let Some((a, b)) = z.min_max() {
            self.expand_range_z(a, b);
        }
    }

    fn expand_z_series(&mut self, z: &Series) {
        if let Some((a, b)) = z.min_max() {
            self.expand_range_z(a, b);
        }
    }

    fn expand_range_x(&mut self, lo: f64, hi: f64) {
        if self.x_lim_manual {
            return;
        }
        let (lo, hi) = ordered(lo, hi);
        if !self.x_seeded {
            self.x_data_min = lo;
            self.x_data_max = hi;
            self.x_seeded = true;
        } else {
            self.x_data_min = self.x_data_min.min(lo);
            self.x_data_max = self.x_data_max.max(hi);
        }
        let (vlo, vhi) = padded(self.x_data_min, self.x_data_max);
        self.x_min = vlo;
        self.x_max = vhi;
    }

    fn expand_range_y(&mut self, lo: f64, hi: f64) {
        if self.y_lim_manual {
            return;
        }
        let (lo, hi) = ordered(lo, hi);
        if !self.y_seeded {
            self.y_data_min = lo;
            self.y_data_max = hi;
            self.y_seeded = true;
        } else {
            self.y_data_min = self.y_data_min.min(lo);
            self.y_data_max = self.y_data_max.max(hi);
        }
        let (vlo, vhi) = padded(self.y_data_min, self.y_data_max);
        self.y_min = vlo;
        self.y_max = vhi;
    }

    fn expand_range_z(&mut self, lo: f64, hi: f64) {
        if self.z_lim_manual {
            return;
        }
        let (lo, hi) = ordered(lo, hi);
        if !self.z_seeded {
            self.z_data_min = lo;
            self.z_data_max = hi;
            self.z_seeded = true;
        } else {
            self.z_data_min = self.z_data_min.min(lo);
            self.z_data_max = self.z_data_max.max(hi);
        }
        let (vlo, vhi) = padded_z(self.z_data_min, self.z_data_max);
        self.z_min = vlo;
        self.z_max = vhi;
    }

    /// Axis view ranges used by the projection (includes bar footprints / mesh coords).
    pub(crate) fn ranges(&self) -> ((f64, f64), (f64, f64), (f64, f64)) {
        // Re-derive auto limits from artists so `.x()` / `.y()` / `.dx()` after add work.
        let mut xd = Extents::new();
        let mut yd = Extents::new();
        let mut zd = Extents::new();
        let mut any = false;

        for el in &self.elements {
            match el {
                PlotElement3D::Line(l) => {
                    any |= xd.include_series(&l.x);
                    any |= yd.include_series(&l.y);
                    any |= zd.include_series(&l.z);
                }
                PlotElement3D::Scatter(s) => {
                    any |= xd.include_series(&s.x);
                    any |= yd.include_series(&s.y);
                    any |= zd.include_series(&s.z);
                }
                PlotElement3D::Surface(s) => {
                    let (xs, ys) = mesh_xy(s.nx, s.ny, &s.x, &s.y);
                    any |= xd.include_slice(&xs);
                    any |= yd.include_slice(&ys);
                    any |= zd.include_series(&s.z);
                }
                PlotElement3D::Wireframe(w) => {
                    let (xs, ys) = mesh_xy(w.nx, w.ny, &w.x, &w.y);
                    any |= xd.include_slice(&xs);
                    any |= yd.include_slice(&ys);
                    any |= zd.include_series(&w.z);
                }
                PlotElement3D::Bar(b) => {
                    let dx = b.dx.abs();
                    let dy = b.dy.abs();
                    let n = b.x.len().min(b.y.len()).min(b.z.len());
                    for i in 0..n {
                        let x = b.x.as_slice()[i];
                        let y = b.y.as_slice()[i];
                        let z = b.z.as_slice()[i];
                        if x.is_finite() {
                            any |= xd.include(x);
                            any |= xd.include(x + dx);
                        }
                        if y.is_finite() {
                            any |= yd.include(y);
                            any |= yd.include(y + dy);
                        }
                        if z.is_finite() {
                            any |= zd.include(0.0);
                            any |= zd.include(z);
                        }
                    }
                }
                PlotElement3D::Contour(c) => {
                    let (xs, ys) = mesh_xy(c.nx, c.ny, &c.x, &c.y);
                    any |= xd.include_slice(&xs);
                    any |= yd.include_slice(&ys);
                    any |= zd.include_series(&c.z);
                }
                PlotElement3D::Quiver(q) => {
                    any |= xd.include_series(&q.x);
                    any |= yd.include_series(&q.y);
                    any |= zd.include_series(&q.z);
                }
            }
        }

        let xr = if self.x_lim_manual {
            (self.x_min, self.x_max)
        } else if any && xd.valid() {
            padded(xd.min, xd.max)
        } else {
            (self.x_min, self.x_max)
        };
        let yr = if self.y_lim_manual {
            (self.y_min, self.y_max)
        } else if any && yd.valid() {
            padded(yd.min, yd.max)
        } else {
            (self.y_min, self.y_max)
        };
        let zr = if self.z_lim_manual {
            (self.z_min, self.z_max)
        } else if any && zd.valid() {
            padded_z(zd.min, zd.max)
        } else {
            (self.z_min, self.z_max)
        };
        (xr, yr, zr)
    }
}

/// Resolve mesh sample coordinates (explicit series or integer indices).
pub(crate) fn mesh_xy(
    nx: usize,
    ny: usize,
    x: &Option<Series>,
    y: &Option<Series>,
) -> (Vec<f64>, Vec<f64>) {
    let xs = match x {
        Some(s) if s.len() >= nx => s.as_slice()[..nx].to_vec(),
        _ => (0..nx).map(|i| i as f64).collect(),
    };
    let ys = match y {
        Some(s) if s.len() >= ny => s.as_slice()[..ny].to_vec(),
        _ => (0..ny).map(|j| j as f64).collect(),
    };
    (xs, ys)
}

#[derive(Clone, Copy)]
struct Extents {
    min: f64,
    max: f64,
    seeded: bool,
}

impl Extents {
    fn new() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            seeded: false,
        }
    }
    fn valid(self) -> bool {
        self.seeded && self.min.is_finite() && self.max.is_finite()
    }
    fn include(&mut self, v: f64) -> bool {
        if !v.is_finite() {
            return false;
        }
        if !self.seeded {
            self.min = v;
            self.max = v;
            self.seeded = true;
        } else {
            self.min = self.min.min(v);
            self.max = self.max.max(v);
        }
        true
    }
    fn include_slice(&mut self, s: &[f64]) -> bool {
        let mut any = false;
        for &v in s {
            any |= self.include(v);
        }
        any
    }
    fn include_series(&mut self, s: &Series) -> bool {
        self.include_slice(s.as_slice())
    }
}

fn ordered(a: f64, b: f64) -> (f64, f64) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Matplotlib `Axes3D` x/y autoscale: `ax.margins` (5%) then `_view_margin` (1/48).
fn padded(min: f64, max: f64) -> (f64, f64) {
    padded_with_margin(min, max, crate::mpl_policy::margin::PAD)
}

/// Matplotlib z-axis: `_zmargin` defaults to 0, then `_view_margin` still applies.
fn padded_z(min: f64, max: f64) -> (f64, f64) {
    padded_with_margin(min, max, 0.0)
}

fn padded_with_margin(min: f64, max: f64, margin: f64) -> (f64, f64) {
    let span = (max - min).abs().max(1e-12);
    let mut lo = min - span * margin;
    let mut hi = max + span * margin;
    let vm = crate::mpl_policy::axes3d::VIEW_MARGIN;
    let delta = (hi - lo) * vm;
    lo -= delta;
    hi += delta;
    (lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xy_padding_matches_mplot3d_margins_plus_view_margin() {
        // data [-1,1] → 5% → [-1.1,1.1] → ×(1±1/48) ≈ ±1.145833
        let (lo, hi) = padded(-1.0, 1.0);
        assert!((lo - -1.145_833_333_333_333_5).abs() < 1e-12);
        assert!((hi - 1.145_833_333_333_333_5).abs() < 1e-12);
    }

    #[test]
    fn z_padding_skips_cartesian_margin() {
        // zmargin=0; only view_margin on [0, 4π]
        let z1 = 4.0 * std::f64::consts::PI;
        let (lo, hi) = padded_z(0.0, z1);
        assert!((lo - -0.261_799_387_799_149_4).abs() < 1e-12);
        assert!((hi - 12.828_170_002_158_322).abs() < 1e-12);
    }
}
