# Introduction

**plotine** is a high-level, LLM-friendly Rust native scientific plotting library
(static 2D + basic 3D).

It provides publication-quality defaults, a
small predictable API surface, and errors that tell agents how to fix themselves.
Visual defaults track stock matplotlib; the API stays Rust-native (pyplot facade = M12 opt-in).

## Design principles

1. **No global state** — always `Figure::new()…save(...)`.
2. **Intent over mechanism** — `ax.line(&x, &y)` is enough; style via builder chains.
3. **Strong types** — `Color::CRIMSON` / `ScaleType::Log`, not magic strings (with
   optional `Color::from_str` for convenience).
4. **Actionable errors** — every `PlotError` carries a `suggestion` string.
5. **Static export first** — PNG, deterministic SVG, and PDF. Optional
   interactive window (`gui`), offline animation (`gif`), geo maps,
   `plotine-pyplot`, and external LaTeX (`feature = "latex"` / `usetex`).

## Status

| Milestone | Status |
|-----------|--------|
| M0–M3 (core charts, SVG, scales, polars/ndarray/evcxr) | Complete |
| M4 (agent docs, mdBook, API freeze, crates.io cadence) | Docs ready; first publish = **0.5.0** |
| M5–M7 (paper chrome, mpl 2D breadth, static 3D) | Complete |
| M8 (deeper matplotlib static alignment) | Largely landed in 0.3.x / 0.5.x |
| M9 (interactive GUI: pan/zoom, 3D rotate, export) | Complete (`feature = "gui"`) |
| M10 (offline animation: PNG sequence / GIF) | Complete (`Figure::animate` / `gif`) |
| M11 (geo: PlateCarree / Mercator + coastline) | Complete |
| M12 (pyplot facade) | Complete (`plotine-pyplot` crate) |
| M13 (external LaTeX) | Complete (`feature = "latex"` / `Figure::usetex`) |
| M14 (EPS / PGF / MP4) | Complete |
| M15 (non-blocking show + widgets) | Complete (`gui`) |
| M16 (stats + GeoJSON) | Complete |
| M17 (gap docs + tutorials + CONTRIBUTING) | Complete |

MSRV: **1.85**. License: **MIT**. Version: **0.5.0**. Gap scores: repo `docs/MPL_GAP.md`.

## Where to go next

- Humans: [Quick start](quickstart.md)
- Coding agents: [Agent guide](agents.md) and the repo-root `AGENTS.md` / `llms.txt`
- API reference: [docs.rs/plotine](https://docs.rs/plotine) (after publish)
