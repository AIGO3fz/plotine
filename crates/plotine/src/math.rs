//! Lightweight Unicode helpers for axis labels and titles.
//!
//! This module is a **pure string** path: TeX-like markup is rewritten to UTF-8
//! (Greek letters, superscripts / subscripts) and rendered as ordinary text with
//! DejaVu Sans. Prefer writing Unicode directly, or use the helpers below when
//! generating labels from ASCII / agent codegen.
//!
//! For laid-out math (`\frac`, nested scripts, inline gaps), use
//! [`crate::mathtext`] with `$...$` in titles / labels — that path does **not**
//! require an external LaTeX binary.
//!
//! # Examples
//!
//! ```
//! use plotine::math::{self, unicode};
//!
//! assert_eq!(format!("{}{}", "x", math::sup("2")), "x\u{00B2}");
//! assert_eq!(format!("H{}O", math::sub("2")), "H\u{2082}O");
//! assert_eq!(unicode(r"$\alpha$ vs $x^2$"), "\u{03B1} vs x\u{00B2}");
//! ```

/// Greek alpha
pub const ALPHA: &str = "\u{03B1}";
/// Greek beta
pub const BETA: &str = "\u{03B2}";
/// Greek gamma
pub const GAMMA: &str = "\u{03B3}";
/// Greek delta
pub const DELTA: &str = "\u{03B4}";
/// Greek epsilon
pub const EPSILON: &str = "\u{03B5}";
/// Greek zeta
pub const ZETA: &str = "\u{03B6}";
/// Greek eta
pub const ETA: &str = "\u{03B7}";
/// Greek theta
pub const THETA: &str = "\u{03B8}";
/// Greek iota
pub const IOTA: &str = "\u{03B9}";
/// Greek kappa
pub const KAPPA: &str = "\u{03BA}";
/// Greek lambda
pub const LAMBDA: &str = "\u{03BB}";
/// Greek mu
pub const MU: &str = "\u{03BC}";
/// Greek nu
pub const NU: &str = "\u{03BD}";
/// Greek xi
pub const XI: &str = "\u{03BE}";
/// Greek pi
pub const PI: &str = "\u{03C0}";
/// Greek rho
pub const RHO: &str = "\u{03C1}";
/// Greek sigma
pub const SIGMA: &str = "\u{03C3}";
/// Greek tau
pub const TAU: &str = "\u{03C4}";
/// Greek upsilon
pub const UPSILON: &str = "\u{03C5}";
/// Greek phi
pub const PHI: &str = "\u{03C6}";
/// Greek chi
pub const CHI: &str = "\u{03C7}";
/// Greek psi
pub const PSI: &str = "\u{03C8}";
/// Greek omega
pub const OMEGA: &str = "\u{03C9}";

/// Greek Gamma
pub const GAMMA_U: &str = "\u{0393}";
/// Greek Delta
pub const DELTA_U: &str = "\u{0394}";
/// Greek Theta
pub const THETA_U: &str = "\u{0398}";
/// Greek Lambda
pub const LAMBDA_U: &str = "\u{039B}";
/// Greek Xi
pub const XI_U: &str = "\u{039E}";
/// Greek Pi
pub const PI_U: &str = "\u{03A0}";
/// Greek Sigma
pub const SIGMA_U: &str = "\u{03A3}";
/// Greek Phi
pub const PHI_U: &str = "\u{03A6}";
/// Greek Psi
pub const PSI_U: &str = "\u{03A8}";
/// Greek Omega
pub const OMEGA_U: &str = "\u{03A9}";

/// Infinity
pub const INFTY: &str = "\u{221E}";
/// Plus-minus
pub const PM: &str = "\u{00B1}";
/// Less-or-equal
pub const LEQ: &str = "\u{2264}";
/// Greater-or-equal
pub const GEQ: &str = "\u{2265}";
/// Approximately equal
pub const APPROX: &str = "\u{2248}";
/// Degree
pub const DEG: &str = "\u{00B0}";
/// Multiplication sign
pub const TIMES: &str = "\u{00D7}";
/// Middle dot
pub const CDOT: &str = "\u{00B7}";
/// Partial derivative
pub const PARTIAL: &str = "\u{2202}";
/// Square root symbol (not a layout radical)
pub const SQRT: &str = "\u{221A}";

/// Map ASCII digits / letters / signs to Unicode superscript characters.
///
/// Unknown characters are left unchanged.
pub fn sup(s: impl AsRef<str>) -> String {
    s.as_ref().chars().map(to_sup).collect()
}

/// Map ASCII digits / letters / signs to Unicode subscript characters.
///
/// Unknown characters are left unchanged.
pub fn sub(s: impl AsRef<str>) -> String {
    s.as_ref().chars().map(to_sub).collect()
}

/// Convert a limited TeX-like markup string into Unicode for labels.
///
/// Supported (no layout — pure string rewrite):
/// - Greek / symbols: `\alpha` … `\omega`, `\Gamma` … `\Omega`, `\infty`,
///   `\pm`, `\leq`, `\geq`, `\approx`, `\cdot`, `\times`, `\circ`, `\partial`, `\sqrt`
/// - Superscripts: `x^2`, `x^{10}`, `e^{-1}`
/// - Subscripts: `H_2`, `a_{i}`, `x_{10}`
/// - Paired `$...$` delimiters (including multiple segments) are stripped
///
/// Not supported: `\frac`, nested scripts, matrices, font switches.
/// Use [`crate::mathtext`] when you need layout, not just Unicode rewriting.
pub fn unicode(input: impl AsRef<str>) -> String {
    let s = strip_math_dollars(input.as_ref());
    let s = replace_commands(&s);
    apply_scripts(&s)
}

/// Remove paired `$...$` delimiters; unpaired `$` are kept as-is.
fn strip_math_dollars(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' {
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '$') {
                let close = i + 1 + end;
                out.extend(chars[i + 1..close].iter().copied());
                i = close + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn to_sup(c: char) -> char {
    match c {
        '0' => '\u{2070}',
        '1' => '\u{00B9}',
        '2' => '\u{00B2}',
        '3' => '\u{00B3}',
        '4' => '\u{2074}',
        '5' => '\u{2075}',
        '6' => '\u{2076}',
        '7' => '\u{2077}',
        '8' => '\u{2078}',
        '9' => '\u{2079}',
        '+' => '\u{207A}',
        '-' | '\u{2212}' => '\u{207B}',
        '=' => '\u{207C}',
        '(' => '\u{207D}',
        ')' => '\u{207E}',
        'a' => '\u{1D43}',
        'b' => '\u{1D47}',
        'c' => '\u{1D9C}',
        'd' => '\u{1D48}',
        'e' => '\u{1D49}',
        'f' => '\u{1DA0}',
        'g' => '\u{1D4D}',
        'h' => '\u{02B0}',
        'i' => '\u{2071}',
        'j' => '\u{02B2}',
        'k' => '\u{1D4F}',
        'l' => '\u{02E1}',
        'm' => '\u{1D50}',
        'n' => '\u{207F}',
        'o' => '\u{1D52}',
        'p' => '\u{1D56}',
        'r' => '\u{02B3}',
        's' => '\u{02E2}',
        't' => '\u{1D57}',
        'u' => '\u{1D58}',
        'v' => '\u{1D5B}',
        'w' => '\u{02B7}',
        'x' => '\u{02E3}',
        'y' => '\u{02B8}',
        'z' => '\u{1DBB}',
        other => other,
    }
}

fn to_sub(c: char) -> char {
    match c {
        '0' => '\u{2080}',
        '1' => '\u{2081}',
        '2' => '\u{2082}',
        '3' => '\u{2083}',
        '4' => '\u{2084}',
        '5' => '\u{2085}',
        '6' => '\u{2086}',
        '7' => '\u{2087}',
        '8' => '\u{2088}',
        '9' => '\u{2089}',
        '+' => '\u{208A}',
        '-' | '\u{2212}' => '\u{208B}',
        '=' => '\u{208C}',
        '(' => '\u{208D}',
        ')' => '\u{208E}',
        'a' => '\u{2090}',
        'e' => '\u{2091}',
        'h' => '\u{2095}',
        'i' => '\u{1D62}',
        'j' => '\u{2C7C}',
        'k' => '\u{2096}',
        'l' => '\u{2097}',
        'm' => '\u{2098}',
        'n' => '\u{2099}',
        'o' => '\u{2092}',
        'p' => '\u{209A}',
        'r' => '\u{1D63}',
        's' => '\u{209B}',
        't' => '\u{209C}',
        'u' => '\u{1D64}',
        'v' => '\u{1D65}',
        'x' => '\u{2093}',
        other => other,
    }
}

fn replace_commands(s: &str) -> String {
    const CMDS: &[(&str, &str)] = &[
        ("\\varepsilon", EPSILON),
        ("\\vartheta", "\u{03D1}"),
        ("\\varphi", PHI),
        ("\\varrho", RHO),
        ("\\varsigma", "\u{03C2}"),
        ("\\alpha", ALPHA),
        ("\\beta", BETA),
        ("\\gamma", GAMMA),
        ("\\delta", DELTA),
        ("\\epsilon", EPSILON),
        ("\\zeta", ZETA),
        ("\\eta", ETA),
        ("\\theta", THETA),
        ("\\iota", IOTA),
        ("\\kappa", KAPPA),
        ("\\lambda", LAMBDA),
        ("\\mu", MU),
        ("\\nu", NU),
        ("\\xi", XI),
        ("\\pi", PI),
        ("\\rho", RHO),
        ("\\sigma", SIGMA),
        ("\\tau", TAU),
        ("\\upsilon", UPSILON),
        ("\\phi", PHI),
        ("\\chi", CHI),
        ("\\psi", PSI),
        ("\\omega", OMEGA),
        ("\\Gamma", GAMMA_U),
        ("\\Delta", DELTA_U),
        ("\\Theta", THETA_U),
        ("\\Lambda", LAMBDA_U),
        ("\\Xi", XI_U),
        ("\\Pi", PI_U),
        ("\\Sigma", SIGMA_U),
        ("\\Phi", PHI_U),
        ("\\Psi", PSI_U),
        ("\\Omega", OMEGA_U),
        ("\\infty", INFTY),
        ("\\partial", PARTIAL),
        ("\\approx", APPROX),
        ("\\cdot", CDOT),
        ("\\times", TIMES),
        ("\\leq", LEQ),
        ("\\geq", GEQ),
        ("\\pm", PM),
        ("\\circ", DEG),
        ("\\deg", DEG),
        ("\\sqrt", SQRT),
    ];
    let mut out = s.to_string();
    for &(cmd, repl) in CMDS {
        out = out.replace(cmd, repl);
    }
    out
}

fn apply_scripts(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if (c == '^' || c == '_') && i + 1 < chars.len() {
            let map = if c == '^' { to_sup } else { to_sub };
            if chars[i + 1] == '{' {
                if let Some(end) = find_closing_brace(&chars, i + 1) {
                    let body: String = chars[i + 2..end].iter().collect();
                    out.push_str(&body.chars().map(map).collect::<String>());
                    i = end + 1;
                    continue;
                }
            } else {
                out.push(map(chars[i + 1]));
                i += 2;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

fn find_closing_brace(chars: &[char], open_at: usize) -> Option<usize> {
    let mut depth = 0isize;
    for (j, &ch) in chars.iter().enumerate().skip(open_at) {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sup_digits() {
        assert_eq!(sup("210"), "\u{00B2}\u{00B9}\u{2070}");
        assert_eq!(sup("-1"), "\u{207B}\u{00B9}");
    }

    #[test]
    fn sub_digits() {
        assert_eq!(sub("2"), "\u{2082}");
        assert_eq!(format!("H{}O", sub("2")), "H\u{2082}O");
    }

    #[test]
    fn unicode_greek_and_scripts() {
        assert_eq!(unicode(r"\alpha"), ALPHA);
        assert_eq!(unicode(r"$x^2$"), "x\u{00B2}");
        assert_eq!(unicode(r"H_2O"), "H\u{2082}O");
        assert_eq!(unicode(r"e^{-1}"), "e\u{207B}\u{00B9}");
        assert_eq!(unicode(r"x^{10}"), "x\u{00B9}\u{2070}");
        assert_eq!(unicode(r"a_{i}"), "a\u{1D62}");
        assert_eq!(unicode(r"\theta (rad)"), format!("{} (rad)", THETA));
        assert_eq!(unicode(r"\infty"), INFTY);
    }

    #[test]
    fn unicode_strips_multiple_math_segments() {
        assert_eq!(
            unicode(r"$\alpha$ vs $x^2$"),
            format!("{ALPHA} vs x\u{00B2}")
        );
        assert_eq!(unicode(r"$t$ (s)"), "t (s)");
        assert_eq!(unicode(r"plain $x$ and $y$"), "plain x and y");
    }

    #[test]
    fn unicode_keeps_unpaired_dollar() {
        assert_eq!(unicode("cost $5"), "cost $5");
    }

    #[test]
    fn unknown_chars_passthrough() {
        assert_eq!(sup("!"), "!");
        assert_eq!(unicode("plain"), "plain");
    }
}
