//! Secondary axes: transformed tick labels on the opposite spine.
//!
//! Unlike [`crate::axes::Axes::twin_y`] / [`twin_x`](crate::axes::Axes::twin_x),
//! a secondary axis does **not** host artists — it only shows a function of the
//! primary domain (matplotlib `secondary_xaxis` / `secondary_yaxis`).

use plotine_core::{LinearScale, Tick, TickLocator};

use crate::mpl_policy::ticks as ticks_policy;

/// How secondary tick values relate to the primary axis domain.
#[derive(Debug, Clone, Copy)]
pub enum SecondaryTransform {
    /// `secondary = scale * primary + offset`.
    Linear {
        /// Multiplier applied to the primary value.
        scale: f64,
        /// Constant added after scaling.
        offset: f64,
    },
    /// Arbitrary monotonic pair (must be approximate inverses).
    Fn {
        /// Map primary → secondary.
        forward: fn(f64) -> f64,
        /// Map secondary → primary.
        inverse: fn(f64) -> f64,
    },
}

impl SecondaryTransform {
    /// Map a primary-domain value into the secondary domain.
    pub fn forward(self, v: f64) -> f64 {
        match self {
            Self::Linear { scale, offset } => scale * v + offset,
            Self::Fn { forward, .. } => forward(v),
        }
    }

    /// Map a secondary-domain value back into the primary domain.
    pub fn inverse(self, v: f64) -> f64 {
        match self {
            Self::Linear { scale, offset } => {
                if scale.abs() < 1e-15 {
                    f64::NAN
                } else {
                    (v - offset) / scale
                }
            }
            Self::Fn { inverse, .. } => inverse(v),
        }
    }
}

/// Configuration for a secondary axis (label + transform).
#[derive(Debug, Clone)]
pub struct SecondaryAxis {
    pub(crate) transform: SecondaryTransform,
    pub(crate) label: Option<String>,
}

impl SecondaryAxis {
    pub(crate) fn new(transform: SecondaryTransform) -> Self {
        Self {
            transform,
            label: None,
        }
    }

    /// Axis title drawn on the secondary spine side.
    pub fn label(&mut self, text: impl Into<String>) -> &mut Self {
        self.label = Some(text.into());
        self
    }

    /// Locate ticks in the secondary domain; each item is
    /// `(primary_value_for_position, tick_with_secondary_label)`.
    pub(crate) fn mapped_ticks(&self, primary_min: f64, primary_max: f64) -> Vec<(f64, Tick)> {
        let a = self.transform.forward(primary_min);
        let b = self.transform.forward(primary_max);
        if !(a.is_finite() && b.is_finite()) {
            return Vec::new();
        }
        let (sec_min, sec_max) = if a <= b { (a, b) } else { (b, a) };
        if (sec_max - sec_min).abs() < 1e-15 {
            return Vec::new();
        }
        let Ok(scale) = LinearScale::new(sec_min, sec_max) else {
            return Vec::new();
        };
        let ticks = TickLocator::new(ticks_policy::LINEAR_TARGETS).ticks_linear(scale);
        let lo = primary_min.min(primary_max);
        let hi = primary_min.max(primary_max);
        let pad = (hi - lo).abs() * 1e-6 + 1e-12;
        ticks
            .into_iter()
            .filter_map(|tick| {
                let primary = self.transform.inverse(tick.value);
                if primary.is_finite() && primary >= lo - pad && primary <= hi + pad {
                    Some((primary, tick))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_from_radians() {
        let sec = SecondaryAxis::new(SecondaryTransform::Fn {
            forward: f64::to_degrees,
            inverse: f64::to_radians,
        });
        let ticks = sec.mapped_ticks(0.0, std::f64::consts::PI);
        assert!(ticks.len() >= 2);
        for (primary, tick) in &ticks {
            assert!((0.0..=std::f64::consts::PI).contains(primary) || primary.is_finite());
            assert!((0.0..=180.0).contains(&tick.value) || tick.value.is_finite());
            assert!((sec.transform.forward(*primary) - tick.value).abs() < 1e-6);
        }
    }

    #[test]
    fn linear_celsius_fahrenheit() {
        // °F = 1.8 * °C + 32
        let sec = SecondaryAxis::new(SecondaryTransform::Linear {
            scale: 1.8,
            offset: 32.0,
        });
        let ticks = sec.mapped_ticks(0.0, 100.0);
        assert!(ticks.len() >= 2);
        for (primary, tick) in &ticks {
            assert!((0.0..=100.0).contains(primary));
            let expected = 1.8 * primary + 32.0;
            assert!((tick.value - expected).abs() < 1e-6);
        }
    }
}
