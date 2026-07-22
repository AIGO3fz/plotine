# Interactive windows

Requires `features = ["gui"]`.

```rust
// Blocking (matplotlib show(block=True))
figure.show()?;

// Non-blocking (ion() / show(block=False) subset)
let handle = figure.show_nonblocking()?;
// … other work …
handle.join()?;

// Side-panel widgets (Slider / Button via egui)
figure.show_with(|ui, fig| {
    ui.heading("Controls");
    // return true to re-render after mutating artists
    ui.button("Nudge").clicked()
})?;
```

Examples:

```bash
cargo run -p plotine --example interactive_show --features gui
cargo run -p plotine --example interactive_widgets --features gui
```

Capability matrix: repository `docs/GUI_TOOLBAR.md`. Still **egui only** — no Qt/Tk/WebAgg.
