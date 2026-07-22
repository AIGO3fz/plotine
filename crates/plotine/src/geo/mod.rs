//! Geographic projections (cartopy-thin subset) and coastline layer.
//!
//! Set [`Axes::projection`](crate::axes::Axes::projection) then plot lon/lat with
//! ordinary [`line`](crate::axes::Axes::line) / [`scatter`](crate::axes::Axes::scatter).
//! Call [`coastline`](crate::axes::Axes::coastline) for Natural Earth 110m shorelines
//! (public domain; regenerate via `scripts/build_coastline.py`).

mod coastline;
mod geojson;

pub use coastline::{coastline_lonlat, coastline_point_count};
pub use geojson::{parse_geojson, GeoGeom};

use plotine_core::{PlotError, Result};
use std::f64::consts::PI;

use crate::series::Series;

/// Supported map projections (matplotlib / cartopy subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum GeoProjection {
    /// Equirectangular: `(x, y) = (lon°, lat°)`.
    #[default]
    PlateCarree,
    /// Web-Mercator style: `x = lon°`, `y = (180/π)·ln(tan(π/4+φ/2))` (degree-like units).
    Mercator,
}

impl GeoProjection {
    /// Project a single lon/lat point (degrees) into map plane coordinates.
    pub fn project(self, lon: f64, lat: f64) -> (f64, f64) {
        match self {
            Self::PlateCarree => (lon, lat),
            Self::Mercator => (lon, mercator_y(lat)),
        }
    }

    /// Default data extent in projected coordinates (global view).
    pub fn default_extent(self) -> (f64, f64, f64, f64) {
        match self {
            Self::PlateCarree => (-180.0, 180.0, -90.0, 90.0),
            Self::Mercator => {
                let ymax = mercator_y(crate::mpl_policy::geo::MERCATOR_MAX_LAT);
                (-180.0, 180.0, -ymax, ymax)
            }
        }
    }

    /// Axis label hints for the projected plane.
    pub fn default_labels(self) -> (&'static str, &'static str) {
        match self {
            Self::PlateCarree => ("longitude (°)", "latitude (°)"),
            Self::Mercator => ("longitude (°)", "Mercator y"),
        }
    }
}

/// Mercator forward (lat degrees → degree-like y). Lat is clamped to ±max.
pub fn mercator_y(lat_deg: f64) -> f64 {
    let max = crate::mpl_policy::geo::MERCATOR_MAX_LAT;
    let lat = lat_deg.clamp(-max, max);
    let phi = lat * PI / 180.0;
    (180.0 / PI) * (PI / 4.0 + phi / 2.0).tan().ln()
}

/// Inverse Mercator (degree-like y → lat degrees).
pub fn mercator_lat(y: f64) -> f64 {
    let phi = 2.0 * (y * PI / 180.0).exp().atan() - PI / 2.0;
    phi * 180.0 / PI
}

/// Project parallel lon/lat series (NaN breaks preserved).
pub fn project_lonlat(proj: GeoProjection, lon: &Series, lat: &Series) -> Result<(Series, Series)> {
    if lon.len() != lat.len() {
        return Err(PlotError::length_mismatch(lon.len(), lat.len()));
    }
    let mut xs = Vec::with_capacity(lon.len());
    let mut ys = Vec::with_capacity(lat.len());
    for (&lo, &la) in lon.as_slice().iter().zip(lat.as_slice().iter()) {
        if lo.is_finite() && la.is_finite() {
            let (x, y) = proj.project(lo, la);
            xs.push(x);
            ys.push(y);
        } else {
            // Preserve segment breaks (and propagate non-finite pairs).
            xs.push(f64::NAN);
            ys.push(f64::NAN);
        }
    }
    Ok((Series::new(xs), Series::new(ys)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plate_carree_identity() {
        let (x, y) = GeoProjection::PlateCarree.project(12.5, -33.0);
        assert!((x - 12.5).abs() < 1e-12);
        assert!((y - (-33.0)).abs() < 1e-12);
    }

    #[test]
    fn mercator_equator_zero() {
        let (_, y) = GeoProjection::Mercator.project(0.0, 0.0);
        assert!(y.abs() < 1e-12);
    }

    #[test]
    fn mercator_symmetric() {
        let (_, yp) = GeoProjection::Mercator.project(10.0, 45.0);
        let (_, yn) = GeoProjection::Mercator.project(10.0, -45.0);
        assert!((yp + yn).abs() < 1e-9);
        assert!(yp > 45.0); // stretched vs PlateCarree
    }

    #[test]
    fn mercator_roundtrip_lat() {
        for lat in [-60.0, -30.0, 0.0, 30.0, 60.0] {
            let y = mercator_y(lat);
            let back = mercator_lat(y);
            assert!((back - lat).abs() < 1e-9, "lat={lat} back={back}");
        }
    }

    #[test]
    fn coastline_non_empty() {
        let (lon, lat) = coastline_lonlat();
        assert!(lon.len() > 100);
        assert_eq!(lon.len(), lat.len());
        assert_eq!(lon.len(), coastline_point_count());
        let finite = lon.as_slice().iter().filter(|v| v.is_finite()).count();
        assert!(finite > 100);
    }

    #[test]
    fn project_preserves_nan_breaks() {
        let lon = Series::new(vec![0.0, f64::NAN, 10.0]);
        let lat = Series::new(vec![0.0, f64::NAN, 20.0]);
        let (x, y) = project_lonlat(GeoProjection::PlateCarree, &lon, &lat).unwrap();
        assert!(x.as_slice()[1].is_nan());
        assert!(y.as_slice()[1].is_nan());
        assert!((x.as_slice()[2] - 10.0).abs() < 1e-12);
    }
}
