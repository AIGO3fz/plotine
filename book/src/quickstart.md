# Quick start

Add to `Cargo.toml`:

```toml
plotine = "0.5"
```

## Minimal example

```rust
use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    let x = [0.0, 1.0, 2.0, 3.0];
    let y = [0.0, 1.0, 0.5, 1.2];

    Figure::new()
        .size(6.4, 4.8) // inches (matplotlib figsize)
        .dpi(150.0)     // default
        .theme(Theme::light())
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("series A");
            ax.title("Title")
                .x_label("x")
                .y_label("y")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save("out.png")?; // or .svg / .pdf
    Ok(())
}
```

## Defaults

| Setting | Default | Notes |
|---------|---------|-------|
| Figure size | 6.4×4.8 in | Matches matplotlib `figsize` |
| DPI | 150 | Fonts/strokes scale as `px = pt × dpi/72` |
| Theme fonts | title 12 / label 10 / tick 10 pt | Matches stock mpl rcParams |
| Formats | `.png`, `.svg`, `.pdf`, `.pgf` | Extension selects backend; `.eps` with `feature = "eps"` |

## Run examples from the repo

```bash
cargo run -p plotine --example gallery
cargo run -p plotine --example matplotlib_compare
cargo test -p plotine
```
