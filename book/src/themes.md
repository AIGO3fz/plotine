# Themes & styling

## Built-in themes

```rust
Figure::new()
    .theme(Theme::light()) // default
    // .theme(Theme::dark())
    // .theme(Theme::paper())
    .axes(|ax| { /* ... */ });
```

Theme sizes are in **points** and scale with DPI.

## Colors

Prefer named constants:

```rust
Color::CRIMSON
Color::STEEL_BLUE
Color::rgb(70, 130, 180)
Color::from_hex(0xdc143c)
```

Convenience parsing (case-insensitive names or `#rrggbb`):

```rust
use std::str::FromStr;
let c = Color::from_str("crimson")?;
let c = Color::from_str("#4682b4")?;
```

Unlabeled artists cycle through `DEFAULT_CYCLE` (colorblind-friendlier qualitative set).
