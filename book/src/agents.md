# Agent guide

Coding agents should prefer the repository-root files (always kept in sync with the
public API):

| File | Role |
|------|------|
| [`AGENTS.md`](https://github.com/AIGO3fz/plotine/blob/main/AGENTS.md) | Idioms, do/don't, matplotlib → plotine map |
| [`llms.txt`](https://github.com/AIGO3fz/plotine/blob/main/llms.txt) | Compact index |
| [`llms-full.txt`](https://github.com/AIGO3fz/plotine/blob/main/llms-full.txt) | Fuller API + examples |

## Canonical pattern

```rust
use plotine::prelude::*;

Figure::new()
    .axes(|ax| {
        ax.line(&x, &y).color(Color::CRIMSON).label("A");
        ax.title("T").x_label("x").y_label("y").legend(Legend::TopRight);
    })
    .save("out.png")?;
```

## Hard rules

1. Always `Figure::new()…`. Prefer `.save` for static output. Do **not** default to `plotine-pyplot` globals or `usetex`. Optional: `.show()` (`gui`), `.animate()` / GIF, geo maps, `usetex` (`latex` feature + system TeX).
2. Set `x_scale` / `y_scale` **before** artists for Log/Symlog auto-limits.
3. Save `.png`, `.svg`, `.pdf`, or `.pgf` (`.eps` needs `feature = "eps"` + Ghostscript).
4. Prefer `Color::*` constants; `Color::from_str` is optional convenience.
5. Read `PlotError::suggestion()` on every error path.
