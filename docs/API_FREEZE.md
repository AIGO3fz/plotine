# API freeze review (0.5 → 1.0-rc)

> Status: refreshed 2026-07-22 for **v0.5.0**. Public facade names below are
> freeze-candidates (additive OK before 1.0). Remaining work is crates.io upload
> + docs.rs verification. Pre-1.0 may still add **new** methods (e.g. layout).

## Public facade (`plotine`)

### Types & re-exports (stable intent)

| Item | Notes |
|------|-------|
| `Figure` | Entry builder; `.axes` / `.subplots` / `.axes3d` / `.save` / `.show` (`gui`) / `.usetex` (`latex`) / render helpers |
| `ViewSnapshot` / `PanelView` / `Axes3DView` | Interactive view capture/apply |
| `NavMode` / `ViewHistory` | Navigation mode + Home/Back/Forward stack |
| `Animation` / `AnimFrame` | Offline multi-frame; PNG sequence / GIF |
| `GeoProjection` | `PlateCarree` / `Mercator`; `ax.projection` / `coastline` |
| `Axes` | Chart surface; limits, scales, datetime, legend, grid, twins, insets |
| `Axes3D` | Static 3D panel; elev/azim, plot3d / scatter3d / surface / wireframe / bar3d / contour3d / quiver3d |
| `Theme` | `light` / `dark` / `paper` |
| `Legend` | Four corners |
| `GridSpec` / `SubplotGrid` | Multi-panel layout + `at_span` |
| `Color`, `Colormap`, `Norm`, `ScaleType` | From `plotine-core` |
| `PlotError` / `Result` | All variants carry `suggestion` |
| `IntoSeries` / `Series` | Data adapters |
| Artist builders | Returned by `Axes` / `Axes3D` methods |
| `plotine::math` | Unicode helpers (pure string path; not a LaTeX engine) |
| `plotine::mathtext` | `$...$` layout (scripts / `\frac` / Greek; default path) |
| `plotine::latex` | External LaTeX (`feature = "latex"` + `Figure::usetex`; needs system TeX) |
| `plotine::polars` | Feature-gated |
| `prelude` | Re-exports for apps & agents |

### Chart methods (frozen names — additive OK)

**2D core:** `line`, `scatter`, `bar`, `barh`, `hist`, `area`, `errorbar` (`.xerr` / asym),
`heatmap`, `heatmap_array` (ndarray), `boxplot`, `violin`.

**2D breadth:** `fill_between`, `fill_betweenx`, `step`, `stairs`, `stem`,
`hlines`, `vlines`, `axhline`, `axvline`, `axhspan`, `axvspan`, `polygon`,
`pie`, `stackplot`, `eventplot`, `broken_barh`, `hist2d`, `hexbin`,
`contour`, `contourf`, `pcolormesh`, `spy`, `quiver`, `barbs`, `streamplot`,
`polar_line`, `polar_scatter`, `polar_frame`, `text`, `annotate`,
`twin_y`, `twin_x`, `x_categories` / `y_categories`,
`inset_axes`, `secondary_x` / `secondary_y` (+ `_linear`),
`tripcolor`, `tricontour`, `table`,
`projection` / `coastline` (geo).

**3D:** `plot3d`, `scatter3d`, `surface`, `wireframe`, `bar3d`, `contour3d`, `quiver3d` (via `Figure::axes3d`; interactive rotate via `Figure::show` + `feature = "gui"`).

### Features (frozen names)

`png`, `svg`, `pdf`, `pgf`, `eps`, `polars`, `ndarray`, `evcxr`, `cjk`, `gui`,
`gif`, `mp4`, `latex`.

## Decisions from this review

| Topic | Decision |
|-------|----------|
| Empty figure | Keep **runtime** `EmptyFigure` (not type-state) for builder ergonomics |
| Color strings | `Color::from_str` / `FromStr` as convenience; constants remain preferred |
| `recipes` module | Remains advanced/testing; `#![allow(missing_docs)]` OK |
| Backend crates | Publishable but **not** part of the supported app-facing API |
| MSRV | 1.85 (stable − 4 policy) |
| Matplotlib | Visual/behaviour alignment via `mpl_policy`; 0.x primary API has **no** pyplot in `plotine` (optional crate `plotine-pyplot` = M12) |
| Priority (M8+) | Static matplotlib parity; M9–M13 (GUI → animation → geo → pyplot → LaTeX) landed as opt-in |
| First publish target | **0.5.0** (M9–M17 included; skip separate 0.3.0 upload) |

## 1.0-rc checklist

- [x] Facade `#![warn(missing_docs)]` + rustdoc on public items
- [x] Agent docs (`AGENTS.md`, `llms.txt`, `llms-full.txt`)
- [x] mdBook user guide (`book/`)
- [x] Error suggestion audit + unit tests
- [x] Visual snapshot matrix (43 Linux slots including hexbin / streamplot / 3D)
- [ ] First crates.io publish of `0.5.0`
- [ ] docs.rs green for `plotine`
- [ ] Tag + GitHub Release aligned with crates.io
- [ ] Announce cadence in README (done once publish lands)

## Breaking-change watchlist (pre-1.0 only if needed)

- ~~Renaming `set_xticks` / `set_yticks` toward `x_ticks` / `y_ticks`~~ (done in 0.2)
- Whether `Artist` builders should return owned handles vs `&mut` (keep `&mut` for 1.0)
- Optional: stricter compile-time empty-figure type state (deferred)
- M8c layout APIs (`inset_axes`, `secondary_*`, rowspan) — names frozen for 0.3; avoid renames
