# plotine vs matplotlib — Feature Comparison

> Scores relative to matplotlib (= 100). Reflects v0.5 / M17 capabilities.
> Static 2D checklist: [`MPL_2D_COVERAGE.md`](MPL_2D_COVERAGE.md); GUI: [`GUI_TOOLBAR.md`](GUI_TOOLBAR.md).

## How to read this document

plotine is a Rust-native plotting library that uses matplotlib's visual defaults as a
reference for publication-quality output. This document tracks where plotine stands
relative to matplotlib's feature surface and explains our intentional design choices.

matplotlib is a mature, battle-tested library with 20+ years of development and a
massive ecosystem. plotine does not aim to replicate all of matplotlib — it targets
a focused subset with different architectural trade-offs suited to the Rust ecosystem.

## Feature completeness

| Area | Status | Notes |
|---|---|---|
| PNG / SVG / PDF / PGF; EPS; GIF / MP4 | ✅ | Clear `ExternalToolNotFound` errors when system tools are missing |
| Interactive window (pan / zoom / export) | ✅ | Single egui backend; blocking + non-blocking + widget side panels |
| Stats helpers + GeoJSON overlay | ✅ | `corr_heatmap` / `pair_scatter` / `regline` (thin layer) |
| Docs (tutorials / CONTRIBUTING / Issues) | ✅ | mdBook guide + community templates |
| `Norm::TwoSlope` + `SegmentedColormap` | ✅ | Diverging data + custom colormaps |
| Static visual fidelity | Ongoing | Continuous improvement; pixel-exact match is not a goal |

## Intentional design choices (not planned)

plotine makes deliberate architectural decisions that differ from matplotlib:

| Feature | plotine's choice | Rationale |
|---|---|---|
| GUI backend | Single (egui) | Simpler architecture; one well-tested interactive path |
| API style | Builder pattern (`Figure::new().axes(…)`) | No global mutable state; compile-time type safety |
| Full seaborn / plotnine equivalent | Thin `stats` helpers only | Focus on core plotting; avoid scope explosion |
| Full geopandas / arbitrary CRS | Projection + coastline + GeoJSON | Sufficient for publication maps without GIS complexity |
| Large community (SO / books) | Documentation-first approach | Community grows with adoption; cannot be engineered upfront |

## Relative scores (capability snapshot)

| Dimension | Score | Notes |
|---|---|---|
| Static 2D breadth | ~95 | All core Axes methods covered |
| Visual fidelity | ~85 | Skia vs Agg rendering differences; font metric variance |
| Output formats | ~90 | PNG / SVG / PDF / PGF / EPS / GIF / MP4 |
| Interactivity | ~60 | Single backend; not multi-toolkit |
| Ecosystem integration | ~40 | Polars / ndarray adapters + thin stats/geo (not full-stack) |
| Documentation | ~40 | Tutorials complete; community scale takes time |
| Colormap / Norm | ~90 | 82 named maps + TwoSlope + Segmented |
| Math typesetting | ~65 | Built-in mathtext + optional external LaTeX |

## Roadmap

M9–M13 (GUI / animation / geo / pyplot / LaTeX) → M14–M17 (formats / widgets / stats / docs)
→ ongoing visual fidelity + community growth.

Full roadmap: [`DEVELOPMENT_PLAN.md`](DEVELOPMENT_PLAN.md).
