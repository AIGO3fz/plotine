//! External LaTeX demo (`feature = "latex"` + system TeX).
//!
//! ```bash
//! cargo run -p plotine --example usetex_demo --features latex
//! ```
//!
//! Requires `latex` and `dvipng` on PATH (TeX Live / MiKTeX). Without them the
//! example exits with [`PlotError::LatexUnavailable`].

use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    if !plotine::latex::tools_available() {
        eprintln!(
            "usetex_demo: `latex` and/or `dvipng` not on PATH.\n\
             Install TeX Live or MiKTeX, or omit Figure::usetex(true) to use built-in mathtext."
        );
        return Err(PlotError::latex_unavailable(
            "`latex`/`dvipng` not found on PATH",
        ));
    }

    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [0.0, 0.8, 0.2, 1.1, 0.6];
    Figure::new()
        .size(6.0, 4.0)
        .dpi(150.0)
        .usetex(true)
        .axes(|ax| {
            ax.line(&x, &y)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label(r"$f(x)=\sin x$");
            ax.title(r"External LaTeX: $\displaystyle\int_0^1 x^2\,dx$")
                .x_label(r"$x$")
                .y_label(r"$y$")
                .legend(Legend::TopRight)
                .grid(true);
        })
        .save("usetex_demo.png")?;
    println!("wrote usetex_demo.png");
    Ok(())
}
