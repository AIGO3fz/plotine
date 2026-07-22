//! Polars DataFrame → [`Series`](crate::Series) adapters (`feature = "polars"`).

use polars::prelude::{Column, DataFrame, DataType};

use crate::series::Series;
use plotine_core::{PlotError, Result};

/// Extract a numeric column as an owned [`Series`] (nulls → NaN).
///
/// ```no_run
/// # #[cfg(feature = "polars")]
/// # {
/// use plotine::prelude::*;
/// use polars::prelude::*;
///
/// fn demo(df: &DataFrame) -> plotine::Result<()> {
///     let (x, y) = plotine::polars::xy(df, "x", "y")?;
///     Figure::new()
///         .axes(|ax| {
///             ax.line(&x, &y).width(2.0);
///         })
///         .save("out.png")?;
///     Ok(())
/// }
/// # }
/// ```
pub fn column(df: &DataFrame, name: &str) -> Result<Series> {
    let col = df
        .column(name)
        .map_err(|_| PlotError::column_not_found(name))?;
    column_to_series(col, name)
}

/// Extract two numeric columns as `(x, y)`.
pub fn xy(df: &DataFrame, x: &str, y: &str) -> Result<(Series, Series)> {
    Ok((column(df, x)?, column(df, y)?))
}

fn column_to_series(col: &Column, name: &str) -> Result<Series> {
    let casted = col
        .cast(&DataType::Float64)
        .map_err(|_| PlotError::column_not_numeric(name, format!("{:?}", col.dtype())))?;
    let ca = casted
        .f64()
        .map_err(|_| PlotError::column_not_numeric(name, format!("{:?}", casted.dtype())))?;
    let values: Vec<f64> = ca.iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    Ok(Series::new(values))
}
