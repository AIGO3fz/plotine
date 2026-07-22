//! Axes table geometry (matplotlib `ax.table`).

use plotine_core::Rect;

/// Where to place the table relative to the axes box (axes fraction).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TableLoc {
    /// Upper-right corner (default).
    #[default]
    UpperRight,
    /// Upper-left corner.
    UpperLeft,
    /// Lower-right corner.
    LowerRight,
    /// Lower-left corner.
    LowerLeft,
    /// Explicit axes-fraction bbox `[x0, y0, width, height]` (origin bottom-left).
    Bbox([f64; 4]),
}

/// One laid-out cell rectangle in pixel space.
#[derive(Debug, Clone)]
pub struct TableCellGeom {
    /// Pixel rectangle for the cell.
    pub rect: Rect,
    /// Cell text.
    pub text: String,
    /// True for header row / row-label column.
    pub header: bool,
}

/// Compute pixel geometry for a table overlaid on `axes` (pixel rect).
///
/// `col_widths` / `row_heights` are in pixels (already measured + padded).
pub fn table_cell_geoms(
    axes: Rect,
    loc: TableLoc,
    col_widths: &[f64],
    row_heights: &[f64],
    cells: &[Vec<String>],
    has_col_labels: bool,
    has_row_labels: bool,
) -> Vec<TableCellGeom> {
    let ncols = col_widths.len();
    let nrows = row_heights.len();
    if ncols == 0 || nrows == 0 {
        return Vec::new();
    }
    let table_w: f64 = col_widths.iter().sum();
    let table_h: f64 = row_heights.iter().sum();
    let (x0, y0) = match loc {
        TableLoc::UpperRight => (axes.x1 - table_w, axes.y0),
        TableLoc::UpperLeft => (axes.x0, axes.y0),
        TableLoc::LowerRight => (axes.x1 - table_w, axes.y1 - table_h),
        TableLoc::LowerLeft => (axes.x0, axes.y1 - table_h),
        TableLoc::Bbox([fx, fy, fw, fh]) => {
            let x = axes.x0 + fx.clamp(0.0, 1.0) * axes.width();
            // axes fraction y is bottom-origin; pixel y is top-origin.
            let y_bottom = axes.y1 - fy.clamp(0.0, 1.0) * axes.height();
            let w = fw.clamp(0.0, 1.0) * axes.width();
            let h = fh.clamp(0.0, 1.0) * axes.height();
            // Stretch cells into bbox if provided; otherwise pack at lower-left of bbox.
            let _ = (w, h);
            (x, y_bottom - table_h)
        }
    };

    let mut out = Vec::with_capacity(nrows * ncols);
    let mut y = y0;
    for (r, rh) in row_heights.iter().enumerate() {
        let mut x = x0;
        for (c, cw) in col_widths.iter().enumerate() {
            let text = cells
                .get(r)
                .and_then(|row| row.get(c))
                .cloned()
                .unwrap_or_default();
            let header = (has_col_labels && r == 0) || (has_row_labels && c == 0);
            out.push(TableCellGeom {
                rect: Rect::new(x, y, x + cw, y + rh),
                text,
                header,
            });
            x += cw;
        }
        y += rh;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_right_packs_inside_axes() {
        let axes = Rect::new(0.0, 0.0, 100.0, 80.0);
        let cells = vec![vec!["A".into(), "B".into()], vec!["1".into(), "2".into()]];
        let geoms = table_cell_geoms(
            axes,
            TableLoc::UpperRight,
            &[20.0, 30.0],
            &[12.0, 12.0],
            &cells,
            true,
            false,
        );
        assert_eq!(geoms.len(), 4);
        assert!((geoms[0].rect.x0 - 50.0).abs() < 1e-9);
        assert!(geoms[0].header);
        assert!(!geoms[2].header);
    }
}
