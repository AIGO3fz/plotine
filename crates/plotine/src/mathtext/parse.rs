//! TeX-like math tokenizer / AST for mathtext.

use crate::math;

/// True when the string contains at least one `$...$` math span.
pub fn needs_mathtext(s: &str) -> bool {
    let mut in_math = false;
    for ch in s.chars() {
        if ch == '$' {
            if in_math {
                return true;
            }
            in_math = true;
        }
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum AccentKind {
    Hat,
    Bar,
    Vec,
    Tilde,
    Dot,
    Ddot,
    Overline,
    Underline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatrixKind {
    /// `\begin{matrix}` — no delimiters.
    Plain,
    /// `\begin{pmatrix}` — parentheses.
    Paren,
    /// `\begin{bmatrix}` — square brackets.
    Bracket,
    /// `\begin{vmatrix}` — vertical bars (determinant).
    VBar,
    /// `\begin{Vmatrix}` — double vertical bars (norm).
    DoubleVBar,
    /// `\begin{Bmatrix}` — curly braces.
    Brace,
    /// `\begin{smallmatrix}` — no delimiters, reduced size.
    Small,
}

#[derive(Debug, Clone)]
pub(crate) enum Node {
    Text(String),
    /// Math-mode italic (matplotlib default for letters / Greek).
    Italic(String),
    /// Horizontal list of atoms.
    List(Vec<Node>),
    Script {
        base: Box<Node>,
        sup: Option<Box<Node>>,
        sub: Option<Box<Node>>,
        /// `Some(true)` = `\limits`, `Some(false)` = `\nolimits`, `None` = auto from style.
        limits: Option<bool>,
    },
    /// `\displaystyle{…}` / `\textstyle{…}` (or unbraced rest-of-math).
    Style {
        display: bool,
        body: Box<Node>,
    },
    Frac {
        num: Box<Node>,
        den: Box<Node>,
    },
    /// `\sqrt{body}` or `\sqrt[index]{body}`.
    Sqrt {
        index: Option<Box<Node>>,
        body: Box<Node>,
    },
    /// `\begin{matrix|pmatrix|bmatrix} … \end{…}`.
    Matrix {
        kind: MatrixKind,
        rows: Vec<Vec<Node>>,
    },
    /// `\hat{x}`, `\bar{x}`, `\vec{x}`, etc.
    Accent {
        kind: AccentKind,
        body: Box<Node>,
    },
    /// Thin space (em units).
    Space(f64),
}

/// Split a mixed string into plain / math alternating segments.
pub(crate) fn split_dollar_spans(s: &str) -> Vec<(bool, String)> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut math = false;
    for ch in s.chars() {
        if ch == '$' {
            if !cur.is_empty() || out.is_empty() {
                out.push((math, std::mem::take(&mut cur)));
            } else if math {
                // empty math — still push
                out.push((math, String::new()));
            }
            math = !math;
        } else {
            cur.push(ch);
        }
    }
    if !cur.is_empty() || out.is_empty() {
        out.push((math, cur));
    }
    // Drop trailing empty plain after final `$`
    if out.len() > 1 {
        if let Some((false, t)) = out.last() {
            if t.is_empty() {
                out.pop();
            }
        }
    }
    out
}

pub(crate) fn parse_mixed(s: &str) -> Node {
    let spans = split_dollar_spans(s);
    let mut parts = Vec::new();
    for (is_math, text) in spans {
        if text.is_empty() {
            continue;
        }
        if is_math {
            parts.push(parse_math(&text));
        } else {
            parts.push(Node::Text(text));
        }
    }
    if parts.len() == 1 {
        parts.pop().unwrap()
    } else {
        Node::List(parts)
    }
}

fn parse_math(s: &str) -> Node {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut atoms = Vec::new();
    while i < chars.len() {
        let before_ws = i;
        skip_ws(&chars, &mut i);
        if i >= chars.len() {
            break;
        }
        // Ordinary spaces between atoms → thin space (mpl / TeX inter-atom gap).
        // Without this, `$\int_0^1 x^2$` packs the integrand against the limits.
        if i > before_ws && !atoms.is_empty() {
            atoms.push(Node::Space(0.1667));
        }
        let atom = parse_atom(&chars, &mut i);
        // Optional `\limits` / `\nolimits` between operator and scripts.
        let limits = parse_limits_flag(&chars, &mut i);
        // Attach scripts
        let mut base = atom;
        let mut attached = false;
        loop {
            skip_ws(&chars, &mut i);
            if i >= chars.len() {
                break;
            }
            match chars[i] {
                '^' => {
                    i += 1;
                    let sup = parse_script_body(&chars, &mut i);
                    let sub = if i < chars.len() && chars[i] == '_' {
                        i += 1;
                        Some(Box::new(parse_script_body(&chars, &mut i)))
                    } else {
                        None
                    };
                    base = Node::Script {
                        base: Box::new(base),
                        sup: Some(Box::new(sup)),
                        sub,
                        // Only the first script layer carries the op-limits flag.
                        limits: if attached { None } else { limits },
                    };
                    attached = true;
                }
                '_' => {
                    i += 1;
                    let sub = parse_script_body(&chars, &mut i);
                    let sup = if i < chars.len() && chars[i] == '^' {
                        i += 1;
                        Some(Box::new(parse_script_body(&chars, &mut i)))
                    } else {
                        None
                    };
                    base = Node::Script {
                        base: Box::new(base),
                        sup,
                        sub: Some(Box::new(sub)),
                        limits: if attached { None } else { limits },
                    };
                    attached = true;
                }
                _ => break,
            }
        }
        atoms.push(base);
    }
    if atoms.len() == 1 {
        atoms.pop().unwrap()
    } else {
        Node::List(atoms)
    }
}

/// Consume `\limits` / `\nolimits` if present; otherwise leave the cursor unchanged.
fn parse_limits_flag(chars: &[char], i: &mut usize) -> Option<bool> {
    skip_ws(chars, i);
    if *i >= chars.len() || chars[*i] != '\\' {
        return None;
    }
    let mut j = *i + 1;
    let mut name = String::new();
    while j < chars.len() && chars[j].is_ascii_alphabetic() {
        name.push(chars[j]);
        j += 1;
    }
    match name.as_str() {
        "limits" => {
            *i = j;
            Some(true)
        }
        "nolimits" => {
            *i = j;
            Some(false)
        }
        _ => None,
    }
}

fn parse_atom(chars: &[char], i: &mut usize) -> Node {
    skip_ws(chars, i);
    if *i >= chars.len() {
        return Node::Text(String::new());
    }
    match chars[*i] {
        '{' => {
            *i += 1;
            let inner = parse_until(chars, i, '}');
            parse_math(&inner)
        }
        '\\' => parse_command(chars, i),
        ch if ch.is_ascii_alphabetic() => {
            // Matplotlib mathtext: variables are italic.
            *i += 1;
            Node::Italic(ch.to_string())
        }
        ch if ch.is_ascii_digit()
            || matches!(
                ch,
                '+' | '-' | '=' | '(' | ')' | '[' | ']' | '.' | ',' | '/' | '|' | '<' | '>'
            ) =>
        {
            *i += 1;
            Node::Text(ch.to_string())
        }
        ch => {
            *i += 1;
            Node::Text(ch.to_string())
        }
    }
}

fn parse_script_body(chars: &[char], i: &mut usize) -> Node {
    skip_ws(chars, i);
    if *i < chars.len() && chars[*i] == '{' {
        *i += 1;
        let inner = parse_until(chars, i, '}');
        parse_math(&inner)
    } else {
        parse_atom(chars, i)
    }
}

fn parse_command(chars: &[char], i: &mut usize) -> Node {
    *i += 1; // skip '\'
    let mut name = String::new();
    while *i < chars.len() && chars[*i].is_ascii_alphabetic() {
        name.push(chars[*i]);
        *i += 1;
    }
    // Single-char spacing / escapes: `\,` `\;` `\:` `\!` (name stays empty).
    if name.is_empty() {
        if *i < chars.len() {
            let ch = chars[*i];
            *i += 1;
            return match ch {
                ',' => Node::Space(0.1667),  // thin
                ':' => Node::Space(0.2222),  // medium
                ';' => Node::Space(0.2778),  // thick
                '!' => Node::Space(-0.1667), // negative thin
                _ => Node::Text(ch.to_string()),
            };
        }
        return Node::Text(String::new());
    }
    match name.as_str() {
        "frac" => {
            let num = Box::new(parse_script_body(chars, i));
            let den = Box::new(parse_script_body(chars, i));
            Node::Frac { num, den }
        }
        "sqrt" => {
            let index = if *i < chars.len() && chars[*i] == '[' {
                *i += 1;
                let inner = parse_until(chars, i, ']');
                Some(Box::new(parse_math(&inner)))
            } else {
                None
            };
            let body = Box::new(parse_script_body(chars, i));
            Node::Sqrt { index, body }
        }
        "begin" => parse_begin_env(chars, i),
        "quad" => Node::Space(1.0),
        "qquad" => Node::Space(2.0),
        "thinspace" => Node::Space(0.1667),
        "negthinspace" => Node::Space(-0.1667),
        // Style switches: braced group, or the remainder of the current math list.
        "displaystyle" => parse_style_body(chars, i, true),
        "textstyle" => parse_style_body(chars, i, false),
        "mathrm" | "rm" => force_italic(parse_script_body(chars, i), false),
        "mathit" | "it" => force_italic(parse_script_body(chars, i), true),
        // Consumed next to operators in `parse_limits_flag`; bare occurrences are no-ops.
        "limits" | "nolimits" => Node::Text(String::new()),
        "sin" | "cos" | "tan" | "log" | "ln" | "exp" | "max" | "min" | "lim" | "det" | "arcsin"
        | "arccos" | "arctan" | "sinh" | "cosh" | "tanh" | "sec" | "csc" | "cot" | "sup"
        | "inf" | "arg" | "ker" | "dim" | "hom" | "gcd" | "lcm" | "Pr" | "mod" => Node::Text(name),
        "left" | "right" => {
            // Consume following delimiter if present; ignore sizing.
            if *i < chars.len() && !chars[*i].is_ascii_alphabetic() {
                let ch = chars[*i];
                *i += 1;
                Node::Text(ch.to_string())
            } else {
                Node::Text(String::new())
            }
        }
        "hat" | "widehat" => {
            let body = Box::new(parse_script_body(chars, i));
            Node::Accent {
                kind: AccentKind::Hat,
                body,
            }
        }
        "bar" | "overline" => {
            let body = Box::new(parse_script_body(chars, i));
            Node::Accent {
                kind: AccentKind::Bar,
                body,
            }
        }
        "vec" => {
            let body = Box::new(parse_script_body(chars, i));
            Node::Accent {
                kind: AccentKind::Vec,
                body,
            }
        }
        "tilde" | "widetilde" => {
            let body = Box::new(parse_script_body(chars, i));
            Node::Accent {
                kind: AccentKind::Tilde,
                body,
            }
        }
        "dot" => {
            let body = Box::new(parse_script_body(chars, i));
            Node::Accent {
                kind: AccentKind::Dot,
                body,
            }
        }
        "ddot" => {
            let body = Box::new(parse_script_body(chars, i));
            Node::Accent {
                kind: AccentKind::Ddot,
                body,
            }
        }
        "underline" => {
            let body = Box::new(parse_script_body(chars, i));
            Node::Accent {
                kind: AccentKind::Underline,
                body,
            }
        }
        // Bare `\sqrt` without args used to map to Unicode; keep symbol-only aliases.
        other => match command_to_unicode(other) {
            Some(s) if is_math_letter_name(other) => Node::Italic(s),
            Some(s) => Node::Text(s),
            None => Node::Text(format!("\\{other}")),
        },
    }
}

fn parse_style_body(chars: &[char], i: &mut usize, display: bool) -> Node {
    skip_ws(chars, i);
    let body = if *i < chars.len() && chars[*i] == '{' {
        parse_script_body(chars, i)
    } else {
        let rest: String = chars[*i..].iter().collect();
        *i = chars.len();
        parse_math(&rest)
    };
    Node::Style {
        display,
        body: Box::new(body),
    }
}

fn parse_begin_env(chars: &[char], i: &mut usize) -> Node {
    skip_ws(chars, i);
    if *i >= chars.len() || chars[*i] != '{' {
        return Node::Text("\\begin".into());
    }
    *i += 1;
    let env = parse_until(chars, i, '}');
    let kind = match env.as_str() {
        "matrix" => MatrixKind::Plain,
        "pmatrix" => MatrixKind::Paren,
        "bmatrix" => MatrixKind::Bracket,
        "vmatrix" => MatrixKind::VBar,
        "Vmatrix" => MatrixKind::DoubleVBar,
        "Bmatrix" => MatrixKind::Brace,
        "smallmatrix" => MatrixKind::Small,
        other => return Node::Text(format!("\\begin{{{other}}}")),
    };
    let body = parse_until_end_env(chars, i, &env);
    let rows = parse_matrix_rows(&body);
    Node::Matrix { kind, rows }
}

fn parse_until_end_env(chars: &[char], i: &mut usize, env: &str) -> String {
    let needle: Vec<char> = format!("\\end{{{env}}}").chars().collect();
    let mut out = String::new();
    while *i < chars.len() {
        if chars[*i..].starts_with(&needle) {
            *i += needle.len();
            break;
        }
        out.push(chars[*i]);
        *i += 1;
    }
    out
}

fn parse_matrix_rows(body: &str) -> Vec<Vec<Node>> {
    let chars: Vec<char> = body.chars().collect();
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut i = 0usize;
    let flush_cell = |cell: &mut String, row: &mut Vec<Node>| {
        let t = cell.trim();
        if !t.is_empty() {
            row.push(parse_math(t));
        } else {
            row.push(Node::Text(String::new()));
        }
        cell.clear();
    };
    while i < chars.len() {
        if chars[i] == '&' {
            flush_cell(&mut cell, &mut row);
            i += 1;
            continue;
        }
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '\\' {
            flush_cell(&mut cell, &mut row);
            rows.push(std::mem::take(&mut row));
            i += 2;
            // Optional whitespace / `\cr`-style trailing spaces
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            continue;
        }
        cell.push(chars[i]);
        i += 1;
    }
    let trimmed = cell.trim();
    if !trimmed.is_empty() || !row.is_empty() {
        flush_cell(&mut cell, &mut row);
        rows.push(row);
    }
    if rows.is_empty() {
        rows.push(vec![Node::Text(String::new())]);
    }
    rows
}

fn force_italic(node: Node, italic: bool) -> Node {
    match node {
        Node::Text(s) | Node::Italic(s) => {
            if italic {
                Node::Italic(s)
            } else {
                Node::Text(s)
            }
        }
        Node::List(items) => {
            Node::List(items.into_iter().map(|n| force_italic(n, italic)).collect())
        }
        Node::Script {
            base,
            sup,
            sub,
            limits,
        } => Node::Script {
            base: Box::new(force_italic(*base, italic)),
            sup: sup.map(|n| Box::new(force_italic(*n, italic))),
            sub: sub.map(|n| Box::new(force_italic(*n, italic))),
            limits,
        },
        Node::Style { display, body } => Node::Style {
            display,
            body: Box::new(force_italic(*body, italic)),
        },
        Node::Frac { num, den } => Node::Frac {
            num: Box::new(force_italic(*num, italic)),
            den: Box::new(force_italic(*den, italic)),
        },
        Node::Sqrt { index, body } => Node::Sqrt {
            index: index.map(|n| Box::new(force_italic(*n, italic))),
            body: Box::new(force_italic(*body, italic)),
        },
        Node::Accent { kind, body } => Node::Accent {
            kind,
            body: Box::new(force_italic(*body, italic)),
        },
        Node::Matrix { kind, rows } => Node::Matrix {
            kind,
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(|n| force_italic(n, italic)).collect())
                .collect(),
        },
        other => other,
    }
}

fn is_math_letter_name(name: &str) -> bool {
    matches!(
        name,
        "alpha"
            | "beta"
            | "gamma"
            | "delta"
            | "epsilon"
            | "varepsilon"
            | "zeta"
            | "eta"
            | "theta"
            | "vartheta"
            | "iota"
            | "kappa"
            | "lambda"
            | "mu"
            | "nu"
            | "xi"
            | "pi"
            | "varpi"
            | "rho"
            | "varrho"
            | "sigma"
            | "varsigma"
            | "tau"
            | "upsilon"
            | "phi"
            | "varphi"
            | "chi"
            | "psi"
            | "omega"
            | "Gamma"
            | "Delta"
            | "Theta"
            | "Lambda"
            | "Xi"
            | "Pi"
            | "Sigma"
            | "Upsilon"
            | "Phi"
            | "Psi"
            | "Omega"
            | "ell"
            | "imath"
            | "jmath"
            | "partial"
    )
}

fn command_to_unicode(name: &str) -> Option<String> {
    let s = match name {
        "alpha" => math::ALPHA,
        "beta" => math::BETA,
        "gamma" => math::GAMMA,
        "delta" => math::DELTA,
        "epsilon" | "varepsilon" => math::EPSILON,
        "zeta" => math::ZETA,
        "eta" => math::ETA,
        "theta" | "vartheta" => math::THETA,
        "iota" => math::IOTA,
        "kappa" => math::KAPPA,
        "lambda" => math::LAMBDA,
        "mu" => math::MU,
        "nu" => math::NU,
        "xi" => math::XI,
        "pi" => math::PI,
        "rho" | "varrho" => math::RHO,
        "sigma" | "varsigma" => math::SIGMA,
        "tau" => math::TAU,
        "upsilon" => math::UPSILON,
        "phi" | "varphi" => math::PHI,
        "chi" => math::CHI,
        "psi" => math::PSI,
        "omega" => math::OMEGA,
        "Gamma" => math::GAMMA_U,
        "Delta" => math::DELTA_U,
        "Theta" => math::THETA_U,
        "Lambda" => math::LAMBDA_U,
        "Xi" => math::XI_U,
        "Pi" => math::PI_U,
        "Sigma" => math::SIGMA_U,
        "Phi" => math::PHI_U,
        "Psi" => math::PSI_U,
        "Omega" => math::OMEGA_U,
        "infty" => math::INFTY,
        "partial" => math::PARTIAL,
        "approx" => math::APPROX,
        "cdot" => math::CDOT,
        "times" => math::TIMES,
        "leq" => math::LEQ,
        "geq" => math::GEQ,
        "pm" => math::PM,
        "circ" | "deg" => math::DEG,
        "neq" => "\u{2260}",
        "rightarrow" => "\u{2192}",
        "leftarrow" => "\u{2190}",
        "uparrow" => "\u{2191}",
        "downarrow" => "\u{2193}",
        "leftrightarrow" => "\u{2194}",
        "Rightarrow" => "\u{21D2}",
        "Leftarrow" => "\u{21D0}",
        "Leftrightarrow" => "\u{21D4}",
        "mapsto" => "\u{21A6}",
        "longrightarrow" => "\u{27F6}",
        "longleftarrow" => "\u{27F5}",
        "sum" => "\u{2211}",
        "int" => "\u{222B}",
        "prod" => "\u{220F}",
        "iint" => "\u{222C}",
        "iiint" => "\u{222D}",
        "oint" => "\u{222E}",
        "nabla" => "\u{2207}",
        "forall" => "\u{2200}",
        "exists" => "\u{2203}",
        "nexists" => "\u{2204}",
        "in" => "\u{2208}",
        "notin" => "\u{2209}",
        "subset" => "\u{2282}",
        "supset" => "\u{2283}",
        "subseteq" => "\u{2286}",
        "supseteq" => "\u{2287}",
        "cup" => "\u{222A}",
        "cap" => "\u{2229}",
        "emptyset" => "\u{2205}",
        "wedge" | "land" => "\u{2227}",
        "vee" | "lor" => "\u{2228}",
        "neg" | "lnot" => "\u{00AC}",
        "mp" => "\u{2213}",
        "div" => "\u{00F7}",
        "ast" => "\u{2217}",
        "star" => "\u{22C6}",
        "oplus" => "\u{2295}",
        "otimes" => "\u{2297}",
        "odot" => "\u{2299}",
        "ll" => "\u{226A}",
        "gg" => "\u{226B}",
        "prec" => "\u{227A}",
        "succ" => "\u{227B}",
        "sim" => "\u{223C}",
        "simeq" => "\u{2243}",
        "cong" => "\u{2245}",
        "equiv" => "\u{2261}",
        "propto" => "\u{221D}",
        "perp" => "\u{22A5}",
        "parallel" => "\u{2225}",
        "angle" => "\u{2220}",
        "Re" => "\u{211C}",
        "Im" => "\u{2111}",
        "hbar" => "\u{210F}",
        "ell" => "\u{2113}",
        "wp" => "\u{2118}",
        "aleph" => "\u{2135}",
        "ldots" | "dots" => "\u{2026}",
        "cdots" => "\u{22EF}",
        "vdots" => "\u{22EE}",
        "ddots" => "\u{22F1}",
        "dagger" => "\u{2020}",
        "ddagger" => "\u{2021}",
        "bullet" => "\u{2022}",
        "prime" => "\u{2032}",
        "triangle" => "\u{25B3}",
        "diamond" => "\u{25C7}",
        "square" => "\u{25A1}",
        "clubsuit" => "\u{2663}",
        "heartsuit" => "\u{2665}",
        "spadesuit" => "\u{2660}",
        "diamondsuit" => "\u{2666}",
        "checkmark" => "\u{2713}",
        "therefore" => "\u{2234}",
        "because" => "\u{2235}",
        "cdotp" => math::CDOT,
        // aliases
        "to" => "\u{2192}",
        "gets" => "\u{2190}",
        "iff" => "\u{27FA}",
        "implies" => "\u{27F9}",
        // additional arrows
        "hookrightarrow" => "\u{21AA}",
        "hookleftarrow" => "\u{21A9}",
        "nearrow" => "\u{2197}",
        "searrow" => "\u{2198}",
        "nwarrow" => "\u{2199}",
        "swarrow" => "\u{2198}",
        "longmapsto" => "\u{27FC}",
        "longleftrightarrow" => "\u{27F7}",
        "Longrightarrow" => "\u{27F9}",
        "Longleftarrow" => "\u{27F8}",
        "Longleftrightarrow" => "\u{27FA}",
        "Uparrow" => "\u{21D1}",
        "Downarrow" => "\u{21D3}",
        "Updownarrow" => "\u{21D5}",
        "updownarrow" => "\u{2195}",
        "rightharpoonup" => "\u{21C0}",
        "rightharpoondown" => "\u{21C1}",
        "leftharpoonup" => "\u{21BC}",
        "leftharpoondown" => "\u{21BD}",
        "rightleftharpoons" => "\u{21CC}",
        // additional relations
        "ni" | "owns" => "\u{220B}",
        "subsetneq" => "\u{228A}",
        "supsetneq" => "\u{228B}",
        "nsubseteq" => "\u{2288}",
        "nsupseteq" => "\u{2289}",
        "varnothing" => "\u{2205}",
        "complement" => "\u{2201}",
        "top" => "\u{22A4}",
        "bot" => "\u{22A5}",
        "vdash" => "\u{22A2}",
        "dashv" => "\u{22A3}",
        "models" => "\u{22A8}",
        "mid" => "\u{2223}",
        "nmid" => "\u{2224}",
        "nparallel" => "\u{2226}",
        "asymp" => "\u{224D}",
        "doteq" => "\u{2250}",
        "triangleq" => "\u{225C}",
        "lesssim" => "\u{2272}",
        "gtrsim" => "\u{2273}",
        "leqslant" => "\u{2A7D}",
        "geqslant" => "\u{2A7E}",
        // additional operators
        "setminus" => "\u{2216}",
        "wr" => "\u{2240}",
        "amalg" => "\u{2A3F}",
        "coprod" => "\u{2210}",
        "bigcup" => "\u{22C3}",
        "bigcap" => "\u{22C2}",
        "bigvee" => "\u{22C1}",
        "bigwedge" => "\u{22C0}",
        "bigodot" => "\u{2A00}",
        "bigoplus" => "\u{2A01}",
        "bigotimes" => "\u{2A02}",
        "biguplus" => "\u{2A04}",
        "bigsqcup" => "\u{2A06}",
        "sqcup" => "\u{2294}",
        "sqcap" => "\u{2293}",
        "sqsubseteq" => "\u{2291}",
        "sqsupseteq" => "\u{2292}",
        "uplus" => "\u{228E}",
        "triangleleft" => "\u{25C3}",
        "triangleright" => "\u{25B9}",
        "lhd" => "\u{22B2}",
        "rhd" => "\u{22B3}",
        "unlhd" => "\u{22B4}",
        "unrhd" => "\u{22B5}",
        // delimiters
        "langle" => "\u{27E8}",
        "rangle" => "\u{27E9}",
        "lceil" => "\u{2308}",
        "rceil" => "\u{2309}",
        "lfloor" => "\u{230A}",
        "rfloor" => "\u{230B}",
        "lbrace" => "{",
        "rbrace" => "}",
        "lbrack" => "[",
        "rbrack" => "]",
        "lvert" | "vert" => "\u{2223}",
        "rvert" => "\u{2223}",
        "lVert" | "Vert" => "\u{2225}",
        "rVert" => "\u{2225}",
        // additional greek
        "varkappa" => "\u{03F0}",
        "varpi" => "\u{03D6}",
        "digamma" => "\u{03DD}",
        // dotless letters
        "imath" => "\u{0131}",
        "jmath" => "\u{0237}",
        // music / misc
        "flat" => "\u{266D}",
        "sharp" => "\u{266F}",
        "natural" => "\u{266E}",
        "blacksquare" => "\u{25A0}",
        "lozenge" => "\u{25CA}",
        "blacklozenge" => "\u{29EB}",
        "blacktriangle" => "\u{25B2}",
        "blacktriangledown" => "\u{25BC}",
        "triangledown" => "\u{25BD}",
        "bigstar" => "\u{2605}",
        "maltese" => "\u{2720}",
        "degree" => "\u{00B0}",
        "backslash" => "\\",
        "S" => "\u{00A7}",
        "P" => "\u{00B6}",
        "copyright" => "\u{00A9}",
        _ => return None,
    };
    Some(s.to_string())
}

fn parse_until(chars: &[char], i: &mut usize, end: char) -> String {
    let mut depth = 0usize;
    let mut out = String::new();
    while *i < chars.len() {
        let ch = chars[*i];
        *i += 1;
        if ch == '{' {
            depth += 1;
            out.push(ch);
        } else if ch == '}' {
            if depth == 0 && end == '}' {
                break;
            }
            depth = depth.saturating_sub(1);
            out.push(ch);
        } else if ch == end && depth == 0 {
            break;
        } else {
            out.push(ch);
        }
    }
    out
}

fn skip_ws(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i].is_whitespace() {
        *i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_math() {
        assert!(needs_mathtext(r"$x^2$"));
        assert!(needs_mathtext(r"a $b$ c"));
        assert!(!needs_mathtext("plain"));
        assert!(!needs_mathtext("unclosed $"));
    }

    #[test]
    fn parses_frac_and_script() {
        let n = parse_math(r"e^{-t} + \frac{a}{b}");
        match n {
            Node::List(items) => assert!(items.len() >= 3),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn parses_sqrt_with_index() {
        let n = parse_math(r"\sqrt[3]{8}");
        match n {
            Node::Sqrt {
                index: Some(_),
                body: _,
            } => {}
            other => panic!("expected sqrt, got {other:?}"),
        }
    }

    #[test]
    fn parses_hat_accent() {
        let n = parse_math(r"\hat{x}");
        match n {
            Node::Accent {
                kind: AccentKind::Hat,
                ..
            } => {}
            other => panic!("expected accent, got {other:?}"),
        }
    }

    #[test]
    fn parses_pmatrix() {
        let n = parse_math(r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}");
        match n {
            Node::Matrix {
                kind: MatrixKind::Paren,
                rows,
            } => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
                assert_eq!(rows[1].len(), 2);
            }
            other => panic!("expected matrix, got {other:?}"),
        }
    }

    #[test]
    fn parses_thin_and_neg_space() {
        let n = parse_math(r"a\,b\!c");
        match n {
            Node::List(items) => {
                assert!(matches!(items[1], Node::Space(w) if (w - 0.1667).abs() < 1e-4));
                assert!(matches!(items[3], Node::Space(w) if (w + 0.1667).abs() < 1e-4));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn maps_common_set_symbols() {
        assert_eq!(command_to_unicode("emptyset").as_deref(), Some("\u{2205}"));
        assert_eq!(command_to_unicode("subseteq").as_deref(), Some("\u{2286}"));
        assert_eq!(command_to_unicode("nabla").as_deref(), Some("\u{2207}"));
    }

    #[test]
    fn parses_int_with_limits() {
        let n = parse_math(r"\int_0^1");
        match n {
            Node::Script {
                base,
                sup: Some(sup),
                sub: Some(sub),
                limits: None,
            } => {
                assert!(matches!(base.as_ref(), Node::Text(s) if s.contains('\u{222B}')));
                assert!(matches!(sup.as_ref(), Node::Text(s) if s == "1"));
                assert!(matches!(sub.as_ref(), Node::Text(s) if s == "0"));
            }
            other => panic!("expected Script with auto limits, got {other:?}"),
        }
    }

    #[test]
    fn parses_int_limits_flag() {
        let n = parse_math(r"\int\limits_0^1");
        match n {
            Node::Script {
                limits: Some(true), ..
            } => {}
            other => panic!("expected Script with \\limits, got {other:?}"),
        }
    }

    #[test]
    fn parses_displaystyle() {
        let n = parse_math(r"\displaystyle\int_0^1");
        match n {
            Node::Style {
                display: true,
                body,
            } => match body.as_ref() {
                Node::Script { .. } => {}
                other => panic!("expected Script body, got {other:?}"),
            },
            other => panic!("expected Style, got {other:?}"),
        }
    }

    #[test]
    fn letters_are_italic_in_math() {
        let n = parse_mixed("$e^{-t}\\sin(2t)$");
        fn has_italic(n: &Node) -> bool {
            match n {
                Node::Italic(_) => true,
                Node::List(items) => items.iter().any(has_italic),
                Node::Script { base, sup, sub, .. } => {
                    has_italic(base)
                        || sup.as_deref().is_some_and(has_italic)
                        || sub.as_deref().is_some_and(has_italic)
                }
                Node::Style { body, .. } => has_italic(body),
                _ => false,
            }
        }
        assert!(has_italic(&n), "expected italic variables in {n:?}");
        fn has_sin(n: &Node) -> bool {
            match n {
                Node::Text(s) if s == "sin" => true,
                Node::List(items) => items.iter().any(has_sin),
                Node::Script { base, sup, sub, .. } => {
                    has_sin(base)
                        || sup.as_deref().is_some_and(has_sin)
                        || sub.as_deref().is_some_and(has_sin)
                }
                _ => false,
            }
        }
        assert!(has_sin(&n), "expected upright sin in {n:?}");
    }
}
