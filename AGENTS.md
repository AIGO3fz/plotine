# AGENTS.md — coding with plotine

This file is for AI coding agents (and humans teaching them). Prefer these patterns over inventing a matplotlib/pyplot-style API.

## Canonical pattern

```rust
use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    let x = [0.0, 1.0, 2.0, 3.0];
    let y = [0.0, 1.0, 0.5, 1.2];

    Figure::new()
        .size(6.4, 4.8)          // inches (matplotlib figsize; also Figure::new default)
        .dpi(150.0)              // default (mpl stock is 100); pt × dpi/72 → px
        .theme(Theme::light())   // title 12 / label 10 / tick 10 (mpl stock)
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .linestyle(LineStyle::Dashed)
                .label("series A");
            ax.scatter(&x, &y)
                .marker(MarkerStyle::Square)
                .size(6.0);
            ax.despine() // hide top/right spines
                .minor_ticks(true);
            ax.title("Title")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save("out.png")?;
    Ok(())
}
```

## Do / Don't

| Do | Don't |
|---|---|
| `Figure::new().axes(\|ax\| { ... }).save(...)` | Global `plt.plot` / module-level state |
| `Color::STEEL_BLUE` or `Color::rgb(r,g,b)` (optional `Color::from_str`) | Rely on string kwargs as the only API |
| `ax.x_scale(ScaleType::Log)` then add artists | Add artists first, then switch to log |
| `ax.x_range(min, max)` with `min < max` | Equal or reversed bounds without checking |
| Match `x`/`y` lengths | Silent truncation |
| `.label(...); ax.legend(Legend::TopRight)` / `Best` | Expect auto-legend without labels |
| Extension `.png` / `.svg` / `.pdf` / `.pgf` (`.eps` needs `feature = "eps"` + Ghostscript) | `.jpg` (not supported); MP4 via `Animation::save_mp4` (`mp4` + ffmpeg), not `Figure::save` |

## 3D plots

```rust
use plotine::prelude::*;

// 3D line (helix)
let t: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
let x: Vec<f64> = t.iter().map(|v| v.cos()).collect();
let y: Vec<f64> = t.iter().map(|v| v.sin()).collect();
let z: Vec<f64> = t.clone();

Figure::new()
    .axes3d(|ax| {
        ax.plot3d(&x, &y, &z).color(Color::CRIMSON).width(2.0).label("helix");
        ax.scatter3d(&x, &y, &z).size(3.0);
        ax.title("3D Demo").elev(30.0).azim(-60.0);
        ax.legend(Legend::TopRight);
    })
    .save("3d.png")?;

// Surface / wireframe (pass sample coords like mpl plot_surface(X,Y,Z))
let nx = 20; let ny = 20;
let x: Vec<f64> = /* length nx */;
let y: Vec<f64> = /* length ny */;
let z: Vec<f64> = /* ny*nx row-major grid */;
Figure::new().axes3d(|ax| {
 ax.surface(nx, ny, &z).x(&x).y(&y).cmap(Colormap::Viridis).alpha(0.9);
 // or: ax.wireframe(nx, ny, &z).x(&x).y(&y).color(Color::STEEL_BLUE);
 ax.title("Surface");
}).save("surface.png")?;

// 3D bar chart
Figure::new().axes3d(|ax| {
    ax.bar3d([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [3.0, 5.0, 2.0])
        .color(Color::STEEL_BLUE);
}).save("bar3d.png")?;

// 3D contour (iso-z) + quiver (static export; interactive rotate via `.show()` + feature = "gui")
Figure::new().axes3d(|ax| {
 ax.contour3d(nx, ny, &z).x(&x).y(&y).levels(8);
 ax.quiver3d(&x, &y, &z, &u, &v, &w).scale(1.0).color(Color::STEEL_BLUE);
}).save("field3d.png")?;
```

## Subplots

```rust
Figure::new().subplots(2, 2, |g| {
 g.hspace(0.3).wspace(0.25);
 g.at(0, 0, |ax| { ax.line(&x, &y); ax.title("A"); });
 g.at(0, 1, |ax| { ax.scatter(&x, &y); ax.title("B"); });
 g.at(1, 0, |ax| { ax.bar([1.0, 2.0], [3.0, 4.0]); });
 g.at(1, 1, |ax| { ax.hist(&y).bins(8); });
})
.save("grid.png")?;

// Span cells (matplotlib GridSpec rowspan/colspan)
Figure::new().subplots(2, 2, |g| {
 g.at_span(0, 0, 2, 1, |ax| { ax.line(&x, &y); ax.title("tall"); });
 g.at(0, 1, |ax| { ax.scatter(&x, &y); });
 g.at(1, 1, |ax| { ax.hist(&y).bins(8); });
}).save("span.png")?;

// Mosaic layout (matplotlib subplot_mosaic)
Figure::new().subplot_mosaic("AAB;CDB", |name, ax| match name {
 'A' => { ax.line(&x, &y); ax.title("A"); },
 'B' => { ax.scatter(&x, &y); ax.title("B"); },
 'C' => { ax.bar([1.0, 2.0], [3.0, 4.0]); },
 'D' => { ax.hist(&y).bins(8); },
 _ => {},
}).save("mosaic.png")?;
```

## Scales & datetime

```rust
// Log: domain must be strictly positive
ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
ax.line(&x_pos, &y_pos);

// Data crossing zero → Symlog
ax.y_scale(ScaleType::Symlog { linthresh: 1.0 });

// Unix UTC seconds → calendar tick labels
ax.x_datetime(true);
```

Log scale on non-positive data returns `PlotError::LogScaleNonPositive` with suggestion:
`use ScaleType::Symlog { linthresh: ... }, or filter/clip values so the domain is > 0`.

## Optional features

```toml
plotine = { version = "0.5", features = ["polars", "ndarray", "evcxr", "cjk", "gui", "gif", "latex"] }
```

```rust
// polars
let (x, y) = plotine::polars::xy(&df, "x", "y")?;
Figure::new().axes(|ax| { ax.line(&x, &y); }).save("p.png")?;

// ndarray heatmap
ax.heatmap_array(&array2);

// evcxr notebook (last expr in cell)
Figure::new().axes(|ax| { ax.line(&x, &y); }).evcxr_display()?;
```

## Common compile / runtime failures

1. **Empty figure** — forgot `.axes(...)` / `.subplots(...)` before `.save`.
2. **Length mismatch** — `x` and `y` (or `yerr`) lengths differ.
3. **Heatmap size** — `values.len() != nrows * ncols` (row-major).
4. **Unsupported format** — path is not `.png`, `.svg`, `.pdf`, `.pgf`, or `.eps` (enable matching features).
5. **Log domain** — non-positive limits; switch to `Symlog` or filter.

Every `PlotError` variant includes a `suggestion` string — surface it to the user or auto-fix from it.

## Naming map (vs matplotlib)

| matplotlib | plotine |
|---|---|
| `plt.subplots` / `fig, ax = ...` | `Figure::new().axes` / `.subplots` |
| `ax.plot` | `ax.line` |
| `linestyle='--'/':'/'-.'` | `.linestyle(LineStyle::Dashed/Dotted/DashDot)` |
| `marker='s'/'^'/'D'/'x'/'+'/'*'` | `.marker(MarkerStyle::Square/Triangle/Diamond/Cross/Plus/Star)` |
| `ax.minorticks_on()` | `ax.minor_ticks(true)` / `x_minor_ticks` / `y_minor_ticks` |
| `ax.spines['top'].set_visible(False)` | `ax.despine()` 或 `ax.spines(Spines::bottom_left())` |
| `ax.fill_between` | `ax.fill_between` |
| `ax.step` / `ax.stairs` | `ax.step` / `ax.stairs` (`.mode(StepMode::…)`) |
| `ax.stem` | `ax.stem` |
| `ax.barh` / `ax.hlines` / `ax.vlines` | same names |
| `ax.pie` / `ax.stackplot` / `ax.eventplot` | same names |
| `ax.broken_barh` / `ax.axhspan` / `ax.polygon` | same names |
| `ax.contour` / `ax.contourf` / `ax.pcolormesh` / `ax.spy` | same; contour `.clabel(true)` |
| `ax.quiver` / `ax.barbs` / `ax.streamplot` | streamplot `.density(1.0)` / `.arrow_size` (mpl-like) |
| polar `plot(θ,r)` | `ax.polar_line` / `polar_scatter` (+ `polar_frame`) |
| `ax.text` / `ax.annotate` | same; `.arrow` / `.arrow_style(ArrowStyle::…)` (data coords; `.ha` / `.va`) |
| `errorbar(..., yerr=(lo,hi))` | `.yerr_asym(lo, hi)` / `.xerr_asym(lo, hi)`（或对称 `.xerr`） |
| `imshow(..., extent=, alpha=)` | `ax.heatmap(...).extent([l,r,b,t]).alpha(...)` |
| `ax.twinx()` | `ax.twin_y(\|ax2\| { ... })` (shared x, right y) |
| `ax.twiny()` | `ax.twin_x(\|ax2\| { ... })` (shared y, top x) |
| mathtext / `$\\alpha$` | 直接写 `$...$`（默认内置 `mathtext`）；可选 `Figure::usetex(true)` + `features = ["latex"]`（需系统 `latex`/`dvipng`）；或 `math::unicode` |
| `ax.table` | `ax.table([[..]]).col_labels(..).loc(TableLoc::UpperRight)` |
| categorical ticks | `ax.x_categories([...])` + `category_indices(n)` (`0..n`, mpl categorical) |
| `LogNorm` | `.norm(Norm::Log)` on heatmap / hist2d / hexbin / … |
| `ax.set_xlabel` | `ax.x_label` |
| `ax.set_xlim` | `ax.x_range` |
| `ax.set_xscale("log")` | `ax.x_scale(ScaleType::Log)` |
| `ax.legend(loc=...)` | `ax.legend(Legend::TopRight)` / `Best` / `OutsideUpperRight` / … |
| `bar(..., hatch='//')` | `.hatch(Hatch::Diagonal)` / `Cross` / `Grid` / `Dots` / … (bar / barh / hist) |
| `ax.grid(True, axis="y")` | `ax.grid(true).grid_axis(GridAxis::Y)` |
| `ax.grid(..., linestyle='--')` | `.grid_linestyle(LineStyle::Dashed)` |
| `FuncFormatter` / `%` ticks | `ax.y_tick_formatter(TickFormatter::percent(0))` / `fixed(2)` / `new(|v| …)` |
| `Rectangle` / `Circle` / `Ellipse` | `ax.rectangle` / `circle` / `ellipse`（数据坐标；rect 支持 `.hatch`） |
| `set_title(..., fontsize=)` | `.title(...).title_fontsize(14.0)`（另有 `x_label_fontsize` / `y_label_fontsize`） |
| `ax.inset_axes([x0,y0,w,h])` | `ax.inset_axes([x0, y0, w, h], \|inset\| { ... })`（axes fraction；可嵌套一层） |
| `ax.secondary_xaxis('top', functions=(f,finv))` | `ax.secondary_x(f, finv, \|sec\| { sec.label(...); })` |
| `ax.secondary_yaxis('right', functions=(f,finv))` | `ax.secondary_y(f, finv, …)` / `secondary_y_linear(scale, offset, …)` |
| `plt.savefig` | `figure.save`（或旁路 `plotine_pyplot::savefig`） |
| `plt.show` / `fig.show` | `figure.show()` / `show_nonblocking` / `show_with`（`feature = "gui"`） |
| `plt.plot` / `plt.subplots` | **主路径** `ax.line` / `Figure::subplots`；旁路见 `plotine-pyplot` |
| `FuncAnimation` / `anim.save` | `figure.animate(...).save_png_sequence` / `.save_gif`（`gif`） / `.save_mp4`（`mp4`） |
| `line.set_ydata` | `ax.line_at_mut(i).set_y(...)` |
| cartopy `PlateCarree` / `Mercator` | `ax.projection(GeoProjection::PlateCarree/Mercator)` |
| `ax.coastlines()` | `ax.coastline()`（嵌入 NE 110m） |
| `cmap='viridis'` / `coolwarm` / `RdBu_r` | `.cmap(Colormap::Viridis)` / `Coolwarm` / `RdBuR`；82 named maps + `.sample_reversed()` |
| `LinearSegmentedColormap` | `SegmentedColormap::from_colors([...])` / `from_stops` → `.cmap(...)` |
| `TwoSlopeNorm(vcenter=…)` | `.norm(Norm::TwoSlope { vcenter })` |
| `cmap_r` (reversed) | `cmap.sample_reversed(t)` / `map_reversed(v, lo, hi)` |
| `ax.tripcolor` / `tricontour` | same；`.triangles(...)` 可选（省略时自动 Delaunay） |
| `plt.subplot_mosaic("AB;CC")` | `Figure::new().subplot_mosaic("AB;CC", \|name, ax\| { ... })` |

| `fig.add_subplot(projection='3d')` | `Figure::new().axes3d(\|ax\| { ... })` |
| `ax.plot(x,y,z)` (3D) | `ax.plot3d(&x, &y, &z)` |
| `ax.scatter(x,y,z)` (3D) | `ax.scatter3d(&x, &y, &z)` |
| `ax.plot_surface` | `ax.surface(nx, ny, &z)` |
| `ax.plot_wireframe` | `ax.wireframe(nx, ny, &z)` |
| `ax.bar3d` | `ax.bar3d(&x, &y, &z)` |
| `ax.contour` (Axes3D) | `ax.contour3d(nx, ny, &z)` |
| `ax.quiver` (Axes3D) | `ax.quiver3d(&x, &y, &z, &u, &v, &w)` |
| `ax.view_init(elev, azim)` | `ax.elev(30.0).azim(-60.0)` |

Coverage checklist vs matplotlib static 2D: [`docs/MPL_2D_COVERAGE.md`](docs/MPL_2D_COVERAGE.md).

## Matplotlib alignment policy

Visual constants that encode stock matplotlib behaviour live in
[`crates/plotine/src/mpl_policy.rs`](crates/plotine/src/mpl_policy.rs)
(`figure` / `font` / `subplot` / `colorbar` / `chrome` / `datetime` / `polar` /
`pie` / `barbs` / `margin`).

| Do | Don't |
|---|---|
| Add / adjust a named constant in `mpl_policy` | Sprinkle compare-tuned literals (`0.255`, `1.026`, …) into recipes |
| Reuse `chrome_expands_stock_insets` for tight-layout | Special-case colorbar / datetime insets per chart |
| Derive pie radius from `RADIUS / (2·VIEW)` | Hard-code `0.40` beside a separate `VIEW` |

## Testing / gallery

```bash
python scripts/benchmark.py --tier smoke             # L1 Tier S vs mpl (release, median/p95)
python scripts/benchmark.py --tier stress            # include Tier B stress cases
cargo bench -p plotine --bench hotspots -- --quick   # L2 Criterion (contourf/stream/mathtext)
python scripts/size_benchmark.py                     # M9–M13 size/time smoke vs matplotlib
python scripts/pixel_align_features.py               # M9–M13 pixel MAE (after matplotlib_compare)
cargo run -p plotine --example gallery              # → ./gallery/
cargo run -p plotine --example matplotlib_compare   # → ./compare/plotine_*.png
# then: python scripts/matplotlib_compare.py  → 84 pairs + compare/index.html
cargo test -p plotine --test visual_snapshots
```

When changing rendering, update insta snapshots only after human review (`cargo insta review`).

## Interactive GUI (`feature = "gui"`)

Blocking window (matplotlib `show(block=True)` subset). Static `.save` remains the default path.
Toolbar capability matrix vs matplotlib: [`docs/GUI_TOOLBAR.md`](docs/GUI_TOOLBAR.md).

```rust
use plotine::prelude::*;

Figure::new()
    .axes(|ax| {
        ax.line(&x, &y);
    })
    .show()?; // requires features = ["gui"]
```

| Do | Don't |
|---|---|
| `Figure::… .show()?` with `features = ["gui"]` | Expect Qt/Tk/WebAgg backends |
| Pan (p) / Zoom box (o) / scroll zoom / Home (h) / Save (s) | Expect Configure Subplots / picking |
| `show_with(\|ui, fig\| { … })` for egui side-panel widgets | Expect `matplotlib.widgets`-style API |
| 3D: drag elev/azim, scroll zoom data box | Expect `ion()` or non-blocking REPL integration |

Example: `cargo run -p plotine --example interactive_show --features gui`.

## Offline animation (M10)

Matplotlib `FuncAnimation` subset — **no GUI required**. Fix axis ranges, update artists per frame, export PNG sequence and/or GIF.

```rust
use plotine::prelude::*;

let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();
let fig = Figure::new().axes(|ax| {
    ax.line(&x, &y0);
    ax.y_range(-1.2, 1.2); // fix limits before animating
});
let anim = fig
    .animate(0..30, |fig, i| {
        let t = i as f64 * 0.15;
        let y: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
        fig.axes_at_mut(0).unwrap().line_at_mut(0).unwrap().set_y(&y)?;
        Ok(())
    })?
    .interval_ms(50);
anim.save_png_sequence("frames")?;
// anim.save_gif("wave.gif")?; // features = ["gif"]
```

| Do | Don't |
|---|---|
| `y_range` / `x_range` before `animate` | Expect auto-rescale after `set_y` |
| `line_at_mut(0).set_y(...)` or `Animation::map` | Depend on `feature = "gui"` for export |
| `features = ["gif"]` / `["mp4"]` for `.save_gif` / `.save_mp4` | Expect MP4 without system `ffmpeg` |

Example: `cargo run -p plotine --example animate_wave --features gif`.

## Geographic maps (M11)

Cartopy-thin: set a projection, plot lon/lat with ordinary `line`/`scatter`, optionally draw coastlines.

```rust
Figure::new().axes(|ax| {
    ax.projection(GeoProjection::Mercator); // or PlateCarree
    ax.coastline().color(Color::rgb(0x55, 0x55, 0x55));
    ax.scatter(&lons, &lats).size(4.0); // degrees
    ax.title("Map");
}).save("map.png")?;
```

| Do | Don't |
|---|---|
| `ax.projection(GeoProjection::…)` before lon/lat artists | Expect full GIS / shapefile / mixed CRS |
| `coastline()` for NE 110m shores | Expect high-res / political borders |
| Fix extent via `x_range`/`y_range` if needed | Combine with `polar_*` on the same axes |

## pyplot facade (opt-in crate `plotine-pyplot`)

**Do not use for default agent codegen.** Primary API remains `Figure` builder.
Only when the user explicitly wants matplotlib.pyplot-style globals:

```rust
use plotine_pyplot as plt;
plt::plot(&x, &y)?;
plt::xlabel("x")?;
plt::savefig("out.png")?;
```

| Do | Don't |
|---|---|
| `plotine` builder for new / LLM code | Put TLS / `plt::` in the main `plotine` crate |
| `plotine-pyplot` only when migrating pyplot scripts | Treat facade as the stable long-term surface |

## External LaTeX (`feature = "latex"`)

Opt-in only. Default codegen must keep using built-in mathtext (`$...$` without `usetex`).

```rust
Figure::new()
    .usetex(true) // requires features = ["latex"] + latex/dvipng on PATH
    .axes(|ax| {
        ax.title(r"$\int_0^1 x^2\,dx$");
    })
    .save("tex.png")?;
```

| Do | Don't |
|---|---|
| Built-in `$...$` mathtext for scripts/CI | Assume TeX is installed |
| `usetex(true)` only when user asks + TeX available | Enable `latex` feature by default in agent templates |

## Post-M8 alignment (M9–M17 done)

See `docs/DEVELOPMENT_PLAN.md` §1.2. Default codegen: static `Figure` builder + `.save` + built-in mathtext (not pyplot, not usetex).

1. ~~Interactive GUI (M9)~~ — `feature = "gui"` / `Figure::show`
2. ~~Animation (M10)~~ — `Figure::animate` / PNG sequence / `feature = "gif"`
3. ~~Geographic projections (M11)~~ — `GeoProjection` / `coastline`
4. ~~pyplot facade (M12)~~ — opt-in crate `plotine-pyplot`
5. ~~External LaTeX (M13)~~ — `feature = "latex"` / `Figure::usetex`
6. ~~Output formats (M14)~~ — PGF backend, EPS (`feature = "eps"` + Ghostscript), MP4 (`feature = "mp4"` + ffmpeg)
7. ~~Interactive deepening (M15)~~ — `show_nonblocking` + `show_with` (egui Slider/Button side panel)
8. ~~Ecosystem thin layer (M16)~~ — `plotine::stats` + `ax.geojson` (not full seaborn/geopandas)
9. ~~Docs & community (M17)~~ — `MPL_GAP.md`, mdBook tutorials, `CONTRIBUTING`, issue templates

Also deferred: Grammar of Graphics DSL, WASM/browser canvas, MCP server.
