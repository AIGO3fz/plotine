# plotine

**English** | [中文](README.zh-CN.md)

A high-level, LLM-friendly **Rust native scientific plotting library** (static 2D + basic 3D).

> Plotters' spiritual successor: publication-quality defaults, type-safe APIs that fail at compile time when possible, and errors that tell agents how to fix themselves. Visual defaults track stock matplotlib — the API stays Rust-native.
>
> Post-M8 alignment: **M9–M13** done; **M14–M17** add EPS/PGF/MP4, non-blocking GUI + widgets, stats/GeoJSON, docs. Gap scores: [`docs/MPL_GAP.md`](docs/MPL_GAP.md).

## Status

**v0.5.0** — M0–M8 static charts + M9–M17 (GUI, animation, geo, pyplot façade, LaTeX,
PGF/EPS/MP4, widgets, stats/GeoJSON, docs). Gap scores: [`docs/MPL_GAP.md`](docs/MPL_GAP.md).
First crates.io upload: follow [`docs/RELEASING.md`](docs/RELEASING.md) (`cargo login` required).

## Gallery

<p align="center">
  <img src="docs/images/02_line.png" width="32%" alt="line" />
  <img src="docs/images/18_heatmap.png" width="32%" alt="heatmap" />
  <img src="docs/images/30_contour_pcolor.png" width="32%" alt="contour" />
</p>
<p align="center">
  <img src="docs/images/44_surface_3d.png" width="32%" alt="surface 3d" />
  <img src="docs/images/52_mathtext.png" width="32%" alt="mathtext" />
  <img src="docs/images/69_geo_map.png" width="32%" alt="geo map" />
</p>

Full set: `cargo run -p plotine --example gallery` → `./gallery/`.

## Installation

```toml
plotine = "0.5"
# Optional extras:
# plotine = { version = "0.5", features = ["gui", "gif", "mp4", "eps", "latex"] }
```

## Quick start

```rust
use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    Figure::new()
        .subplots(2, 1, |g| {
            g.at(0, 0, |ax| {
                ax.line(&x, &y).color(Color::CRIMSON).width(2.0);
                ax.title("Top");
            });
            g.at(1, 0, |ax| {
                ax.scatter(&x, &y).size(3.0);
                ax.title("Bottom");
            });
        })
        .save("out.png")?; // also .svg / .pdf
    Ok(())
}
```

```bash
cargo run -p plotine --example gallery                 # → ./gallery/ (69 figures)
cargo run -p plotine --example matplotlib_compare      # → ./compare/plotine_*.png
python scripts/matplotlib_compare.py                   # → ./compare/mpl_*.png + index.html
cargo run -p plotine --example violin_demo
cargo run -p plotine --features polars --example polars_line
cargo test -p plotine --features polars,ndarray,evcxr,cjk
cargo run -p plotine --example cjk_labels --features cjk   # system CJK font
cargo run -p plotine --example interactive_show --features gui  # pan/zoom / 3D rotate
cargo run -p plotine --example interactive_widgets --features gui
cargo run -p plotine --example animate_wave --features "gif,mp4" # PNG + GIF + optional MP4
cargo run -p plotine --example export_formats                   # png/svg/pdf/pgf (+ eps)
cargo run -p plotine --example usetex_demo --features latex     # system LaTeX (needs TeX)
cargo run -p plotine-pyplot --example migrate_pyplot            # opt-in pyplot facade
python scripts/benchmark.py --tier smoke                       # Tier S vs mpl (release)
python scripts/benchmark.py                                    # full Tier A
python scripts/size_benchmark.py                               # size/time smoke: M9–M13 vs mpl
```

```rust
// Polars: three lines after you have a DataFrame
let (x, y) = plotine::polars::xy(&df, "x", "y")?;
Figure::new().axes(|ax| { ax.line(&x, &y); }).save("out.png")?;

// evcxr Jupyter (feature = "evcxr")
Figure::new().axes(|ax| { ax.line(&x, &y); }).evcxr_display()?;
```

## Performance

End-to-end `Figure` build + export (release, median of 7 iters after 2 warmups;
5.0×3.5 in @ 150 DPI). Numbers below are indicative (Windows laptop); re-run
locally or via the [Benchmark](.github/workflows/benchmark.yml) workflow.

| Scenario | plotine | matplotlib | speedup |
|---|---:|---:|---:|
| `series.line_n10000` | ~16 ms | ~42 ms | ~2.7× |
| `stat.heatmap_128` | ~24 ms | ~52 ms | ~2.2× |
| `d3.surface_40` | ~42 ms | ~92 ms | ~2.2× |
| `layout.subplots_2x2` | ~12 ms | ~112 ms | ~9× |
| `fmt.svg_line_n1000` | ~0.7 ms | ~25 ms | ~35× |

`speedup` = mpl / plotine (>1 ⇒ plotine faster). See [`docs/BENCHMARK.md`](docs/BENCHMARK.md)
for the full suite (`smoke` / `default` / `stress` tiers).

```bash
python scripts/benchmark.py --tier smoke    # CI-sized
python scripts/benchmark.py --tier stress   # includes 1e6-point line, 512² heatmap, …
cargo bench -p plotine --bench hotspots -- --quick   # L2 Criterion (maintainers)
```

## Charts & features

| API | Notes |
|---|---|
| `line` / `scatter` / `bar` / `hist` / `area` / `errorbar` | M1 core set |
| `heatmap` + `.cmap` / `.colorbar` | row-major grid; Viridis…Cividis |
| `boxplot` / `violin` | Tukey / Gaussian KDE |
| `plotine::polars::xy` (`feature = "polars"`) | DataFrame columns → `Series` |
| `Array1` / `heatmap_array` (`feature = "ndarray"`) | ndarray adapters |
| `Figure::evcxr_display` (`feature = "evcxr"`) | Jupyter inline PNG |
| `Figure::subplots` / `GridSpec` | multi-panel grid + hspace/wspace |
| `x_datetime` / `y_datetime` | Unix seconds → date tick labels |
| `legend(Legend::…)` | 13 positions incl. `Best` / `Outside*` |
| `x_scale` / `y_scale` (`Linear` / `Log` / `Symlog`) | set **before** artists for log auto-limits |
| `Theme::light/dark/paper` | built-in themes |
| `.save("out.png"\|"out.svg"\|"out.pdf"\|"out.pgf")` | + `.eps` with `feature = "eps"` (Ghostscript) |
| `Figure::show` / `show_nonblocking` / `show_with` (`gui`) | Blocking or ion()-like; Slider/Button side panel |
| `Figure::animate` / `Animation` | PNG sequence / GIF (`gif`) / MP4 (`mp4` + ffmpeg) |
| `ax.projection` / `coastline` / `geojson` | PlateCarree / Mercator + NE 110m + GeoJSON overlay |
| `plotine::stats` | `corr_heatmap` / `pair_scatter` / `regline` (seaborn-thin) |
| `plotine-pyplot` (separate crate) | Opt-in `plt::plot` / `savefig` facade — **not** the primary API |
| `Figure::usetex` (`feature = "latex"`) | System `latex`+`dvipng`; default remains built-in mathtext |

## Documentation

### English

| Document | Description |
|----------|-------------|
| [`book/`](book/) | mdBook user guide + tutorials (`mdbook serve book`) |
| [`docs/MPL_GAP.md`](docs/MPL_GAP.md) | Feature comparison & scores vs matplotlib |
| [`docs/API_FREEZE.md`](docs/API_FREEZE.md) | 0.5 → 1.0-rc API stability review |
| [`docs/RELEASING.md`](docs/RELEASING.md) | crates.io publish cadence & checklist |
| [`docs/BENCHMARK.md`](docs/BENCHMARK.md) | Benchmark design (scenarios + methodology) |
| [`docs/GUI_TOOLBAR.md`](docs/GUI_TOOLBAR.md) | Interactive GUI capability matrix |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | PR workflow & visual-review expectations |
| [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | Community standards (Contributor Covenant) |
| [`SECURITY.md`](SECURITY.md) | Vulnerability reporting policy |
| [`AGENTS.md`](AGENTS.md) | Idioms & migration guide for coding agents |
| [`llms.txt`](llms.txt) / [`llms-full.txt`](llms-full.txt) | LLM-oriented API index |
| [`CHANGELOG.md`](CHANGELOG.md) | Release history & breaking changes |

### 中文 (Chinese)

| 文档 | 说明 |
|------|------|
| [`docs/DEVELOPMENT_PLAN.md`](docs/DEVELOPMENT_PLAN.md) | 顶层架构设计 & 里程碑路线图 / Architecture & roadmap |
| [`docs/MPL_2D_COVERAGE.md`](docs/MPL_2D_COVERAGE.md) | matplotlib 静态 2D 图型覆盖清单 / Chart coverage checklist |

## License

MIT — see [LICENSE](LICENSE).
Embedded DejaVu Sans retains its own license (see `crates/plotine-text/fonts/LICENSE`).
Natural Earth 110m coastline data is public domain.
Full third-party attribution: [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md).
