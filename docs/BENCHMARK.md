# plotine Benchmark Design

Status: **P0–P4 implemented** (L1 product suite + L2 Criterion hotspots).  
Related:
- L1: `crates/plotine/examples/bench_suite.rs` + `scripts/benchmark.py`
- L2: `crates/plotine/benches/hotspots.rs` (`cargo bench -p plotine --bench hotspots`)
- `.github/workflows/benchmark.yml` (`workflow_dispatch` + weekly smoke)
- legacy: `scripts/size_benchmark.py`

Tiers (`BENCH_TIER` / `--tier`):

| Value | Scope |
|-------|--------|
| `smoke` | Tier S (~7 cases) — CI default |
| `default` | Tier A (+S), excludes stress |
| `stress` / `all` | A + Tier B stress |

### L2 Criterion notes (maintainer)

```bash
cargo bench -p plotine --bench hotspots
cargo bench -p plotine --bench hotspots -- --quick
# HTML: target/criterion/report/index.html
```

| Bench | What it isolates |
|-------|------------------|
| `recipe.contourf_fills_80` | marching-squares fills only |
| `recipe.streamlines_20` | streamplot integration only |
| `mathtext.measure_mixed` | mathtext measure (no full figure) |
| `e2e_render_png/*` | full PNG for contourf / streamplot / mathtext / subplots |
| `e2e.chrome_empty` | empty axes chrome |

**Finding:** `contourf` e2e (~100 ms) ≫ `contourf_fills` recipe (~9 ms) on a typical
dev machine — most cost is path rasterization / paint, not level geometry. Prefer
optimizing the draw path for filled contours before further recipe tuning.

This document defines a complete product-oriented benchmark for plotine:
which figures to generate, how to time them, how to compare to matplotlib,
and what to put in CI.

---

## 1. Research summary

### 1.1 What peers measure

| Source | What they time | Chart / scale focus |
|--------|----------------|---------------------|
| [matplotlib/mpl-bench](https://github.com/matplotlib/mpl-bench) (ASV) | `canvas.draw` / `savefig`; param sweeps | empty `subplots(n×n)`, projections (polar…), tiny `plot`, savefig |
| Matplotlib perf PRs (pyperf) | init / clear / reuse Axes, plot+legend | **layout chrome** and tick materialization often dominate |
| Plotly / Bokeh studies | FPS, CPU/RAM, render latency | line + scatter across **1k → 1M** points (interactive/WebGL) |
| Academic viz surveys | functionality + speed on common types | line, scatter, bar, heatmap as the “default four” |

Takeaways for plotine:

1. **Layout/chrome cost** matters as much as artist geometry (mpl-bench’s
   `time_subplots` is intentionally empty).
2. **Point-count scaling** (1k / 10k / 100k) is the industry stress pattern for
   series charts — not “one sin wave of 100 points”.
3. **Interactive FPS** is out of scope for default plotine (static-first;
   GUI is opt-in). Measure `render_png` / export, not egui frame rate in v1.
4. Fair cross-library comparison needs **fixed figsize, DPI, data, and theme**.

### 1.2 What plotine already has

| Corpus | Size | Role |
|--------|------|------|
| `size_benchmark` TIMING cases | ~8–10 | line@DPI ladder + M9–M13 smoke; **debug**, single-shot, no warmup |
| gallery | ~72 panels | visual demo corpus — best recipe source |
| compare pairs | 84 | visual parity vs mpl — **not** timed |
| MPL_2D_COVERAGE | full static 2D ✅ | API completeness ≠ bench coverage |

Gap: visual coverage ≫ timing coverage. The suite below closes that gap without
turning into “time every gallery PNG”.

---

## 2. Goals and non-goals

### Goals

| ID | Goal |
|----|------|
| G1 | **Product latency**: end-to-end `Figure` build + raster/vector export time |
| G2 | **Size narrative**: PNG/SVG/PDF/GIF bytes vs matplotlib (extend size_bench) |
| G3 | **Hot-path coverage**: charts that dominate CPU (mesh, field, 3D, large N) |
| G4 | **Scaling curves**: same chart at N ∈ {1e3, 1e4, 1e5} where relevant |
| G5 | **Regression signal**: stable median on `--release` for local / optional CI |
| G6 | **Parity harness**: same scenarios runnable in plotine and matplotlib |

### Non-goals (v1)

- Criterion microbenches of internal helpers (defer until profiles show hotspots)
- GUI toolbar FPS / interactive pan-zoom latency
- Pixel MAE (belongs to `matplotlib_compare` / `pixel_align_features`)
- Full 72-gallery × all formats matrix
- Claiming wall-clock comparability across machines without a published machine profile

---

## 3. Methodology

### 3.1 Build and timing protocol

| Knob | Value |
|------|--------|
| Profile | **`--release` only** for published numbers |
| Figsize | `5.0 × 3.5` in (keep size_bench default) |
| DPI (raster) | `150` default; DPI ladder only for `core.line` |
| Theme | `Theme::light()` stock fonts |
| Warmup | 2 iterations (discarded) |
| Measured | 7 iterations → report **median** and **p95** ms |
| Timer scope | A: build+artists+`render_png` (primary); B: encode/write bytes (secondary) |
| RNG | shared LCG+Box–Muller seed `42` (`bench_suite::seeded_samples` ≡ `benchmark.py::seeded_samples`; **not** `numpy.default_rng`) |
| Output line | `BENCH name=… median_ms=… p95_ms=… bytes=… fmt=png n=…` |

Split timing when cheap:

```text
t_build_render  = Instant around Figure construction + render_png()/render_svg()
t_encode_write  = optional; or include in end-to-end for size_bench compatibility
```

v1 default: **end-to-end** (build + render + encode to memory or tempfile),
matching current `TIMING` semantics so Python can keep one parser.

### 3.2 Two layers (do not merge)

| Layer | Binary / script | Audience |
|-------|-----------------|----------|
| **L1 Product** | `bench_suite` example + `scripts/benchmark.py` | README / releases / vs mpl |
| **L2 Micro** | `benches/hotspots.rs` + Criterion | maintainers profiling hotspots |

L1 is the public narrative. L2 is **not** on PR CI (noise + time).

### 3.3 Formats

| Format | When |
|--------|------|
| PNG | all L1 scenarios (primary) |
| SVG | subset `fmt.svg_*` (vector path / text) |
| PDF | subset `fmt.pdf_*` |
| GIF | only `feat.anim_gif` (`feature = "gif"`) |

---

## 4. Scenario catalog

Naming: `{family}.{case}[_{param}]`  
Example: `series.line_n10000`, `field.streamplot`, `layout.subplots_2x2`.

### Tier S — Smoke / CI-optional (≤ 30 s release, Linux)

Must stay green locally; optional `workflow_dispatch` CI job.

| ID | Figure to generate | Why |
|----|-----------------|-----|
| `series.line_n1000` | line, N=1000, PNG | baseline series path |
| `series.scatter_n1000` | scatter, N=1000 | marker path ≠ polyline |
| `stat.heatmap_64` | heatmap 64×64 | image/colormap path |
| `layout.subplots_2x2` | 4× line panels | layout/chrome (mpl-bench analogue) |
| `chrome.empty` | empty axes + title/labels/grid | pure chrome cost |
| `math.mathtext` | line + `$...$` title/labels | text shaping hotspot |
| `fmt.svg_line_n1000` | same as line → SVG | vector backend |

### Tier A — Core product suite (README table)

All Tier S, plus:

#### A1 Series & stats (common “default four” + friends)

| ID | Figure | Data |
|----|--------|------|
| `series.line_n1e3` / `_n1e4` / `_n1e5` | line | N points |
| `series.scatter_n1e3` / `_n1e4` | scatter | N points (skip 1e5 if too slow) |
| `series.bar_n50` | bar | 50 categories |
| `series.hist_n1e4` | hist | 10k samples, bins=30 |
| `series.area_n1e3` | area / fill under curve | N=1000 |
| `series.errorbar_n200` | errorbar + caps | N=200 |
| `series.multiline_5x1e3` | 5 lines | 5 × 1000 |
| `stat.boxplot` | boxplot | 4 groups × 100 |
| `stat.violin` | violin | same |
| `stat.heatmap_64` / `_128` | heatmap | 64² / 128² |
| `stat.hist2d_1e4` | hist2d | 10k pts |
| `stat.hexbin_1e4` | hexbin | 10k pts |

#### A2 Field / mesh (CPU-heavy)

| ID | Figure | Data |
|----|--------|------|
| `field.contourf_80` | contourf | 80×80 grid |
| `field.pcolormesh_80` | pcolormesh | 80×80 |
| `field.quiver_20` | quiver | 20×20 |
| `field.streamplot_20` | streamplot | 20×20 |
| `field.tripcolor_1e3` | tripcolor | ~1k pts, auto Delaunay |
| `field.spy_40` | spy | 40×40 sparse-ish |

#### A3 Layout / axes semantics

| ID | Figure | Notes |
|----|--------|-------|
| `layout.subplots_2x2` | 2×2 lines | |
| `layout.mosaic` | `subplot_mosaic` 4 panels | |
| `layout.twin_y` | line + twin_y bar | |
| `layout.inset` | host line + inset scatter | |
| `layout.secondary_x` | secondary top axis | |
| `scale.loglog` | log–log line | positive data |
| `scale.datetime` | datetime x + line | ConciseDateFormatter path |
| `polar.line` | polar_line | polar chrome |

#### A4 Annotation / proportion

| ID | Figure |
|----|--------|
| `anno.annotate_styles` | several FancyArrow styles (costly stroke) |
| `anno.table` | small table overlay |
| `prop.pie` | 5 wedges + labels |
| `prop.stackplot` | 3 series stack |

#### A5 3D

| ID | Figure | Data |
|----|--------|------|
| `d3.helix` | plot3d | 200 pts |
| `d3.scatter` | scatter3d | 200 pts |
| `d3.surface_40` | surface | 40×40 |
| `d3.wireframe_40` | wireframe | 40×40 |
| `d3.bar` | bar3d | ~20 bars |

#### A6 Formats & features

| ID | Figure |
|----|--------|
| `fmt.png_line_dpi{100,150,200,300}` | DPI ladder (keep size_bench) |
| `fmt.svg_line_n1000` | SVG |
| `fmt.pdf_line_n1000` | PDF |
| `feat.static_render` | `render_png` only (M9 path) |
| `feat.anim_20f` | 20-frame animate + PNG sequence |
| `feat.anim_gif_20f` | optional `gif` |
| `feat.geo` | Mercator/PlateCarree + coastline + scatter |
| `feat.mathtext` | mathtext stress (Tier S overlap OK) |
| `feat.usetex` | optional `latex` + tools on PATH |
| `feat.pyplot_line` | `plotine-pyplot` façade (separate example OK) |

### Tier B — Stress / nightly (not PR CI)

| ID | Intent |
|----|--------|
| `series.line_n1e6` | optional; document if skipped on CI runners |
| `stat.heatmap_512` | large image |
| `field.streamplot_40` | denser field |
| `d3.surface_80` | heavier 3D |
| `layout.subplots_4x4` | mpl-bench-like empty-ish grid + light artists |
| `composite.dashboard` | one figure: twin + colorbar heatmap + legend outside | “real app” smoke |

### Tier X — Explicitly out of suite

| Item | Reason |
|------|--------|
| Every compare pair (84) | visual, not perf |
| GUI show / widgets | different metric (FPS); document later |
| EPS / MP4 | external process noise (gs / ffmpeg) |
| CJK font load | machine-dependent system fonts |

---

## 5. Suggested figures by real-world scenario

Mapping “who generates what” → which Tier A IDs to care about:

| User scenario | Typical figures | Bench IDs |
|---------------|-----------------|-----------|
| Time series / sensors | line, multiline, datetime, area | `series.line_*`, `series.multiline_*`, `scale.datetime`, `series.area_*` |
| Experiment scatter | scatter, errorbar, log scales | `series.scatter_*`, `series.errorbar_*`, `scale.loglog` |
| Categorical report | bar, barh, pie, stackplot | `series.bar_*`, `prop.*` |
| Distributions | hist, boxplot, violin, hist2d, hexbin | `series.hist_*`, `stat.*` |
| Scientific fields | contourf, quiver, streamplot, pcolormesh | `field.*` |
| Images / matrices | heatmap, spy | `stat.heatmap_*`, `field.spy_*` |
| Papers / LaTeX | mathtext, SVG/PDF, table, annotate | `math.*`, `fmt.svg_*`, `fmt.pdf_*`, `anno.*` |
| Dashboards | subplots, mosaic, twin, inset | `layout.*` |
| Maps | geo + coastline | `feat.geo` |
| Simulation 3D | surface, wireframe, helix | `d3.*` |
| Animation export | FuncAnimation-like GIF/PNG seq | `feat.anim_*` |
| LLM/agent codegen | line+legend+labels (canonical AGENTS pattern) | `series.line_n1000`, `chrome.empty` |

---

## 6. Matplotlib counterpart contract

For every Tier A ID that is “fair vs mpl”:

1. Same figsize, DPI, data arrays (export shared `.npz` or generate in both from seed).
2. Stock style as close as practical (`plt.style.use('default')`; plotine `Theme::light()`).
3. Measure mpl with `fig.canvas.draw()` **or** `savefig` to `BytesIO` — pick one and document; prefer **`savefig` to memory** to match plotine encode cost.
4. Python harness prints the same `BENCH` / `TIMING` schema with `impl=matplotlib|plotine`.

Reuse and evolve `scripts/size_benchmark.py` → `scripts/benchmark.py`:

- Keep size table (bytes).
- Add median/p95 columns.
- Require `cargo run --release`.

---

## 7. Implementation sketch

```
crates/plotine/examples/bench_suite.rs   # L1 scenarios, prints BENCH lines
crates/plotine-pyplot/examples/…         # feat.pyplot_line only
scripts/benchmark.py                     # orchestrate release run + mpl twins + markdown/CSV
compare/bench/                           # gitignored outputs (JSON/CSV/PNG samples)
docs/BENCHMARK.md                        # this file
```

Phased delivery:

| Phase | Deliverable |
|-------|-------------|
| **P0** | Protocol: release, warmup, median; migrate size_bench to `--release` ✅ |
| **P1** | Day-one suite (~16) in `bench_suite` + mpl twins in `scripts/benchmark.py` ✅ |
| **P2** | Expand A1–A6 Tier A + fairer geo/anim byte metrics ✅ |
| **P3** | Tier B + `benchmark.yml` + README Performance ✅ |
| **P4** | Criterion L2 hotspots (`benches/hotspots.rs`) ✅ |

Do **not** delete `size_benchmark` until `benchmark.py` absorbs DPI ladder + M9–M13 cases.

---

## 8. CI policy

| Job | When | Scope |
|-----|------|--------|
| PR `test` | every PR | **no** full bench (noise + time) |
| `benchmark` workflow | `workflow_dispatch` + optional weekly | Tier S on `ubuntu-latest`, release, upload CSV artifact |
| Release checklist | before crates.io | run Tier A locally; paste summary into GitHub Release notes |

Regression gate (optional, loose): fail if Tier S median > **2×** previous artifact on same runner image — warn-only for the first month.

---

## 9. Success metrics

The suite is “complete enough” when:

1. Tier A covers every **family** in §4 (not every API method).
2. Published numbers are always `--release` + median of ≥5 iters.
3. README can show a small table: plotine vs mpl for
   `line_n1e4`, `heatmap_128`, `surface_40`, `subplots_2x2`.
4. A contributor can add a scenario by copying one `case!` block and a matching mpl function.

---

## 10. Relation to visual tools

| Tool | Question answered |
|------|-------------------|
| `benchmark` (this design) | How fast / how large? |
| `matplotlib_compare` | Does it look like mpl? |
| `visual_snapshots` (insta) | Did pixels regress? |
| `pixel_align_features` | MAE for feature packs |

Never mix MAE into BENCH lines.

---

## 11. Open decisions (resolve at P0)

1. **Tempfile vs memory**: prefer `render_png()` / SVG string in memory for timing stability; write samples only when `BENCH_SAVE=1` (plotine + mpl under `compare/bench/`; included in `compare/dashboard.html` via `scripts/write_review_dashboard.py`).
2. **100k line points**: include in Tier A or only Tier B on Windows CI runners?
3. **Shared data fixtures**: Rust `include_bytes!` of `.npz` vs generate identically in Python — recommend generate-from-seed in both for less repo weight.
4. **Whether `chrome.empty` uses tight_layout** — yes, to stress inset measurement.

---

## Appendix A — Count summary

| Tier | Approx. scenario count |
|------|------------------------|
| S | 7 |
| A (incl. scaling variants) | ~45–55 |
| B | ~6 |
| **v1 implement target** | **Tier S + ~25 Tier A representatives** (not every N variant on day one) |

Implemented: Tier S + A (~45) + Tier B stress (6). Select with
`BENCH_TIER=smoke|default|stress` or `python scripts/benchmark.py --tier …`.
