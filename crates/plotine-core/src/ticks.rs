//! Tick location for linear / log / symlog axes.

use crate::scale::{LinearScale, LogScale, Scale, ScaleKind, SymlogScale};

/// A single major tick with its data value and formatted label.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    pub value: f64,
    pub label: String,
}

impl Tick {
    /// Format a data value with the same rules as the auto locator.
    pub fn from_value(value: f64) -> Self {
        Self {
            value,
            label: format_tick(value),
        }
    }
}

/// Locates aesthetically pleasing ticks inside a scale domain.
#[derive(Debug, Clone, Copy)]
pub struct TickLocator {
    pub target_count: usize,
}

impl Default for TickLocator {
    fn default() -> Self {
        Self { target_count: 6 }
    }
}

impl TickLocator {
    pub fn new(target_count: usize) -> Self {
        Self {
            target_count: target_count.max(2),
        }
    }

    pub fn ticks(&self, scale: ScaleKind) -> Vec<Tick> {
        match scale {
            ScaleKind::Linear(s) => self.ticks_linear(s),
            ScaleKind::Log(s) => self.ticks_log(s),
            ScaleKind::Symlog(s) => self.ticks_symlog(s),
        }
    }

    /// Compute major ticks for a linear scale using a simplified Wilkinson/nice-number approach.
    pub fn ticks_linear(&self, scale: LinearScale) -> Vec<Tick> {
        let (min, max) = scale.domain();
        let span = max - min;
        if !span.is_finite() || span <= 0.0 {
            return vec![Tick {
                value: min,
                label: format_tick(min),
            }];
        }

        // `target_count - 1` ≈ MaxNLocator `nbins` (max intervals). Use the
        // ceiling nice-number path so step is never smaller than `rough`
        // (round-to-nearest was producing 0.5 when nbins required ≥1.0).
        let rough = span / (self.target_count as f64 - 1.0).max(1.0);
        let mut step = nice_number(rough, false);
        if step <= 0.0 || !step.is_finite() {
            return vec![
                Tick {
                    value: min,
                    label: format_tick(min),
                },
                Tick {
                    value: max,
                    label: format_tick(max),
                },
            ];
        }

        let mut values = linear_tick_values(min, max, step);
        // MaxNLocator `min_n_ticks` (default 2): if the ceiling step left too
        // few in-domain ticks (tiny inset axes), step down the nice ladder
        // instead of labeling raw padded endpoints (`0.114` / `0.914`).
        const MIN_N_TICKS: usize = 2;
        let mut guard = 0;
        while values.len() < MIN_N_TICKS && guard < 8 {
            let smaller = next_smaller_nice(step);
            if smaller <= 0.0 || !smaller.is_finite() || smaller >= step * (1.0 - 1e-12) {
                break;
            }
            step = smaller;
            values = linear_tick_values(min, max, step);
            guard += 1;
        }

        if values.is_empty() {
            values.push(min);
            if (max - min).abs() > f64::EPSILON {
                values.push(max);
            }
        }

        let labels = format_aligned_ticks(&values, step);
        values
            .into_iter()
            .zip(labels)
            .map(|(value, label)| Tick { label, value })
            .collect()
    }

    pub fn ticks_log(&self, scale: LogScale) -> Vec<Tick> {
        let (min, max) = scale.domain();
        let mut values = Vec::new();
        let start_exp = min.log10().floor() as i32;
        let end_exp = max.log10().ceil() as i32;
        for exp in start_exp..=end_exp {
            let v = 10f64.powi(exp);
            if v >= min * (1.0 - 1e-12) && v <= max * (1.0 + 1e-12) {
                values.push(v);
            }
        }
        if values.is_empty() {
            values.push(min);
            if (max - min).abs() > f64::EPSILON * min {
                values.push(max);
            }
        }
        values
            .into_iter()
            .map(|value| Tick {
                label: format_log_tick(value),
                value,
            })
            .collect()
    }

    pub fn ticks_symlog(&self, scale: SymlogScale) -> Vec<Tick> {
        // Reuse linear locator on the data domain; good enough for M2.
        match LinearScale::new(scale.min(), scale.max()) {
            Ok(linear) => self.ticks_linear(linear),
            Err(_) => vec![Tick {
                value: 0.0,
                label: "0".into(),
            }],
        }
    }
}

fn linear_tick_values(min: f64, max: f64, step: f64) -> Vec<f64> {
    let start = (min / step).ceil() * step;
    let mut values = Vec::new();
    let mut v = start;
    let mut guard = 0;
    while v <= max + step * 1e-9 && guard < 64 {
        if v >= min - step * 1e-9 {
            // Keep nice step values — do not snap to raw domain endpoints
            // (that produced labels like `5.551e-17` / long floats).
            let mut tick_v = v;
            if tick_v.abs() < step * 1e-10 {
                tick_v = 0.0;
            }
            if values
                .last()
                .map(|last: &f64| (tick_v - *last).abs() > step * 1e-9)
                .unwrap_or(true)
            {
                values.push(tick_v);
            }
        }
        v += step;
        guard += 1;
    }
    values
}

/// Next-smaller AutoLocator step on the `[1, 2, 2.5, 5, 10]` ladder.
fn next_smaller_nice(step: f64) -> f64 {
    if step <= 0.0 || !step.is_finite() {
        return step;
    }
    let exp = step.log10().floor();
    let base = 10f64.powf(exp);
    let frac = step / base;
    let (next_frac, next_exp) = if (frac - 10.0).abs() < 1e-9 {
        (5.0, exp)
    } else if (frac - 5.0).abs() < 1e-9 {
        (2.5, exp)
    } else if (frac - 2.5).abs() < 1e-9 {
        (2.0, exp)
    } else if (frac - 2.0).abs() < 1e-9 {
        (1.0, exp)
    } else if (frac - 1.0).abs() < 1e-9 {
        (5.0, exp - 1.0)
    } else {
        // Non-canonical step: halve, then re-nice.
        return nice_number(step * 0.5, true);
    };
    next_frac * 10f64.powf(next_exp)
}

fn nice_number(x: f64, round: bool) -> f64 {
    if x <= 0.0 || !x.is_finite() {
        return 1.0;
    }
    let exp = x.log10().floor();
    let frac = x / 10f64.powf(exp);
    // `round=true` (step from rough span/nbins): AutoLocator-ish `[1, 2, 5, 10]`.
    // Thresholds bias toward *larger* steps (fewer ticks): rough=0.33 → 0.5,
    // rough=3 → 5 (not 2). Keep 2.5 only in the `round=false` ceiling path.
    let nice_frac = if round {
        if frac < 1.5 {
            1.0
        } else if frac < 2.25 {
            2.0
        } else if frac < 7.0 {
            5.0
        } else {
            10.0
        }
    } else if frac <= 1.0 {
        1.0
    } else if frac <= 2.0 {
        2.0
    } else if frac <= 2.5 {
        2.5
    } else if frac <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice_frac * 10f64.powf(exp)
}

fn format_tick(value: f64) -> String {
    format_tick_with_step(value, 1.0)
}

/// Build ticks from explicit values with axis-wide decimal alignment.
pub fn ticks_from_values(values: &[f64]) -> Vec<Tick> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut step = f64::INFINITY;
    for w in values.windows(2) {
        let d = (w[1] - w[0]).abs();
        if d.is_finite() && d > 0.0 {
            step = step.min(d);
        }
    }
    if !step.is_finite() {
        step = 1.0;
    }
    let labels = format_aligned_ticks(values, step);
    values
        .iter()
        .copied()
        .zip(labels)
        .map(|(value, label)| Tick { value, label })
        .collect()
}

/// Hard cap on fractional digits for auto tick labels (avoids float-noise
/// labels like `0.913580` on tiny inset axes).
pub const MAX_TICK_DECIMALS: usize = 3;

/// Format every tick on an axis to the same decimal width.
///
/// Decimals = max(step precision, fractional digits present in any value),
/// capped at [`MAX_TICK_DECIMALS`], so `1 / 0.75 / 0.5` becomes
/// `1.00 / 0.75 / 0.50`. Integer-only axes stay compact (`0`, `1`, `2`).
pub fn format_aligned_ticks(values: &[f64], step: f64) -> Vec<String> {
    let mut decimals = step_decimals(step);
    for &v in values {
        decimals = decimals.max(value_frac_decimals(v));
    }
    decimals = decimals.min(MAX_TICK_DECIMALS);
    values
        .iter()
        .map(|&v| format_tick_fixed(v, decimals))
        .collect()
}

/// Format a tick using the locator step (matplotlib `ScalarFormatter`-ish).
///
/// Fractional steps keep trailing decimals (`0.0`, `1.0` for step `0.5`);
/// integer steps stay compact (`0`, `1`, `2`).
fn format_tick_with_step(value: f64, step: f64) -> String {
    format_tick_fixed(value, step_decimals(step))
}

fn value_frac_decimals(value: f64) -> usize {
    if !value.is_finite() {
        return 0;
    }
    let v = if value.abs() < 1e-12 { 0.0 } else { value };
    if (v - v.round()).abs() < 1e-9 * v.abs().max(1.0) {
        return 0;
    }
    if !(1e-3..1e4).contains(&v.abs()) {
        return 0;
    }
    let s = format!("{v:.6}");
    let Some(frac) = s.split('.').nth(1) else {
        return 0;
    };
    frac.trim_end_matches('0').len().min(MAX_TICK_DECIMALS)
}

fn format_tick_fixed(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return format!("{value}");
    }
    // Floating-point dust around zero must not become `5.551e-17`.
    if value == 0.0 || value.abs() < 1e-12 {
        return if decimals == 0 {
            "0".to_string()
        } else {
            format!("{:.*}", decimals, 0.0)
        };
    }
    let abs = value.abs();
    let body = if !(1e-3..1e4).contains(&abs) {
        // Keep sign in scientific form; rare on plot axes.
        return format!("{value:.3e}");
    } else if decimals > 0 {
        format!("{:.*}", decimals, abs)
    } else {
        let near_int = (value - value.round()).abs() < 1e-9 * abs.max(1.0);
        if near_int {
            format!("{:.0}", abs.round())
        } else if abs >= 100.0 {
            format!("{abs:.0}")
        } else if abs >= 10.0 {
            format!("{abs:.1}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            format!("{abs:.2}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
    };
    // Matplotlib `ScalarFormatter` uses Unicode minus (U+2212).
    if value < 0.0 {
        format!("\u{2212}{body}")
    } else {
        body
    }
}

fn step_decimals(step: f64) -> usize {
    let s = step.abs();
    if !s.is_finite() || s >= 0.999 {
        return 0;
    }
    // Enough decimals for the step to format cleanly (0.5→1, 0.25→2).
    for d in 1..=MAX_TICK_DECIMALS {
        let scaled = s * 10f64.powi(d as i32);
        if (scaled - scaled.round()).abs() < 1e-8 * scaled.max(1.0) {
            return d;
        }
    }
    MAX_TICK_DECIMALS
}

fn format_log_tick(value: f64) -> String {
    if value <= 0.0 {
        return format_tick(value);
    }
    let exp = value.log10();
    if (exp - exp.round()).abs() < 1e-9 {
        // Matplotlib `LogFormatterMathtext` / `LogFormatterSciNotation`: `$10^{n}$`.
        let e = exp.round() as i32;
        format!("$10^{{{e}}}$")
    } else {
        format_tick(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{LinearScale, LogScale, ScaleKind};
    use proptest::prelude::*;

    #[test]
    fn produces_ticks_inside_domain() {
        let scale = ScaleKind::Linear(LinearScale::new(0.0, 1.0).unwrap());
        let ticks = TickLocator::default().ticks(scale);
        assert!(ticks.len() >= 2);
        for t in &ticks {
            assert!(t.value >= -1e-9 && t.value <= 1.0 + 1e-9);
        }
    }

    #[test]
    fn formats_near_zero_as_zero() {
        assert_eq!(format_tick(0.0), "0");
        assert_eq!(format_tick(5.551115123125783e-17), "0");
        assert_eq!(format_tick(-1e-13), "0");
    }

    #[test]
    fn fractional_step_keeps_decimals() {
        assert_eq!(format_tick_with_step(0.0, 0.5), "0.0");
        assert_eq!(format_tick_with_step(1.0, 0.5), "1.0");
        assert_eq!(format_tick_with_step(0.5, 0.5), "0.5");
        assert_eq!(format_tick_with_step(0.0, 0.25), "0.00");
        assert_eq!(format_tick_with_step(0.25, 0.25), "0.25");
        assert_eq!(format_tick_with_step(-0.5, 0.5), "\u{2212}0.5");
    }

    #[test]
    fn aligned_decimals_pad_axis() {
        let labels = format_aligned_ticks(&[1.0, 0.75, 0.5, 0.25], 0.25);
        assert_eq!(labels, vec!["1.00", "0.75", "0.50", "0.25"]);
        let labels = format_aligned_ticks(&[0.0, 1.0, 2.0], 1.0);
        assert_eq!(labels, vec!["0", "1", "2"]);
        let ticks = ticks_from_values(&[1.0, 0.5, 0.25]);
        assert_eq!(
            ticks.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(),
            vec!["1.00", "0.50", "0.25"]
        );
    }

    #[test]
    fn aligned_decimals_cap_at_three() {
        // Float-noise endpoints must not produce 6-digit inset labels.
        let labels = format_aligned_ticks(&[0.113639482, 0.913580247], 0.1);
        for lab in &labels {
            let frac = lab.split('.').nth(1).unwrap_or("");
            assert!(
                frac.len() <= MAX_TICK_DECIMALS,
                "label {lab} exceeds {MAX_TICK_DECIMALS} decimals"
            );
        }
    }

    #[test]
    fn rough_three_ceils_to_five() {
        // Ceiling path (used by TickLocator): never shrink below rough.
        assert!((nice_number(3.0, false) - 5.0).abs() < 1e-12);
        assert!((nice_number(0.33, false) - 0.5).abs() < 1e-12);
        assert!((nice_number(0.61, false) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn colorbar_compressed_x_matches_mpl_maxn() {
        // Stock mpl + colorbar: get_tick_space ≈ 7–8 → targets≈8–9.
        // Flooring targets to 11 densified coolwarm to step 2.5 / "0.0" labels.
        let scale = ScaleKind::Linear(LinearScale::new(-0.5, 23.5).unwrap());
        let ticks = TickLocator::new(8).ticks(scale);
        let labels: Vec<&str> = ticks.iter().map(|t| t.label.as_str()).collect();
        assert_eq!(labels, ["0", "5", "10", "15", "20"]);

        // Short tripcolor span: mpl uses 0.5 (not 0.25) when axes are narrow.
        let scale = ScaleKind::Linear(LinearScale::new(-0.1, 2.1).unwrap());
        let ticks = TickLocator::new(8).ticks(scale);
        let step = ticks[1].value - ticks[0].value;
        assert!((step - 0.5).abs() < 1e-12, "expected step 0.5, got {step}");
        assert_eq!(ticks[0].label, "0.0");
    }

    #[test]
    fn locator_respects_max_intervals() {
        // span=5.5, nbins≈8 → targets=9 → must not land on step 0.5.
        let scale = ScaleKind::Linear(LinearScale::new(0.0, 5.5).unwrap());
        let ticks = TickLocator::new(9).ticks(scale);
        let step = ticks[1].value - ticks[0].value;
        assert!(
            step >= 0.99,
            "expected step≥1, got {step} from {} ticks",
            ticks.len()
        );
    }

    #[test]
    fn tiny_inset_span_uses_nice_steps_not_endpoints() {
        // Nested inset y after 5% margins — ceiling step 1.0 would be empty.
        let scale = ScaleKind::Linear(LinearScale::new(0.113639482, 0.913580247).unwrap());
        let ticks = TickLocator::new(2).ticks(scale);
        assert!(ticks.len() >= 2);
        let vals: Vec<f64> = ticks.iter().map(|t| t.value).collect();
        assert!(
            vals.iter().any(|v| (*v - 0.5).abs() < 1e-9),
            "expected a 0.5 tick, got {vals:?}"
        );
        for t in &ticks {
            assert!(
                !t.label.contains("0.114") && !t.label.contains("0.913"),
                "raw endpoint label {}",
                t.label
            );
        }
    }

    #[test]
    fn log_ticks_are_powers_of_ten() {
        let scale = ScaleKind::Log(LogScale::new(1.0, 1000.0).unwrap());
        let ticks = TickLocator::default().ticks(scale);
        assert!(ticks.iter().any(|t| (t.value - 1.0).abs() < 1e-12));
        assert!(ticks.iter().any(|t| (t.value - 10.0).abs() < 1e-12));
        assert!(ticks.iter().any(|t| (t.value - 100.0).abs() < 1e-12));
        assert!(ticks.iter().any(|t| (t.value - 1000.0).abs() < 1e-12));
        let ten = ticks
            .iter()
            .find(|t| (t.value - 10.0).abs() < 1e-12)
            .unwrap();
        assert_eq!(ten.label, "$10^{1}$");
        let tenth = TickLocator::default().ticks(ScaleKind::Log(LogScale::new(0.1, 10.0).unwrap()));
        let lab = tenth
            .iter()
            .find(|t| (t.value - 0.1).abs() < 1e-12)
            .unwrap();
        assert_eq!(lab.label, "$10^{-1}$");
    }

    #[test]
    fn handles_large_range() {
        let scale = ScaleKind::Linear(LinearScale::new(0.0, 1e6).unwrap());
        let ticks = TickLocator::new(5).ticks(scale);
        assert!(ticks.len() >= 2);
        assert!(ticks.len() <= 12);
    }

    proptest! {
        #[test]
        fn ticks_never_panic(a in -1e6f64..1e6, b in -1e6f64..1e6) {
            prop_assume!((a - b).abs() > 1e-6);
            let (min, max) = if a < b { (a, b) } else { (b, a) };
            let scale = ScaleKind::Linear(LinearScale::new(min, max).unwrap());
            let ticks = TickLocator::default().ticks(scale);
            prop_assert!(!ticks.is_empty());
            prop_assert!(ticks.len() <= 64);
        }
    }
}
