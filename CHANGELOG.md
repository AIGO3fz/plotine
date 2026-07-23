# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.5.2] — 2026-07-23

### Fixed
- README: convert all relative links and image paths to absolute GitHub URLs so
  they render correctly on crates.io (which resolves paths relative to the crate
  directory, not the repository root).

## [0.5.1] — 2026-07-23

### Changed
- Remove "Plotters' spiritual successor" framing from README, llms.txt, book, and
  development plan. plotine is an independent project; matplotlib is credited for
  visual-default alignment only.

## [0.5.0] — 2026-07-22

### Changed
- **Identity**: repository owner updated to `AIGO3fz`; all URLs, authors, and
  license copyright now reference `github.com/AIGO3fz/plotine`.
- **Version**: bumped from 0.4.x to 0.5.0 for first public open-source release.
- Unified version references across README, llms.txt, book, AGENTS.md, and docs.

### Added
- `.github/PULL_REQUEST_TEMPLATE.md` with review checklist.
- `deny.toml` (cargo-deny) for license and vulnerability auditing.
- `THIRD_PARTY_LICENSES.md` (DejaVu fonts, Natural Earth, dependency summary).
- `scripts/requirements-dev.txt` for Python development dependencies.
- `readme` field to 7 sub-crate `Cargo.toml` for crates.io pages.
- `package.metadata.docs.rs` config (all features except gui/cjk).
- Exclude `tests/snapshots/` from crate package.

### Fixed
- `projection.rs`: `partial_cmp().unwrap()` → `unwrap_or` for NaN safety.
- `bar.rs`: fix mojibake (invalid UTF-8 → `∈`).
- Mathtext layout: refine em-box metrics for closer matplotlib alignment.
- Theme font sizes in docs: `13/11/10` → `12/10/10` (matches `mpl_policy.rs`).
- Book/docs: sync stale version refs, update gallery count, feature lists, and
  publish order.
- Remove generated `compare/*.html` from tracked files; add to `.gitignore`.

## [0.4.4] — 2026-07-22

### Added
- `.github/PULL_REQUEST_TEMPLATE.md` with review checklist.
- `deny.toml` (cargo-deny) for license and vulnerability auditing.
- `THIRD_PARTY_LICENSES.md` (DejaVu fonts, Natural Earth, dependency summary).
- `scripts/requirements-dev.txt` for Python development dependencies.
- `readme` field to 7 sub-crate `Cargo.toml` for crates.io pages.
- `package.metadata.docs.rs` config (all features except gui/cjk).
- Exclude `tests/snapshots/` from crate package.

### Fixed
- `projection.rs`: `partial_cmp().unwrap()` → `unwrap_or` for NaN safety.
- `bar.rs`: fix mojibake (invalid UTF-8 → `∈`).
- Mathtext layout: refine em-box metrics for closer matplotlib alignment.
- Theme font sizes in docs: `13/11/10` → `12/10/10` (matches `mpl_policy.rs`).
- Book/docs: sync stale version refs (`0.3` → `0.4`, `0.4.0` → `0.4.3`),
  update gallery count, feature lists, and publish order.
- Remove generated `compare/*.html` from tracked files; add to `.gitignore`.

## [0.4.3] — 2026-07-21

### Added
- Product L1 benchmark suite: `bench_suite` example + `scripts/benchmark.py`
  (warmup / median / p95, matplotlib twins, `compare/bench/results.{csv,md}`);
  tiers `smoke` / `default` / `stress` (Tier B: 1e6 line, 512² heatmap, …);
  GitHub Actions [`benchmark.yml`](.github/workflows/benchmark.yml)
  (`workflow_dispatch` + weekly smoke); README Performance table;
  L2 Criterion hotspots (`cargo bench -p plotine --bench hotspots`);
  see [`docs/BENCHMARK.md`](docs/BENCHMARK.md).
- Open-source hygiene: Dependabot, `rust-toolchain.toml`, DejaVu font LICENSE,
  README gallery showcase images.

### Changed
- `scripts/size_benchmark.py` runs cargo examples with `--release`.
- L1 bench harness: Python `seeded_samples` matches Rust LCG+Box–Muller (not
  `numpy.default_rng`); hist2d/hexbin get colorbars like plotine defaults.
- Violin: median/extrema stem half-width = `0.25 * widths` (mpl `line_ends`);
  default `show_median=false` (stock `showmedians=False`); limits use `points=100`.
- Hexbin: document pointy-top lattice (was mislabeled flat-top); polygon scale
  `1.0` via `mpl_policy::hexbin` (drop 1.02 AA oversize; crisp fill already seals).
- Colorbar axes: drop the artificial x-tick target floor (`LINEAR_TARGETS+2`).
  Stock mpl shrinks axes width → *fewer* ticks (`MaxNLocator`); the floor had
  densified coolwarm/hist2d/hexbin bottoms to step `2.5` / `0.0` labels.
- `contourf`: merge cell polygons into one multi-contour path per band
  (O(levels) draw calls instead of O(cells × levels)); skip hairline seal
  stroke on those fills (crisp fill is enough; large raster speedup).
- Multi-panel / `at_span` y-axis titles: reserve tick→label geometry in
  tight-layout (`Y_LABEL_AIR_PT` 7→10) so rotated labels clear dense ticks.
- Mathtext box metrics use em-box ascent/descent (not cosmic-text line
  height) so scripts/fractions sit closer to matplotlib `_mathtext`.
- `tripcolor` / `tricontourf`: crisp fill without hairline seal stroke
  (same rationale as `contourf`; large meshes paint much faster).
- `hexbin`: crisp fill without hairline seal stroke.
- `scatter` (filled markers): chunked multi-contour batch fill; circles use
  regular octagons instead of dense Bézier tessellation.
- 3D `surface`: drop per-face edge stroke (mpl default omits mesh edges;
  halves paint work on dense grids).
- 3D `wireframe`: single multi-contour stroke (skip pointless depth sort).
- `line`: drop consecutive points that round to the same pixel (helps N≫width).
- `heatmap` (≥256 cells): rasterize to one RGBA blit instead of per-cell fills.

### Fixed
- `plotine-render` rustdoc: drop broken cross-crate intra-doc links to optional
  backend crates.

## [0.4.0] — 2026-07-21

First crates.io publish target (skipping a separate 0.3.0 upload). Primary API
remains the `Figure` builder; pyplot / usetex / multi-toolkit are opt-in or out of
scope — see [`docs/MPL_GAP.md`](docs/MPL_GAP.md).

### Added
- **M9 GUI** (`feature = "gui"`): `Figure::show` with pan / box zoom / scroll zoom /
  home / history / save; 3D elev/azim drag (egui only).
- **M10 Animation**: `Figure::animate` / `Animation::map` → PNG sequence;
  `.save_gif` (`gif`) / `.save_mp4` (`mp4` + ffmpeg).
- **M11 Geo**: `GeoProjection::{PlateCarree,Mercator}` + `ax.coastline()` (NE 110m).
- **M12** crate `plotine-pyplot`: opt-in `plt::plot` / `savefig` façade (TLS state).
- **M13** `feature = "latex"`: `Figure::usetex(true)` via system `latex` + `dvipng`.
- **M14** formats: PGF backend (`.pgf`); EPS (`eps` + Ghostscript); MP4 as above.
- **M15** `show_nonblocking` / `ShowHandle` + `show_with` (egui Slider/Button).
- **M16** `plotine::stats` (`corr_heatmap` / `pair_scatter` / `regline`) + `ax.geojson`.
- **M17** docs: `MPL_GAP.md`, mdBook tutorials, `CONTRIBUTING`, Issue templates.
- Colormap / norm: 82 named maps, `SegmentedColormap`, `Norm::TwoSlope`,
  `sample_reversed` / `map_reversed`.
- Style surface: `Legend::Best` / `Outside*`, `Hatch`, `TickFormatter`,
  `rectangle` / `circle` / `ellipse`, grid linestyle + title/label fontsizes.
- Auto Delaunay for `tripcolor` / `tricontour` when `.triangles` omitted.
- Examples: `interactive_show`, `interactive_widgets`, `animate_wave`,
  `export_formats`, `usetex_demo`; gallery 70+.
- Size / pixel tooling: `scripts/size_benchmark.py`, `scripts/pixel_align_features.py`.
- M9–M17 maturity tests: animation frame-diff + GIF/PNG magic headers, geo/stats
  smoke, view-history contract, optional GUI `show_nonblocking` soft smoke
  (`tests/m9_m17_maturity.rs`); pyplot TLS isolation test; CI covers `gif`.
- Mathtext `\displaystyle` / `\textstyle` / `\limits` / `\nolimits` for large-op
  limit placement.

### Changed
- Default features include `pgf` alongside `png` / `svg` / `pdf`.
- Publish order documents `plotine-backend-pgf` and `plotine-pyplot`.
- Inset axes: fraction box is the axes bbox (mpl `inset_axes`); nested `inner`
  no longer shrinks under a chrome pad into endpoint tick labels.
- `TickLocator` steps down the nice ladder when fewer than `min_n_ticks` (2)
  fall in-domain (MaxNLocator-like; fixes tiny-inset `0.114`/`0.914` labels).
- `aspect_equal` / geo: keep stock subplot box, then fit spines/ticks/grid to the
  artist plot rect (`adjustable='box'`) so PlateCarree fills like mpl/cartopy.
- Mathtext default for `\int_0^1` etc. is textstyle (side scripts), matching
  matplotlib titles; use `\displaystyle` or `\limits` for above/below.
- Polar radial ring hint `7` → `5` (`mpl_policy::polar::RING_N_HINT`) so unit-`r`
  charts get `0.2/0.4/0.6/0.8` like stock mpl polar.
- Y-axis titles may paint into figure margins (full-bleed / `at_span`), fixing
  cramped `y` on tall subplot spans.
- Colorbar panels keep a denser x-tick floor (`LINEAR_TARGETS`) so tripcolor-like
  short spans get mpl-like `0.25` steps.
- 3D chrome: draw only the three tick-bearing axis spines (mpl pane borders are
  nearly invisible; no dark front cage).
- Barh sticky `x=0` survives later `hlines`/`expand_range_x` (flush to y spine).
- Errorbar default width `1.5` + larger markers; legend uses marker+cross handle.
- Polar / spy titles use `top_x_chrome_anchors` (tick band + cell clamp) so they
  are not clipped at the figure top or drawn onto the spines.
- Quiver matches stock mpl geometry (`mpl_policy::quiver`: ~1.4 pt filled shaft,
  headlength/headwidth multiples of width, auto-scale density).
- Spy matches mpl: inverted y, square markers, x ticks on top.
- Log tick labels use mathtext `$10^{n}$` (matplotlib `LogFormatterMathtext`).
- Mathtext italicizes variables / Greek; `\sin`/`\mathrm` stay roman (`DejaVuSans-Oblique`).
- Linear tick labels align decimals per axis (`format_aligned_ticks`: max frac digits).
- `tricontour` / `tricontourf` use ContourSet-style sticky edges (tight limits) so
  `y=0` sits on the spine when combined with tripcolor.
- Datetime ticks use matplotlib `ConciseDateFormatter`-style labels + offset.
- Auto tick decimals capped at 3 (`MAX_TICK_DECIMALS`) to avoid float-noise
  labels on tiny inset axes.
- Polar subplot titles clear `90°` (in-cell top inset + margin placement).
- Polar multi-panel top inset no longer double-counts title chrome (disk size ≈ mpl).
- Scatter default diameter matches matplotlib `s=36` (`mpl_policy::scatter`).
- Area / `fill_between` / spans peek the color cycle (following `line` stays C0 like mpl).
- Patch legend handles use artist alpha (area fill swatch matches the band).
- Axes3D ticks follow mplot3d: `highs` edge pick, inward/outward stubs,
  `_move_from_center` label offsets (`pad=3.5` + `offset=8`), `ha=center`/`va=top`.
- M11 geo compare uses figsize `7×3.5` so PlateCarree 2:1 `aspect_equal` fills
  the axes (5×3.5 letterboxed the map).
- Polygon / patch legend + stroke use artist alpha (mpl `Patch(alpha=…)`).
- Mathtext textstyle ∫/∑ slightly larger with nestled side scripts (mpl titles).
- Annotate styles match mpl FancyArrowPatch: `mutation_scale=10`, `shrinkA/B=2 pt`,
  `_get_arrow_wedge` pad, `BracketB` full bar `2·widthB·ms`, round cap/join,
  `BothEnds`=`<->` open V (stroked); shaft clears text via padded `patchA` box.
- Mathtext ∫/scripts use mpl `FontConstantsBase` (DejaVu) dropsub kerns.

### Notes
- GUI: no Configure Subplots / picking / Qt·Tk·WebAgg (intentional).
- Ecosystem: thin stats/geo helpers only — not full seaborn / geopandas.

## [0.3.0] — 2026-07-19

### Added
- `GridAxis::{Both,X,Y}` + `ax.grid_axis(...)` (matplotlib `grid(..., axis=)`).
- `ax.inset_axes([x0, y0, w, h], |inset| { ... })` (matplotlib `inset_axes`, axes fractions; one nesting level).
- `ax.secondary_x` / `secondary_y` (+ `_linear`) — transformed tick labels (≠ twin).
- `SubplotGrid::at_span(row, col, rowspan, colspan, …)` for multi-cell panels.
- `ax.tripcolor` / `ax.tricontour` with `.triangles([[i,j,k], …])` (no auto-Delaunay).
- Colormaps: `Tab10`, `Coolwarm`, `RdBuR` (matplotlib `RdBu_r`).
- `mpl_policy::ticks::LINEAR_TARGETS` (2D/3D major-tick density ≈ MaxNLocator).
- `mpl_policy::annotate` mutation-scale constants for FancyArrow-style annotate heads.
- Asymmetric error bars: `.yerr_asym(lo, hi)` / `.xerr_asym(lo, hi)` (`ErrBars`).
- Annotate arrow styles: `.arrow_style(ArrowStyle::{Triangle, Simple, Bracket, BothEnds})`.
- Heatmap `.extent([l, r, b, t])` and `.alpha(...)` (matplotlib `imshow` kwargs).
- `ax.table` with `.col_labels` / `.row_labels` / `.loc(TableLoc::…)` (axes-fraction overlay).
- Mathtext layout engine (`plotine::mathtext`): `$...$` in titles/labels/text/annotate
  supports scripts, `\frac`, Greek/symbols, `\sin`/`\cos` (no external LaTeX).
- Linux visual snapshot baselines for the full pragmatic matrix (43 PNG/SVG slots),
  including hexbin, streamplot, twin axes, and static 3D.

### Changed
- `streamplot` / `quiver` / `spy`: cartesian grid off by default (like heatmap/contour).
- Linear tick target count 6 → 9 via `mpl_policy` (denser short spans, e.g. bar).
- `annotate` arrows use fixed point-sized heads (not quiver length-fraction heads).
- Multi-panel layout: outer label/title chrome moves into figure margins; axes boxes
  are equal-sized (mpl `tight_layout`-like).
- Datetime: `autofmt` bottom fraction is final (ticks + xlabel share the band).
- Hist: no default edge stroke (matches mpl transparent `edgecolor`).
- 3D: fit cube into stock Axes3D subplot box; per-axis pane colors; legend/title
  anchored to that box; tick labels use fixed decimals + Unicode minus.
- Linear tick labels: step-aware decimals (`0.0`/`1.0` for step 0.5) and Unicode minus.
- `math::unicode` strips multiple paired `$...$` segments (e.g. `$\alpha$ vs $x^2$`).

### Fixed
- Clippy `-D warnings` clean under workspace `--all-targets` (too-many-arguments allows
  on geometry helpers; `TableLoc` derive Default; const colorbar assertion).

## [0.2.0] — 2026-07-18

### Changed
- **streamplot**: structural port of `matplotlib.streamplot` (mask, RK2/Heun,
  spiral seeds, mid-stream arrows). Trajectory count matches mpl 3.10 on the
  gallery field (`density=1.2` → 98 lines). `.density(1.0)` / `.arrow_size(1.0)`.
- **polar**: matplotlib-like circular spine, `0°`/`45°`/… + radial labels,
  equal-aspect disk (no cartesian box/grid).
- **Figure::new** default size: `7×5` → **`6.4×4.8`** in (matplotlib `figsize`).
  DPI remains 150.
- **layout**: single-panel axes box uses matplotlib stock `figure.subplot.*`
  (≈0.775×0.77); expand only for colorbar / twin / polar.
- **limits**: sticky zero for bar/barh/hist (and other `include_zero` artists) —
  baseline stays flush to the spine; open-side margin matches mpl (~5%).

### Added
- Wind barbs: `ax.barbs(x, y, u, v)` (matplotlib `barbs`; flag/full/half +
  calm circle; `.length` / `.flip` / `.increments`). Gallery `41_barbs.png`.
- Contour level labels: `ax.contour(...).clabel(true)` (matplotlib `clabel`;
  one label per level, inline line gaps, MaxNLocator-style levels, `%.3g`-like
  fmt). Gallery `40_clabel.png`.
- Visual snapshots for scatter / hist / area, `Theme::dark` / `Theme::paper`,
  and SVG snapshots for bar + heatmap.
- Agent docs at repo root: `llms.txt`, `llms-full.txt`, `AGENTS.md`.
- Matplotlib side-by-side comparison: `examples/matplotlib_compare` +
  `scripts/matplotlib_compare.py` → `./compare/`.
- `PlotError::suggestion()` plus constructors `empty_figure` /
  `unsupported_format` / `log_non_positive`; unit test covering every variant.
- mdBook user guide in `book/` (CI job builds it).
- API freeze review (`docs/API_FREEZE.md`) and release runbook
  (`docs/RELEASING.md` + `scripts/publish.ps1`).
- `Color::from_str` / `FromStr` for named colors and `#rgb` / `#rrggbb` /
  `#rrggbbaa` hex (plus `ColorParseError` with suggestion).
- Runnable doctests on all chart methods (`line`…`violin`).

### Changed
- **plotine**: theme font/stroke sizes are points and scale with DPI
  (`px = pt × dpi/72`, matplotlib convention). Default DPI raised **120 → 150**
  for sharper raster output.
- **plotine**: default theme sized a notch above matplotlib stock rcParams
  (title 13 / label 11 / tick 10 pt vs mpl 12 / 10 / 10; figure **6.4×4.8** @
  150 DPI vs mpl 6.4×4.8 @ 100 DPI).
- **errorbar**: default connecting line through markers (matplotlib `fmt='o-'`).
- **boxplot**: orange median (`TAB_ORANGE` / mpl C1), open-circle fliers, face α≈0.7.
- **violin**: Scott bandwidth, density clipped to data range, extrema stems,
  default width 0.5 / 100 KDE points (matplotlib-compatible).
- **datetime**: x tick labels rotate −30° with right alignment (`autofmt_xdate`).
- **layout**: x/y tick-label pad unified; insets measured from real tick strings
  so left/bottom chrome stay aligned (fixes boxplot/violin oversized left margin).
- **boxplot / violin**: tight categorical x-limits `[0.5, n+0.5]` and integer xticks.
- **backend-skia**: PNG encoder now uses `Compression::Best` + adaptive Paeth
  filtering, and writes RGB (not RGBA) when every pixel is opaque — smaller
  files without changing pixel content.
- **backend-svg**: added `SvgRenderer::to_svg_string(&self)` — `save_svg` no longer
  clones the entire node buffer; `into_svg()` now delegates to the shared impl.
- **backend-skia**: replaced `with_clip` (mem::take round-trip) with `clipped_draw`
  using a raw-pointer split-borrow for zero-overhead clip access.
- **plotine**: extracted artist rendering from `Figure::draw_artists` into a dedicated
  `draw` module (`draw::draw_element`), reducing `figure.rs` by ~180 lines.
- **plotine**: introduced `ArtistProps` trait + `impl_artist_props!` macro to eliminate
  repetitive per-variant match dispatch in `PlotElement`.
- Clarified log-scale / length-mismatch error suggestions for agents.

### Fixed
- `recipes/line.rs`: replaced garbled UTF-8 in test comment.

### Docs
- Expanded runnable doctests on `Figure`, `Axes`, `Series`, `IntoSeries`, `Theme`,
  and `PlotError`.
- Added rustdoc to `Figure`, `Axes`, `Series`, `IntoSeries`, `Color`, `Theme`,
  and the `Renderer` trait.
- Enabled `#![warn(missing_docs)]` on the `plotine` facade; documented public
  modules, artist builders, `Legend` / `GridSpec` / `Insets` / `Theme` fields,
  and `SubplotGrid`. Low-level `recipes` allows missing docs by design.

## [0.1.0] — 2026-07-17

### Added
- Project renamed to **plotine** (crate + GitHub repo)
- Workspace crates: core / render / text / backend-skia / **backend-svg** / facade
- Scales: `Linear`, `Log`, `Symlog` via `ScaleType` + actionable `LogScaleNonPositive`
- Themes: `Theme::light`, `dark`, `paper`
- Charts: line, scatter, bar, hist, area, errorbar + legend
- Subplots: `Figure::subplots` / `subplots_with` + `GridSpec` cell layout
- Tight-layout: column/row-aligned insets across subplot panels
- Datetime axes: `Axes::x_datetime` / `y_datetime` + `DatetimeLocator` (calendar-aligned ticks)
- Export: PNG (tiny-skia) and deterministic SVG
- Colormaps: Viridis / Plasma / Inferno / Magma / Cividis
- Charts: `heatmap` (+ colorbar), `boxplot` (Tukey), `violin` (Gaussian KDE)
- Optional features: `polars`, `ndarray`, `evcxr` (`Figure::evcxr_display`)
- Gallery example: 20 figures (`cargo run -p plotine --example gallery`)
- Visual snapshots (PNG binary + SVG text) via insta
- CI: Linux / macOS / Windows + wasm32 + optional features job
