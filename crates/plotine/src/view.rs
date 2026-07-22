//! Interactive view state: capture / apply axis limits and 3D camera.

use plotine_core::{Rect, Result, Size};
use plotine_render::Renderer;

use crate::axes::Axes;
use crate::axes3d::Axes3D;
use crate::figure::Figure;
use crate::layout::Layout;

/// Per-panel 2D data limits for interactive navigation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelView {
    /// X-axis lower bound.
    pub x_min: f64,
    /// X-axis upper bound.
    pub x_max: f64,
    /// Y-axis lower bound.
    pub y_min: f64,
    /// Y-axis upper bound.
    pub y_max: f64,
}

/// 3D camera + data-box limits for interactive navigation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axes3DView {
    /// Elevation in degrees.
    pub elev: f64,
    /// Azimuth in degrees.
    pub azim: f64,
    /// X-axis lower bound.
    pub x_min: f64,
    /// X-axis upper bound.
    pub x_max: f64,
    /// Y-axis lower bound.
    pub y_min: f64,
    /// Y-axis upper bound.
    pub y_max: f64,
    /// Z-axis lower bound.
    pub z_min: f64,
    /// Z-axis upper bound.
    pub z_max: f64,
}

/// Snapshot of all interactive view parameters on a [`Figure`].
#[derive(Debug, Clone, PartialEq)]
pub struct ViewSnapshot {
    /// One entry per 2D panel (empty when the figure is 3D-only).
    pub panels: Vec<PanelView>,
    /// 3D view when the figure uses [`Figure::axes3d`](crate::figure::Figure::axes3d).
    pub axes3d: Option<Axes3DView>,
}

impl PanelView {
    pub(crate) fn from_axes(ax: &Axes) -> Self {
        Self {
            x_min: ax.x_min,
            x_max: ax.x_max,
            y_min: ax.y_min,
            y_max: ax.y_max,
        }
    }

    pub(crate) fn apply_to(self, ax: &mut Axes) {
        ax.x_range(self.x_min, self.x_max);
        ax.y_range(self.y_min, self.y_max);
    }
}

impl Axes3DView {
    pub(crate) fn from_axes3d(ax: &Axes3D) -> Self {
        Self {
            elev: ax.camera.elev,
            azim: ax.camera.azim,
            x_min: ax.x_min,
            x_max: ax.x_max,
            y_min: ax.y_min,
            y_max: ax.y_max,
            z_min: ax.z_min,
            z_max: ax.z_max,
        }
    }

    pub(crate) fn apply_to(self, ax: &mut Axes3D) {
        ax.elev(self.elev).azim(self.azim);
        ax.x_range(self.x_min, self.x_max);
        ax.y_range(self.y_min, self.y_max);
        ax.z_range(self.z_min, self.z_max);
    }
}

impl Figure {
    /// Capture current 2D limits / 3D camera for Home / history navigation.
    pub fn capture_view(&self) -> ViewSnapshot {
        let panels = self
            .panels
            .iter()
            .map(|p| PanelView::from_axes(&p.axes))
            .collect();
        let axes3d = self.axes3d.as_ref().map(Axes3DView::from_axes3d);
        ViewSnapshot { panels, axes3d }
    }

    /// Restore a previously captured [`ViewSnapshot`].
    ///
    /// Sets manual limits so auto-expand does not undo the interactive view.
    pub fn apply_view(&mut self, view: &ViewSnapshot) {
        for (panel, pv) in self.panels.iter_mut().zip(view.panels.iter()) {
            pv.apply_to(&mut panel.axes);
        }
        if let (Some(ax), Some(v)) = (self.axes3d.as_mut(), view.axes3d.as_ref()) {
            v.apply_to(ax);
        }
    }

    /// Compute axes-box rectangles (figure pixels) for each 2D panel.
    ///
    /// Used for hit-testing during interactive pan/zoom. Requires a renderer for
    /// text measurement (same path as [`Figure::draw`](Self::draw)).
    pub fn panel_axes_rects(&self, renderer: &mut dyn Renderer) -> Result<Vec<Rect>> {
        if self.axes3d.is_some() {
            let (pw, ph) = self.pixel_size();
            return Ok(vec![Rect::new(0.0, 0.0, pw as f64, ph as f64)]);
        }
        if self.panels.is_empty() {
            return Err(plotine_core::PlotError::empty_figure());
        }
        for panel in &self.panels {
            panel.axes.validate()?;
        }
        let (pw, ph) = self.pixel_size();
        let figure_size = Size::new(pw as f64, ph as f64);
        let panel_refs: Vec<(usize, usize, usize, usize, &Axes)> = self
            .panels
            .iter()
            .map(|p| (p.row, p.col, p.rowspan, p.colspan, &p.axes))
            .collect();
        let (grid, insets) = crate::layout::tight_layout_for_grid(
            self.grid,
            &panel_refs,
            figure_size,
            &self.theme,
            renderer,
            self.dpi,
        );
        let mut out = Vec::with_capacity(self.panels.len());
        for (panel, panel_insets) in self.panels.iter().zip(insets) {
            let cell = grid.span_rect(
                figure_size,
                panel.row,
                panel.col,
                panel.rowspan,
                panel.colspan,
            );
            let layout = Layout::from_insets(cell, panel_insets);
            out.push(layout.axes);
        }
        Ok(out)
    }

    /// Number of 2D panels (0 when using [`axes3d`](Self::axes3d)).
    pub fn panel_count(&self) -> usize {
        self.panels.len()
    }

    /// True when this figure hosts a 3D axes instead of 2D panels.
    pub fn is_3d(&self) -> bool {
        self.axes3d.is_some()
    }

    /// Mutable access to the 2D axes panel at `index` (for animation updates).
    pub fn axes_at_mut(&mut self, index: usize) -> Option<&mut Axes> {
        self.panels.get_mut(index).map(|p| &mut p.axes)
    }

    /// Scale types for panel `index` (used by interactive navigation).
    #[cfg(feature = "gui")]
    pub(crate) fn panel_scale_types(
        &self,
        index: usize,
    ) -> Option<(plotine_core::ScaleType, plotine_core::ScaleType)> {
        self.panels
            .get(index)
            .map(|p| (p.axes.x_scale_type, p.axes.y_scale_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::Figure;

    #[test]
    fn capture_apply_roundtrip_2d() {
        let mut fig = Figure::new().axes(|ax| {
            ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
            ax.x_range(0.0, 2.0).y_range(-1.0, 1.0);
        });
        let home = fig.capture_view();
        assert_eq!(home.panels.len(), 1);
        assert!((home.panels[0].x_max - 2.0).abs() < 1e-12);

        let zoomed = ViewSnapshot {
            panels: vec![PanelView {
                x_min: 0.5,
                x_max: 1.5,
                y_min: -0.5,
                y_max: 0.5,
            }],
            axes3d: None,
        };
        fig.apply_view(&zoomed);
        let cur = fig.capture_view();
        assert!((cur.panels[0].x_min - 0.5).abs() < 1e-12);
        assert!((cur.panels[0].y_max - 0.5).abs() < 1e-12);

        fig.apply_view(&home);
        let back = fig.capture_view();
        assert_eq!(back, home);
    }

    #[test]
    fn capture_apply_roundtrip_3d() {
        let mut fig = Figure::new().axes3d(|ax| {
            ax.plot3d([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]);
            ax.elev(25.0).azim(-45.0);
            ax.x_range(0.0, 1.0).y_range(0.0, 1.0).z_range(0.0, 1.0);
        });
        let home = fig.capture_view();
        assert!(fig.is_3d());
        let mut rotated = home.clone();
        if let Some(ref mut v) = rotated.axes3d {
            v.elev = 40.0;
            v.azim = -30.0;
        }
        fig.apply_view(&rotated);
        let cur = fig.capture_view();
        assert!((cur.axes3d.unwrap().elev - 40.0).abs() < 1e-12);
        fig.apply_view(&home);
        assert_eq!(fig.capture_view(), home);
    }
}
