//! Offline multi-frame animation (matplotlib `FuncAnimation` semantics).
//!
//! Does **not** require the `gui` feature. Typical flow:
//!
//! 1. Build a [`Figure`](crate::figure::Figure) with fixed axis ranges.
//! 2. Call [`Figure::animate`](crate::figure::Figure::animate) to update artists each frame.
//! 3. Export with [`Animation::save_png_sequence`] or `Animation::save_gif`
//!    (`feature = "gif"`).
//!
//! Alternatively, rebuild a figure per frame with [`Animation::map`].

use std::fs;
use std::path::{Path, PathBuf};

use plotine_core::{PlotError, Result};

use crate::figure::Figure;

/// One rendered RGBA8 frame.
#[derive(Debug, Clone)]
pub struct AnimFrame {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Tight RGBA8 buffer (`width * height * 4`).
    pub rgba: Vec<u8>,
}

/// Offline animation: a sequence of rendered frames plus a playback interval.
#[derive(Debug, Clone)]
pub struct Animation {
    frames: Vec<AnimFrame>,
    interval_ms: u32,
}

impl Animation {
    /// Default interval is 50 ms (20 fps), matching a common matplotlib default.
    pub fn new(frames: Vec<AnimFrame>) -> Self {
        Self {
            frames,
            interval_ms: 50,
        }
    }

    /// Build by mapping each frame key to a fresh [`Figure`] (rebuild-each-frame).
    #[cfg(feature = "png")]
    pub fn map<I, F>(frame_keys: I, mut build: F) -> Result<Self>
    where
        I: IntoIterator,
        F: FnMut(I::Item) -> Result<Figure>,
    {
        let mut frames = Vec::new();
        let mut expected: Option<(u32, u32)> = None;
        for key in frame_keys {
            let fig = build(key)?;
            let (w, h, rgba) = fig.render_rgba()?;
            if let Some((ew, eh)) = expected {
                if w != ew || h != eh {
                    return Err(PlotError::render(format!(
                        "animation frame size mismatch: expected {ew}x{eh}, got {w}x{h}"
                    )));
                }
            } else {
                expected = Some((w, h));
            }
            frames.push(AnimFrame {
                width: w,
                height: h,
                rgba,
            });
        }
        if frames.is_empty() {
            return Err(PlotError::render(
                "animation has no frames; pass a non-empty frame iterator",
            ));
        }
        Ok(Self::new(frames))
    }

    /// Delay between frames in milliseconds (GIF delay uses centiseconds).
    pub fn interval_ms(mut self, ms: u32) -> Self {
        self.interval_ms = ms.max(1);
        self
    }

    /// Number of frames.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// `true` when there are no frames.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Pixel size shared by all frames `(width, height)`.
    pub fn frame_size(&self) -> Option<(u32, u32)> {
        self.frames.first().map(|f| (f.width, f.height))
    }

    /// Borrow rendered frames.
    pub fn frames(&self) -> &[AnimFrame] {
        &self.frames
    }

    /// Playback interval in milliseconds.
    pub fn interval(&self) -> u32 {
        self.interval_ms
    }

    /// Write `frame_0000.png`, `frame_0001.png`, … into `dir` (created if needed).
    #[cfg(feature = "png")]
    pub fn save_png_sequence(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir).map_err(|e| PlotError::io(e.to_string()))?;
        for (i, frame) in self.frames.iter().enumerate() {
            let path = dir.join(format!("frame_{i:04}.png"));
            write_rgba_png(&path, frame)?;
        }
        Ok(())
    }

    /// Encode all frames as an animated GIF.
    #[cfg(feature = "gif")]
    pub fn save_gif(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| PlotError::io(e.to_string()))?;
            }
        }
        let (w, h) = self
            .frame_size()
            .ok_or_else(|| PlotError::render("cannot save empty animation as GIF"))?;
        if w > u16::MAX as u32 || h > u16::MAX as u32 {
            return Err(PlotError::render(format!(
                "GIF dimensions {w}x{h} exceed u16::MAX"
            )));
        }
        let file = fs::File::create(path).map_err(|e| PlotError::io(e.to_string()))?;
        let mut encoder = gif::Encoder::new(file, w as u16, h as u16, &[])
            .map_err(|e| PlotError::io(format!("GIF encoder: {e}")))?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| PlotError::io(format!("GIF repeat: {e}")))?;
        // GIF delay unit = 10 ms.
        let delay = ((self.interval_ms + 5) / 10).max(1) as u16;
        for frame in &self.frames {
            let mut rgba = frame.rgba.clone();
            let mut gif_frame = gif::Frame::from_rgba_speed(w as u16, h as u16, &mut rgba, 10);
            gif_frame.delay = delay;
            encoder
                .write_frame(&gif_frame)
                .map_err(|e| PlotError::io(format!("GIF write frame: {e}")))?;
        }
        Ok(())
    }

    /// Encode all frames as an H.264 MP4 via system `ffmpeg` (`feature = "mp4"`).
    ///
    /// Writes a temporary PNG sequence, then shells out to `ffmpeg`. Requires
    /// `ffmpeg` on `PATH` (same model as matplotlib `FFMpegWriter`).
    #[cfg(feature = "mp4")]
    pub fn save_mp4(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if self.frames.is_empty() {
            return Err(PlotError::render("cannot save empty animation as MP4"));
        }
        let dir = tempfile::tempdir().map_err(|e| {
            PlotError::external_tool_failed("ffmpeg", format!("temp directory: {e}"))
        })?;
        self.save_png_sequence(dir.path())?;
        let fps = 1000.0 / f64::from(self.interval_ms.max(1));
        crate::ext_tools::png_sequence_to_mp4(dir.path(), "frame_%04d.png", fps, path)
    }
}

impl Figure {
    /// Offline FuncAnimation-style loop: update `self` each frame, then render.
    ///
    /// Does not open a GUI window. Prefer fixed `x_range` / `y_range` so limits
    /// do not jump when calling [`LinePlot::set_y`](crate::artist::LinePlot::set_y).
    ///
    /// ```
    /// # #[cfg(feature = "png")]
    /// # {
    /// use plotine::prelude::*;
    ///
    /// let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.1).collect();
    /// let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    /// let fig = Figure::new().size(3.0, 2.0).dpi(72.0).axes(|ax| {
    ///     ax.line(&x, &y0).color(Color::CRIMSON);
    ///     ax.y_range(-1.2, 1.2);
    /// });
    /// let expected = fig.pixel_size();
    /// let anim = fig
    ///     .animate(0..4, |fig, i| {
    ///         let t = i as f64 * 0.3;
    ///         let y: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
    ///         fig.axes_at_mut(0)
    ///             .unwrap()
    ///             .line_at_mut(0)
    ///             .unwrap()
    ///             .set_y(&y)?;
    ///         Ok(())
    ///     })
    ///     .unwrap();
    /// assert_eq!(anim.len(), 4);
    /// assert_eq!(anim.frame_size(), Some(expected));
    /// # }
    /// ```
    #[cfg(feature = "png")]
    pub fn animate<I, F>(mut self, frame_keys: I, mut update: F) -> Result<Animation>
    where
        I: IntoIterator,
        F: FnMut(&mut Figure, I::Item) -> Result<()>,
    {
        let mut frames = Vec::new();
        let mut expected: Option<(u32, u32)> = None;
        for key in frame_keys {
            update(&mut self, key)?;
            let (w, h, rgba) = self.render_rgba()?;
            if let Some((ew, eh)) = expected {
                if w != ew || h != eh {
                    return Err(PlotError::render(format!(
                        "animation frame size mismatch: expected {ew}x{eh}, got {w}x{h}"
                    )));
                }
            } else {
                expected = Some((w, h));
            }
            frames.push(AnimFrame {
                width: w,
                height: h,
                rgba,
            });
        }
        if frames.is_empty() {
            return Err(PlotError::render(
                "animation has no frames; pass a non-empty frame iterator",
            ));
        }
        Ok(Animation::new(frames))
    }
}

#[cfg(feature = "png")]
fn write_rgba_png(path: &Path, frame: &AnimFrame) -> Result<()> {
    let mut encoder = png::Encoder::new(
        fs::File::create(path).map_err(|e| PlotError::io(e.to_string()))?,
        frame.width,
        frame.height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| PlotError::io(format!("PNG header: {e}")))?;
    writer
        .write_image_data(&frame.rgba)
        .map_err(|e| PlotError::io(format!("PNG data: {e}")))?;
    Ok(())
}

/// Convenience: directory path for tests / examples under `target/`.
#[allow(dead_code)]
pub(crate) fn target_anim_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::Figure;
    #[cfg(feature = "gif")]
    use plotine_core::Color;

    #[cfg(feature = "png")]
    #[test]
    fn animate_frame_count_and_size() {
        let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.1).collect();
        let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();
        let fig = Figure::new().size(3.2, 2.4).dpi(50.0).axes(|ax| {
            ax.line(&x, &y0);
            ax.y_range(-1.2, 1.2);
        });
        let (ew, eh) = fig.pixel_size();
        let anim = fig
            .animate(0..5, |fig, i| {
                let t = i as f64 * 0.25;
                let y: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
                fig.axes_at_mut(0)
                    .expect("panel")
                    .line_at_mut(0)
                    .expect("line")
                    .set_y(&y)?;
                Ok(())
            })
            .expect("animate");
        assert_eq!(anim.len(), 5);
        assert_eq!(anim.frame_size(), Some((ew, eh)));
        for f in anim.frames() {
            assert_eq!(f.rgba.len(), (ew * eh * 4) as usize);
        }
    }

    /// Prove `set_y` actually changes pixels between frames (not a frozen first frame).
    #[cfg(feature = "png")]
    #[test]
    fn animate_frames_differ() {
        let x: Vec<f64> = (0..40).map(|i| i as f64 * 0.1).collect();
        let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();
        let fig = Figure::new().size(3.2, 2.4).dpi(50.0).axes(|ax| {
            ax.line(&x, &y0);
            ax.y_range(-1.2, 1.2);
        });
        let anim = fig
            .animate(0..3, |fig, i| {
                let t = i as f64 * 0.5;
                let y: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
                fig.axes_at_mut(0)
                    .expect("panel")
                    .line_at_mut(0)
                    .expect("line")
                    .set_y(&y)?;
                Ok(())
            })
            .expect("animate");
        assert_ne!(
            anim.frames()[0].rgba,
            anim.frames()[1].rgba,
            "frame 0 and 1 should differ after set_y"
        );
        assert_ne!(
            anim.frames()[1].rgba,
            anim.frames()[2].rgba,
            "frame 1 and 2 should differ after set_y"
        );
    }

    #[cfg(feature = "png")]
    #[test]
    fn map_rebuild_frames() {
        let anim = Animation::map(0..3, |i| {
            let x = [0.0, 1.0, 2.0];
            let y = [0.0, i as f64, 0.0];
            Ok(Figure::new().size(2.0, 1.5).dpi(40.0).axes(|ax| {
                ax.line(x, y);
                ax.y_range(-1.0, 3.0);
            }))
        })
        .expect("map");
        assert_eq!(anim.len(), 3);
        let (w, h) = anim.frame_size().unwrap();
        assert_eq!((w, h), (80, 60));
    }

    #[cfg(feature = "png")]
    #[test]
    fn save_png_sequence_writes_files() {
        let dir = target_anim_dir("plotine_anim_png_test");
        let _ = fs::remove_dir_all(&dir);
        let anim = Animation::map(0..2, |_| {
            Ok(Figure::new().size(1.0, 1.0).dpi(32.0).axes(|ax| {
                ax.line([0.0, 1.0], [0.0, 1.0]);
            }))
        })
        .unwrap();
        anim.save_png_sequence(&dir).unwrap();
        assert!(dir.join("frame_0000.png").is_file());
        assert!(dir.join("frame_0001.png").is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(feature = "gif")]
    #[test]
    fn save_gif_writes_file() {
        let path = target_anim_dir("plotine_anim_gif_test").join("wave.gif");
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        let _ = fs::remove_file(&path);
        let anim = Animation::map(0..3, |i| {
            let x = [0.0, 1.0, 2.0, 3.0];
            let y = [0.0, (i as f64 * 0.5).sin(), (i as f64).sin(), 0.0];
            Ok(Figure::new().size(1.5, 1.0).dpi(40.0).axes(|ax| {
                ax.line(x, y).color(Color::CRIMSON);
                ax.y_range(-1.2, 1.2);
            }))
        })
        .unwrap()
        .interval_ms(80);
        anim.save_gif(&path).unwrap();
        let bytes = fs::read(&path).expect("read gif");
        assert!(
            bytes.starts_with(b"GIF89a") || bytes.starts_with(b"GIF87a"),
            "GIF magic missing; got {:?}",
            &bytes[..bytes.len().min(8)]
        );
        assert!(bytes.len() > 32);
        let _ = fs::remove_file(&path);
    }

    #[cfg(feature = "png")]
    #[test]
    fn empty_frames_errors() {
        let err = Animation::map(0..0, |_| {
            Ok(Figure::new().axes(|ax| {
                ax.line([0.0], [0.0]);
            }))
        });
        assert!(err.is_err());
    }

    #[test]
    fn set_y_length_mismatch() {
        let mut fig = Figure::new().axes(|ax| {
            ax.line([0.0, 1.0, 2.0], [0.0, 1.0, 0.0]);
        });
        let err = fig
            .axes_at_mut(0)
            .unwrap()
            .line_at_mut(0)
            .unwrap()
            .set_y([0.0, 1.0]);
        assert!(err.is_err());
    }
}
