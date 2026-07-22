# Errors & suggestions

Every fallible path returns `plotine::Result<T>` = `Result<T, PlotError>`.

Each variant includes a **suggestion** string. Agents should surface or auto-apply it:

```rust
use plotine::prelude::*;

match figure.save("out.jpg") {
    Err(e) => {
        eprintln!("{e}");
        eprintln!("fix: {}", e.suggestion());
    }
    Ok(()) => {}
}
```

## Common failures

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `EmptyFigure` | Forgot `.axes` / `.subplots` | Add a panel before `save` |
| `LengthMismatch` | `x`/`y`/`yerr` lengths differ | Align series lengths |
| `LogScaleNonPositive` | Log domain ≤ 0 | Use `Symlog` or filter data |
| `HeatmapSizeMismatch` | `values.len() != nrows * ncols` | Pass row-major flat data |
| `UnsupportedFormat` | Path not `.png`/`.svg`/`.pdf`/`.pgf`/`.eps` | Change extension (`.eps` needs `feature = "eps"`) |

## Empty figures

Empty figures fail at **render time** with `EmptyFigure` (not a type-state machine).
This keeps the builder ergonomic while still failing loudly before writing a blank file.
