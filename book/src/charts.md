# Charts

All 2D chart methods live on [`Axes`](https://docs.rs/plotine/latest/plotine/struct.Axes.html)
inside `Figure::axes` / `subplots` closures. 3D charts use `Figure::axes3d` and
[`Axes3D`](https://docs.rs/plotine/latest/plotine/struct.Axes3D.html).

Visual defaults track stock matplotlib (see `mpl_policy` and `compare/`); the
API is Rust builder style — optional pyplot globals live in the `plotine-pyplot` crate (not the primary API).

## Core 2D

| Method | Purpose | Common builders |
|--------|---------|-----------------|
| `line(x, y)` | Polyline | `.color` `.width` `.label` |
| `scatter(x, y)` | Markers | `.size` `.color` `.label` |
| `bar(x, heights)` / `barh(y, widths)` | Bars | `.width` `.baseline` `.label` |
| `hist(data)` | Histogram | `.bins(n)` |
| `area(x, y)` | Filled under curve | `.alpha` `.baseline` |
| `errorbar(x, y, yerr)` | Points ± error | `.xerr` `.capsize` `.connect` |
| `heatmap(nrows, ncols, values)` | Row-major grid | `.cmap` `.colorbar` `.norm` |
| `boxplot(groups)` | Tukey box-and-whisker | groups at x = 1..n |
| `violin(groups)` | Gaussian KDE violins | same categorical x |

## Curves, spans, composition

| Method | Notes |
|--------|-------|
| `fill_between` / `fill_betweenx` | Band between curves |
| `step` / `stairs` | `.mode(StepMode::…)` |
| `stem` | Stems + markers |
| `hlines` / `vlines` / `axhline` / `axvline` | Segments / full-span refs |
| `axhspan` / `axvspan` / `polygon` | Bands and arbitrary fills |
| `pie` / `stackplot` / `eventplot` / `broken_barh` | Composition & events |

## Fields, density, polar

| Method | Notes |
|--------|-------|
| `hist2d` / `hexbin` | Density; colorbar |
| `contour` / `contourf` | `.levels` / `.clabel(true)` |
| `pcolormesh` / `spy` | Mesh / sparse pattern |
| `quiver` / `barbs` / `streamplot` | Vector fields |
| `polar_line` / `polar_scatter` / `polar_frame` | θ, r (radians) |

## Annotation & axes chrome

| Method | Notes |
|--------|-------|
| `text` / `annotate` | Data coords; `.ha` / `.va` / `.arrow` |
| `twin_y` / `twin_x` | Shared x→right y; shared y→top x |
| `x_categories` / `y_categories` | With `category_indices(n)` |
| `$...$` mathtext / `math::unicode` | Built-in math; optional `usetex` + `latex` feature |

## 3D (static)

```rust
Figure::new().axes3d(|ax| {
    ax.plot3d(&x, &y, &z).color(Color::CRIMSON).width(2.0);
    // ax.scatter3d / surface / wireframe / bar3d
    ax.title("3D").elev(30.0).azim(-60.0);
}).save("out.png")?;
```

A figure is either 2D panels **or** one 3D axes — not mixed in one figure yet.

## Data input

Anything implementing `IntoSeries` works: `&[f64]`, `Vec<f64>`, arrays, and
(with features) Polars columns / ndarray views.

Lengths of `x` and `y` (and `yerr` / `xerr`) must match — mismatches return
`PlotError::LengthMismatch` with a suggestion.

## Legend

Call `.label(...)` on artists, then `ax.legend(Legend::TopRight)` (or another corner).
No label → no legend entry.

Full matplotlib static-2D checklist: `docs/MPL_2D_COVERAGE.md`.
