# Scales & datetime

## Scale types

```rust
use plotine::prelude::*;

// Set scale BEFORE adding artists when you want log-aware auto-limits.
ax.x_scale(ScaleType::Log).y_scale(ScaleType::Log);
ax.line(&x_pos, &y_pos);

// Data that crosses zero → Symlog
ax.y_scale(ScaleType::Symlog { linthresh: 1.0 });
```

| Scale | Domain rule |
|-------|-------------|
| `Linear` | Any finite interval with `min < max` |
| `Log` | Strictly positive; else `LogScaleNonPositive` |
| `Symlog { linthresh }` | Linear near zero, log outside |

## Datetime axes

Unix UTC seconds → calendar tick labels:

```rust
ax.x_datetime(true);
ax.line(&timestamps, &values);
```

Tick labels are rotated (−30°) for readability (matplotlib `autofmt_xdate` style).
