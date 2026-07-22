use plotine_core::{DataToPixel, Point};
use plotine_render::{TextAlign, TextBaseline};

use super::quiver::QuiverArrow;
use crate::artist::ArrowStyle;
use crate::mpl_policy::annotate as ann_policy;

/// Pixel-space text box used to keep the arrow from painting through the label
/// (matplotlib `Annotation.update_positions` → `patchA` + `relpos`).
#[derive(Debug, Clone, Copy)]
pub struct AnnotateTextBox {
    pub width: f64,
    pub height: f64,
    pub size_px: f64,
    pub align: TextAlign,
    pub baseline: TextBaseline,
    /// Padding around the ink box (matplotlib uses 4 pt).
    pub pad_px: f64,
}

/// Arrow from text anchor `xytext` toward the annotated point `xy` (data coords).
///
/// Matches matplotlib's default annotate orientation: tip at `xy`. Head size is
/// fixed in points via `mutation_scale_px` (FancyArrowPatch `mutation_scale`),
/// not a fraction of the shaft like quiver.
pub fn annotation_arrow(
    xy: (f64, f64),
    xytext: (f64, f64),
    transform: &DataToPixel,
    mutation_scale_px: f64,
) -> Option<QuiverArrow> {
    annotation_arrow_styled(
        xy,
        xytext,
        transform,
        mutation_scale_px,
        ArrowStyle::Triangle,
        ann_policy::LINEWIDTH_PT * mutation_scale_px / ann_policy::MUTATION_SCALE_PT,
        None,
    )
}

/// Arrow geometry for a chosen [`ArrowStyle`].
///
/// `linewidth_px` is the FancyArrowPatch stroke width in pixels (used for the
/// projected-cap pad on wedge heads). When `text` is set, the shaft starts at
/// the exit of the padded text box (mpl `patchA` clip), not at `xytext`.
pub fn annotation_arrow_styled(
    xy: (f64, f64),
    xytext: (f64, f64),
    transform: &DataToPixel,
    mutation_scale_px: f64,
    style: ArrowStyle,
    linewidth_px: f64,
    text: Option<AnnotateTextBox>,
) -> Option<QuiverArrow> {
    let (x0, y0) = xytext;
    let (x1, y1) = xy;
    if !(x0.is_finite() && y0.is_finite() && x1.is_finite() && y1.is_finite()) {
        return None;
    }
    let anchor = transform.map(Point::new(x0, y0));
    let mut tip = transform.map(Point::new(x1, y1));

    // mpl: arrow_begin = text_bbox.p0 + size * relpos(0.5, 0.5), then clip by
    // padded patchA, then shrinkA/shrinkB circles.
    let mut p0 = if let Some(tb) = text.filter(|t| t.width > 1e-6 && t.height > 1e-6) {
        let (x0, y0, x1, y1) = text_ink_rect(anchor, &tb);
        let pad = tb.pad_px.max(0.0);
        let px0 = x0 - pad;
        let py0 = y0 - pad;
        let px1 = x1 + pad;
        let py1 = y1 + pad;
        let center = Point::new(0.5 * (x0 + x1), 0.5 * (y0 + y1));
        ray_exit_aabb(center, tip, px0, py0, px1, py1).unwrap_or(center)
    } else {
        anchor
    };

    let sx = tip.x - p0.x;
    let sy = tip.y - p0.y;
    let len = (sx * sx + sy * sy).sqrt();
    if len < 1e-9 {
        return None;
    }
    let ux = sx / len;
    let uy = sy / len;
    let nx = -uy;
    let ny = ux;

    let px_per_pt = mutation_scale_px / ann_policy::MUTATION_SCALE_PT.max(1e-9);
    let shrink_px = ann_policy::SHRINK_PT * px_per_pt;
    let len = if len > 2.0 * shrink_px + 1e-9 {
        p0 = Point::new(p0.x + ux * shrink_px, p0.y + uy * shrink_px);
        tip = Point::new(tip.x - ux * shrink_px, tip.y - uy * shrink_px);
        len - 2.0 * shrink_px
    } else {
        len
    };

    let ms = mutation_scale_px.max(0.0);
    let max_frac = if matches!(style, ArrowStyle::BothEnds) {
        0.45
    } else {
        0.85
    };
    let hl = (ann_policy::HEAD_LENGTH * ms).min(len * max_frac);
    let hw = ann_policy::HEAD_WIDTH * ms;
    let linewidth_px = linewidth_px.max(0.0);
    if hl < 1e-9 && !matches!(style, ArrowStyle::Bracket) {
        return None;
    }

    match style {
        ArrowStyle::Triangle => filled_head(p0, tip, ux, uy, hl, hw, linewidth_px),
        ArrowStyle::Simple => open_v_head(p0, tip, ux, uy, hl, hw, linewidth_px),
        ArrowStyle::Bracket => bracket_head(p0, tip, ux, uy, nx, ny, ms),
        ArrowStyle::BothEnds => both_ends_open_v(p0, tip, ux, uy, hl, hw, linewidth_px),
    }
}

/// Ink rectangle for the annotation label (image y-down), matching draw_text
/// origin rules for align / baseline.
fn text_ink_rect(pos: Point, tb: &AnnotateTextBox) -> (f64, f64, f64, f64) {
    let w = tb.width.max(0.0);
    let h = tb.height.max(0.0);
    let size = tb.size_px.max(1.0);
    let mut x0 = pos.x;
    match tb.align {
        TextAlign::Left => {}
        TextAlign::Center => x0 -= w * 0.5,
        TextAlign::Right => x0 -= w,
    }
    let (y0, y1) = match tb.baseline {
        // mpl Text va='baseline': ~0.8em ascent / 0.2em descent around the anchor.
        TextBaseline::Alphabetic => (pos.y - 0.8 * size, pos.y + 0.2 * size),
        TextBaseline::Top => (pos.y, pos.y + h.max(size)),
        TextBaseline::Middle => {
            let half = 0.5 * h.max(size);
            (pos.y - half, pos.y + half)
        }
        TextBaseline::Bottom => (pos.y - h.max(size), pos.y),
    };
    (x0, y0, x0 + w, y1)
}

/// First exit of the ray `origin → tip` from the interior of an axis-aligned box.
fn ray_exit_aabb(origin: Point, tip: Point, x0: f64, y0: f64, x1: f64, y1: f64) -> Option<Point> {
    let dx = tip.x - origin.x;
    let dy = tip.y - origin.y;
    if dx.abs() < 1e-12 && dy.abs() < 1e-12 {
        return None;
    }
    let mut t = f64::INFINITY;
    if dx > 1e-12 {
        t = t.min((x1 - origin.x) / dx);
    } else if dx < -1e-12 {
        t = t.min((x0 - origin.x) / dx);
    }
    if dy > 1e-12 {
        t = t.min((y1 - origin.y) / dy);
    } else if dy < -1e-12 {
        t = t.min((y0 - origin.y) / dy);
    }
    if !t.is_finite() || t < 0.0 {
        return None;
    }
    // Tip still inside the pad box → nothing to draw outside.
    if t >= 1.0 {
        return None;
    }
    Some(Point::new(origin.x + t * dx, origin.y + t * dy))
}

/// Matplotlib `ArrowStyle._Curve._get_arrow_wedge`: wing tips + pad for
/// projected line caps (`pad_projected = 0.5 * linewidth / sin_t`).
fn wedge_at_tip(
    tip: Point,
    ux: f64,
    uy: f64,
    hl: f64,
    hw: f64,
    linewidth_px: f64,
) -> (Point, Point, Point) {
    let head_dist = (hl * hl + hw * hw).sqrt().max(1e-9);
    let cos_t = hl / head_dist;
    let sin_t = (hw / head_dist).max(1e-9);
    let pad = 0.5 * linewidth_px / sin_t;
    let tip = Point::new(tip.x - ux * pad, tip.y - uy * pad);
    let dx = -ux * head_dist;
    let dy = -uy * head_dist;
    let left = Point::new(
        tip.x + cos_t * dx + sin_t * dy,
        tip.y + -sin_t * dx + cos_t * dy,
    );
    let right = Point::new(
        tip.x + cos_t * dx - sin_t * dy,
        tip.y + sin_t * dx + cos_t * dy,
    );
    (left, tip, right)
}

fn filled_head(
    p0: Point,
    tip: Point,
    ux: f64,
    uy: f64,
    hl: f64,
    hw: f64,
    linewidth_px: f64,
) -> Option<QuiverArrow> {
    let (left, tip, right) = wedge_at_tip(tip, ux, uy, hl, hw, linewidth_px);
    let mut shaft = kurbo::BezPath::new();
    shaft.move_to(p0.to_kurbo());
    shaft.line_to(tip.to_kurbo());
    let mut head = kurbo::BezPath::new();
    head.move_to(left.to_kurbo());
    head.line_to(tip.to_kurbo());
    head.line_to(right.to_kurbo());
    head.close_path();
    Some(QuiverArrow { shaft, head })
}

fn open_v_head(
    p0: Point,
    tip: Point,
    ux: f64,
    uy: f64,
    hl: f64,
    hw: f64,
    linewidth_px: f64,
) -> Option<QuiverArrow> {
    let (left, tip, right) = wedge_at_tip(tip, ux, uy, hl, hw, linewidth_px);
    let mut shaft = kurbo::BezPath::new();
    shaft.move_to(p0.to_kurbo());
    shaft.line_to(tip.to_kurbo());
    let mut head = kurbo::BezPath::new();
    head.move_to(left.to_kurbo());
    head.line_to(tip.to_kurbo());
    head.line_to(right.to_kurbo());
    Some(QuiverArrow { shaft, head })
}

fn both_ends_open_v(
    p0: Point,
    tip: Point,
    ux: f64,
    uy: f64,
    hl: f64,
    hw: f64,
    linewidth_px: f64,
) -> Option<QuiverArrow> {
    let (l0, tip_p, r0) = wedge_at_tip(tip, ux, uy, hl, hw, linewidth_px);
    let (l1, p0_p, r1) = wedge_at_tip(p0, -ux, -uy, hl, hw, linewidth_px);
    let mut shaft = kurbo::BezPath::new();
    shaft.move_to(p0_p.to_kurbo());
    shaft.line_to(tip_p.to_kurbo());
    let mut head = kurbo::BezPath::new();
    head.move_to(l0.to_kurbo());
    head.line_to(tip_p.to_kurbo());
    head.line_to(r0.to_kurbo());
    head.move_to(l1.to_kurbo());
    head.line_to(p0_p.to_kurbo());
    head.line_to(r1.to_kurbo());
    Some(QuiverArrow { shaft, head })
}

/// Matplotlib `BracketB` / `-[`: outward square bracket at the tip.
fn bracket_head(
    p0: Point,
    tip: Point,
    ux: f64,
    uy: f64,
    nx: f64,
    ny: f64,
    ms: f64,
) -> Option<QuiverArrow> {
    let half = ann_policy::BRACKET_WIDTH * ms;
    let arm = ann_policy::BRACKET_LENGTH * ms;
    let left = Point::new(tip.x + nx * half, tip.y + ny * half);
    let right = Point::new(tip.x - nx * half, tip.y - ny * half);
    let left_out = Point::new(left.x + ux * arm, left.y + uy * arm);
    let right_out = Point::new(right.x + ux * arm, right.y + uy * arm);

    let mut shaft = kurbo::BezPath::new();
    shaft.move_to(p0.to_kurbo());
    shaft.line_to(tip.to_kurbo());
    let mut head = kurbo::BezPath::new();
    head.move_to(left_out.to_kurbo());
    head.line_to(left.to_kurbo());
    head.line_to(right.to_kurbo());
    head.line_to(right_out.to_kurbo());
    Some(QuiverArrow { shaft, head })
}

/// Pixel position of a data point (for text anchors).
pub fn data_to_pixel(x: f64, y: f64, transform: &DataToPixel) -> Option<Point> {
    if x.is_finite() && y.is_finite() {
        Some(transform.map(Point::new(x, y)))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn transform() -> DataToPixel {
        DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 4.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 4.0).unwrap()),
            Rect::new(0.0, 0.0, 200.0, 200.0),
        )
    }

    #[test]
    fn arrow_present() {
        let t = transform();
        let a = annotation_arrow((3.0, 1.0), (1.0, 2.0), &t, 10.0).expect("arrow");
        assert!(!a.shaft.is_empty());
        assert!(!a.head.is_empty());
    }

    #[test]
    fn styles_produce_geometry() {
        let t = transform();
        for style in [
            ArrowStyle::Triangle,
            ArrowStyle::Simple,
            ArrowStyle::Bracket,
            ArrowStyle::BothEnds,
        ] {
            let a = annotation_arrow_styled((3.0, 1.0), (1.0, 2.0), &t, 10.0, style, 1.0, None)
                .unwrap_or_else(|| panic!("style {style:?}"));
            assert!(!a.shaft.is_empty(), "{style:?}");
            assert!(!a.head.is_empty(), "{style:?}");
        }
    }

    #[test]
    fn both_ends_is_open_not_closed_polygon() {
        let t = transform();
        let a = annotation_arrow_styled(
            (3.0, 1.0),
            (1.0, 2.0),
            &t,
            10.0,
            ArrowStyle::BothEnds,
            1.0,
            None,
        )
        .expect("both");
        let has_close = a
            .head
            .elements()
            .iter()
            .any(|el| matches!(el, kurbo::PathEl::ClosePath));
        assert!(
            !has_close,
            "BothEnds must be stroke open-V, not filled triangles"
        );
    }

    #[test]
    fn bracket_has_four_vertices_path() {
        let t = transform();
        let a = annotation_arrow_styled(
            (3.0, 1.0),
            (1.0, 2.0),
            &t,
            10.0,
            ArrowStyle::Bracket,
            1.0,
            None,
        )
        .expect("bracket");
        let n_line = a
            .head
            .elements()
            .iter()
            .filter(|el| matches!(el, kurbo::PathEl::LineTo(..)))
            .count();
        assert!(n_line >= 3, "expected U-bracket polyline, line_to={n_line}");
    }

    #[test]
    fn bracket_crossbar_is_two_widthb() {
        let t = transform();
        let ms = 20.0_f64;
        let a = annotation_arrow_styled(
            (3.0, 1.0),
            (1.0, 2.0),
            &t,
            ms,
            ArrowStyle::Bracket,
            1.0,
            None,
        )
        .expect("bracket");
        let pts: Vec<_> = a
            .head
            .elements()
            .iter()
            .filter_map(|el| match el {
                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => Some(*p),
                _ => None,
            })
            .collect();
        assert_eq!(pts.len(), 4);
        let bar = ((pts[1].x - pts[2].x).powi(2) + (pts[1].y - pts[2].y).powi(2)).sqrt();
        let expected = 2.0 * ann_policy::BRACKET_WIDTH * ms;
        assert!(
            (bar - expected).abs() < 1e-6,
            "crossbar={bar} expected={expected}"
        );
    }

    #[test]
    fn text_box_moves_start_past_label() {
        let t = transform();
        let tb = AnnotateTextBox {
            width: 40.0,
            height: 16.0,
            size_px: 16.0,
            align: TextAlign::Left,
            baseline: TextBaseline::Alphabetic,
            pad_px: 4.0,
        };
        let with = annotation_arrow_styled(
            (3.0, 1.0),
            (1.0, 2.0),
            &t,
            10.0,
            ArrowStyle::Triangle,
            1.0,
            Some(tb),
        )
        .expect("with text");
        let without = annotation_arrow_styled(
            (3.0, 1.0),
            (1.0, 2.0),
            &t,
            10.0,
            ArrowStyle::Triangle,
            1.0,
            None,
        )
        .expect("without");
        // Shaft start should move toward the tip when clearing the label.
        let start_with = match with.shaft.elements().first() {
            Some(kurbo::PathEl::MoveTo(p)) => *p,
            _ => panic!("start"),
        };
        let start_without = match without.shaft.elements().first() {
            Some(kurbo::PathEl::MoveTo(p)) => *p,
            _ => panic!("start"),
        };
        let tip = t.map(Point::new(3.0, 1.0));
        let d_with = (start_with.x - tip.x).hypot(start_with.y - tip.y);
        let d_without = (start_without.x - tip.x).hypot(start_without.y - tip.y);
        assert!(
            d_with < d_without - 1.0,
            "cleared start should be closer to tip: {d_with} vs {d_without}"
        );
    }

    #[test]
    fn zero_length_skips() {
        let t = transform();
        assert!(annotation_arrow((1.0, 1.0), (1.0, 1.0), &t, 10.0).is_none());
    }

    #[test]
    fn data_to_pixel_maps() {
        let t = transform();
        let p = data_to_pixel(2.0, 1.0, &t).unwrap();
        assert!(p.x.is_finite() && p.y.is_finite());
    }
}
