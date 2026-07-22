# Subplots & layout

```rust
use plotine::prelude::*;

Figure::new().subplots(2, 2, |g| {
    g.hspace(0.3).wspace(0.25);
    g.at(0, 0, |ax| { ax.line(&x, &y); ax.title("A"); });
    g.at(0, 1, |ax| { ax.scatter(&x, &y); ax.title("B"); });
    g.at(1, 0, |ax| { ax.bar([1.0, 2.0], [3.0, 4.0]); });
    g.at(1, 1, |ax| { ax.hist(&y).bins(8); });
})
.save("grid.png")?;
```

## Tight layout

plotine measures tick/label chrome and aligns insets so that:

- panels in the **same column** share left/right margins
- panels in the **same row** share top/bottom margins

This avoids overlapping y-labels across a grid without a separate `tight_layout()` call.
