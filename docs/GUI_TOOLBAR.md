# M9/M15 GUI vs matplotlib NavigationToolbar2

`Figure::show` (`feature = "gui"`) is an egui window over the same static render
path as `render_png` / `.save`. It is **not** a full matplotlib backend
(Qt5Agg / TkAgg / WebAgg).

## Capability matrix

| Capability | matplotlib toolbar | plotine GUI | Notes |
|---|---|---|---|
| Pan | ✅ | ✅ | Mode `p` / toolbar Pan |
| Box zoom | ✅ | ✅ | Mode `o` / Zoom |
| Scroll zoom | ✅ (backend-dependent) | ✅ | Wheel; log/symlog aware |
| Home / reset | ✅ | ✅ | `h` / `r` |
| Back / Forward | ✅ | ✅ | View history |
| Save figure | ✅ | ✅ | PNG / SVG / PDF / PGF / EPS (features) via `rfd` |
| 3D rotate | ✅ (Axes3D) | ✅ | Drag elev/azim |
| Configure Subplots | ✅ | ❌ | Deferred |
| Axis / plot picking | ✅ | ❌ | Deferred |
| Rubberband / live pan preview | ✅ | partial | Box zoom rubberband only |
| Non-blocking / `ion()` | ✅ | ✅ (M15) | `Figure::show_nonblocking` → [`ShowHandle`] |
| Side-panel widgets | ✅ (Slider/Button/…) | ✅ (M15) | `Figure::show_with` + egui Slider/Button |
| Multiple GUI toolkits | Qt / Tk / … | egui only | Opt-in feature |

## APIs

```rust
// Blocking (matplotlib show(block=True))
figure.show()?;

// Non-blocking (ion() / show(block=False) subset)
let handle = figure.show_nonblocking()?;
// … other work …
handle.join()?; // or handle.close()

// Widgets side panel
figure.show_with(|ui, fig| {
    // return true to re-render
    ui.add(plotine::egui::Slider::new(&mut t, 0.0..=1.0)).changed()
})?;
```

## What size_benchmark `static_render_*` measures

The bench case formerly labeled `gui_frame` is renamed **`static_render`**: it
times `Figure::render_png` only. That is the **pixel path** shared with the GUI
window, **not** proof of toolbar/UX alignment with matplotlib. Use this matrix
for GUI parity claims; use `compare/` visual pairs for static chrome.

## Related

- Example: `cargo run -p plotine --example interactive_show --features gui`
- Widgets: `cargo run -p plotine --example interactive_widgets --features gui`
- Plan: `docs/DEVELOPMENT_PLAN.md` § M9 / M15
