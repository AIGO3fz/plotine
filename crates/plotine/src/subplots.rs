//! Subplot grid builder used by [`Figure::subplots`](crate::Figure::subplots).

use crate::axes::Axes;
use crate::layout::GridSpec;

/// One axes panel placed at a grid location (optionally spanning cells).
#[derive(Debug, Clone)]
pub(crate) struct Panel {
    pub row: usize,
    pub col: usize,
    pub rowspan: usize,
    pub colspan: usize,
    pub axes: Axes,
}

/// Mutable builder handed to the [`Figure::subplots`](crate::Figure::subplots) closure.
///
/// Use [`at`](Self::at) / [`at_span`](Self::at_span) / [`flat`](Self::flat) to
/// configure panels and [`hspace`](Self::hspace) / [`wspace`](Self::wspace) to
/// tune gaps.
pub struct SubplotGrid<'a> {
    pub(crate) spec: GridSpec,
    pub(crate) panels: &'a mut Vec<Panel>,
    pub(crate) sharex: bool,
    pub(crate) sharey: bool,
}

impl SubplotGrid<'_> {
    /// Configure the axes at `(row, col)` (0-based, row-major; 1×1 cell).
    pub fn at<F>(&mut self, row: usize, col: usize, f: F) -> &mut Self
    where
        F: FnOnce(&mut Axes),
    {
        self.at_span(row, col, 1, 1, f)
    }

    /// Configure axes at `(row, col)` spanning `rowspan × colspan` cells.
    ///
    /// Spans are clamped to the grid. Overlapping spans are allowed but draw
    /// order follows insertion (later panels paint on top).
    ///
    /// ```
    /// use plotine::prelude::*;
    /// let png = Figure::new().size(5.0, 3.5).dpi(72.0).subplots(2, 2, |g| {
    ///     g.at_span(0, 0, 2, 1, |ax| {
    ///         ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.5]);
    ///         ax.title("tall");
    ///     });
    ///     g.at(0, 1, |ax| { ax.scatter([0.0, 1.0], [1.0, 0.0]); ax.title("A"); });
    ///     g.at(1, 1, |ax| { ax.bar([1.0, 2.0], [3.0, 2.0]); ax.title("B"); });
    /// }).render_png().unwrap();
    /// assert!(!png.is_empty());
    /// ```
    pub fn at_span<F>(
        &mut self,
        row: usize,
        col: usize,
        rowspan: usize,
        colspan: usize,
        f: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Axes),
    {
        let row = row.min(self.spec.nrows.saturating_sub(1));
        let col = col.min(self.spec.ncols.saturating_sub(1));
        let rowspan = rowspan
            .max(1)
            .min(self.spec.nrows.saturating_sub(row).max(1));
        let colspan = colspan
            .max(1)
            .min(self.spec.ncols.saturating_sub(col).max(1));
        let mut ax = Axes::new();
        f(&mut ax);
        ax.finalize_artist_limits();
        // Replace existing panel at same origin if reconfigured.
        if let Some(existing) = self
            .panels
            .iter_mut()
            .find(|p| p.row == row && p.col == col)
        {
            existing.rowspan = rowspan;
            existing.colspan = colspan;
            existing.axes = ax;
        } else {
            self.panels.push(Panel {
                row,
                col,
                rowspan,
                colspan,
                axes: ax,
            });
        }
        self
    }

    /// Configure axes by flat 0-based index in row-major order.
    pub fn flat<F>(&mut self, index: usize, f: F) -> &mut Self
    where
        F: FnOnce(&mut Axes),
    {
        let ncols = self.spec.ncols.max(1);
        let row = index / ncols;
        let col = index % ncols;
        self.at(row, col, f)
    }

    /// Vertical gap between rows as a fraction of average cell height.
    pub fn hspace(&mut self, v: f64) -> &mut Self {
        self.spec.hspace = v.clamp(0.0, 1.0);
        self
    }

    /// Horizontal gap between columns as a fraction of average cell width.
    pub fn wspace(&mut self, v: f64) -> &mut Self {
        self.spec.wspace = v.clamp(0.0, 1.0);
        self
    }

    /// Share x-axis limits across all panels in each column (matplotlib `sharex=True`).
    ///
    /// When enabled, all panels in the same column use the union of their
    /// x-data ranges. Tick labels on non-bottom panels are hidden.
    pub fn sharex(&mut self, share: bool) -> &mut Self {
        self.sharex = share;
        self
    }

    /// Share y-axis limits across all panels in each row (matplotlib `sharey=True`).
    ///
    /// When enabled, all panels in the same row use the union of their
    /// y-data ranges. Tick labels on non-left panels are hidden.
    pub fn sharey(&mut self, share: bool) -> &mut Self {
        self.sharey = share;
        self
    }
}

/// Parse a mosaic layout string into `(name_char, row, col, rowspan, colspan)`.
///
/// Each unique ASCII letter becomes a named region spanning its rectangular extent.
/// `.` (dot) marks empty cells. Rows are separated by newlines or `;`.
///
/// # Example
/// ```
/// use plotine::subplots::parse_mosaic;
/// // "AAB;CDB" → A spans (0,0)-(0,1), B spans (0,2)-(1,2), C is (1,0), D is (1,1)
/// let regions = parse_mosaic("AAB;CDB");
/// assert_eq!(regions.len(), 4);
/// ```
pub fn parse_mosaic(layout: &str) -> Vec<(char, usize, usize, usize, usize)> {
    let rows: Vec<Vec<char>> = layout
        .split(['\n', ';'])
        .map(|row| row.chars().filter(|c| !c.is_whitespace()).collect())
        .filter(|row: &Vec<char>| !row.is_empty())
        .collect();

    if rows.is_empty() {
        return Vec::new();
    }

    let mut seen = std::collections::BTreeMap::<char, (usize, usize, usize, usize)>::new();
    for (r, row) in rows.iter().enumerate() {
        for (c, &ch) in row.iter().enumerate() {
            if ch == '.' {
                continue;
            }
            seen.entry(ch)
                .and_modify(|e| {
                    e.0 = e.0.min(r);
                    e.1 = e.1.min(c);
                    e.2 = e.2.max(r);
                    e.3 = e.3.max(c);
                })
                .or_insert((r, c, r, c));
        }
    }

    seen.into_iter()
        .map(|(ch, (r0, c0, r1, c1))| (ch, r0, c0, r1 - r0 + 1, c1 - c0 + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mosaic_basic() {
        let regions = parse_mosaic("AAB;CDB");
        assert_eq!(regions.len(), 4);
        // A: row 0, col 0, 1×2
        assert!(regions.contains(&('A', 0, 0, 1, 2)));
        // B: row 0, col 2, 2×1
        assert!(regions.contains(&('B', 0, 2, 2, 1)));
        // C: row 1, col 0, 1×1
        assert!(regions.contains(&('C', 1, 0, 1, 1)));
        // D: row 1, col 1, 1×1
        assert!(regions.contains(&('D', 1, 1, 1, 1)));
    }

    #[test]
    fn mosaic_with_dots() {
        let regions = parse_mosaic("A.;.B");
        assert_eq!(regions.len(), 2);
        assert!(regions.contains(&('A', 0, 0, 1, 1)));
        assert!(regions.contains(&('B', 1, 1, 1, 1)));
    }

    #[test]
    fn mosaic_newline_separator() {
        let regions = parse_mosaic("AB\nCC");
        assert_eq!(regions.len(), 3);
        assert!(regions.contains(&('A', 0, 0, 1, 1)));
        assert!(regions.contains(&('B', 0, 1, 1, 1)));
        assert!(regions.contains(&('C', 1, 0, 1, 2)));
    }

    #[test]
    fn mosaic_empty_returns_empty() {
        assert!(parse_mosaic("").is_empty());
    }
}
