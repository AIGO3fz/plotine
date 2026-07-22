//! Offline wave animation → PNG sequence + GIF (+ optional MP4).
//!
//! ```text
//! cargo run -p plotine --example animate_wave --features gif
//! cargo run -p plotine --example animate_wave --features "gif,mp4"
//! ```
//!
//! Writes `target/animate_wave/frame_*.png`, `wave.gif`, and (with `mp4`) `wave.mp4`.

use plotine::prelude::*;
use std::path::PathBuf;

fn main() -> plotine::Result<()> {
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/animate_wave");
    std::fs::create_dir_all(&out).ok();

    let x: Vec<f64> = (0..120).map(|i| i as f64 * 0.08).collect();
    let y0: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    let fig = Figure::new().size(5.0, 3.0).dpi(100.0).axes(|ax| {
        ax.line(&x, &y0)
            .color(Color::CRIMSON)
            .width(2.0)
            .label("sin(x+t)");
        ax.title("FuncAnimation-style wave")
            .x_label("x")
            .y_label("y")
            .y_range(-1.2, 1.2)
            .legend(Legend::TopRight)
            .grid(true);
    });

    let anim = fig
        .animate(0..40, |fig, i| {
            let t = i as f64 * 0.15;
            let y: Vec<f64> = x.iter().map(|v| (v + t).sin()).collect();
            fig.axes_at_mut(0)
                .expect("axes")
                .line_at_mut(0)
                .expect("line")
                .set_y(&y)?;
            Ok(())
        })?
        .interval_ms(50);

    println!("Rendered {} frames at {:?}", anim.len(), anim.frame_size());
    anim.save_png_sequence(&out)?;
    let gif_path = out.join("wave.gif");
    anim.save_gif(&gif_path)?;
    println!("Wrote PNG sequence + {}", gif_path.display());
    #[cfg(feature = "mp4")]
    {
        let mp4_path = out.join("wave.mp4");
        match anim.save_mp4(&mp4_path) {
            Ok(()) => println!("Wrote {}", mp4_path.display()),
            Err(e) => eprintln!("MP4 skipped: {e}"),
        }
    }
    Ok(())
}
