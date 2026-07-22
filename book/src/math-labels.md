# Math labels

## Default: built-in mathtext

Write `$...$` in titles/labels. plotine layouts a practical TeX-like subset
(scripts, `\frac`, `\sqrt`, matrices, Greek) with the embedded font — **no**
external LaTeX binary required.

```rust
ax.title(r"Amplitude $e^{-t/\tau}$");
ax.y_label(r"$\partial_x f$");
```

## Unicode helpers

For plain UTF-8 without math layout:

```rust
use plotine::prelude::*;
use plotine::math;

ax.y_label("θ (rad)");
ax.y_label(math::unicode(r"$\theta$ (rad)"));
ax.x_label(format!("x{}", math::sup("2")));
```

`math::unicode` rewrites a **limited** TeX-like subset into Unicode. It does
**not** layout fractions, roots, or nested scripts.

## Optional: external LaTeX (`feature = "latex"`)

When you need full TeX and have TeX Live / MiKTeX installed (`latex` +
`dvipng` on `PATH`):

```rust
Figure::new()
    .usetex(true)
    .axes(|ax| {
        ax.title(r"$\displaystyle\int_0^1 x^2\,dx$");
    })
    .save("tex.png")?;
```

```bash
cargo run -p plotine --example usetex_demo --features latex
```

Without TeX tools (or without the Cargo feature), `usetex(true)` returns
`PlotError::LatexUnavailable` with a `suggestion`. Prefer mathtext for CI and
default agent codegen.

Built-in mathtext defaults to **textstyle** (inline): `\int_0^1` puts limits to
the side of the symbol, matching matplotlib titles. Use `\displaystyle\int_0^1`
or `\int\limits_0^1` for limits above/below.
