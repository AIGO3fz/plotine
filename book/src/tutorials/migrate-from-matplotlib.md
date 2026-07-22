# Migrate from matplotlib

plotine keeps a **Rust builder** API. Prefer this over the optional `plotine-pyplot` facade.

| matplotlib | plotine |
|---|---|
| `fig, ax = plt.subplots()` | `Figure::new().axes(\|ax\| { … })` |
| `ax.plot(x, y)` | `ax.line(&x, &y)` |
| `ax.set_xlabel` / `set_xlim` | `ax.x_label` / `ax.x_range` |
| `ax.set_xscale("log")` | `ax.x_scale(ScaleType::Log)` **before** artists |
| `plt.savefig` | `figure.save("out.png")` |
| `plt.show()` | `figure.show()?` (`feature = "gui"`) |

```rust
use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    let x = [0.0, 1.0, 2.0, 3.0];
    let y = [0.0, 1.0, 0.5, 1.2];
    Figure::new()
        .axes(|ax| {
            ax.line(&x, &y).color(Color::CRIMSON).width(2.0).label("A");
            ax.title("Migrated").x_label("x").y_label("y")
                .legend(Legend::TopRight).grid(true);
        })
        .save("out.png")?;
    Ok(())
}
```

See the repo-root `AGENTS.md` naming map for a fuller table.
