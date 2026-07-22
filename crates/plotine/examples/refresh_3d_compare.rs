//! Regenerate only the 3D compare PNGs (data matched to `matplotlib_compare`).
//!
//! ```bash
//! cargo run -p plotine --example refresh_3d_compare
//! ```

use std::f64::consts::PI;
use std::path::PathBuf;

use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../compare");
    std::fs::create_dir_all(&dir).ok();

    // Match matplotlib_compare / mpl: linspace(0, 4π, 200)
    let t3: Vec<f64> = (0..200).map(|i| i as f64 * 4.0 * PI / 199.0).collect();
    let hx: Vec<f64> = t3.iter().map(|t| t.cos()).collect();
    let hy: Vec<f64> = t3.iter().map(|t| t.sin()).collect();
    let hz = t3.clone();
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.plot3d(&hx, &hy, &hz)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("helix");
            ax.title("3D Helix").legend(Legend::TopRight);
        })
        .save(dir.join("plotine_helix_3d.png"))?;

    let n3 = 200usize;
    let sx3: Vec<f64> = (0..n3)
        .map(|i| ((i as f64) * 0.1).cos() + (i as f64 * 0.037).sin() * 0.3)
        .collect();
    let sy3: Vec<f64> = (0..n3)
        .map(|i| ((i as f64) * 0.1).sin() + (i as f64 * 0.029).cos() * 0.3)
        .collect();
    let sz3: Vec<f64> = (0..n3).map(|i| i as f64 / n3 as f64 * 10.0).collect();
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.scatter3d(&sx3, &sy3, &sz3).color(Color::STEEL_BLUE);
            ax.title("3D Scatter");
        })
        .save(dir.join("plotine_scatter_3d.png"))?;

    let sn = 40usize;
    let surf_x: Vec<f64> = (0..sn).map(|i| -5.0 + i as f64 * 0.25).collect();
    let surf_y = surf_x.clone();
    let mut surf_z = Vec::with_capacity(sn * sn);
    for &yv in &surf_y {
        for &xv in &surf_x {
            let r = (xv * xv + yv * yv).sqrt();
            surf_z.push(r.sin());
        }
    }
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.surface(sn, sn, &surf_z)
                .x(&surf_x)
                .y(&surf_y)
                .cmap(Colormap::Plasma)
                .alpha(0.95);
            ax.title("3D Surface").elev(30.0).azim(-60.0);
        })
        .save(dir.join("plotine_surface_3d.png"))?;

    let gn = 25usize;
    let g_x: Vec<f64> = (0..gn)
        .map(|i| (i as f64 / (gn - 1) as f64) * 4.0 - 2.0)
        .collect();
    let g_y = g_x.clone();
    let mut g_z = Vec::with_capacity(gn * gn);
    for &yv in &g_y {
        for &xv in &g_x {
            g_z.push((-(xv * xv + yv * yv) * 0.5).exp());
        }
    }
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.surface(gn, gn, &g_z)
                .x(&g_x)
                .y(&g_y)
                .cmap(Colormap::Plasma)
                .alpha(0.9);
            ax.title("3D Gaussian").elev(35.0).azim(-50.0);
        })
        .save(dir.join("plotine_gaussian_3d.png"))?;

    let wn = 30usize;
    let wire_x: Vec<f64> = (0..wn)
        .map(|i| -3.0 + i as f64 * 6.0 / (wn - 1) as f64)
        .collect();
    let wire_y = wire_x.clone();
    let mut wire_z = Vec::with_capacity(wn * wn);
    for &yv in &wire_y {
        for &xv in &wire_x {
            wire_z.push(xv.sin() * yv.cos());
        }
    }
    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.wireframe(wn, wn, &wire_z)
                .x(&wire_x)
                .y(&wire_y)
                .color(Color::STEEL_BLUE)
                .width(0.7);
            ax.title("3D Wireframe").elev(25.0).azim(-70.0);
        })
        .save(dir.join("plotine_wireframe_3d.png"))?;

    Figure::new()
        .size(6.0, 5.0)
        .dpi(150.0)
        .axes3d(|ax| {
            ax.bar3d(
                [0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0],
                [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
                [3.0, 5.0, 2.0, 4.0, 1.0, 6.0, 3.0, 2.0],
            )
            .dx(0.6)
            .dy(0.6)
            .color(Color::STEEL_BLUE)
            .alpha(0.85);
            ax.title("3D Bar").elev(30.0).azim(-55.0);
        })
        .save(dir.join("plotine_bar_3d.png"))?;

    println!("refreshed 3D compare PNGs in {}", dir.display());
    Ok(())
}
