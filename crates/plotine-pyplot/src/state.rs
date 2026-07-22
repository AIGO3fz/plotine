//! Thread-local current figure / axes (matplotlib `gcf` / `gca` subset).

use std::cell::RefCell;
use std::collections::HashMap;

use plotine::{Axes, Figure, PlotError, Result};

thread_local! {
    static STATE: RefCell<PyplotState> = RefCell::new(PyplotState::new());
}

struct PyplotState {
    figures: HashMap<usize, Figure>,
    current: usize,
    current_axes: usize,
    next_num: usize,
}

impl PyplotState {
    fn new() -> Self {
        Self {
            figures: HashMap::new(),
            current: 0,
            current_axes: 0,
            next_num: 1,
        }
    }

    fn ensure_current(&mut self) {
        if self.current == 0 || !self.figures.contains_key(&self.current) {
            let num = self.next_num;
            self.next_num += 1;
            self.figures.insert(num, Figure::with_empty_axes());
            self.current = num;
            self.current_axes = 0;
        } else if self
            .figures
            .get(&self.current)
            .is_some_and(|f| f.panel_count() == 0)
        {
            // Replace an empty shell with a 1×1 axes panel.
            self.figures.insert(self.current, Figure::with_empty_axes());
            self.current_axes = 0;
        }
    }
}

/// Create or select a figure by number (matplotlib `plt.figure(num)`).
///
/// Pass `None` to create a new figure and make it current.
pub fn figure(num: Option<usize>) -> usize {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        match num {
            Some(n) => {
                s.figures.entry(n).or_insert_with(Figure::with_empty_axes);
                s.current = n;
                if n >= s.next_num {
                    s.next_num = n + 1;
                }
                s.current_axes = 0;
                n
            }
            None => {
                let n = s.next_num;
                s.next_num += 1;
                s.figures.insert(n, Figure::with_empty_axes());
                s.current = n;
                s.current_axes = 0;
                n
            }
        }
    })
}

/// Clear the current figure (matplotlib `plt.clf`).
pub fn clf() {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.ensure_current();
        let cur = s.current;
        s.figures.insert(cur, Figure::with_empty_axes());
        s.current_axes = 0;
    });
}

/// Number of the current figure (0 if none yet — next mutating call creates one).
pub fn gcf_num() -> usize {
    STATE.with(|cell| cell.borrow().current)
}

/// Index of the current axes panel within the current figure.
pub fn gca_index() -> usize {
    STATE.with(|cell| cell.borrow().current_axes)
}

/// Set the current axes panel by 0-based index (matplotlib `plt.sca` subset).
pub fn sca(index: usize) -> Result<()> {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.ensure_current();
        let n = s
            .figures
            .get(&s.current)
            .map(|f| f.panel_count())
            .unwrap_or(0);
        if index >= n {
            return Err(PlotError::render(format!(
                "axes index {index} out of range (figure has {n} panels)"
            )));
        }
        s.current_axes = index;
        Ok(())
    })
}

/// Replace the current figure with `fig` and select axes 0.
pub(crate) fn set_current_figure(fig: Figure) {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.ensure_current();
        let cur = s.current;
        s.figures.insert(cur, fig);
        s.current_axes = 0;
    });
}

/// Borrow the current figure mutably and run `f`.
pub(crate) fn with_gcf_mut<R>(f: impl FnOnce(&mut Figure) -> Result<R>) -> Result<R> {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.ensure_current();
        let cur = s.current;
        let fig = s
            .figures
            .get_mut(&cur)
            .ok_or_else(|| PlotError::render("no current figure"))?;
        f(fig)
    })
}

/// Borrow the current figure immutably and run `f`.
pub(crate) fn with_gcf<R>(f: impl FnOnce(&Figure) -> Result<R>) -> Result<R> {
    STATE.with(|cell| {
        {
            let mut s = cell.borrow_mut();
            s.ensure_current();
        }
        let s = cell.borrow();
        let fig = s
            .figures
            .get(&s.current)
            .ok_or_else(|| PlotError::render("no current figure"))?;
        f(fig)
    })
}

/// Borrow the current axes mutably and run `f`.
pub(crate) fn with_gca_mut<R>(f: impl FnOnce(&mut Axes) -> Result<R>) -> Result<R> {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.ensure_current();
        let cur = s.current;
        let ax_i = s.current_axes;
        let fig = s
            .figures
            .get_mut(&cur)
            .ok_or_else(|| PlotError::render("no current figure"))?;
        let ax = fig.axes_at_mut(ax_i).ok_or_else(|| {
            PlotError::render(format!("no axes at index {ax_i} on current figure"))
        })?;
        f(ax)
    })
}

/// Take ownership of the current figure (leaving an empty replacement).
#[cfg(feature = "gui")]
pub(crate) fn take_gcf() -> Result<Figure> {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.ensure_current();
        let cur = s.current;
        let fig = s
            .figures
            .remove(&cur)
            .ok_or_else(|| PlotError::render("no current figure"))?;
        s.figures.insert(cur, Figure::with_empty_axes());
        s.current_axes = 0;
        Ok(fig)
    })
}
