//! Matplotlib-aligned navigation math (pan / zoom / 3D rotate) and view history.

use plotine_core::{PlotError, Result, ScaleKind, ScaleType};

use crate::view::{Axes3DView, PanelView, ViewSnapshot};

/// Interaction mode (matplotlib NavigationToolbar2 pan / zoom).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavMode {
    /// Left-drag pans; scroll zooms (default exploration mode).
    #[default]
    Pan,
    /// Left-drag draws a zoom box; scroll still zooms.
    Zoom,
}

/// Browser-like view history with a fixed Home snapshot.
#[derive(Debug, Clone)]
pub struct ViewHistory {
    home: ViewSnapshot,
    stack: Vec<ViewSnapshot>,
    index: usize,
}

impl ViewHistory {
    /// Create a history whose Home is `home` (also the initial current view).
    pub fn new(home: ViewSnapshot) -> Self {
        Self {
            home: home.clone(),
            stack: vec![home],
            index: 0,
        }
    }

    /// Home (initial) view.
    pub fn home(&self) -> &ViewSnapshot {
        &self.home
    }

    /// Current view in the history stack.
    pub fn current(&self) -> &ViewSnapshot {
        &self.stack[self.index]
    }

    /// Push a new view after a completed gesture (truncates forward history).
    pub fn push(&mut self, view: ViewSnapshot) {
        if views_approx_eq(self.current(), &view) {
            return;
        }
        self.stack.truncate(self.index + 1);
        self.stack.push(view);
        self.index = self.stack.len() - 1;
    }

    /// Jump to Home and record it as current (clears forward stack from Home).
    pub fn go_home(&mut self) -> ViewSnapshot {
        self.stack.clear();
        self.stack.push(self.home.clone());
        self.index = 0;
        self.home.clone()
    }

    /// Whether Back is available.
    pub fn can_back(&self) -> bool {
        self.index > 0
    }

    /// Whether Forward is available.
    pub fn can_forward(&self) -> bool {
        self.index + 1 < self.stack.len()
    }

    /// Move back one step.
    pub fn back(&mut self) -> Option<ViewSnapshot> {
        if !self.can_back() {
            return None;
        }
        self.index -= 1;
        Some(self.stack[self.index].clone())
    }

    /// Move forward one step.
    pub fn forward(&mut self) -> Option<ViewSnapshot> {
        if !self.can_forward() {
            return None;
        }
        self.index += 1;
        Some(self.stack[self.index].clone())
    }
}

fn views_approx_eq(a: &ViewSnapshot, b: &ViewSnapshot) -> bool {
    if a.panels.len() != b.panels.len() {
        return false;
    }
    for (pa, pb) in a.panels.iter().zip(b.panels.iter()) {
        if !panel_approx_eq(*pa, *pb) {
            return false;
        }
    }
    match (a.axes3d, b.axes3d) {
        (None, None) => true,
        (Some(aa), Some(bb)) => axes3d_approx_eq(aa, bb),
        _ => false,
    }
}

fn panel_approx_eq(a: PanelView, b: PanelView) -> bool {
    approx(a.x_min, b.x_min)
        && approx(a.x_max, b.x_max)
        && approx(a.y_min, b.y_min)
        && approx(a.y_max, b.y_max)
}

fn axes3d_approx_eq(a: Axes3DView, b: Axes3DView) -> bool {
    approx(a.elev, b.elev)
        && approx(a.azim, b.azim)
        && approx(a.x_min, b.x_min)
        && approx(a.x_max, b.x_max)
        && approx(a.y_min, b.y_min)
        && approx(a.y_max, b.y_max)
        && approx(a.z_min, b.z_min)
        && approx(a.z_max, b.z_max)
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * (1.0 + a.abs().max(b.abs()))
}

/// Pan a single axis by a pixel delta (positive `pixel_delta` shifts the window
/// toward larger data values when dragging content with the mouse — i.e. limits
/// decrease, matching matplotlib LMB pan).
pub fn pan_limits(
    scale_type: ScaleType,
    min: f64,
    max: f64,
    pixel_delta: f64,
    axis_pixel_span: f64,
) -> Result<(f64, f64)> {
    if !(axis_pixel_span.is_finite() && axis_pixel_span > 0.0) {
        return Ok((min, max));
    }
    let scale = ScaleKind::build(scale_type, min, max)?;
    let unit_delta = pixel_delta / axis_pixel_span;
    // Drag right → data under cursor follows → limits move left in unit space.
    let new_min = scale.denormalize(0.0 - unit_delta);
    let new_max = scale.denormalize(1.0 - unit_delta);
    sanitize_limits(scale_type, new_min, new_max)
}

/// Zoom a single axis about an anchor in \[0, 1\] unit space (`factor` > 1 zooms in).
pub fn zoom_limits(
    scale_type: ScaleType,
    min: f64,
    max: f64,
    anchor_unit: f64,
    factor: f64,
) -> Result<(f64, f64)> {
    if !(factor.is_finite() && factor > 0.0) {
        return Ok((min, max));
    }
    let scale = ScaleKind::build(scale_type, min, max)?;
    let a = anchor_unit.clamp(0.0, 1.0);
    let new_min_u = a + (0.0 - a) / factor;
    let new_max_u = a + (1.0 - a) / factor;
    let new_min = scale.denormalize(new_min_u);
    let new_max = scale.denormalize(new_max_u);
    sanitize_limits(scale_type, new_min, new_max)
}

/// Set limits from two data-space corners (box zoom).
pub fn box_limits(scale_type: ScaleType, x0: f64, x1: f64) -> Result<(f64, f64)> {
    let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
    sanitize_limits(scale_type, lo, hi)
}

/// Map a pixel x inside an axes box to data x.
pub fn pixel_to_data_x(
    scale_type: ScaleType,
    min: f64,
    max: f64,
    axes_x0: f64,
    axes_width: f64,
    px: f64,
) -> Result<f64> {
    if !(axes_width.is_finite() && axes_width > 0.0) {
        return Ok(min);
    }
    let scale = ScaleKind::build(scale_type, min, max)?;
    let u = ((px - axes_x0) / axes_width).clamp(0.0, 1.0);
    Ok(scale.denormalize(u))
}

/// Map a pixel y inside an axes box to data y (screen y grows downward).
pub fn pixel_to_data_y(
    scale_type: ScaleType,
    min: f64,
    max: f64,
    axes_y0: f64,
    axes_height: f64,
    py: f64,
) -> Result<f64> {
    if !(axes_height.is_finite() && axes_height > 0.0) {
        return Ok(min);
    }
    let scale = ScaleKind::build(scale_type, min, max)?;
    // Screen y down → unit y up.
    let u = (1.0 - (py - axes_y0) / axes_height).clamp(0.0, 1.0);
    Ok(scale.denormalize(u))
}

/// Pan a 2D panel view. `dx_px` / `dy_px` are mouse deltas (screen space).
pub fn pan_panel(
    view: PanelView,
    x_scale: ScaleType,
    y_scale: ScaleType,
    dx_px: f64,
    dy_px: f64,
    axes_w: f64,
    axes_h: f64,
) -> Result<PanelView> {
    let (x_min, x_max) = pan_limits(x_scale, view.x_min, view.x_max, dx_px, axes_w)?;
    // Screen y down: drag down → content follows → data limits increase.
    let (y_min, y_max) = pan_limits(y_scale, view.y_min, view.y_max, -dy_px, axes_h)?;
    Ok(PanelView {
        x_min,
        x_max,
        y_min,
        y_max,
    })
}

/// Wheel-zoom a 2D panel about a cursor position in axes pixels.
#[allow(clippy::too_many_arguments)]
pub fn zoom_panel(
    view: PanelView,
    x_scale: ScaleType,
    y_scale: ScaleType,
    cursor_x: f64,
    cursor_y: f64,
    axes_x0: f64,
    axes_y0: f64,
    axes_w: f64,
    axes_h: f64,
    factor: f64,
) -> Result<PanelView> {
    let ax = if axes_w > 0.0 {
        ((cursor_x - axes_x0) / axes_w).clamp(0.0, 1.0)
    } else {
        0.5
    };
    let ay = if axes_h > 0.0 {
        (1.0 - (cursor_y - axes_y0) / axes_h).clamp(0.0, 1.0)
    } else {
        0.5
    };
    let (x_min, x_max) = zoom_limits(x_scale, view.x_min, view.x_max, ax, factor)?;
    let (y_min, y_max) = zoom_limits(y_scale, view.y_min, view.y_max, ay, factor)?;
    Ok(PanelView {
        x_min,
        x_max,
        y_min,
        y_max,
    })
}

/// Box-zoom a 2D panel from two screen corners.
#[allow(clippy::too_many_arguments)]
pub fn box_zoom_panel(
    view: PanelView,
    x_scale: ScaleType,
    y_scale: ScaleType,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    axes_x0: f64,
    axes_y0: f64,
    axes_w: f64,
    axes_h: f64,
) -> Result<PanelView> {
    let dx0 = pixel_to_data_x(x_scale, view.x_min, view.x_max, axes_x0, axes_w, x0)?;
    let dx1 = pixel_to_data_x(x_scale, view.x_min, view.x_max, axes_x0, axes_w, x1)?;
    let dy0 = pixel_to_data_y(y_scale, view.y_min, view.y_max, axes_y0, axes_h, y0)?;
    let dy1 = pixel_to_data_y(y_scale, view.y_min, view.y_max, axes_y0, axes_h, y1)?;
    let (x_min, x_max) = box_limits(x_scale, dx0, dx1)?;
    let (y_min, y_max) = box_limits(y_scale, dy0, dy1)?;
    Ok(PanelView {
        x_min,
        x_max,
        y_min,
        y_max,
    })
}

/// Rotate a 3D camera from mouse drag (`dx`/`dy` in pixels).
pub fn rotate_3d(view: Axes3DView, dx: f64, dy: f64, sensitivity: f64) -> Axes3DView {
    let mut out = view;
    out.azim += dx * sensitivity;
    out.elev = (out.elev - dy * sensitivity).clamp(-90.0, 90.0);
    out
}

/// Zoom a 3D data box about its center (`factor` > 1 zooms in).
pub fn zoom_3d(view: Axes3DView, factor: f64) -> Result<Axes3DView> {
    if !(factor.is_finite() && factor > 0.0) {
        return Ok(view);
    }
    let (x_min, x_max) = zoom_range_linear(view.x_min, view.x_max, factor)?;
    let (y_min, y_max) = zoom_range_linear(view.y_min, view.y_max, factor)?;
    let (z_min, z_max) = zoom_range_linear(view.z_min, view.z_max, factor)?;
    Ok(Axes3DView {
        elev: view.elev,
        azim: view.azim,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    })
}

fn zoom_range_linear(min: f64, max: f64, factor: f64) -> Result<(f64, f64)> {
    let mid = 0.5 * (min + max);
    let half = 0.5 * (max - min) / factor;
    if !half.is_finite() || half <= 0.0 {
        return Err(PlotError::invalid_range(min, max));
    }
    Ok((mid - half, mid + half))
}

fn sanitize_limits(scale_type: ScaleType, min: f64, max: f64) -> Result<(f64, f64)> {
    if !min.is_finite() || !max.is_finite() {
        return Err(PlotError::InvalidRange {
            min,
            max,
            message: "interactive limits must be finite",
            suggestion: "reset the view with Home, or zoom out",
        });
    }
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    if lo >= hi {
        return Err(PlotError::invalid_range(lo, hi));
    }
    if matches!(scale_type, ScaleType::Log) && (lo <= 0.0 || hi <= 0.0) {
        return Err(PlotError::log_non_positive(lo.min(hi)));
    }
    // Ensure the scale builder accepts the domain.
    ScaleKind::build(scale_type, lo, hi)?;
    Ok((lo, hi))
}

/// Scroll wheel steps → zoom factor (>1 zooms in).
pub fn wheel_zoom_factor(scroll_y: f64) -> f64 {
    // One "notch" ≈ zoom in/out by 1.2× (matplotlib-like).
    if scroll_y > 0.0 {
        1.2_f64.powf(scroll_y.min(5.0))
    } else if scroll_y < 0.0 {
        1.0 / 1.2_f64.powf((-scroll_y).min(5.0))
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::PanelView;

    #[test]
    fn pan_linear_shifts_domain() {
        let (lo, hi) = pan_limits(ScaleType::Linear, 0.0, 10.0, 50.0, 100.0).unwrap();
        // unit_delta = 0.5 → new domain [-5, 5]
        assert!((lo - (-5.0)).abs() < 1e-9);
        assert!((hi - 5.0).abs() < 1e-9);
    }

    #[test]
    fn zoom_linear_about_center() {
        let (lo, hi) = zoom_limits(ScaleType::Linear, 0.0, 10.0, 0.5, 2.0).unwrap();
        assert!((lo - 2.5).abs() < 1e-9);
        assert!((hi - 7.5).abs() < 1e-9);
    }

    #[test]
    fn zoom_log_stays_positive() {
        let (lo, hi) = zoom_limits(ScaleType::Log, 1.0, 100.0, 0.5, 2.0).unwrap();
        assert!(lo > 0.0 && hi > lo);
        // Midpoint in log space is 10; zoom-in should tighten around it.
        assert!(lo > 1.0 && hi < 100.0);
    }

    #[test]
    fn pan_log_stays_positive() {
        // Extreme pan stays in log-legal positive domain via denormalize.
        let (lo, hi) = pan_limits(ScaleType::Log, 1.0, 10.0, 10_000.0, 100.0).unwrap();
        assert!(lo > 0.0 && hi > lo);
    }

    #[test]
    fn history_back_forward_home() {
        let home = ViewSnapshot {
            panels: vec![PanelView {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            }],
            axes3d: None,
        };
        let mut h = ViewHistory::new(home.clone());
        h.push(ViewSnapshot {
            panels: vec![PanelView {
                x_min: 1.0,
                x_max: 2.0,
                y_min: 0.0,
                y_max: 1.0,
            }],
            axes3d: None,
        });
        h.push(ViewSnapshot {
            panels: vec![PanelView {
                x_min: 2.0,
                x_max: 3.0,
                y_min: 0.0,
                y_max: 1.0,
            }],
            axes3d: None,
        });
        assert!(h.can_back());
        let b = h.back().unwrap();
        assert!((b.panels[0].x_min - 1.0).abs() < 1e-12);
        let f = h.forward().unwrap();
        assert!((f.panels[0].x_min - 2.0).abs() < 1e-12);
        let home_v = h.go_home();
        assert_eq!(home_v, home);
        assert!(!h.can_back());
    }

    #[test]
    fn rotate_3d_clamps_elev() {
        let v = Axes3DView {
            elev: 80.0,
            azim: -60.0,
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
            z_min: 0.0,
            z_max: 1.0,
        };
        let out = rotate_3d(v, 10.0, -50.0, 1.0);
        assert!((out.elev - 90.0).abs() < 1e-12);
        assert!((out.azim - (-50.0)).abs() < 1e-12);
    }

    #[test]
    fn zoom_3d_shrinks_box() {
        let v = Axes3DView {
            elev: 30.0,
            azim: -60.0,
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
            z_min: 0.0,
            z_max: 10.0,
        };
        let out = zoom_3d(v, 2.0).unwrap();
        assert!((out.x_min - 2.5).abs() < 1e-9);
        assert!((out.x_max - 7.5).abs() < 1e-9);
    }

    #[test]
    fn box_zoom_maps_screen_corners() {
        let view = PanelView {
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        };
        // Axes box [0,100]×[0,100]; select lower-left quarter in screen coords
        // (y grows downward → data y upper half).
        let out = box_zoom_panel(
            view,
            ScaleType::Linear,
            ScaleType::Linear,
            0.0,
            50.0,
            50.0,
            100.0,
            0.0,
            0.0,
            100.0,
            100.0,
        )
        .unwrap();
        assert!((out.x_min - 0.0).abs() < 1e-9);
        assert!((out.x_max - 5.0).abs() < 1e-9);
        assert!((out.y_min - 0.0).abs() < 1e-9);
        assert!((out.y_max - 5.0).abs() < 1e-9);
    }

    #[test]
    fn wheel_factor_zooms_in_on_positive_scroll() {
        assert!(wheel_zoom_factor(1.0) > 1.0);
        assert!(wheel_zoom_factor(-1.0) < 1.0);
        assert!((wheel_zoom_factor(0.0) - 1.0).abs() < 1e-12);
    }
}
