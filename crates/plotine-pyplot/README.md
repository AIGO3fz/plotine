# plotine-pyplot

Optional **matplotlib.pyplot-style** facade over [`plotine`](https://crates.io/crates/plotine).

> This is an **opt-in bypass**. The supported primary API remains the Rust
> `Figure` builder (`Figure::new().axes(|ax| { … }).save(...)`).
> Prefer the builder for new code and for LLM/agent codegen (`AGENTS.md`).

## Install

```toml
plotine-pyplot = "0.5"
# optional interactive window:
# plotine-pyplot = { version = "0.5", features = ["gui"] }
```

## Quick migrate

```rust
use plotine_pyplot as plt;

fn main() -> plotine::Result<()> {
    let x = [0.0, 1.0, 2.0, 3.0];
    let y = [0.0, 1.0, 0.5, 1.2];
    plt::plot(&x, &y)?;
    plt::xlabel("x")?;
    plt::ylabel("y")?;
    plt::title("pyplot facade")?;
    plt::grid(true)?;
    plt::legend()?;
    plt::savefig("out.png")?;
    Ok(())
}
```

Equivalent builder form:

```rust
use plotine::prelude::*;
Figure::new().axes(|ax| {
    ax.line(&x, &y);
    ax.x_label("x").y_label("y").title("pyplot facade").grid(true).legend(Legend::Best);
}).save("out.png")?;
```

## Example

```bash
cargo run -p plotine-pyplot --example migrate_pyplot
```
