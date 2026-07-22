//! eframe application hosting an interactive [`Figure`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect as EguiRect, TextureHandle, Vec2};
use plotine_core::{Point, Rect, Result};

use crate::figure::Figure;
use crate::nav::{
    box_zoom_panel, pan_panel, rotate_3d, wheel_zoom_factor, zoom_3d, zoom_panel, NavMode,
    ViewHistory,
};
use crate::view::ViewSnapshot;

use super::{gui_error, WidgetCallback};

const ROTATE_SENSITIVITY: f64 = 0.4;
const MIN_BOX_PX: f32 = 4.0;

pub(super) fn run(
    figure: Figure,
    widgets: Option<WidgetCallback>,
    close_flag: Option<Arc<AtomicBool>>,
) -> Result<()> {
    let (fw, fh) = figure.pixel_size();
    let side = if widgets.is_some() { 200.0 } else { 0.0 };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([
                (fw as f32 + 16.0 + side).max(480.0),
                (fh as f32 + 48.0).max(360.0),
            ])
            .with_title("plotine"),
        ..Default::default()
    };

    let app = PlotApp::new(figure, widgets, close_flag)?;
    eframe::run_native("plotine", options, Box::new(|_cc| Ok(Box::new(app))))
        .map_err(|e| gui_error(format!("GUI event loop failed: {e}")))
}

struct PlotApp {
    figure: Figure,
    history: ViewHistory,
    mode: NavMode,
    dirty: bool,
    texture: Option<TextureHandle>,
    /// Cached axes rects in figure pixel space.
    axes_rects: Vec<Rect>,
    active_panel: usize,
    /// Drag state for pan / rotate / box-zoom.
    drag: Option<DragState>,
    /// Rubber-band box in figure pixels (zoom mode).
    box_start: Option<Pos2>,
    box_end: Option<Pos2>,
    status: String,
    save_path: String,
    widgets: Option<WidgetCallback>,
    close_flag: Option<Arc<AtomicBool>>,
}

#[derive(Clone)]
struct DragState {
    last_pos: Pos2,
    panel: usize,
    start_view: ViewSnapshot,
    /// True when 3D rotate; false when 2D pan.
    is_3d: bool,
}

impl PlotApp {
    fn new(
        figure: Figure,
        widgets: Option<WidgetCallback>,
        close_flag: Option<Arc<AtomicBool>>,
    ) -> Result<Self> {
        let home = figure.capture_view();
        let mut app = Self {
            figure,
            history: ViewHistory::new(home),
            mode: NavMode::Pan,
            dirty: true,
            texture: None,
            axes_rects: Vec::new(),
            active_panel: 0,
            drag: None,
            box_start: None,
            box_end: None,
            status: String::new(),
            save_path: "plotine_export.png".into(),
            widgets,
            close_flag,
        };
        app.refresh_layout()?;
        Ok(app)
    }

    fn refresh_layout(&mut self) -> Result<()> {
        let (w, h) = self.figure.pixel_size();
        let mut renderer = plotine_backend_skia::SkiaRenderer::new(w.max(1), h.max(1))?;
        self.axes_rects = self.figure.panel_axes_rects(&mut renderer)?;
        Ok(())
    }

    fn rerender(&mut self, ctx: &egui::Context) {
        if !self.dirty && self.texture.is_some() {
            return;
        }
        match self.figure.render_rgba() {
            Ok((w, h, rgba)) => {
                let image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                match &mut self.texture {
                    Some(tex) => tex.set(image, Default::default()),
                    None => {
                        self.texture =
                            Some(ctx.load_texture("plotine-figure", image, Default::default()));
                    }
                }
                self.dirty = false;
                if let Err(e) = self.refresh_layout() {
                    self.status = format!("layout: {e}");
                }
            }
            Err(e) => {
                self.status = format!("render: {e}");
                self.dirty = false;
            }
        }
    }

    fn apply_snapshot(&mut self, snap: ViewSnapshot) {
        self.figure.apply_view(&snap);
        self.dirty = true;
    }

    fn push_history(&mut self) {
        let snap = self.figure.capture_view();
        self.history.push(snap);
    }

    fn hit_panel(&self, fig_pos: Pos2) -> Option<usize> {
        let p = Point::new(fig_pos.x as f64, fig_pos.y as f64);
        for (i, r) in self.axes_rects.iter().enumerate() {
            if r.contains(p) {
                return Some(i);
            }
        }
        None
    }

    /// Map pointer position inside the painted image rect → figure pixels.
    fn to_figure_pos(&self, pointer: Pos2, image_rect: EguiRect) -> Option<Pos2> {
        let (fw, fh) = self.figure.pixel_size();
        if fw == 0 || fh == 0 || image_rect.width() <= 0.0 || image_rect.height() <= 0.0 {
            return None;
        }
        let u = ((pointer.x - image_rect.min.x) / image_rect.width()).clamp(0.0, 1.0);
        let v = ((pointer.y - image_rect.min.y) / image_rect.height()).clamp(0.0, 1.0);
        Some(Pos2::new(u * fw as f32, v * fh as f32))
    }

    fn handle_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .button("Home")
                .on_hover_text("Reset view (h/r)")
                .clicked()
            {
                let snap = self.history.go_home();
                self.apply_snapshot(snap);
                self.status = "Home".into();
            }
            ui.add_enabled_ui(self.history.can_back(), |ui| {
                if ui
                    .button("Back")
                    .on_hover_text("Previous view (←)")
                    .clicked()
                {
                    if let Some(snap) = self.history.back() {
                        self.apply_snapshot(snap);
                    }
                }
            });
            ui.add_enabled_ui(self.history.can_forward(), |ui| {
                if ui
                    .button("Forward")
                    .on_hover_text("Next view (→)")
                    .clicked()
                {
                    if let Some(snap) = self.history.forward() {
                        self.apply_snapshot(snap);
                    }
                }
            });
            ui.separator();
            ui.selectable_value(&mut self.mode, NavMode::Pan, "Pan")
                .on_hover_text("Pan mode (p)");
            ui.selectable_value(&mut self.mode, NavMode::Zoom, "Zoom")
                .on_hover_text("Box-zoom mode (o)");
            ui.separator();
            ui.label("Save:");
            ui.add(
                egui::TextEdit::singleline(&mut self.save_path)
                    .desired_width(180.0)
                    .hint_text("out.png"),
            );
            if ui
                .button("Save")
                .on_hover_text("Export PNG/SVG/PDF (s)")
                .clicked()
            {
                self.do_save();
            }
            if ui.button("Browse…").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .add_filter("SVG", &["svg"])
                    .add_filter("PDF", &["pdf"])
                    .set_file_name(&self.save_path)
                    .save_file()
                {
                    self.save_path = path.to_string_lossy().into_owned();
                    self.do_save();
                }
            }
        });
        if !self.status.is_empty() {
            ui.label(&self.status);
        }
    }

    fn do_save(&mut self) {
        match self.figure.save(&self.save_path) {
            Ok(()) => self.status = format!("Saved {}", self.save_path),
            Err(e) => self.status = format!("Save failed: {e}"),
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let mut quit = false;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Q) || i.key_pressed(egui::Key::Escape) {
                quit = true;
            }
            if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::R) {
                let snap = self.history.go_home();
                self.apply_snapshot(snap);
                self.status = "Home".into();
            }
            if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::C) {
                if let Some(snap) = self.history.back() {
                    self.apply_snapshot(snap);
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::V) {
                if let Some(snap) = self.history.forward() {
                    self.apply_snapshot(snap);
                }
            }
            if i.key_pressed(egui::Key::P) {
                self.mode = NavMode::Pan;
            }
            if i.key_pressed(egui::Key::O) {
                self.mode = NavMode::Zoom;
            }
            if i.key_pressed(egui::Key::S) {
                self.do_save();
            }
        });
        if quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn on_scroll(&mut self, fig_pos: Pos2, scroll_y: f64) {
        let factor = wheel_zoom_factor(scroll_y);
        if (factor - 1.0).abs() < 1e-12 {
            return;
        }
        if self.figure.is_3d() {
            let Some(v) = self.figure.capture_view().axes3d else {
                return;
            };
            match zoom_3d(v, factor) {
                Ok(nv) => {
                    let mut snap = self.figure.capture_view();
                    snap.axes3d = Some(nv);
                    self.apply_snapshot(snap);
                    self.push_history();
                }
                Err(e) => self.status = format!("{e}"),
            }
            return;
        }
        let panel = self.hit_panel(fig_pos).unwrap_or(self.active_panel);
        self.active_panel = panel;
        let Some(rect) = self.axes_rects.get(panel).copied() else {
            return;
        };
        let Some((xs, ys)) = self.figure.panel_scale_types(panel) else {
            return;
        };
        let mut snap = self.figure.capture_view();
        let Some(pv) = snap.panels.get(panel).copied() else {
            return;
        };
        match zoom_panel(
            pv,
            xs,
            ys,
            fig_pos.x as f64,
            fig_pos.y as f64,
            rect.x0,
            rect.y0,
            rect.width(),
            rect.height(),
            factor,
        ) {
            Ok(nv) => {
                snap.panels[panel] = nv;
                self.apply_snapshot(snap);
                self.push_history();
            }
            Err(e) => self.status = format!("{e}"),
        }
    }

    fn begin_drag(&mut self, fig_pos: Pos2) {
        let is_3d = self.figure.is_3d();
        let panel = if is_3d {
            0
        } else {
            self.hit_panel(fig_pos).unwrap_or(self.active_panel)
        };
        self.active_panel = panel;
        if self.mode == NavMode::Zoom && !is_3d {
            self.box_start = Some(fig_pos);
            self.box_end = Some(fig_pos);
            self.drag = None;
            return;
        }
        self.drag = Some(DragState {
            last_pos: fig_pos,
            panel,
            start_view: self.figure.capture_view(),
            is_3d,
        });
    }

    fn continue_drag(&mut self, fig_pos: Pos2) {
        if self.mode == NavMode::Zoom && self.box_start.is_some() && !self.figure.is_3d() {
            self.box_end = Some(fig_pos);
            return;
        }
        let Some(drag) = self.drag.clone() else {
            return;
        };
        let dx = (fig_pos.x - drag.last_pos.x) as f64;
        let dy = (fig_pos.y - drag.last_pos.y) as f64;
        if dx.abs() < 1e-6 && dy.abs() < 1e-6 {
            return;
        }
        if drag.is_3d {
            let Some(v) = self.figure.capture_view().axes3d else {
                return;
            };
            let nv = rotate_3d(v, dx, dy, ROTATE_SENSITIVITY);
            let mut snap = self.figure.capture_view();
            snap.axes3d = Some(nv);
            self.apply_snapshot(snap);
        } else {
            let Some(rect) = self.axes_rects.get(drag.panel).copied() else {
                return;
            };
            let Some((xs, ys)) = self.figure.panel_scale_types(drag.panel) else {
                return;
            };
            let mut snap = self.figure.capture_view();
            let Some(pv) = snap.panels.get(drag.panel).copied() else {
                return;
            };
            match pan_panel(pv, xs, ys, dx, dy, rect.width(), rect.height()) {
                Ok(nv) => {
                    snap.panels[drag.panel] = nv;
                    self.apply_snapshot(snap);
                }
                Err(e) => self.status = format!("{e}"),
            }
        }
        if let Some(d) = self.drag.as_mut() {
            d.last_pos = fig_pos;
        }
    }

    fn end_drag(&mut self) {
        if let (Some(start), Some(end)) = (self.box_start.take(), self.box_end.take()) {
            let w = (end.x - start.x).abs();
            let h = (end.y - start.y).abs();
            if w >= MIN_BOX_PX && h >= MIN_BOX_PX {
                let panel = self.hit_panel(start).unwrap_or(self.active_panel);
                self.active_panel = panel;
                if let Some(rect) = self.axes_rects.get(panel).copied() {
                    if let Some((xs, ys)) = self.figure.panel_scale_types(panel) {
                        let mut snap = self.figure.capture_view();
                        if let Some(pv) = snap.panels.get(panel).copied() {
                            match box_zoom_panel(
                                pv,
                                xs,
                                ys,
                                start.x as f64,
                                start.y as f64,
                                end.x as f64,
                                end.y as f64,
                                rect.x0,
                                rect.y0,
                                rect.width(),
                                rect.height(),
                            ) {
                                Ok(nv) => {
                                    snap.panels[panel] = nv;
                                    self.apply_snapshot(snap);
                                    self.push_history();
                                }
                                Err(e) => self.status = format!("{e}"),
                            }
                        }
                    }
                }
            }
            return;
        }
        if let Some(drag) = self.drag.take() {
            let current = self.figure.capture_view();
            if current != drag.start_view {
                self.history.push(current);
            }
        }
    }
}

impl eframe::App for PlotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self
            .close_flag
            .as_ref()
            .is_some_and(|f| f.load(Ordering::SeqCst))
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.handle_keys(ctx);
        self.rerender(ctx);

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.handle_toolbar(ui);
        });

        if self.widgets.is_some() {
            egui::SidePanel::right("widgets")
                .default_width(180.0)
                .show(ctx, |ui| {
                    let mut need_redraw = false;
                    if let Some(cb) = self.widgets.as_mut() {
                        need_redraw = cb(ui, &mut self.figure);
                    }
                    if need_redraw {
                        self.dirty = true;
                        self.history = ViewHistory::new(self.figure.capture_view());
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(tex) = self.texture.as_ref() else {
                ui.label("Rendering…");
                return;
            };
            let (fw, fh) = self.figure.pixel_size();
            let avail = ui.available_size();
            let scale = (avail.x / fw as f32).min(avail.y / fh as f32).max(0.05);
            let disp = Vec2::new(fw as f32 * scale, fh as f32 * scale);
            let (response, painter) = ui.allocate_painter(disp, egui::Sense::click_and_drag());
            let image_rect = response.rect;
            painter.image(
                tex.id(),
                image_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            // Rubber-band overlay in screen space.
            if let (Some(a), Some(b)) = (self.box_start, self.box_end) {
                let to_screen = |p: Pos2| {
                    egui::pos2(
                        image_rect.min.x + (p.x / fw as f32) * image_rect.width(),
                        image_rect.min.y + (p.y / fh as f32) * image_rect.height(),
                    )
                };
                let r = EguiRect::from_two_pos(to_screen(a), to_screen(b));
                painter.rect(
                    r,
                    0.0,
                    Color32::from_rgba_unmultiplied(30, 120, 200, 40),
                    egui::Stroke::new(1.0, Color32::from_rgb(30, 120, 200)),
                    egui::StrokeKind::Outside,
                );
            }

            if response.hovered() {
                // Prefer discrete wheel notches over smooth_scroll to avoid
                // flooding view history while the wheel is spinning.
                let scroll = ctx.input(|i| {
                    let raw = i.raw_scroll_delta.y;
                    if raw.abs() > 0.0 {
                        raw
                    } else {
                        i.smooth_scroll_delta.y
                    }
                });
                if scroll.abs() > 0.0 {
                    if let Some(pointer) = response
                        .interact_pointer_pos()
                        .or_else(|| ctx.input(|i| i.pointer.hover_pos()))
                    {
                        if let Some(fig_pos) = self.to_figure_pos(pointer, image_rect) {
                            // One notch ≈ |scroll|≈40–120 on most platforms.
                            let steps = (scroll as f64 / 40.0).clamp(-3.0, 3.0);
                            self.on_scroll(fig_pos, steps);
                        }
                    }
                }
            }

            if response.drag_started() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    if let Some(fig_pos) = self.to_figure_pos(pointer, image_rect) {
                        self.begin_drag(fig_pos);
                    }
                }
            }
            if response.dragged() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    if let Some(fig_pos) = self.to_figure_pos(pointer, image_rect) {
                        self.continue_drag(fig_pos);
                    }
                }
            }
            if response.drag_stopped() {
                self.end_drag();
            }
        });
    }
}
