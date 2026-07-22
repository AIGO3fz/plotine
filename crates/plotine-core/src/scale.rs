//! Continuous scales mapping data values into a normalized [0, 1] domain.

use crate::error::{PlotError, Result};

/// Mapping from data space to unit interval (and back).
pub trait Scale: Clone {
    fn domain(&self) -> (f64, f64);
    fn normalize(&self, value: f64) -> f64;
    fn denormalize(&self, unit: f64) -> f64;
}

/// Which continuous scale an axis uses.
///
/// Prefer setting the scale on axes **before** adding artists so auto-limits use
/// the correct padding. Log domains must be strictly positive; use
/// [`ScaleType::Symlog`] when data crosses zero.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[non_exhaustive]
pub enum ScaleType {
    /// Identity mapping over `[min, max]` (default).
    #[default]
    Linear,
    /// Base-10 logarithmic scale. Domain must be `> 0`.
    Log,
    /// Linear near zero within `±linthresh`, logarithmic outside.
    Symlog {
        /// Half-width of the linear region around zero (must be `> 0`).
        linthresh: f64,
    },
}

/// Type-erased scale used by the transform / tick pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleKind {
    Linear(LinearScale),
    Log(LogScale),
    Symlog(SymlogScale),
}

impl ScaleKind {
    pub fn build(scale_type: ScaleType, min: f64, max: f64) -> Result<Self> {
        match scale_type {
            ScaleType::Linear => Ok(Self::Linear(LinearScale::from_values(min, max)?)),
            ScaleType::Log => Ok(Self::Log(LogScale::from_values(min, max)?)),
            ScaleType::Symlog { linthresh } => {
                Ok(Self::Symlog(SymlogScale::from_values(min, max, linthresh)?))
            }
        }
    }

    pub fn domain(self) -> (f64, f64) {
        match self {
            Self::Linear(s) => s.domain(),
            Self::Log(s) => s.domain(),
            Self::Symlog(s) => s.domain(),
        }
    }

    pub fn normalize(self, value: f64) -> f64 {
        match self {
            Self::Linear(s) => s.normalize(value),
            Self::Log(s) => s.normalize(value),
            Self::Symlog(s) => s.normalize(value),
        }
    }

    /// Inverse of [`normalize`](Self::normalize): unit interval → data value.
    pub fn denormalize(self, unit: f64) -> f64 {
        match self {
            Self::Linear(s) => s.denormalize(unit),
            Self::Log(s) => s.denormalize(unit),
            Self::Symlog(s) => s.denormalize(unit),
        }
    }
}

/// Linear scale over a closed interval `[min, max]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    min: f64,
    max: f64,
}

impl LinearScale {
    pub fn new(min: f64, max: f64) -> Result<Self> {
        if !min.is_finite() || !max.is_finite() {
            return Err(PlotError::InvalidRange {
                min,
                max,
                message: "bounds must be finite",
                suggestion: "replace NaN/Inf with finite data-derived limits",
            });
        }
        if min >= max {
            return Err(PlotError::invalid_range(min, max));
        }
        Ok(Self { min, max })
    }

    pub fn from_values(min: f64, max: f64) -> Result<Self> {
        if (max - min).abs() < f64::EPSILON {
            let pad = if min.abs() < 1.0 {
                0.5
            } else {
                min.abs() * 0.05
            };
            return Self::new(min - pad, max + pad);
        }
        Self::new(min, max)
    }

    pub fn min(self) -> f64 {
        self.min
    }

    pub fn max(self) -> f64 {
        self.max
    }

    pub fn span(self) -> f64 {
        self.max - self.min
    }
}

impl Scale for LinearScale {
    fn domain(&self) -> (f64, f64) {
        (self.min, self.max)
    }

    fn normalize(&self, value: f64) -> f64 {
        (value - self.min) / self.span()
    }

    fn denormalize(&self, unit: f64) -> f64 {
        self.min + unit * self.span()
    }
}

/// Logarithmic scale (base 10). Domain must be strictly positive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogScale {
    min: f64,
    max: f64,
}

impl LogScale {
    pub fn new(min: f64, max: f64) -> Result<Self> {
        if !min.is_finite() || !max.is_finite() {
            return Err(PlotError::InvalidRange {
                min,
                max,
                message: "bounds must be finite",
                suggestion: "replace NaN/Inf with finite positive limits",
            });
        }
        if min <= 0.0 {
            return Err(PlotError::log_non_positive(min));
        }
        if max <= 0.0 {
            return Err(PlotError::log_non_positive(max));
        }
        if min >= max {
            return Err(PlotError::invalid_range(min, max));
        }
        Ok(Self { min, max })
    }

    pub fn from_values(min: f64, max: f64) -> Result<Self> {
        let min = if min <= 0.0 { f64::NAN } else { min };
        let max = if max <= 0.0 { f64::NAN } else { max };
        if !min.is_finite() {
            return Err(PlotError::log_non_positive(0.0));
        }
        if !max.is_finite() {
            return Err(PlotError::log_non_positive(0.0));
        }
        if (max - min).abs() < f64::EPSILON * min.abs().max(1.0) {
            return Self::new(min / 10.0_f64.sqrt(), max * 10.0_f64.sqrt());
        }
        Self::new(min, max)
    }

    pub fn min(self) -> f64 {
        self.min
    }

    pub fn max(self) -> f64 {
        self.max
    }
}

impl Scale for LogScale {
    fn domain(&self) -> (f64, f64) {
        (self.min, self.max)
    }

    fn normalize(&self, value: f64) -> f64 {
        if value <= 0.0 {
            return f64::NAN;
        }
        let a = self.min.log10();
        let b = self.max.log10();
        (value.log10() - a) / (b - a)
    }

    fn denormalize(&self, unit: f64) -> f64 {
        let a = self.min.log10();
        let b = self.max.log10();
        10f64.powf(a + unit * (b - a))
    }
}

/// Symlog scale: linear in `[-linthresh, linthresh]`, log outside.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymlogScale {
    min: f64,
    max: f64,
    linthresh: f64,
}

impl SymlogScale {
    pub fn new(min: f64, max: f64, linthresh: f64) -> Result<Self> {
        if !min.is_finite() || !max.is_finite() || !linthresh.is_finite() {
            return Err(PlotError::InvalidRange {
                min,
                max,
                message: "bounds and linthresh must be finite",
                suggestion: "provide finite symlog parameters",
            });
        }
        if linthresh <= 0.0 {
            return Err(PlotError::InvalidRange {
                min: linthresh,
                max: linthresh,
                message: "linthresh must be positive",
                suggestion: "pass Symlog { linthresh: 1.0 } or another positive threshold",
            });
        }
        if min >= max {
            return Err(PlotError::invalid_range(min, max));
        }
        Ok(Self {
            min,
            max,
            linthresh,
        })
    }

    pub fn from_values(min: f64, max: f64, linthresh: f64) -> Result<Self> {
        let linthresh = if linthresh > 0.0 { linthresh } else { 1.0 };
        if (max - min).abs() < f64::EPSILON {
            let pad = linthresh.max(0.5);
            return Self::new(min - pad, max + pad, linthresh);
        }
        Self::new(min, max, linthresh)
    }

    fn forward(self, value: f64) -> f64 {
        let lt = self.linthresh;
        if value.abs() <= lt {
            value
        } else {
            value.signum() * (lt + (value.abs() / lt).ln())
        }
    }

    pub fn min(self) -> f64 {
        self.min
    }

    pub fn max(self) -> f64 {
        self.max
    }

    pub fn linthresh(self) -> f64 {
        self.linthresh
    }
}

impl Scale for SymlogScale {
    fn domain(&self) -> (f64, f64) {
        (self.min, self.max)
    }

    fn normalize(&self, value: f64) -> f64 {
        let a = self.forward(self.min);
        let b = self.forward(self.max);
        let v = self.forward(value);
        (v - a) / (b - a)
    }

    fn denormalize(&self, unit: f64) -> f64 {
        let a = self.forward(self.min);
        let b = self.forward(self.max);
        let t = a + unit * (b - a);
        let lt = self.linthresh;
        if t.abs() <= lt {
            t
        } else {
            t.signum() * lt * (t.abs() - lt).exp()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inverted_domain() {
        assert!(LinearScale::new(1.0, 0.0).is_err());
    }

    #[test]
    fn normalizes_endpoints() {
        let s = LinearScale::new(0.0, 10.0).unwrap();
        assert!((s.normalize(0.0) - 0.0).abs() < 1e-12);
        assert!((s.normalize(10.0) - 1.0).abs() < 1e-12);
        assert!((s.normalize(5.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn log_rejects_non_positive() {
        assert!(matches!(
            LogScale::new(-1.0, 10.0),
            Err(PlotError::LogScaleNonPositive { .. })
        ));
    }

    #[test]
    fn log_midpoint() {
        let s = LogScale::new(1.0, 100.0).unwrap();
        assert!((s.normalize(10.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn symlog_passes_through_zero() {
        let s = SymlogScale::new(-100.0, 100.0, 1.0).unwrap();
        assert!(s.normalize(0.0).is_finite());
        assert!((s.normalize(0.0) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn scale_kind_denormalize_roundtrip() {
        for st in [
            ScaleType::Linear,
            ScaleType::Log,
            ScaleType::Symlog { linthresh: 1.0 },
        ] {
            let (min, max) = match st {
                ScaleType::Log => (1.0, 100.0),
                _ => (-10.0, 10.0),
            };
            let s = ScaleKind::build(st, min, max).unwrap();
            for u in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let v = s.denormalize(u);
                let u2 = s.normalize(v);
                assert!(
                    (u - u2).abs() < 1e-9,
                    "roundtrip failed for {st:?} at u={u}: got {u2}"
                );
            }
        }
    }
}
