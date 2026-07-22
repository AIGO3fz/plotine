# API stability

This page summarizes the **0.5 → 1.0-rc freeze review** (see also
`docs/API_FREEZE.md` in the repository). Refreshed for M17 (2026-07-22).

## Stability tiers

| Tier | Surface | Policy before 1.0 |
|------|---------|-------------------|
| **Stable intent** | `Figure` / `Axes` / `Axes3D` chart methods, `Theme`, `Legend`, `ScaleType`, `Color` constants, `PlotError` | Additive preferred; renames need `#[deprecated]` |
| **Semi-stable** | Artist builders (`.color` / `.width` / …), `GridSpec`, optional features | May grow fields/methods |
| **Internal** | `recipes::*` geometry helpers, backend crates, `mpl_policy` | No stability promise; may move |

## Frozen for 1.0-rc (do not break casually)

- Entry: `Figure::new`, `.axes`, `.subplots`, `.axes3d`, `.save`, `.render_png`, `.render_svg`, `.render` PDF via `.save`
- 2D charts: core set (`line`…`violin`) plus M5–M6 breadth (`fill_between`, `contour`, `streamplot`, `polar_*`, `twin_*`, …)
- 3D charts: `plot3d` / `scatter3d` / `surface` / `wireframe` / `bar3d` / `contour3d` / `quiver3d` (static only)
- Scales: `ScaleType::{Linear, Log, Symlog}`; `Norm::{Linear, Log}`
- Themes: `Theme::{light, dark, paper}`
- Features: `png`, `svg`, `pdf`, `pgf`, `eps`, `polars`, `ndarray`, `evcxr`, `gui`, `gif`, `mp4`, `latex`, `cjk`
- Errors: every variant keeps a non-empty `suggestion`

## Explicitly deferred (not blockers for 0.x)

Post-M8 milestones (all done — see `docs/DEVELOPMENT_PLAN.md` §1.2):

1. ~~Interactive GUI (M9)~~ — `feature = "gui"` / `Figure::show`
2. ~~Animation (M10)~~ — `Figure::animate` / `Animation` / `feature = "gif"`
3. ~~Geographic projections (M11)~~ — `GeoProjection` / `coastline`
4. ~~pyplot facade (M12)~~ — opt-in crate `plotine-pyplot`
5. ~~External LaTeX (M13)~~ — `feature = "latex"` / `Figure::usetex`
6. ~~Output formats (M14)~~ — PGF, EPS (`feature = "eps"`), MP4 (`feature = "mp4"`)
7. ~~Interactive deepening (M15)~~ — `show_nonblocking` + `show_with` (egui widgets)
8. ~~Ecosystem thin layer (M16)~~ — `plotine::stats` + `ax.geojson`
9. ~~Docs & community (M17)~~ — `MPL_GAP.md`, mdBook, `CONTRIBUTING`, issue templates

Also deferred for 0.x:

- Type-state empty figures (compile-time) — runtime `EmptyFigure` is intentional
- WASM / browser canvas backend
- Grammar of Graphics DSL
- MCP server for agents

## Current focus

First crates.io publish (0.5.x), deeper matplotlib visual fidelity (`compare/`),
`constrained_layout`, mathtext/usetex refinement. Not an API-compatibility layer.

## Semver after 1.0

Breaking changes require a major bump. Prefer `#[deprecated]` migration paths for
at least one minor release so LLM training lag does not strand agents.
