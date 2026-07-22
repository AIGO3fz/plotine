//! Interactive figure window (`feature = "gui"`).
//!
//! Provides [`Figure::show`](crate::figure::Figure::show) — a blocking egui/winit
//! window with matplotlib-like pan/zoom, 3D rotation, view history, and export.
//! Also [`Figure::show_nonblocking`] (ion()-like) and [`Figure::show_with`] for
//! side-panel widgets (Slider / Button).

mod app;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use egui::Ui;
use plotine_core::{PlotError, Result};

use crate::figure::Figure;

/// Callback drawn in the GUI side panel. Return `true` to force a figure re-render.
pub type WidgetCallback = Box<dyn FnMut(&mut Ui, &mut Figure) -> bool + Send>;

/// Handle for a non-blocking GUI window (matplotlib `ion()` / `show(block=False)` subset).
pub struct ShowHandle {
    join: Option<JoinHandle<Result<()>>>,
    close: Arc<AtomicBool>,
}

impl ShowHandle {
    /// Request the window to close (non-blocking).
    pub fn close(&self) {
        self.close.store(true, Ordering::SeqCst);
    }

    /// Whether the GUI thread has finished.
    pub fn is_finished(&self) -> bool {
        self.join.as_ref().map(|j| j.is_finished()).unwrap_or(true)
    }

    /// Block until the window closes and return the GUI result.
    pub fn join(mut self) -> Result<()> {
        if let Some(handle) = self.join.take() {
            match handle.join() {
                Ok(r) => r,
                Err(_) => Err(gui_error("GUI thread panicked")),
            }
        } else {
            Ok(())
        }
    }
}

impl Drop for ShowHandle {
    fn drop(&mut self) {
        self.close.store(true, Ordering::SeqCst);
        // Detach: do not join on drop (avoids deadlock if dropped on GUI thread).
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

impl Figure {
    /// Open an interactive window and block until it is closed.
    ///
    /// Requires the `gui` feature. Aligns with matplotlib `plt.show(block=True)`:
    /// pan/zoom (2D), elev/azim drag (3D), Home/Back/Forward, and Save to
    /// PNG/SVG/PDF. Does not implement Configure Subplots, picking, or multi-toolkit backends.
    ///
    /// ```no_run
    /// # #[cfg(feature = "gui")]
    /// # {
    /// use plotine::prelude::*;
    /// Figure::new()
    ///     .axes(|ax| {
    ///         ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
    ///     })
    ///     .show()
    ///     .unwrap();
    /// # }
    /// ```
    pub fn show(self) -> Result<()> {
        app::run(self, None, None)
    }

    /// Open an interactive window with a side-panel widget callback (blocking).
    ///
    /// The callback receives `&mut egui::Ui` and `&mut Figure`. Return `true` when
    /// the figure should re-render (e.g. after changing line data).
    ///
    /// ```no_run
    /// # #[cfg(feature = "gui")]
    /// # {
    /// use plotine::prelude::*;
    /// let mut phase = 0.0_f64;
    /// Figure::new()
    ///     .axes(|ax| { ax.line([0.0, 1.0], [0.0, 1.0]); })
    ///     .show_with(move |ui, fig| {
    ///         let mut dirty = false;
    ///         ui.heading("Controls");
    ///         if ui.add(egui::Slider::new(&mut phase, 0.0..=6.28)).changed() {
    ///             dirty = true;
    ///         }
    ///         dirty
    ///     })
    ///     .unwrap();
    /// # }
    /// ```
    pub fn show_with<F>(self, widgets: F) -> Result<()>
    where
        F: FnMut(&mut Ui, &mut Figure) -> bool + Send + 'static,
    {
        app::run(self, Some(Box::new(widgets)), None)
    }

    /// Open an interactive window on a background thread (non-blocking).
    ///
    /// Aligns with matplotlib `plt.show(block=False)` / `ion()` **subset**: the
    /// caller regains control immediately and can [`ShowHandle::join`] or
    /// [`ShowHandle::close`] later. Still uses the egui backend only (no Qt/Tk).
    ///
    /// Note: some platforms prefer GUI on the main thread; if window creation
    /// fails, use blocking [`Self::show`] instead.
    pub fn show_nonblocking(self) -> Result<ShowHandle> {
        let close = Arc::new(AtomicBool::new(false));
        let close_flag = Arc::clone(&close);
        let join = thread::Builder::new()
            .name("plotine-gui".into())
            .spawn(move || app::run(self, None, Some(close_flag)))
            .map_err(|e| gui_error(format!("failed to spawn GUI thread: {e}")))?;
        Ok(ShowHandle {
            join: Some(join),
            close,
        })
    }
}

pub(crate) fn gui_error(message: impl Into<String>) -> PlotError {
    PlotError::Render {
        message: message.into(),
        suggestion: "ensure a display is available; GUI requires the `gui` feature and a windowing environment",
    }
}
