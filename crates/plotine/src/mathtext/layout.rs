//! Box layout + draw for mathtext AST.

use plotine_core::{Point, Result};
use plotine_render::{Renderer, TextAlign, TextBaseline, TextStyle};

use super::parse::{parse_mixed, AccentKind, MatrixKind, Node};
use crate::mpl_policy::mathtext as mt_policy;

const SCRIPT_SCALE: f32 = 0.7;
/// Matplotlib `Box.shrink()` twice → scriptscript size for `\sqrt[n]`.
const SQRT_INDEX_SCALE: f32 = SCRIPT_SCALE * SCRIPT_SCALE;
const FRAC_GAP: f64 = 0.10; // × size around rule
const MATRIX_COL_GAP: f64 = 0.40; // × size between columns
const MATRIX_ROW_GAP: f64 = 0.22; // × size between rows

#[derive(Debug, Clone)]
struct Run {
    text: String,
    x: f64,
    y: f64, // screen offset from baseline (↓ positive)
    size: f32,
    italic: bool,
}

#[derive(Debug, Clone)]
struct BoxMetrics {
    width: f64,
    /// Distance above baseline (positive).
    ascent: f64,
    /// Distance below baseline (positive).
    descent: f64,
    runs: Vec<Run>,
    /// Horizontal rules (x0, x1, y, thickness) in box coords.
    rules: Vec<(f64, f64, f64, f64)>,
    /// General strokes (x0, y0, x1, y1, thickness) in box coords.
    segs: Vec<(f64, f64, f64, f64, f64)>,
}

impl BoxMetrics {
    fn empty() -> Self {
        Self {
            width: 0.0,
            ascent: 0.0,
            descent: 0.0,
            runs: Vec::new(),
            rules: Vec::new(),
            segs: Vec::new(),
        }
    }

    fn height(&self) -> f64 {
        self.ascent + self.descent
    }

    fn shift(mut self, dx: f64, dy: f64) -> Self {
        for r in &mut self.runs {
            r.x += dx;
            r.y += dy;
        }
        for rule in &mut self.rules {
            rule.0 += dx;
            rule.1 += dx;
            rule.2 += dy;
        }
        for seg in &mut self.segs {
            seg.0 += dx;
            seg.1 += dy;
            seg.2 += dx;
            seg.3 += dy;
        }
        self
    }
}

/// Measure a mixed plain/math string.
pub fn measure_mathtext(renderer: &dyn Renderer, text: &str, size_px: f32) -> Result<(f64, f64)> {
    let node = parse_mixed(text);
    let b = layout_node(renderer, &node, size_px)?;
    Ok((b.width, b.height().max(size_px as f64)))
}

/// Draw a mixed plain/math string at `pos` with the given style.
pub fn draw_mathtext(
    renderer: &mut dyn Renderer,
    text: &str,
    pos: Point,
    style: &TextStyle,
) -> Result<()> {
    let node = parse_mixed(text);
    let boxm = layout_node(renderer, &node, style.size_px)?;
    let (anchor_x, anchor_y) = align_anchor(&boxm, style.align, style.baseline);
    let color = style.color;
    let rot = style.rotation_deg;
    let (sin, cos) = {
        let rad = rot.to_radians();
        (rad.sin(), rad.cos())
    };

    for run in &boxm.runs {
        if run.text.is_empty() {
            continue;
        }
        let lx = run.x - anchor_x;
        let ly = run.y - anchor_y;
        let (dx, dy) = if rot.abs() < 1e-6 {
            (lx, ly)
        } else {
            // Match tiny-skia / SVG: rotate around anchor, y-down.
            (lx * cos - ly * sin, lx * sin + ly * cos)
        };
        let run_style = TextStyle::new(color, run.size)
            .align(TextAlign::Left)
            .baseline(TextBaseline::Alphabetic)
            .rotation(rot)
            .italic(run.italic);
        renderer.draw_text(&run.text, Point::new(pos.x + dx, pos.y + dy), &run_style)?;
    }

    use plotine_render::StrokeStyle;
    let stroke_seg = |renderer: &mut dyn Renderer,
                      x0: f64,
                      y0: f64,
                      x1: f64,
                      y1: f64,
                      thick: f64|
     -> Result<()> {
        let lx0 = x0 - anchor_x;
        let ly0 = y0 - anchor_y;
        let lx1 = x1 - anchor_x;
        let ly1 = y1 - anchor_y;
        let (ax, ay) = if rot.abs() < 1e-6 {
            (lx0, ly0)
        } else {
            (lx0 * cos - ly0 * sin, lx0 * sin + ly0 * cos)
        };
        let (bx, by) = if rot.abs() < 1e-6 {
            (lx1, ly1)
        } else {
            (lx1 * cos - ly1 * sin, lx1 * sin + ly1 * cos)
        };
        renderer.draw_line(
            Point::new(pos.x + ax, pos.y + ay),
            Point::new(pos.x + bx, pos.y + by),
            &StrokeStyle::new(color, thick.max(1.0)),
        )
    };

    for &(x0, x1, y, thick) in &boxm.rules {
        stroke_seg(renderer, x0, y, x1, y, thick)?;
    }
    for &(x0, y0, x1, y1, thick) in &boxm.segs {
        stroke_seg(renderer, x0, y0, x1, y1, thick)?;
    }
    Ok(())
}

fn align_anchor(b: &BoxMetrics, align: TextAlign, baseline: TextBaseline) -> (f64, f64) {
    let ax = match align {
        TextAlign::Left => 0.0,
        TextAlign::Center => b.width * 0.5,
        TextAlign::Right => b.width,
    };
    let ay = match baseline {
        TextBaseline::Alphabetic => 0.0,
        TextBaseline::Top => -b.ascent,
        TextBaseline::Middle => (b.descent - b.ascent) * 0.5,
        TextBaseline::Bottom => b.descent,
    };
    (ax, ay)
}

fn layout_node(renderer: &dyn Renderer, node: &Node, size: f32) -> Result<BoxMetrics> {
    // Titles / labels are matplotlib textstyle (inline) by default.
    layout_node_ctx(renderer, node, size, false)
}

fn layout_node_ctx(
    renderer: &dyn Renderer,
    node: &Node,
    size: f32,
    display: bool,
) -> Result<BoxMetrics> {
    match node {
        Node::Text(s) => layout_text(renderer, s, size, false),
        Node::Italic(s) => layout_text(renderer, s, size, true),
        Node::Space(em) => Ok(BoxMetrics {
            width: (*em) * size as f64,
            ascent: 0.0,
            descent: 0.0,
            runs: Vec::new(),
            rules: Vec::new(),
            segs: Vec::new(),
        }),
        Node::List(items) => {
            let mut out = BoxMetrics::empty();
            for item in items {
                let b = layout_node_ctx(renderer, item, size, display)?;
                let shifted = b.shift(out.width, 0.0);
                out.width += shifted.width;
                out.ascent = out.ascent.max(shifted.ascent);
                out.descent = out.descent.max(shifted.descent);
                out.runs.extend(shifted.runs);
                out.rules.extend(shifted.rules);
                out.segs.extend(shifted.segs);
            }
            Ok(out)
        }
        Node::Style {
            display: style_display,
            body,
        } => layout_node_ctx(renderer, body, size, *style_display),
        Node::Script {
            base,
            sup,
            sub,
            limits,
        } => layout_script(
            renderer,
            base,
            sup.as_deref(),
            sub.as_deref(),
            size,
            display,
            *limits,
        ),
        Node::Frac { num, den } => layout_frac(renderer, num, den, size, display),
        Node::Sqrt { index, body } => layout_sqrt(renderer, index.as_deref(), body, size, display),
        Node::Matrix { kind, rows } => layout_matrix(renderer, *kind, rows, size, display),
        Node::Accent { kind, body } => layout_accent(renderer, *kind, body, size, display),
    }
}

fn layout_text(renderer: &dyn Renderer, s: &str, size: f32, italic: bool) -> Result<BoxMetrics> {
    if s.is_empty() {
        return Ok(BoxMetrics::empty());
    }
    let (w, _line_h) = renderer.measure_text_styled(s, size, italic)?;
    // Cosmic-text line height includes leading (~1.2×em). Math box metrics must
    // track the em box like mpl `_mathtext` (ascent/descent vs font size), not
    // the layout line height — otherwise scripts/fracs sit too high/low.
    let em = f64::from(size);
    let ascent = em * 0.8;
    let descent = em * 0.2;
    Ok(BoxMetrics {
        width: w,
        ascent,
        descent,
        runs: vec![Run {
            text: s.to_string(),
            x: 0.0,
            y: 0.0,
            size,
            italic,
        }],
        rules: Vec::new(),
        segs: Vec::new(),
    })
}

fn layout_script(
    renderer: &dyn Renderer,
    base: &Node,
    sup: Option<&Node>,
    sub: Option<&Node>,
    size: f32,
    display: bool,
    limits: Option<bool>,
) -> Result<BoxMetrics> {
    // Inline (default): side scripts like mpl title mathtext.
    // Above/below only for `\limits` or `\displaystyle` large ops.
    let use_op_limits = match limits {
        Some(true) => is_large_op(base),
        Some(false) => false,
        None => display && is_large_op(base),
    };
    if use_op_limits && (sup.is_some() || sub.is_some()) {
        return layout_op_limits(renderer, base, sup, sub, size, display);
    }

    // Textstyle: same size as surrounding (mpl); side scripts via FontConstantsBase.
    let large = is_large_op(base);
    let base_size = if large {
        size * mt_policy::LARGE_OP_TEXTSTYLE_SCALE
    } else {
        size
    };
    let base_b = layout_node_ctx(renderer, base, base_size, display)?;
    let script_size = (size * SCRIPT_SCALE).max(6.0);
    let mut width = base_b.width;
    let mut ascent = base_b.ascent;
    let mut descent = base_b.descent;
    let mut runs = base_b.runs;
    let mut rules = base_b.rules;

    // mpl `_mathtext.Parser.subsuper` for slanted dropsub (∫/∮):
    //   superkern = (3·δ + δ_∫)·height
    //   subkern   = (3·δ − δ_∫)·height
    //   shift_up  = height − subdrop·xHeight
    //   shift_down = depth + subdrop·xHeight
    let x_height = mt_policy::X_HEIGHT_FRAC * size as f64;
    let both = sup.is_some() && sub.is_some();
    let (superkern, subkern, shift_up, shift_down) = if large {
        // Slanted dropsub (∫): mpl `delta` / `delta_integral` formulas.
        let h = base_b.ascent;
        let d = base_b.descent;
        let sk = (3.0 * mt_policy::DELTA + mt_policy::DELTA_INTEGRAL) * h;
        let bk = (3.0 * mt_policy::DELTA - mt_policy::DELTA_INTEGRAL) * h;
        let up = h - mt_policy::SUBDROP * x_height;
        let down = d + mt_policy::SUBDROP * x_height;
        (sk, bk, up, down)
    } else {
        // Ordinary nuclei: `sup1` / `sub1` / `sub2` × xHeight + `delta` kern.
        let sk = mt_policy::DELTA * x_height;
        let bk = mt_policy::DELTA * x_height;
        let up = mt_policy::SUP1 * x_height;
        let down = if both {
            mt_policy::SUB2 * x_height
        } else {
            mt_policy::SUB1 * x_height
        };
        (sk, bk, up, down)
    };

    let mut segs = base_b.segs;
    if let Some(s) = sup {
        let sb = layout_node_ctx(renderer, s, script_size, display)?;
        let sx = base_b.width + superkern;
        let dy = -shift_up;
        let shifted = sb.shift(sx, dy);
        width = width.max(sx + shifted.width);
        ascent = ascent.max(-dy + shifted.ascent);
        runs.extend(shifted.runs);
        rules.extend(shifted.rules);
        segs.extend(shifted.segs);
    }
    if let Some(s) = sub {
        let sb = layout_node_ctx(renderer, s, script_size, display)?;
        let sx = (base_b.width + subkern).max(0.0);
        let dy = shift_down;
        let shifted = sb.shift(sx, dy);
        width = width.max(sx + shifted.width);
        descent = descent.max(dy + shifted.descent);
        runs.extend(shifted.runs);
        rules.extend(shifted.rules);
        segs.extend(shifted.segs);
    }
    // Ordinary scripts get `script_space`; dropsub (∫) does not (mpl).
    if !large && (sup.is_some() || sub.is_some()) {
        width += mt_policy::SCRIPT_SPACE * x_height;
    }

    Ok(BoxMetrics {
        width,
        ascent,
        descent,
        runs,
        rules,
        segs,
    })
}

/// True for Unicode large operators that can take limits above/below.
fn is_large_op(base: &Node) -> bool {
    match base {
        Node::Italic(s) | Node::Text(s) => s.chars().any(|c| {
            matches!(
                c,
                '\u{222B}' // ∫
                | '\u{222C}' // ∬
                | '\u{222D}' // ∭
                | '\u{222E}' // ∮
                | '\u{2211}' // ∑
                | '\u{220F}' // ∏
                | '\u{22C3}' // ⋃
                | '\u{22C2}' // ⋂
            )
        }),
        Node::List(items) if items.len() == 1 => is_large_op(&items[0]),
        _ => false,
    }
}

/// TeX `\limits` / `\displaystyle` placement for ∫₀¹ …
fn layout_op_limits(
    renderer: &dyn Renderer,
    base: &Node,
    sup: Option<&Node>,
    sub: Option<&Node>,
    size: f32,
    display: bool,
) -> Result<BoxMetrics> {
    let op_size = size * mt_policy::LARGE_OP_DISPLAY_SCALE;
    let script_size = (size * SCRIPT_SCALE).max(6.0);
    let gap = mt_policy::LARGE_OP_LIMIT_GAP * size as f64;

    let base_b = layout_node_ctx(renderer, base, op_size, display)?;
    let mut width = base_b.width;
    let mut ascent = base_b.ascent;
    let mut descent = base_b.descent;

    let sup_b = if let Some(s) = sup {
        Some(layout_node_ctx(renderer, s, script_size, display)?)
    } else {
        None
    };
    let sub_b = if let Some(s) = sub {
        Some(layout_node_ctx(renderer, s, script_size, display)?)
    } else {
        None
    };
    if let Some(ref sb) = sup_b {
        width = width.max(sb.width);
    }
    if let Some(ref sb) = sub_b {
        width = width.max(sb.width);
    }

    let base_x = (width - base_b.width) * 0.5;
    let mut runs = Vec::new();
    let mut rules = Vec::new();
    let mut segs = Vec::new();

    let shifted_base = base_b.shift(base_x, 0.0);
    runs.extend(shifted_base.runs);
    rules.extend(shifted_base.rules);
    segs.extend(shifted_base.segs);

    if let Some(sb) = sup_b {
        let dy = -(ascent + gap + sb.descent);
        let sx = (width - sb.width) * 0.5;
        let shifted = sb.shift(sx, dy);
        ascent = -dy + shifted.ascent;
        runs.extend(shifted.runs);
        rules.extend(shifted.rules);
        segs.extend(shifted.segs);
    }
    if let Some(sb) = sub_b {
        let dy = descent + gap + sb.ascent;
        let sx = (width - sb.width) * 0.5;
        let shifted = sb.shift(sx, dy);
        descent = dy + shifted.descent;
        runs.extend(shifted.runs);
        rules.extend(shifted.rules);
        segs.extend(shifted.segs);
    }

    Ok(BoxMetrics {
        width,
        ascent,
        descent,
        runs,
        rules,
        segs,
    })
}

fn layout_frac(
    renderer: &dyn Renderer,
    num: &Node,
    den: &Node,
    size: f32,
    display: bool,
) -> Result<BoxMetrics> {
    let num_b = layout_node_ctx(renderer, num, size * SCRIPT_SCALE.max(0.75), display)?;
    let den_b = layout_node_ctx(renderer, den, size * SCRIPT_SCALE.max(0.75), display)?;
    let gap = FRAC_GAP * size as f64;
    let rule_t = (0.06 * size as f64).max(1.0);
    let inner_w = num_b.width.max(den_b.width);
    let width = inner_w + size as f64 * 0.15;
    let num_y = -(gap + rule_t * 0.5 + num_b.descent);
    let den_y = gap + rule_t * 0.5 + den_b.ascent;
    let num_x = (width - num_b.width) * 0.5;
    let den_x = (width - den_b.width) * 0.5;
    let mut runs = Vec::new();
    let mut rules = vec![(0.05 * size as f64, width - 0.05 * size as f64, 0.0, rule_t)];
    let n = num_b.shift(num_x, num_y);
    let d = den_b.shift(den_x, den_y);
    let ascent = -num_y + n.ascent;
    let descent = den_y + d.descent;
    let mut segs = Vec::new();
    runs.extend(n.runs);
    runs.extend(d.runs);
    rules.extend(n.rules);
    rules.extend(d.rules);
    segs.extend(n.segs);
    segs.extend(d.segs);
    Ok(BoxMetrics {
        width,
        ascent,
        descent,
        runs,
        rules,
        segs,
    })
}

fn layout_sqrt(
    renderer: &dyn Renderer,
    index: Option<&Node>,
    body: &Node,
    size: f32,
    display: bool,
) -> Result<BoxMetrics> {
    // Layout mirrors `matplotlib._mathtext.Parser.sqrt`:
    // Hlist([root, Kern(-0.5*check.width), check, rightside(Hrule+body)]).
    let body_b = layout_node_ctx(renderer, body, size, display)?;
    let thickness = (0.06 * size as f64).max(1.0);
    let side_pad = mt_policy::SQRT_BODY_SIDE_PAD_THICKNESS * thickness;
    let mut height = body_b.ascent + mt_policy::SQRT_BODY_CLEARANCE_THICKNESS * thickness;
    let mut depth = body_b.descent;

    // Geometric stand-in for AutoHeightChar(r'\__sqrt__', …).
    let check_width = 0.45 * size as f64;
    let hook_w = check_width;
    // After the sized check glyph, mpl re-reads height/depth from the check.
    height = height.max(body_b.ascent + thickness);
    depth = depth.max(body_b.descent);

    let rule_t = thickness;
    let rule_y = -height + rule_t * 0.5;
    let tip_y = depth * 0.35;
    let notch_y = depth * 0.55;

    // Root index: double `shrink()` → scriptscript; empty → 0.5*check.width box.
    let (index_box, root_advance) = if let Some(idx) = index {
        let script_size = (size * SQRT_INDEX_SCALE).max(5.0);
        let b = layout_node_ctx(renderer, idx, script_size, display)?;
        let w = b.width;
        (b, w)
    } else {
        (
            BoxMetrics::empty(),
            mt_policy::SQRT_EMPTY_ROOT_FRAC * check_width,
        )
    };

    // Hlist advance: root → Kern(-0.5*check.width) → check → padded body.
    let kern = -mt_policy::SQRT_ROOT_KERN * check_width;
    let check_x = root_advance + kern;
    let body_x = check_x + hook_w + side_pad;
    let width = body_x + body_b.width + side_pad;

    let tip_x = check_x;
    let notch_x = check_x + hook_w * 0.35;
    let join_x = check_x + hook_w;

    let mut rules = vec![(join_x, width - side_pad * 0.25, rule_y, rule_t)];
    let mut segs = vec![
        (tip_x, tip_y, notch_x, notch_y, rule_t * 0.85),
        (notch_x, notch_y, join_x, rule_y, rule_t),
    ];

    let body_shifted = body_b.shift(body_x, 0.0);
    let mut runs = body_shifted.runs;
    rules.extend(body_shifted.rules);
    segs.extend(body_shifted.segs);

    let mut ascent = height;
    let descent = depth.max(notch_y);

    // `root_vlist.shift_amount = -height * 0.6` (TeX y-up → our y-down).
    if index.is_some() {
        let idx_y = -height * mt_policy::SQRT_ROOT_SHIFT;
        let shifted = index_box.shift(0.0, idx_y);
        ascent = ascent.max(-idx_y + shifted.ascent);
        runs.extend(shifted.runs);
        rules.extend(shifted.rules);
        segs.extend(shifted.segs);
    }

    Ok(BoxMetrics {
        width,
        ascent,
        descent,
        runs,
        rules,
        segs,
    })
}

fn layout_matrix(
    renderer: &dyn Renderer,
    kind: MatrixKind,
    rows: &[Vec<Node>],
    size: f32,
    display: bool,
) -> Result<BoxMetrics> {
    let scale = if kind == MatrixKind::Small {
        0.7
    } else {
        SCRIPT_SCALE.max(0.85)
    };
    let cell_size = size * scale;
    let col_gap = MATRIX_COL_GAP * size as f64 * if kind == MatrixKind::Small { 0.6 } else { 1.0 };
    let row_gap = MATRIX_ROW_GAP * size as f64 * if kind == MatrixKind::Small { 0.6 } else { 1.0 };
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);

    let mut cells: Vec<Vec<BoxMetrics>> = Vec::with_capacity(rows.len());
    let mut col_w: Vec<f64> = vec![0.0; ncols];
    let mut row_ascent: Vec<f64> = Vec::with_capacity(rows.len());
    let mut row_descent: Vec<f64> = Vec::with_capacity(rows.len());

    for row in rows {
        let mut crow = Vec::with_capacity(ncols);
        let mut ra: f64 = 0.0;
        let mut rd: f64 = 0.0;
        for (c, cw) in col_w.iter_mut().enumerate().take(ncols) {
            let node = row
                .get(c)
                .cloned()
                .unwrap_or_else(|| Node::Text(String::new()));
            let b = layout_node_ctx(renderer, &node, cell_size, display)?;
            *cw = cw.max(b.width);
            ra = ra.max(b.ascent);
            rd = rd.max(b.descent);
            crow.push(b);
        }
        row_ascent.push(ra);
        row_descent.push(rd);
        cells.push(crow);
    }

    let mut row_heights = Vec::with_capacity(rows.len());
    for i in 0..rows.len() {
        row_heights.push(row_ascent[i] + row_descent[i]);
    }
    let inner_h: f64 =
        row_heights.iter().sum::<f64>() + row_gap * (rows.len().saturating_sub(1) as f64);

    let (ldelim, rdelim) = match kind {
        MatrixKind::Plain | MatrixKind::Small => ("", ""),
        MatrixKind::Paren => ("(", ")"),
        MatrixKind::Bracket => ("[", "]"),
        MatrixKind::VBar => ("\u{2223}", "\u{2223}"),
        MatrixKind::DoubleVBar => ("\u{2225}", "\u{2225}"),
        MatrixKind::Brace => ("{", "}"),
    };
    let delim_w = if ldelim.is_empty() {
        0.0
    } else {
        let (w, _) = renderer.measure_text(ldelim, size)?;
        w * 1.15
    };

    let inner_w: f64 = col_w.iter().sum::<f64>() + col_gap * (ncols.saturating_sub(1) as f64);
    let width = inner_w + 2.0 * delim_w + size as f64 * 0.1;
    let total_ascent = inner_h * 0.5 + size as f64 * 0.1;
    let total_descent = inner_h * 0.5 + size as f64 * 0.1;

    let mut runs = Vec::new();
    let mut rules = Vec::new();
    let mut segs = Vec::new();
    let content_x0 = delim_w + size as f64 * 0.05;
    let mut y = -total_ascent + size as f64 * 0.05;

    for (ri, crow) in cells.iter().enumerate() {
        let rh = row_heights[ri];
        let baseline = y + row_ascent[ri];
        let mut x = content_x0;
        for (ci, cell) in crow.iter().enumerate() {
            let cx = x + (col_w[ci] - cell.width) * 0.5;
            let shifted = cell.clone().shift(cx, baseline);
            runs.extend(shifted.runs);
            rules.extend(shifted.rules);
            segs.extend(shifted.segs);
            x += col_w[ci] + col_gap;
        }
        y += rh + row_gap;
    }

    if !ldelim.is_empty() {
        let delim_size = (size * (1.0 + (rows.len() as f32 - 1.0) * 0.15)).min(size * 1.8);
        let (lw, lh) = renderer.measure_text(ldelim, delim_size)?;
        let (rw, _) = renderer.measure_text(rdelim, delim_size)?;
        let dy = lh * 0.15;
        runs.push(Run {
            text: ldelim.into(),
            x: (delim_w - lw) * 0.5,
            y: dy,
            size: delim_size,
            italic: false,
        });
        runs.push(Run {
            text: rdelim.into(),
            x: width - delim_w + (delim_w - rw) * 0.5,
            y: dy,
            size: delim_size,
            italic: false,
        });
    }

    Ok(BoxMetrics {
        width,
        ascent: total_ascent,
        descent: total_descent,
        runs,
        rules,
        segs,
    })
}

fn layout_accent(
    renderer: &dyn Renderer,
    kind: AccentKind,
    body: &Node,
    size: f32,
    display: bool,
) -> Result<BoxMetrics> {
    let b = layout_node_ctx(renderer, body, size, display)?;
    let pad = size as f64 * 0.08;

    if matches!(kind, AccentKind::Underline) {
        let line_t = (0.05 * size as f64).max(1.0);
        let underline_y = b.descent + pad;
        let total_descent = underline_y + line_t;
        let mut rules = vec![(0.0, b.width, underline_y, line_t)];
        let runs = b.runs;
        rules.extend(b.rules);
        let segs = b.segs;
        return Ok(BoxMetrics {
            width: b.width,
            ascent: b.ascent,
            descent: total_descent,
            runs,
            rules,
            segs,
        });
    }

    let accent_char = match kind {
        AccentKind::Hat => '\u{0302}',
        AccentKind::Bar | AccentKind::Overline => '\u{0305}',
        AccentKind::Vec => '\u{20D7}',
        AccentKind::Tilde => '\u{0303}',
        AccentKind::Dot => '\u{0307}',
        AccentKind::Ddot => '\u{0308}',
        AccentKind::Underline => unreachable!(),
    };
    let accent_size = (size * 0.85).max(6.0);
    let accent_str = accent_char.to_string();
    let (aw, ah) = renderer.measure_text(&accent_str, accent_size)?;
    let accent_h = ah.max(size as f64 * 0.15);
    let accent_y = -(b.ascent + pad + accent_h * 0.5);
    let accent_x = (b.width - aw) * 0.5;

    let mut runs = b.runs;
    runs.push(Run {
        text: accent_str,
        x: accent_x.max(0.0),
        y: accent_y,
        size: accent_size,
        italic: false,
    });
    let total_ascent = b.ascent + pad + accent_h;

    Ok(BoxMetrics {
        width: b.width,
        ascent: total_ascent,
        descent: b.descent,
        runs,
        rules: b.rules,
        segs: b.segs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_backend_skia::SkiaRenderer;

    #[test]
    fn int_default_uses_side_scripts() {
        let renderer = SkiaRenderer::new(120, 80).expect("renderer");
        let node = parse_mixed(r"$\int_0^1$");
        let boxm = layout_node(&renderer, &node, 24.0).expect("layout");
        let texts: Vec<&str> = boxm.runs.iter().map(|r| r.text.as_str()).collect();
        assert!(texts.contains(&"1"), "missing upper limit, got {texts:?}");
        assert!(texts.contains(&"0"), "missing lower limit, got {texts:?}");
        let one = boxm.runs.iter().find(|r| r.text == "1").unwrap();
        let zero = boxm.runs.iter().find(|r| r.text == "0").unwrap();
        let op = boxm
            .runs
            .iter()
            .find(|r| r.text.contains('\u{222B}'))
            .expect("integral run");
        assert!(
            one.y < zero.y,
            "sup above sub: one.y={} zero.y={}",
            one.y,
            zero.y
        );
        // Side scripts sit to the right of ∫ (x > 0), not stacked on the op center.
        assert!(
            one.x > 2.0 && zero.x > 2.0,
            "expected side scripts, got one.x={} zero.x={}",
            one.x,
            zero.x
        );
        // mpl dropsub: scripts sit near/past the op advance (sup further right).
        let op_w = renderer.measure_text(&op.text, op.size).unwrap().0;
        assert!(
            one.x >= op.x + op_w * 0.85,
            "sup too far inside op: one.x={} op_right={}",
            one.x,
            op.x + op_w
        );
        // Italic correction: superscript further right than subscript.
        assert!(
            one.x > zero.x,
            "expected italic kern one.x > zero.x, got {} vs {}",
            one.x,
            zero.x
        );
    }

    #[test]
    fn int_limits_and_displaystyle_stack_scripts() {
        let renderer = SkiaRenderer::new(120, 80).expect("renderer");
        for src in [r"$\int\limits_0^1$", r"$\displaystyle\int_0^1$"] {
            let node = parse_mixed(src);
            let boxm = layout_node(&renderer, &node, 24.0).expect("layout");
            let one = boxm.runs.iter().find(|r| r.text == "1").expect("upper");
            let zero = boxm.runs.iter().find(|r| r.text == "0").expect("lower");
            assert!(one.y < zero.y, "{src}: one.y={} zero.y={}", one.y, zero.y);
            // Stacked limits are roughly centered on the op (small x).
            assert!(
                one.x < 8.0 && zero.x < 8.0,
                "{src}: expected stacked limits, got one.x={} zero.x={}",
                one.x,
                zero.x
            );
            assert!(
                boxm.ascent > 20.0,
                "{src}: display/limits ascent too small: {}",
                boxm.ascent
            );
        }
    }

    #[test]
    fn sqrt_index_matches_mpl_double_shrink_and_kern() {
        let renderer = SkiaRenderer::new(200, 80).expect("renderer");
        let size = 24.0_f32;
        let node = parse_mixed(r"$\sqrt[3]{8}$");
        let boxm = layout_node(&renderer, &node, size).expect("layout");
        let three = boxm
            .runs
            .iter()
            .find(|r| r.text == "3")
            .expect("root index");
        let eight = boxm.runs.iter().find(|r| r.text == "8").expect("radicand");
        // Double shrink → scriptscript (~0.49×), not a single 0.55× script.
        let expected = size * SQRT_INDEX_SCALE;
        assert!(
            (three.size - expected).abs() < 0.5,
            "index size {} vs expected {}",
            three.size,
            expected
        );
        // Index sits above the radical tick (`-height * 0.6`).
        assert!(
            three.y < -boxm.ascent * mt_policy::SQRT_ROOT_SHIFT * 0.5,
            "index not raised enough: y={} ascent={}",
            three.y,
            boxm.ascent
        );
        // Negative kern overlays the index on the left half of the check; radicand
        // starts after the stem (not after a full reserved index slot).
        assert!(
            eight.x > three.x,
            "radicand should be to the right of the index"
        );
        let check_width = 0.45 * size as f64;
        assert!(
            three.x < check_width,
            "index should overlap the check stem (x={} check_w={})",
            three.x,
            check_width
        );
    }
}
