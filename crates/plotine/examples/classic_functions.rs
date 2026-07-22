//! Plot a handful of classic mathematical functions with plotine.

use plotine::prelude::*;

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![start];
    }
    let step = (end - start) / (n - 1) as f64;
    (0..n).map(|i| start + step * i as f64).collect()
}

fn main() -> Result<()> {
    // --- Trig + hyperbolic on [-π, π] ---
    let x = linspace(-std::f64::consts::PI, std::f64::consts::PI, 400);
    let sin_y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    let cos_y: Vec<f64> = x.iter().map(|v| v.cos()).collect();
    let tanh_y: Vec<f64> = x.iter().map(|v| v.tanh()).collect();

    Figure::new()
        .size(8.0, 5.0)
        .axes(|ax| {
            ax.line(&x, &sin_y)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("sin(x)");
            ax.line(&x, &cos_y)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("cos(x)");
            ax.line(&x, &tanh_y)
                .color(Color::FOREST_GREEN)
                .width(2.0)
                .label("tanh(x)");
            ax.title("Classic Functions: sin / cos / tanh")
                .x_label("x")
                .y_label("y")
                .y_range(-1.2, 1.2)
                .grid(true);
        })
        .save("classic_trig.png")?;
    println!("wrote classic_trig.png");

    // --- Gaussian, sigmoid, softplus-ish (log(1+e^x) scaled) ---
    let x2 = linspace(-4.0, 4.0, 400);
    let gaussian: Vec<f64> = x2
        .iter()
        .map(|v| (-0.5 * v * v).exp() / (2.0 * std::f64::consts::PI).sqrt())
        .collect();
    let sigmoid: Vec<f64> = x2.iter().map(|v| 1.0 / (1.0 + (-v).exp())).collect();
    let relu_smooth: Vec<f64> = x2
        .iter()
        .map(|v| {
            // softplus / ln(2) ≈ smooth ReLU, clipped for display
            ((1.0 + v.exp()).ln() / std::f64::consts::LN_2).min(3.0)
        })
        .collect();

    Figure::new()
        .size(8.0, 5.0)
        .axes(|ax| {
            ax.line(&x2, &gaussian)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("N(0,1) pdf");
            ax.line(&x2, &sigmoid)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("sigmoid(x)");
            ax.line(&x2, &relu_smooth)
                .color(Color::DARK_ORANGE)
                .width(2.0)
                .label("softplus(x)/ln2");
            ax.title("Classic Functions: Gaussian / sigmoid / softplus")
                .x_label("x")
                .y_label("y")
                .grid(true);
        })
        .save("classic_special.png")?;
    println!("wrote classic_special.png");

    // --- Polynomials on [-2, 2] ---
    let x3 = linspace(-2.0, 2.0, 300);
    let quad: Vec<f64> = x3.iter().map(|v| v * v).collect();
    let cubic: Vec<f64> = x3.iter().map(|v| v * v * v).collect();
    let quartic: Vec<f64> = x3.iter().map(|v| v.powi(4) - v * v).collect();

    Figure::new()
        .size(8.0, 5.0)
        .axes(|ax| {
            ax.line(&x3, &quad)
                .color(Color::STEEL_BLUE)
                .width(2.0)
                .label("x²");
            ax.line(&x3, &cubic)
                .color(Color::CRIMSON)
                .width(2.0)
                .label("x³");
            ax.line(&x3, &quartic)
                .color(Color::MEDIUM_PURPLE)
                .width(2.0)
                .label("x⁴ − x²");
            ax.title("Classic Polynomials")
                .x_label("x")
                .y_label("y")
                .grid(true);
        })
        .save("classic_poly.png")?;
    println!("wrote classic_poly.png");

    Ok(())
}
