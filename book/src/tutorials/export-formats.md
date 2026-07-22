# Export formats

| Extension | Feature | Notes |
|---|---|---|
| `.png` | `png` (default) | Raster via tiny-skia |
| `.svg` | `svg` (default) | Deterministic vector |
| `.pdf` | `pdf` (default) | SVG → svg2pdf |
| `.pgf` | `pgf` (default) | TikZ/`pgfpicture` for LaTeX `\input` |
| `.eps` | `eps` | PDF → Ghostscript `eps2write` (needs `gs` on PATH) |

Animation:

| API | Feature | Tool |
|---|---|---|
| `Animation::save_png_sequence` | `png` | — |
| `Animation::save_gif` | `gif` | — |
| `Animation::save_mp4` | `mp4` | system `ffmpeg` |

```toml
plotine = { version = "0.5", features = ["eps", "mp4", "gif"] }
```

Missing tools return `PlotError::ExternalToolUnavailable` with a suggestion string.
