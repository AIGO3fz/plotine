//! Helpers for optional system tools (ffmpeg, Ghostscript).

use std::path::Path;
use std::process::Command;

use plotine_core::{PlotError, Result};

fn tool_runs(name: &str, version_args: &[&str]) -> bool {
    Command::new(name)
        .args(version_args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Locate `ffmpeg` on PATH.
pub fn find_ffmpeg() -> Option<&'static str> {
    const CANDIDATES: &[(&str, &[&str])] = &[("ffmpeg", &["-version"])];
    for (name, args) in CANDIDATES {
        if tool_runs(name, args) {
            return Some(*name);
        }
    }
    None
}

/// Locate Ghostscript (`gs` on Unix, `gswin64c` / `gswin32c` on Windows).
pub fn find_ghostscript() -> Option<&'static str> {
    const CANDIDATES: &[(&str, &[&str])] = &[
        ("gs", &["--version"]),
        ("gswin64c", &["--version"]),
        ("gswin32c", &["--version"]),
    ];
    for (name, args) in CANDIDATES {
        if tool_runs(name, args) {
            return Some(*name);
        }
    }
    None
}

/// Convert a PDF file to EPS via Ghostscript `eps2write`.
#[cfg(feature = "eps")]
pub fn pdf_to_eps(pdf_path: &Path, eps_path: &Path) -> Result<()> {
    let gs = find_ghostscript().ok_or_else(|| {
        PlotError::external_tool_unavailable("gs", "Ghostscript executable not found on PATH")
    })?;
    if let Some(parent) = eps_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| PlotError::io(e.to_string()))?;
        }
    }
    let out = Command::new(gs)
        .args([
            "-dBATCH",
            "-dNOPAUSE",
            "-dSAFER",
            "-sDEVICE=eps2write",
            &format!("-sOutputFile={}", eps_path.display()),
            pdf_path.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| {
            PlotError::external_tool_failed(gs, format!("failed to spawn Ghostscript: {e}"))
        })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(PlotError::external_tool_failed(
            gs,
            format!("Ghostscript exit {}: {stderr}", out.status),
        ));
    }
    Ok(())
}

/// Encode a PNG frame directory as MP4 via ffmpeg.
#[cfg(feature = "mp4")]
pub fn png_sequence_to_mp4(
    frames_dir: &Path,
    pattern: &str,
    fps: f64,
    mp4_path: &Path,
) -> Result<()> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| {
        PlotError::external_tool_unavailable("ffmpeg", "`ffmpeg` executable not found on PATH")
    })?;
    if let Some(parent) = mp4_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| PlotError::io(e.to_string()))?;
        }
    }
    let fps_s = format!("{fps:.3}");
    let input = frames_dir.join(pattern);
    let out = Command::new(ffmpeg)
        .args([
            "-y",
            "-framerate",
            &fps_s,
            "-i",
            input.to_str().unwrap_or(""),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            mp4_path.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| {
            PlotError::external_tool_failed(ffmpeg, format!("failed to spawn ffmpeg: {e}"))
        })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(PlotError::external_tool_failed(
            ffmpeg,
            format!("ffmpeg exit {}: {stderr}", out.status),
        ));
    }
    Ok(())
}
