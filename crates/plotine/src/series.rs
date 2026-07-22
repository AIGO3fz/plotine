//! Data input adapters (`IntoSeries`).

/// Owned numeric series used by plot artists.
///
/// A thin wrapper over `Vec<f64>` that provides convenient min/max scanning
/// with NaN-filtering. You rarely construct this directly — pass slices,
/// `Vec<f64>`, arrays, or ndarray arrays to plotting methods and they will be
/// converted via [`IntoSeries`].
///
/// ```
/// use plotine::Series;
///
/// let s = Series::new(vec![1.0, 2.0, f64::NAN, 4.0]);
/// assert_eq!(s.min_max(), Some((1.0, 4.0)));
/// assert_eq!(s.len(), 4);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    values: Vec<f64>,
}

impl Series {
    /// Wrap an owned `Vec<f64>` as a series.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    /// Borrow the underlying values.
    pub fn as_slice(&self) -> &[f64] {
        &self.values
    }

    /// Mutable borrow of the underlying values (animation / in-place updates).
    pub fn as_mut_slice(&mut self) -> &mut [f64] {
        &mut self.values
    }

    /// Replace all values (used by artist `set_y` / `set_xy`).
    pub fn replace(&mut self, values: Vec<f64>) {
        self.values = values;
    }

    /// Number of samples (including non-finite values).
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// `true` when the series contains no samples.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterate over values by copy.
    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.values.iter().copied()
    }

    /// Finite minimum and maximum, or `None` if every value is non-finite.
    pub fn min_max(&self) -> Option<(f64, f64)> {
        let mut iter = self.values.iter().copied().filter(|v| v.is_finite());
        let first = iter.next()?;
        let (mut min, mut max) = (first, first);
        for v in iter {
            min = min.min(v);
            max = max.max(v);
        }
        Some((min, max))
    }
}

/// Convert common Rust numeric containers into a [`Series`].
///
/// Implemented for `f64` (length-1 broadcast), `Vec<f64>`, `&[f64]`, `[f64; N]`,
/// `Vec<f32>`, `&[f32]`, `ndarray::Array1<f64>` (feature `ndarray`), and
/// `Series` itself. This allows all plot methods to accept diverse input types
/// ergonomically.
///
/// ```
/// use plotine::{IntoSeries, Series};
///
/// let from_slice: Series = [1.0_f64, 2.0, 3.0].into_series();
/// let from_f32: Series = vec![1.0_f32, 2.0].into_series();
/// assert_eq!(from_slice.len(), 3);
/// assert_eq!(from_f32.as_slice(), &[1.0, 2.0]);
/// ```
pub trait IntoSeries {
    /// Convert `self` into an owned [`Series`].
    fn into_series(self) -> Series;
}

impl IntoSeries for Series {
    fn into_series(self) -> Series {
        self
    }
}

impl IntoSeries for &Series {
    fn into_series(self) -> Series {
        self.clone()
    }
}

impl IntoSeries for f64 {
    fn into_series(self) -> Series {
        Series::new(vec![self])
    }
}

impl IntoSeries for Vec<f64> {
    fn into_series(self) -> Series {
        Series::new(self)
    }
}

impl IntoSeries for &[f64] {
    fn into_series(self) -> Series {
        Series::new(self.to_vec())
    }
}

impl IntoSeries for &Vec<f64> {
    fn into_series(self) -> Series {
        Series::new(self.clone())
    }
}

impl<const N: usize> IntoSeries for [f64; N] {
    fn into_series(self) -> Series {
        Series::new(self.to_vec())
    }
}

impl<const N: usize> IntoSeries for &[f64; N] {
    fn into_series(self) -> Series {
        Series::new(self.to_vec())
    }
}

impl IntoSeries for &[f32] {
    fn into_series(self) -> Series {
        Series::new(self.iter().map(|&v| v as f64).collect())
    }
}

impl IntoSeries for Vec<f32> {
    fn into_series(self) -> Series {
        Series::new(self.into_iter().map(|v| v as f64).collect())
    }
}

#[cfg(feature = "ndarray")]
mod ndarray_impl {
    use super::{IntoSeries, Series};

    impl IntoSeries for ndarray::Array1<f64> {
        fn into_series(self) -> Series {
            Series::new(self.to_vec())
        }
    }

    impl IntoSeries for &ndarray::Array1<f64> {
        fn into_series(self) -> Series {
            Series::new(self.iter().copied().collect())
        }
    }

    impl IntoSeries for ndarray::ArrayView1<'_, f64> {
        fn into_series(self) -> Series {
            Series::new(self.iter().copied().collect())
        }
    }

    impl IntoSeries for ndarray::ArrayViewMut1<'_, f64> {
        fn into_series(self) -> Series {
            Series::new(self.iter().copied().collect())
        }
    }

    /// Flatten a 2D array into row-major `(nrows, ncols, values)`.
    pub fn array2_row_major(values: &ndarray::Array2<f64>) -> (usize, usize, Series) {
        let nrows = values.nrows();
        let ncols = values.ncols();
        let mut flat = Vec::with_capacity(nrows * ncols);
        for r in 0..nrows {
            for c in 0..ncols {
                flat.push(values[(r, c)]);
            }
        }
        (nrows, ncols, Series::new(flat))
    }
}

#[cfg(feature = "ndarray")]
pub use ndarray_impl::array2_row_major;
