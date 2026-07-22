//! Unix-timestamp datetime helpers for axis ticks (UTC, no extra deps).
//!
//! Tick labels follow matplotlib [`ConciseDateFormatter`](https://matplotlib.org/stable/api/dates_api.html#matplotlib.dates.ConciseDateFormatter):
//! show only the changing part of the date, with a shared offset string.

use crate::ticks::Tick;

const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Format a Unix timestamp (seconds) for axis ticks, choosing resolution from `span_secs`.
///
/// Prefer [`format_concise_datetime_ticks`] when formatting a whole axis at once.
pub fn format_unix_tick(ts: f64, span_secs: f64) -> String {
    if !ts.is_finite() {
        return String::new();
    }
    let (y, m, d, hh, mm, ss) = civil_from_unix(ts.floor() as i64);
    if span_secs >= 86400.0 * 60.0 {
        format!("{y:04}-{m:02}")
    } else if span_secs >= 86400.0 * 2.0 {
        format!("{y:04}-{m:02}-{d:02}")
    } else if span_secs >= 3600.0 * 6.0 {
        format!("{m:02}-{d:02} {hh:02}h")
    } else if span_secs >= 60.0 {
        format!("{hh:02}:{mm:02}")
    } else {
        format!("{hh:02}:{mm:02}:{ss:02}")
    }
}

/// Concise labels + optional offset for a set of Unix-second tick values.
///
/// Mirrors matplotlib `ConciseDateFormatter.format_ticks` (default formats):
/// `['%Y','%b','%d','%H:%M','%H:%M','%S']` with zero_formats /
/// `offset_formats` of `['','%Y','%Y-%b','%Y-%b-%d',…]`.
pub fn format_concise_datetime_ticks(values: &[f64]) -> (Vec<String>, Option<String>) {
    if values.is_empty() {
        return (Vec::new(), None);
    }
    let civils: Vec<(i32, u32, u32, u32, u32, u32)> = values
        .iter()
        .map(|&v| {
            if v.is_finite() {
                civil_from_unix(v.floor() as i64)
            } else {
                (0, 1, 1, 0, 0, 0)
            }
        })
        .collect();

    // Levels: 0 year, 1 month, 2 day, 3 hour, 4 minute, 5 second.
    let mut level = 5usize;
    for lv in (0..=5).rev() {
        let mut unique = Vec::new();
        for c in &civils {
            let part = match lv {
                0 => c.0 as i64,
                1 => c.1 as i64,
                2 => c.2 as i64,
                3 => c.3 as i64,
                4 => c.4 as i64,
                _ => c.5 as i64,
            };
            if !unique.contains(&part) {
                unique.push(part);
            }
        }
        if unique.len() > 1 {
            level = lv;
            break;
        }
        if lv == 0 {
            level = 5;
        }
    }

    let labels: Vec<String> = civils
        .iter()
        .map(|&(y, m, d, hh, mm, ss)| match level {
            0 => format!("{y:04}"),
            1 if m == 1 => format!("{y:04}"),
            1 => month_abbr(m).to_string(),
            2 if d == 1 => month_abbr(m).to_string(),
            2 => format!("{d:02}"),
            3 if hh == 0 => format!("{}-{d:02}", month_abbr(m)),
            3 | 4 => format!("{hh:02}:{mm:02}"),
            5 if ss == 0 && mm == 0 => format!("{hh:02}:{mm:02}"),
            _ => format!("{hh:02}:{mm:02}:{ss:02}"),
        })
        .collect();

    let offset = match level {
        0 => None,
        1 => civils.last().map(|&(y, _, _, _, _, _)| format!("{y:04}")),
        2 => civils
            .last()
            .map(|&(y, m, _, _, _, _)| format!("{y:04}-{}", month_abbr(m))),
        3 | 4 => civils
            .last()
            .map(|&(y, m, d, _, _, _)| format!("{y:04}-{}-{d:02}", month_abbr(m))),
        _ => civils.last().map(|&(y, m, d, hh, mm, _)| {
            format!("{y:04}-{}-{d:02} {hh:02}:{mm:02}", month_abbr(m))
        }),
    };

    (labels, offset)
}

fn month_abbr(m: u32) -> &'static str {
    MONTH_ABBR
        .get((m.saturating_sub(1) as usize).min(11))
        .copied()
        .unwrap_or("Jan")
}

/// Locates calendar-aligned major ticks for Unix-second domains.
#[derive(Debug, Clone, Copy)]
pub struct DatetimeLocator {
    pub target_count: usize,
}

impl Default for DatetimeLocator {
    fn default() -> Self {
        Self { target_count: 6 }
    }
}

impl DatetimeLocator {
    pub fn new(target_count: usize) -> Self {
        Self {
            target_count: target_count.max(2),
        }
    }

    /// Produce major ticks in `[min, max]` (Unix seconds, UTC).
    pub fn ticks(&self, min: f64, max: f64) -> Vec<Tick> {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        if !min.is_finite() || !max.is_finite() {
            return Vec::new();
        }
        let span = (max - min).max(1.0);
        let rough = span / (self.target_count as f64 - 1.0).max(1.0);
        let unit = choose_unit(rough);

        let values = match unit {
            TimeUnit::Fixed(step) => fixed_step_ticks(min, max, step),
            TimeUnit::Month(n) => month_ticks(min, max, n),
            TimeUnit::Year(n) => year_ticks(min, max, n),
        };

        let (labels, _offset) = format_concise_datetime_ticks(&values);
        // Offset is recomputed by the figure from the same values when drawing.
        values
            .into_iter()
            .zip(labels)
            .map(|(value, label)| Tick { label, value })
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
enum TimeUnit {
    Fixed(f64),
    Month(u32),
    Year(u32),
}

fn choose_unit(rough: f64) -> TimeUnit {
    const SEC: f64 = 1.0;
    const MIN: f64 = 60.0;
    const HOUR: f64 = 3600.0;
    const DAY: f64 = 86_400.0;
    const WEEK: f64 = 7.0 * DAY;
    const MONTH: f64 = 30.0 * DAY;
    const YEAR: f64 = 365.25 * DAY;

    let candidates: &[(f64, TimeUnit)] = &[
        (1.0 * SEC, TimeUnit::Fixed(1.0)),
        (2.0 * SEC, TimeUnit::Fixed(2.0)),
        (5.0 * SEC, TimeUnit::Fixed(5.0)),
        (10.0 * SEC, TimeUnit::Fixed(10.0)),
        (15.0 * SEC, TimeUnit::Fixed(15.0)),
        (30.0 * SEC, TimeUnit::Fixed(30.0)),
        (1.0 * MIN, TimeUnit::Fixed(MIN)),
        (2.0 * MIN, TimeUnit::Fixed(2.0 * MIN)),
        (5.0 * MIN, TimeUnit::Fixed(5.0 * MIN)),
        (10.0 * MIN, TimeUnit::Fixed(10.0 * MIN)),
        (15.0 * MIN, TimeUnit::Fixed(15.0 * MIN)),
        (30.0 * MIN, TimeUnit::Fixed(30.0 * MIN)),
        (1.0 * HOUR, TimeUnit::Fixed(HOUR)),
        (2.0 * HOUR, TimeUnit::Fixed(2.0 * HOUR)),
        (3.0 * HOUR, TimeUnit::Fixed(3.0 * HOUR)),
        (6.0 * HOUR, TimeUnit::Fixed(6.0 * HOUR)),
        (12.0 * HOUR, TimeUnit::Fixed(12.0 * HOUR)),
        (1.0 * DAY, TimeUnit::Fixed(DAY)),
        (2.0 * DAY, TimeUnit::Fixed(2.0 * DAY)),
        (1.0 * WEEK, TimeUnit::Fixed(WEEK)),
        (1.0 * MONTH, TimeUnit::Month(1)),
        (2.0 * MONTH, TimeUnit::Month(2)),
        (3.0 * MONTH, TimeUnit::Month(3)),
        (6.0 * MONTH, TimeUnit::Month(6)),
        (1.0 * YEAR, TimeUnit::Year(1)),
        (2.0 * YEAR, TimeUnit::Year(2)),
        (5.0 * YEAR, TimeUnit::Year(5)),
        (10.0 * YEAR, TimeUnit::Year(10)),
    ];

    let mut best = candidates[0].1;
    let mut best_score = f64::INFINITY;
    for &(size, unit) in candidates {
        let score = (size - rough).abs() / rough.max(1.0);
        // Prefer slightly larger steps over denser clutter.
        let score = if size < rough { score + 0.15 } else { score };
        if score < best_score {
            best_score = score;
            best = unit;
        }
    }
    best
}

fn fixed_step_ticks(min: f64, max: f64, step: f64) -> Vec<f64> {
    if step <= 0.0 || !step.is_finite() {
        return vec![min, max];
    }
    let start = (min / step).ceil() * step;
    let mut out = Vec::new();
    let mut v = start;
    let mut guard = 0;
    while v <= max + step * 1e-9 && guard < 64 {
        if v >= min - step * 1e-9 {
            out.push(v.clamp(min, max));
        }
        v += step;
        guard += 1;
    }
    if out.is_empty() {
        out.push(min);
        if (max - min).abs() > 1e-9 {
            out.push(max);
        }
    }
    dedup_close(&mut out, step * 1e-6);
    out
}

fn month_ticks(min: f64, max: f64, every: u32) -> Vec<f64> {
    let every = every.max(1);
    let (y, m, _, _, _, _) = civil_from_unix(min.floor() as i64);
    let mut cy = y;
    let mut cm = m;
    // Start at month boundary on or after min.
    let mut t = unix_from_civil(cy, cm, 1, 0, 0, 0) as f64;
    if t < min {
        advance_months(&mut cy, &mut cm, 1);
        t = unix_from_civil(cy, cm, 1, 0, 0, 0) as f64;
    }
    // Align to every-N months from year start (Jan = 1).
    let month_index = (cy * 12 + cm as i32 - 1) as i64;
    let rem = month_index.rem_euclid(every as i64);
    if rem != 0 {
        advance_months(&mut cy, &mut cm, (every as i64 - rem) as u32);
        t = unix_from_civil(cy, cm, 1, 0, 0, 0) as f64;
    }

    let mut out = Vec::new();
    let mut guard = 0;
    while t <= max + 1.0 && guard < 64 {
        if t >= min - 1.0 {
            out.push(t.clamp(min, max));
        }
        advance_months(&mut cy, &mut cm, every);
        t = unix_from_civil(cy, cm, 1, 0, 0, 0) as f64;
        guard += 1;
    }
    if out.is_empty() {
        out.push(min);
    }
    out
}

fn year_ticks(min: f64, max: f64, every: u32) -> Vec<f64> {
    let every = every.max(1) as i32;
    let (y, _, _, _, _, _) = civil_from_unix(min.floor() as i64);
    let mut cy = y;
    let mut t = unix_from_civil(cy, 1, 1, 0, 0, 0) as f64;
    if t < min {
        cy += 1;
        t = unix_from_civil(cy, 1, 1, 0, 0, 0) as f64;
    }
    let rem = cy.rem_euclid(every);
    if rem != 0 {
        cy += every - rem;
        t = unix_from_civil(cy, 1, 1, 0, 0, 0) as f64;
    }

    let mut out = Vec::new();
    let mut guard = 0;
    while t <= max + 1.0 && guard < 64 {
        if t >= min - 1.0 {
            out.push(t.clamp(min, max));
        }
        cy += every;
        t = unix_from_civil(cy, 1, 1, 0, 0, 0) as f64;
        guard += 1;
    }
    if out.is_empty() {
        out.push(min);
    }
    out
}

fn advance_months(y: &mut i32, m: &mut u32, n: u32) {
    let idx = *y as i64 * 12 + (*m as i64 - 1) + n as i64;
    *y = (idx.div_euclid(12)) as i32;
    *m = (idx.rem_euclid(12) + 1) as u32;
}

fn dedup_close(values: &mut Vec<f64>, eps: f64) {
    if values.is_empty() {
        return;
    }
    let mut out = vec![values[0]];
    for &v in values.iter().skip(1) {
        if (v - *out.last().unwrap()).abs() > eps {
            out.push(v);
        }
    }
    *values = out;
}

/// Convert Unix seconds to `(year, month, day, hour, minute, second)` in UTC.
pub fn civil_from_unix(ts: i64) -> (i32, u32, u32, u32, u32, u32) {
    let secs_per_day = 86_400_i64;
    let mut days = ts.div_euclid(secs_per_day);
    let mut sod = ts.rem_euclid(secs_per_day);
    if sod < 0 {
        sod += secs_per_day;
        days -= 1;
    }
    let (y, m, d) = civil_from_days(days);
    let hh = (sod / 3600) as u32;
    let mm = ((sod % 3600) / 60) as u32;
    let ss = (sod % 60) as u32;
    (y, m, d, hh, mm, ss)
}

/// Convert a UTC civil datetime to Unix seconds.
pub fn unix_from_civil(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> i64 {
    let days = days_from_civil(y, m, d);
    days * 86_400 + i64::from(hh) * 3600 + i64::from(mm) * 60 + i64::from(ss)
}

/// Howard Hinnant's `civil_from_days` (days since 1970-01-01 → Y-M-D).
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

/// Howard Hinnant's `days_from_civil` (Y-M-D → days since 1970-01-01).
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i64::from(era) * 146_097 + i64::from(doe) - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970_01_01() {
        let (y, m, d, hh, mm, ss) = civil_from_unix(0);
        assert_eq!((y, m, d, hh, mm, ss), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn known_date() {
        // 2020-01-01 00:00:00 UTC
        let (y, m, d, _, _, _) = civil_from_unix(1_577_836_800);
        assert_eq!((y, m, d), (2020, 1, 1));
    }

    #[test]
    fn roundtrip_civil_unix() {
        let ts = unix_from_civil(2020, 1, 1, 12, 30, 45);
        let (y, m, d, hh, mm, ss) = civil_from_unix(ts);
        assert_eq!((y, m, d, hh, mm, ss), (2020, 1, 1, 12, 30, 45));
    }

    #[test]
    fn daily_ticks_land_on_midnight() {
        let start = 1_577_836_800.0; // 2020-01-01
        let end = start + 10.0 * 86_400.0;
        let ticks = DatetimeLocator::new(6).ticks(start, end);
        assert!(ticks.len() >= 3);
        for t in &ticks {
            let (_, _, _, hh, mm, ss) = civil_from_unix(t.value as i64);
            assert_eq!((hh, mm, ss), (0, 0, 0), "tick {}", t.value);
        }
    }

    #[test]
    fn concise_day_span_labels() {
        let start = 1_577_836_800.0; // 2020-01-01
        let vals: Vec<f64> = (0..6).map(|i| start + (i * 2) as f64 * 86_400.0).collect();
        let (labels, offset) = format_concise_datetime_ticks(&vals);
        assert_eq!(labels[0], "Jan");
        assert_eq!(labels[1], "03");
        assert_eq!(offset.as_deref(), Some("2020-Jan"));
    }

    #[test]
    fn month_ticks_on_first() {
        let start = unix_from_civil(2020, 1, 15, 0, 0, 0) as f64;
        let end = unix_from_civil(2020, 8, 15, 0, 0, 0) as f64;
        let ticks = DatetimeLocator::new(6).ticks(start, end);
        assert!(ticks.len() >= 3);
        for t in &ticks {
            let (_, _, d, hh, mm, ss) = civil_from_unix(t.value.round() as i64);
            // Endpoints may be clamped; interior ticks should be month starts.
            if t.value > start + 1.0 && t.value < end - 1.0 {
                assert_eq!((d, hh, mm, ss), (1, 0, 0, 0), "{}", t.label);
            }
        }
    }
}
