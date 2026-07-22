//! Interactive widgets demo (`feature = "gui"`): Slider + Button side panel.
//!
//! ```text
//! cargo run -p plotine --example interactive_widgets --features gui
//! ```

use plotine::prelude::*;
use std::f64::consts::PI;

fn main() -> plotine::Result<()> {
    let x: Vec<f64> = (0..200).map(|i| i as f64 * 0.05).collect();
    let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    let mut phase = 0.0_f64;
    let mut amp = 1.0_f64;
    let x_plot = x.clone();

    println!("Opening widget window (Slider phase/amp, Reset button)…");
    Figure::new()
        .size(6.0, 4.0)
        .dpi(120.0)
        .axes(|ax| {
            ax.line(&x, &y0).color(Color::CRIMSON).width(2.0);
            ax.title("show_with widgets").y_range(-1.5, 1.5).grid(true);
        })
        .show_with(move |ui, fig| {
            let mut dirty = false;
            ui.heading("Controls");
            ui.separator();
            if ui
                .add(plotine::egui::Slider::new(&mut phase, 0.0..=2.0 * PI).text("phase"))
                .changed()
            {
                dirty = true;
            }
            if ui
                .add(plotine::egui::Slider::new(&mut amp, 0.2..=1.5).text("amp"))
                .changed()
            {
                dirty = true;
            }
            if ui.button("Reset").clicked() {
                phase = 0.0;
                amp = 1.0;
                dirty = true;
            }
            if dirty {
                let y: Vec<f64> = x_plot.iter().map(|v| amp * (v + phase).sin()).collect();
                if let Some(ax) = fig.axes_at_mut(0) {
                    if let Some(line) = ax.line_at_mut(0) {
                        let _ = line.set_y(&y);
                    }
                }
            }
            dirty
        })?;
    Ok(())
}
