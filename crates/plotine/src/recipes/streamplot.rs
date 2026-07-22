//! Streamlines of a 2D vector field.
//!
//! Structural port of matplotlib's `matplotlib.streamplot` (tested against
//! 3.10.x): occupancy mask, RK2/Heun adaptive integration, spiral seeds,
//! and mid-stream arrow heads.

use kurbo::BezPath;
use plotine_core::{DataToPixel, Point};

/// One streamline polyline (+ optional arrow heads) in pixel space.
#[derive(Debug, Clone)]
pub struct Streamline {
    pub path: BezPath,
    pub arrows: Vec<BezPath>,
}

#[derive(Clone, Copy)]
struct GridSpec {
    nx: usize,
    ny: usize,
}

impl GridSpec {
    fn within(self, xi: f64, yi: f64) -> bool {
        xi >= 0.0 && yi >= 0.0 && xi <= (self.nx - 1) as f64 && yi <= (self.ny - 1) as f64
    }
}

/// Fast bilinear interpolation on a row-major grid (`a[y, x]` layout).
fn interpgrid(a: &[f64], grid: GridSpec, xi: f64, yi: f64) -> Option<f64> {
    if !grid.within(xi, yi) {
        return None;
    }
    let x = xi as usize; // trunc toward 0; xi >= 0
    let y = yi as usize;
    let xn = if x == grid.nx - 1 { x } else { x + 1 };
    let yn = if y == grid.ny - 1 { y } else { y + 1 };
    let a00 = *a.get(y * grid.nx + x)?;
    let a01 = *a.get(y * grid.nx + xn)?;
    let a10 = *a.get(yn * grid.nx + x)?;
    let a11 = *a.get(yn * grid.nx + xn)?;
    if ![a00, a01, a10, a11].iter().all(|v| v.is_finite()) {
        return None;
    }
    let xt = xi - x as f64;
    let yt = yi - y as f64;
    let a0 = a00 * (1.0 - xt) + a01 * xt;
    let a1 = a10 * (1.0 - xt) + a11 * xt;
    Some(a0 * (1.0 - yt) + a1 * yt)
}

/// Spiral seed order — literal port of matplotlib `_gen_starting_points`.
///
/// Yields `(xm, ym)` mask coordinates; boundary first.
fn gen_starting_points(nx: usize, ny: usize) -> Vec<(usize, usize)> {
    if nx == 0 || ny == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(nx * ny);
    let mut xfirst = 0isize;
    let mut yfirst = 1isize;
    let mut xlast = nx as isize - 1;
    let mut ylast = ny as isize - 1;
    let mut x = 0isize;
    let mut y = 0isize;
    // 0 right, 1 up, 2 left, 3 down
    let mut direction = 0u8;
    for _ in 0..nx * ny {
        out.push((x as usize, y as usize));
        match direction {
            0 => {
                x += 1;
                if x >= xlast {
                    xlast -= 1;
                    direction = 1;
                }
            }
            1 => {
                y += 1;
                if y >= ylast {
                    ylast -= 1;
                    direction = 2;
                }
            }
            2 => {
                x -= 1;
                if x <= xfirst {
                    xfirst += 1;
                    direction = 3;
                }
            }
            _ => {
                y -= 1;
                if y <= yfirst {
                    yfirst += 1;
                    direction = 0;
                }
            }
        }
    }
    out
}

struct StreamMask {
    nx: usize,
    ny: usize,
    mask: Vec<u8>,
    traj: Vec<(usize, usize)>,
    current_xy: Option<(usize, usize)>,
}

impl StreamMask {
    fn new(density: f64) -> Self {
        let n = ((30.0 * density).round() as usize).max(2);
        Self {
            nx: n,
            ny: n,
            mask: vec![0u8; n * n],
            traj: Vec::new(),
            current_xy: None,
        }
    }

    fn get(&self, xm: usize, ym: usize) -> u8 {
        self.mask[ym * self.nx + xm]
    }

    fn start_trajectory(&mut self, xm: usize, ym: usize) -> bool {
        self.traj.clear();
        self.current_xy = None;
        self.update_trajectory(xm, ym, true)
    }

    fn undo_trajectory(&mut self) {
        for &(ym, xm) in &self.traj {
            self.mask[ym * self.nx + xm] = 0;
        }
        self.traj.clear();
        self.current_xy = None;
    }

    /// Returns `false` when the cell is already occupied (`InvalidIndexError`).
    fn update_trajectory(&mut self, xm: usize, ym: usize, broken: bool) -> bool {
        if xm >= self.nx || ym >= self.ny {
            return false;
        }
        if self.current_xy == Some((xm, ym)) {
            return true;
        }
        if self.get(xm, ym) == 0 {
            self.traj.push((ym, xm));
            self.mask[ym * self.nx + xm] = 1;
            self.current_xy = Some((xm, ym));
            true
        } else if broken {
            false
        } else {
            self.current_xy = Some((xm, ym));
            true
        }
    }
}

struct DomainMap {
    x_grid2mask: f64,
    y_grid2mask: f64,
    x_mask2grid: f64,
    y_mask2grid: f64,
}

impl DomainMap {
    fn new(grid: GridSpec, mask_nx: usize, mask_ny: usize) -> Self {
        let x_grid2mask = (mask_nx - 1) as f64 / (grid.nx - 1) as f64;
        let y_grid2mask = (mask_ny - 1) as f64 / (grid.ny - 1) as f64;
        Self {
            x_grid2mask,
            y_grid2mask,
            x_mask2grid: 1.0 / x_grid2mask,
            y_mask2grid: 1.0 / y_grid2mask,
        }
    }

    fn grid2mask(&self, xi: f64, yi: f64) -> (usize, usize) {
        let xm = (xi * self.x_grid2mask).round() as isize;
        let ym = (yi * self.y_grid2mask).round() as isize;
        (xm.max(0) as usize, ym.max(0) as usize)
    }

    fn mask2grid(&self, xm: usize, ym: usize) -> (f64, f64) {
        (xm as f64 * self.x_mask2grid, ym as f64 * self.y_mask2grid)
    }
}

/// Filled `-|>` head matching matplotlib `FancyArrowPatch` geometry.
///
/// `mutation_scale` is in pixels (`10 * arrowsize` points × dpi/72).
fn arrow_head(tip: Point, dir_x: f64, dir_y: f64, mutation_scale: f64) -> BezPath {
    let len = (dir_x * dir_x + dir_y * dir_y).sqrt().max(1e-12);
    let ux = dir_x / len;
    let uy = dir_y / len;
    let nx = -uy;
    let ny = ux;
    // ArrowStyle('-|>'): head_length=0.4, head_width=0.2 (× mutation_scale).
    let hl = 0.4 * mutation_scale;
    let hw = 0.2 * mutation_scale;
    let base = Point::new(tip.x - ux * hl, tip.y - uy * hl);
    let left = Point::new(base.x + nx * hw, base.y + ny * hw);
    let right = Point::new(base.x - nx * hw, base.y - ny * hw);
    let mut head = BezPath::new();
    head.move_to(tip.to_kurbo());
    head.line_to(left.to_kurbo());
    head.line_to(right.to_kurbo());
    head.close_path();
    head
}

fn time_deriv(
    ug: &[f64],
    vg: &[f64],
    speed: &[f64],
    grid: GridSpec,
    xi: f64,
    yi: f64,
    forward: bool,
) -> Option<(f64, f64)> {
    if !grid.within(xi, yi) {
        return None;
    }
    let ds_dt = interpgrid(speed, grid, xi, yi)?;
    if ds_dt == 0.0 {
        return None;
    }
    let dt_ds = 1.0 / ds_dt;
    let ui = interpgrid(ug, grid, xi, yi)?;
    let vi = interpgrid(vg, grid, xi, yi)?;
    let (mut dxi, mut dyi) = (ui * dt_ds, vi * dt_ds);
    if !forward {
        dxi = -dxi;
        dyi = -dyi;
    }
    Some((dxi, dyi))
}

fn euler_to_boundary(
    ug: &[f64],
    vg: &[f64],
    speed: &[f64],
    grid: GridSpec,
    xi: f64,
    yi: f64,
    forward: bool,
) -> Option<(f64, (f64, f64))> {
    let (cx, cy) = time_deriv(ug, vg, speed, grid, xi, yi, forward)?;
    let dsx = if cx == 0.0 {
        f64::INFINITY
    } else if cx < 0.0 {
        xi / -cx
    } else {
        ((grid.nx - 1) as f64 - xi) / cx
    };
    let dsy = if cy == 0.0 {
        f64::INFINITY
    } else if cy < 0.0 {
        yi / -cy
    } else {
        ((grid.ny - 1) as f64 - yi) / cy
    };
    let ds = dsx.min(dsy);
    if !ds.is_finite() || ds < 0.0 {
        return None;
    }
    Some((ds, (xi + cx * ds, yi + cy * ds)))
}

/// Literal port of matplotlib `_integrate_rk12`.
#[allow(clippy::too_many_arguments)]
fn integrate_rk12(
    x0: f64,
    y0: f64,
    forward: bool,
    ug: &[f64],
    vg: &[f64],
    speed: &[f64],
    grid: GridSpec,
    dmap: &DomainMap,
    mask: &mut StreamMask,
    maxlength: f64,
    maxds: f64,
) -> (f64, Vec<(f64, f64)>) {
    const MAXERROR: f64 = 0.003;
    let mut ds = maxds;
    let mut stotal = 0.0;
    let mut xi = x0;
    let mut yi = y0;
    let mut xyf_traj: Vec<(f64, f64)> = Vec::new();

    loop {
        if grid.within(xi, yi) {
            xyf_traj.push((xi, yi));
        } else if let Some((last_x, last_y)) = xyf_traj.last().copied() {
            if let Some((dsb, p)) = euler_to_boundary(ug, vg, speed, grid, last_x, last_y, forward)
            {
                xyf_traj.push(p);
                stotal += dsb;
            }
            break;
        } else {
            break;
        }

        let Some((k1x, k1y)) = time_deriv(ug, vg, speed, grid, xi, yi, forward) else {
            break;
        };
        let Some((k2x, k2y)) =
            time_deriv(ug, vg, speed, grid, xi + ds * k1x, yi + ds * k1y, forward)
        else {
            if let Some((dsb, p)) = euler_to_boundary(ug, vg, speed, grid, xi, yi, forward) {
                xyf_traj.push(p);
                stotal += dsb;
            }
            break;
        };

        let dx1 = ds * k1x;
        let dy1 = ds * k1y;
        let dx2 = ds * 0.5 * (k1x + k2x);
        let dy2 = ds * 0.5 * (k1y + k2y);
        let error = ((dx2 - dx1) / (grid.nx - 1) as f64).hypot((dy2 - dy1) / (grid.ny - 1) as f64);

        if error < MAXERROR {
            xi += dx2;
            yi += dy2;
            if !grid.within(xi, yi) {
                break;
            }
            let (xm, ym) = dmap.grid2mask(xi, yi);
            if !mask.update_trajectory(xm, ym, true) {
                break;
            }
            if stotal + ds > maxlength {
                break;
            }
            stotal += ds;
        }

        if error == 0.0 {
            ds = maxds;
        } else {
            ds = maxds.min(0.85 * ds * (MAXERROR / error).sqrt());
        }
    }
    (stotal, xyf_traj)
}

/// Integrate streamlines on a regular `nrows × ncols` vector field.
///
/// `density` matches matplotlib (`1.0` → 30×30 mask). `arrow_size` is
/// matplotlib `arrowsize` (`1.0` default; `0.0` disables). `px` is points→pixels
/// (`dpi / 72`), used for `mutation_scale = 10 * arrowsize` like mpl.
#[allow(clippy::too_many_arguments)]
pub fn streamlines(
    u: &[f64],
    v: &[f64],
    nrows: usize,
    ncols: usize,
    density: f64,
    arrow_size: f64,
    transform: &DataToPixel,
    px: f64,
) -> Vec<Streamline> {
    streamlines_data(
        u, v, nrows, ncols, density, arrow_size, transform, px, 1.0, 1.0,
    )
}

/// Like [`streamlines`], with explicit data-grid spacing (`dx`, `dy`).
#[allow(clippy::too_many_arguments)]
pub fn streamlines_data(
    u: &[f64],
    v: &[f64],
    nrows: usize,
    ncols: usize,
    density: f64,
    arrow_size: f64,
    transform: &DataToPixel,
    px: f64,
    dx: f64,
    dy: f64,
) -> Vec<Streamline> {
    if nrows < 2 || ncols < 2 || u.len() < nrows * ncols || v.len() < nrows * ncols {
        return Vec::new();
    }
    let density = density.clamp(0.1, 8.0);
    let grid = GridSpec {
        nx: ncols,
        ny: nrows,
    };
    let mut mask = StreamMask::new(density);
    let dmap = DomainMap::new(grid, mask.nx, mask.ny);

    // matplotlib: u,v = data2grid(u,v)  →  multiply by 1/dx, 1/dy
    let dx = dx.max(1e-12);
    let dy = dy.max(1e-12);
    let ug: Vec<f64> = u.iter().map(|&a| a / dx).collect();
    let vg: Vec<f64> = v.iter().map(|&a| a / dy).collect();

    // Axes-normalized components for path-length parameterization.
    let mut speed = vec![0.0; nrows * ncols];
    for i in 0..nrows * ncols {
        let ua = ug[i] / (grid.nx - 1) as f64;
        let va = vg[i] / (grid.ny - 1) as f64;
        speed[i] = (ua * ua + va * va).sqrt();
    }

    // matplotlib streamplot: maxlength /= 2 when integrating both directions,
    // then each RK12 call receives that halved value (default 4 → 2).
    let maxlength = 2.0_f64;
    let minlength = 0.1_f64;
    let maxds = (1.0 / mask.nx as f64).min(1.0 / mask.ny as f64).min(0.1);

    // matplotlib: FancyArrowPatch(..., mutation_scale=10 * arrowsize) in points.
    let arrow_px = (10.0 * arrow_size.max(0.0) * px.max(0.0)).max(0.0);

    let mut out = Vec::new();
    let mask_nx = mask.nx;
    let mask_ny = mask.ny;
    let seeds = gen_starting_points(mask_nx, mask_ny);
    for (xm, ym) in seeds {
        if mask.get(xm, ym) != 0 {
            continue;
        }
        let (xg, yg) = dmap.mask2grid(xm, ym);
        if !mask.start_trajectory(xm, ym) {
            continue;
        }

        let (sb, mut back) = integrate_rk12(
            xg, yg, false, &ug, &vg, &speed, grid, &dmap, &mut mask, maxlength, maxds,
        );
        mask.current_xy = Some(dmap.grid2mask(xg, yg)); // reset_start_point
        let (sf, mut fwd) = integrate_rk12(
            xg, yg, true, &ug, &vg, &speed, grid, &dmap, &mut mask, maxlength, maxds,
        );
        let stotal = sb + sf;

        if stotal <= minlength {
            mask.undo_trajectory();
            continue;
        }

        back.reverse();
        if !fwd.is_empty() {
            fwd.remove(0);
        }
        back.extend(fwd);
        let pts = back;
        if pts.len() < 2 {
            mask.undo_trajectory();
            continue;
        }

        // grid → data (origin 0, spacing dx/dy)
        let data_pts: Vec<(f64, f64)> = pts.iter().map(|&(xg, yg)| (xg * dx, yg * dy)).collect();

        let mapped: Vec<Point> = data_pts
            .iter()
            .map(|&(x, y)| transform.map(Point::new(x, y)))
            .collect();

        let mut path = BezPath::new();
        path.move_to(mapped[0].to_kurbo());
        for p in mapped.iter().skip(1) {
            path.line_to(p.to_kurbo());
        }

        let mut arrows = Vec::new();
        if arrow_size > 0.0 && data_pts.len() >= 3 {
            // matplotlib: place by cumulative data-space distance
            let mut s = Vec::with_capacity(data_pts.len() - 1);
            let mut acc = 0.0;
            for i in 0..data_pts.len() - 1 {
                let (x0, y0) = data_pts[i];
                let (x1, y1) = data_pts[i + 1];
                acc += (x1 - x0).hypot(y1 - y0);
                s.push(acc);
            }
            if let Some(&total) = s.last() {
                if total > 1e-12 {
                    let target = total * 0.5; // num_arrows=1
                    let idx = s.iter().position(|&v| v >= target).unwrap_or(s.len() - 1);
                    let tip = mapped[idx];
                    let head = if idx + 1 < mapped.len() {
                        Point::new(
                            0.5 * (mapped[idx].x + mapped[idx + 1].x),
                            0.5 * (mapped[idx].y + mapped[idx + 1].y),
                        )
                    } else {
                        tip
                    };
                    let dxp = head.x - tip.x;
                    let dyp = head.y - tip.y;
                    // FancyArrowPatch goes from tail→head; tip of triangle at head.
                    if dxp * dxp + dyp * dyp > 1e-8 {
                        arrows.push(arrow_head(head, dxp, dyp, arrow_px));
                    } else if idx > 0 {
                        let prev = mapped[idx - 1];
                        arrows.push(arrow_head(tip, tip.x - prev.x, tip.y - prev.y, arrow_px));
                    }
                }
            }
        }

        out.push(Streamline { path, arrows });
    }
    out
}

/// Number of accepted trajectories (for tests / diagnostics).
pub fn streamlines_count(u: &[f64], v: &[f64], nrows: usize, ncols: usize, density: f64) -> usize {
    use plotine_core::{LinearScale, Rect, ScaleKind};
    let t = DataToPixel::new(
        ScaleKind::Linear(LinearScale::new(0.0, (ncols - 1) as f64).unwrap()),
        ScaleKind::Linear(LinearScale::new(0.0, (nrows - 1) as f64).unwrap()),
        Rect::new(0.0, 0.0, 200.0, 200.0),
    );
    streamlines(u, v, nrows, ncols, density, 0.0, &t, 150.0 / 72.0).len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotine_core::{LinearScale, Rect, ScaleKind};

    fn rot_field(n: usize) -> (Vec<f64>, Vec<f64>) {
        let mut u = vec![0.0; n * n];
        let mut v = vec![0.0; n * n];
        let c0 = (n as f64 - 1.0) / 2.0;
        for r in 0..n {
            for c in 0..n {
                let x = c as f64 - c0;
                let y = r as f64 - c0;
                u[r * n + c] = -y;
                v[r * n + c] = x;
            }
        }
        (u, v)
    }

    #[test]
    fn spiral_matches_matplotlib_corners() {
        let pts = gen_starting_points(36, 36);
        assert_eq!(pts.len(), 36 * 36);
        assert_eq!(pts[0], (0, 0));
        // matplotlib: (34,0), (35,0), (35,1), ...
        assert_eq!(pts[34], (34, 0));
        assert_eq!(pts[35], (35, 0));
        assert_eq!(pts[36], (35, 1));
    }

    #[test]
    fn trajectory_count_near_matplotlib() {
        // matplotlib 3.10.7 reference for this field @ density=1.2 → 98 trajectories
        let n = 12;
        let (u, v) = rot_field(n);
        let count = streamlines_count(&u, &v, n, n, 1.2);
        assert_eq!(count, 98, "must match matplotlib trajectory count");
    }

    #[test]
    fn rotational_field_has_arrows() {
        let n = 12;
        let (u, v) = rot_field(n);
        let t = DataToPixel::new(
            ScaleKind::Linear(LinearScale::new(0.0, 11.0).unwrap()),
            ScaleKind::Linear(LinearScale::new(0.0, 11.0).unwrap()),
            Rect::new(0.0, 0.0, 400.0, 400.0),
        );
        let lines = streamlines(&u, &v, n, n, 1.2, 1.0, &t, 150.0 / 72.0);
        assert!(lines.len() > 50);
        assert!(lines.iter().filter(|l| !l.arrows.is_empty()).count() > 20);
    }
}
