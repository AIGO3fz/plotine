//! Fallible operations return [`Result`] with actionable [`PlotError`] variants.
//!
//! ```
//! use plotine_core::{LogScale, PlotError};
//!
//! let err = LogScale::new(-1.0, 10.0).unwrap_err();
//! match err {
//!     PlotError::LogScaleNonPositive { suggestion, .. } => {
//!         assert!(suggestion.contains("Symlog"));
//!     }
//!     other => panic!("unexpected: {other}"),
//! }
//! ```

use thiserror::Error;

/// Convenient alias used throughout plotine.
pub type Result<T> = std::result::Result<T, PlotError>;

/// Library-wide error type. Every variant carries a human/agent-readable suggestion.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum PlotError {
    #[error("invalid range [{min}, {max}]: {message}. suggestion: {suggestion}")]
    InvalidRange {
        min: f64,
        max: f64,
        message: &'static str,
        suggestion: &'static str,
    },

    #[error("log scale rejected non-positive bound {value}. suggestion: {suggestion}")]
    LogScaleNonPositive {
        value: f64,
        suggestion: &'static str,
    },

    #[error("empty figure cannot be rendered. suggestion: {suggestion}")]
    EmptyFigure { suggestion: &'static str },

    #[error("unsupported output format for path `{path}`. suggestion: {suggestion}")]
    UnsupportedFormat {
        path: String,
        suggestion: &'static str,
    },

    #[error("I/O error: {message}. suggestion: {suggestion}")]
    Io {
        message: String,
        suggestion: &'static str,
    },

    #[error("render error: {message}. suggestion: {suggestion}")]
    Render {
        message: String,
        suggestion: &'static str,
    },

    #[error("text layout error: {message}. suggestion: {suggestion}")]
    Text {
        message: String,
        suggestion: &'static str,
    },

    #[error("x/y length mismatch: x={x_len}, y={y_len}. suggestion: {suggestion}")]
    LengthMismatch {
        x_len: usize,
        y_len: usize,
        suggestion: &'static str,
    },

    #[error(
        "heatmap size mismatch: expected {expected} values (nrows*ncols={nrows}*{ncols}), got {got}. suggestion: {suggestion}"
    )]
    HeatmapSizeMismatch {
        nrows: usize,
        ncols: usize,
        expected: usize,
        got: usize,
        suggestion: &'static str,
    },

    #[error("column `{name}` not found. suggestion: {suggestion}")]
    ColumnNotFound {
        name: String,
        suggestion: &'static str,
    },

    #[error("column `{name}` is not numeric (dtype={dtype}). suggestion: {suggestion}")]
    ColumnNotNumeric {
        name: String,
        dtype: String,
        suggestion: &'static str,
    },

    #[error("external LaTeX unavailable: {message}. suggestion: {suggestion}")]
    LatexUnavailable {
        message: String,
        suggestion: &'static str,
    },

    #[error("external LaTeX failed: {message}. suggestion: {suggestion}")]
    LatexFailed {
        message: String,
        suggestion: &'static str,
    },

    #[error("external tool `{tool}` unavailable: {message}. suggestion: {suggestion}")]
    ExternalToolUnavailable {
        tool: &'static str,
        message: String,
        suggestion: &'static str,
    },

    #[error("external tool `{tool}` failed: {message}. suggestion: {suggestion}")]
    ExternalToolFailed {
        tool: &'static str,
        message: String,
        suggestion: &'static str,
    },
}

impl PlotError {
    pub fn invalid_range(min: f64, max: f64) -> Self {
        Self::InvalidRange {
            min,
            max,
            message: "min must be strictly less than max",
            suggestion: "swap the bounds or widen the interval so max > min",
        }
    }

    pub fn log_non_positive(value: f64) -> Self {
        Self::LogScaleNonPositive {
            value,
            suggestion: "use ScaleType::Symlog, or filter/clip values so the domain is > 0",
        }
    }

    pub fn empty_figure() -> Self {
        Self::EmptyFigure {
            suggestion:
                "call Figure::axes(|ax| { ... }) or Figure::subplots(...) before save/render",
        }
    }

    pub fn unsupported_format(path: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            path: path.into(),
            suggestion:
                "use a path ending in .png, .svg, .pdf, .eps, or .pgf (enable eps/mp4 features when needed)",
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            suggestion: "check that the parent directory exists and is writable",
        }
    }

    pub fn render(message: impl Into<String>) -> Self {
        Self::Render {
            message: message.into(),
            suggestion: "verify figure size, DPI, and that all axes have valid ranges",
        }
    }

    pub fn text(message: impl Into<String>) -> Self {
        Self::Text {
            message: message.into(),
            suggestion: "ensure the embedded font loaded; file a bug if this persists in CI",
        }
    }

    pub fn length_mismatch(x_len: usize, y_len: usize) -> Self {
        Self::LengthMismatch {
            x_len,
            y_len,
            suggestion: "ensure x and y (and yerr for errorbar) have the same length",
        }
    }

    pub fn heatmap_size_mismatch(nrows: usize, ncols: usize, got: usize) -> Self {
        Self::HeatmapSizeMismatch {
            nrows,
            ncols,
            expected: nrows.saturating_mul(ncols),
            got,
            suggestion: "pass a flat row-major slice with length nrows * ncols",
        }
    }

    pub fn column_not_found(name: impl Into<String>) -> Self {
        Self::ColumnNotFound {
            name: name.into(),
            suggestion: "check the column name (case-sensitive) or print df.get_column_names()",
        }
    }

    pub fn column_not_numeric(name: impl Into<String>, dtype: impl Into<String>) -> Self {
        Self::ColumnNotNumeric {
            name: name.into(),
            dtype: dtype.into(),
            suggestion: "cast the column to a numeric dtype (e.g. Float64 / Int64) before plotting",
        }
    }

    pub fn latex_unavailable(message: impl Into<String>) -> Self {
        Self::LatexUnavailable {
            message: message.into(),
            suggestion: "install TeX Live or MiKTeX and ensure `latex` and `dvipng` are on PATH; or omit Figure::usetex(true) / disable feature \"latex\" to use built-in mathtext",
        }
    }

    pub fn latex_failed(message: impl Into<String>) -> Self {
        Self::LatexFailed {
            message: message.into(),
            suggestion: "inspect the LaTeX log in the error message; simplify the formula / preamble; or disable usetex to use built-in mathtext",
        }
    }

    pub fn external_tool_unavailable(tool: &'static str, message: impl Into<String>) -> Self {
        let suggestion = match tool {
            "ffmpeg" => {
                "install ffmpeg and ensure it is on PATH; or use Animation::save_gif / save_png_sequence"
            }
            "gs" | "gswin64c" | "gswin32c" => {
                "install Ghostscript and ensure `gs` (or gswin64c on Windows) is on PATH; or save as .pdf/.svg instead"
            }
            _ => "install the required external tool and ensure it is on PATH",
        };
        Self::ExternalToolUnavailable {
            tool,
            message: message.into(),
            suggestion,
        }
    }

    pub fn external_tool_failed(tool: &'static str, message: impl Into<String>) -> Self {
        Self::ExternalToolFailed {
            tool,
            message: message.into(),
            suggestion:
                "inspect the tool stderr in the error message; verify input paths and permissions",
        }
    }

    /// Extract the agent-facing fix hint from any error variant.
    pub fn suggestion(&self) -> &str {
        match self {
            Self::InvalidRange { suggestion, .. }
            | Self::LogScaleNonPositive { suggestion, .. }
            | Self::EmptyFigure { suggestion }
            | Self::UnsupportedFormat { suggestion, .. }
            | Self::Io { suggestion, .. }
            | Self::Render { suggestion, .. }
            | Self::Text { suggestion, .. }
            | Self::LengthMismatch { suggestion, .. }
            | Self::HeatmapSizeMismatch { suggestion, .. }
            | Self::ColumnNotFound { suggestion, .. }
            | Self::ColumnNotNumeric { suggestion, .. }
            | Self::LatexUnavailable { suggestion, .. }
            | Self::LatexFailed { suggestion, .. }
            | Self::ExternalToolUnavailable { suggestion, .. }
            | Self::ExternalToolFailed { suggestion, .. } => suggestion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_constructor_has_nonempty_suggestion() {
        let samples = [
            PlotError::invalid_range(1.0, 0.0),
            PlotError::log_non_positive(-3.0),
            PlotError::empty_figure(),
            PlotError::unsupported_format("out.jpg"),
            PlotError::io("disk full"),
            PlotError::render("bad pixmap"),
            PlotError::text("font lock"),
            PlotError::length_mismatch(3, 2),
            PlotError::heatmap_size_mismatch(2, 2, 3),
            PlotError::column_not_found("x"),
            PlotError::column_not_numeric("y", "String"),
            PlotError::latex_unavailable("latex not on PATH"),
            PlotError::latex_failed("dvipng exit 1"),
            PlotError::external_tool_unavailable("ffmpeg", "not on PATH"),
            PlotError::external_tool_failed("gs", "exit 1"),
        ];
        for err in samples {
            let s = err.suggestion();
            assert!(!s.is_empty(), "empty suggestion for {err}");
            assert!(s.len() >= 10, "suggestion too vague for {err}: {s:?}");
            let display = err.to_string();
            assert!(
                display.contains("suggestion:"),
                "Display should include suggestion: {display}"
            );
        }
    }
}
