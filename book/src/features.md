# Optional features

```toml
plotine = { version = "0.5", features = ["polars", "ndarray", "evcxr", "cjk", "gui", "gif", "mp4", "eps", "latex"] }
```

| Feature | Default | Purpose |
|---------|---------|---------|
| `png` | yes | tiny-skia PNG backend |
| `svg` | yes | deterministic SVG backend |
| `pdf` | yes | vector PDF (`svg` + svg2pdf) |
| `pgf` | yes | PGF/TikZ fragment for LaTeX |
| `eps` | no | EPS via Ghostscript (`gs`) from PDF |
| `polars` | no | `plotine::polars::xy(&df, "x", "y")` |
| `ndarray` | no | `IntoSeries` for `Array1` + `heatmap_array` |
| `evcxr` | no | `Figure::evcxr_display()` for Jupyter |
| `cjk` | no | system / user CJK font loading (`plotine::fonts`) |
| `gui` | no | `Figure::show` / `show_nonblocking` / `show_with` |
| `gif` | no | `Animation::save_gif` |
| `mp4` | no | `Animation::save_mp4` via system `ffmpeg` |
| `latex` | no | `Figure::usetex` via system `latex`+`dvipng` |

## Polars

```rust
let (x, y) = plotine::polars::xy(&df, "x", "y")?;
Figure::new().axes(|ax| { ax.line(&x, &y); }).save("p.png")?;
```

## ndarray heatmap

```rust
ax.heatmap_array(&array2);
```

## evcxr

```rust
// last expression in a notebook cell
Figure::new().axes(|ax| { ax.line(&x, &y); }).evcxr_display()?;
```

## Stats helpers

```rust
use plotine::stats::{corr_heatmap, pair_scatter, regline};
```

See [Export formats](tutorials/export-formats.md) and [Interactive](tutorials/interactive.md).
