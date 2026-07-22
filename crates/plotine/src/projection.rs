//! 3D→2D projection for Axes3D (matplotlib mplot3d look-at + perspective).
//!
//! Follows matplotlib's `Axes3D.get_proj`:
//! - `elev` / `azim` place the eye on a sphere around the data box
//! - box aspect defaults to `4:4:3` (`set_box_aspect(None)`)
//! - perspective with `focal_length=1`, `dist=10`

use std::f64::consts::PI;

use crate::mpl_policy::axes3d as ax3_policy;

/// A point in 3D data space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Z coordinate.
    pub z: f64,
}

impl Point3 {
    /// Create a new 3D point.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Linear interpolation between two points.
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }
}

/// Camera parameters for 3D projection.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// Elevation angle in degrees (default 30°, like matplotlib).
    pub elev: f64,
    /// Azimuth angle in degrees (default -60°, like matplotlib).
    pub azim: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            elev: ax3_policy::ELEV,
            azim: ax3_policy::AZIM,
        }
    }
}

/// Projected 2D point with depth for painter's algorithm sorting.
#[derive(Debug, Clone, Copy)]
pub struct Projected {
    /// Screen x (right is positive).
    pub x: f64,
    /// Screen y (up is positive).
    pub y: f64,
    /// Depth (larger = farther from camera).
    pub depth: f64,
}

/// View / perspective projection matching matplotlib `Axes3D.get_proj`.
#[derive(Debug, Clone, Copy)]
pub struct Projection {
    /// Camera-right unit vector (world).
    u: Point3,
    /// Camera-up unit vector (world).
    v: Point3,
    /// Camera-out unit vector (world), from box center toward eye.
    w: Point3,
    /// Eye position in world (box) coordinates.
    eye: Point3,
    /// Focal length (matplotlib default 1).
    focal: f64,
}

impl Projection {
    /// Build a projection from camera angles (degrees).
    pub fn from_camera(camera: Camera) -> Self {
        let [ax, ay, az] = ax3_policy::box_aspect();
        let elev = camera.elev * PI / 180.0;
        let azim = camera.azim * PI / 180.0;

        // Eye direction on the unit sphere (matplotlib `get_proj`).
        let p0 = elev.cos() * azim.cos();
        let p1 = elev.cos() * azim.sin();
        let p2 = elev.sin();

        let focal = ax3_policy::FOCAL_LENGTH;
        let cx = 0.5 * ax;
        let cy = 0.5 * ay;
        let cz = 0.5 * az;
        let eye = Point3::new(
            cx + ax3_policy::DIST * focal * p0,
            cy + ax3_policy::DIST * focal * p1,
            cz + ax3_policy::DIST * focal * p2,
        );

        // Viewing axes (`proj3d._view_axes`).
        let w = normalize_vec(Point3::new(eye.x - cx, eye.y - cy, eye.z - cz));
        let up = Point3::new(0.0, 0.0, if elev.abs() > PI / 2.0 { -1.0 } else { 1.0 });
        let u = normalize_vec(cross(up, w));
        let v = cross(w, u);

        Self {
            u,
            v,
            w,
            eye,
            focal,
        }
    }

    /// Project a point already in world/box coordinates `[0, aspect]³`.
    pub fn project(&self, p: Point3) -> Projected {
        // View: rows [u,v,w] applied to (p - eye).
        let qx = p.x - self.eye.x;
        let qy = p.y - self.eye.y;
        let qz = p.z - self.eye.z;
        let cam_x = self.u.x * qx + self.u.y * qy + self.u.z * qz;
        let cam_y = self.v.x * qx + self.v.y * qy + self.v.z * qz;
        let cam_z = self.w.x * qx + self.w.y * qy + self.w.z * qz;

        // Perspective (`proj3d._persp_transformation(-dist, dist, focal)`).
        let clip_x = self.focal * cam_x;
        let clip_y = self.focal * cam_y;
        let clip_w = -cam_z;
        let inv_w = 1.0 / clip_w;
        Projected {
            x: clip_x * inv_w,
            y: clip_y * inv_w,
            // Larger = farther (cam_z is more negative when farther from eye).
            depth: -cam_z,
        }
    }

    /// Project a 3D data point given axis ranges.
    pub fn project_data(
        &self,
        p: Point3,
        x_range: (f64, f64),
        y_range: (f64, f64),
        z_range: (f64, f64),
    ) -> Projected {
        let [ax, ay, az] = ax3_policy::box_aspect();
        // `world_transformation`: data → [0, aspect_i].
        let wx = normalize(p.x, x_range.0, x_range.1) * ax;
        let wy = normalize(p.y, y_range.0, y_range.1) * ay;
        let wz = normalize(p.z, z_range.0, z_range.1) * az;
        self.project(Point3::new(wx, wy, wz))
    }
}

fn normalize(v: f64, min: f64, max: f64) -> f64 {
    let span = (max - min).abs();
    if span < 1e-12 {
        0.5
    } else {
        (v - min) / span
    }
}

fn normalize_vec(p: Point3) -> Point3 {
    let n = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt().max(1e-15);
    Point3::new(p.x / n, p.y / n, p.z / n)
}

fn cross(a: Point3, b: Point3) -> Point3 {
    Point3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

/// Axis bounding box corners in world/box coordinates (`[0, aspect]³`).
pub fn cube_corners() -> [Point3; 8] {
    let [ax, ay, az] = ax3_policy::box_aspect();
    [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(ax, 0.0, 0.0),
        Point3::new(ax, ay, 0.0),
        Point3::new(0.0, ay, 0.0),
        Point3::new(0.0, 0.0, az),
        Point3::new(ax, 0.0, az),
        Point3::new(ax, ay, az),
        Point3::new(0.0, ay, az),
    ]
}

/// The 12 edges of the cube as pairs of corner indices.
pub const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0), // bottom face
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4), // top face
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7), // verticals
];

/// Determine which 3 faces of the cube are "back" (should be drawn first / show axes).
/// Returns edge indices for the 3 back-facing edges that should carry tick labels.
pub fn back_edges(proj: &Projection) -> BackEdges {
    let corners = cube_corners();
    let projected: Vec<Projected> = corners.iter().map(|&c| proj.project(c)).collect();

    // Find the corner with maximum depth (farthest from camera) — that's the
    // "back corner" where 3 axes meet (matplotlib draws axes there).
    let back_idx = projected
        .iter()
        .enumerate()
        .max_by(|a, b| {
            a.1.depth
                .partial_cmp(&b.1.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    // The 3 edges emanating from the back corner.
    let mut edges = [0usize; 3];
    let mut count = 0;
    for (ei, &(a, b)) in CUBE_EDGES.iter().enumerate() {
        if (a == back_idx || b == back_idx) && count < 3 {
            edges[count] = ei;
            count += 1;
        }
    }

    BackEdges {
        edge_indices: edges,
        back_corner: back_idx,
    }
}

/// Result of back-edge detection for axis drawing.
#[derive(Debug, Clone, Copy)]
pub struct BackEdges {
    /// Indices into [`CUBE_EDGES`] for the 3 back-facing axis edges.
    pub edge_indices: [usize; 3],
    /// Index into the 8 corners for the farthest (back) corner.
    pub back_corner: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_camera_projects_center_to_origin() {
        let proj = Projection::from_camera(Camera::default());
        let [ax, ay, az] = ax3_policy::box_aspect();
        let p = proj.project(Point3::new(0.5 * ax, 0.5 * ay, 0.5 * az));
        assert!(p.x.abs() < 1e-9, "x={}", p.x);
        assert!(p.y.abs() < 1e-9, "y={}", p.y);
    }

    #[test]
    fn azim_matches_matplotlib_bottom_tip() {
        // At elev=30, azim=-60 the floor corner (x_max, y_min, z_min) is lowest on screen.
        let proj = Projection::from_camera(Camera {
            elev: 30.0,
            azim: -60.0,
        });
        let ranges = ((0.0, 1.0), (0.0, 1.0), (0.0, 1.0));
        let corners = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
        ];
        let ys: Vec<f64> = corners
            .iter()
            .map(|&c| proj.project_data(c, ranges.0, ranges.1, ranges.2).y)
            .collect();
        let bottom = ys
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        assert_eq!(bottom, 1, "expected (1,0,0) as bottom tip like mplot3d");
    }

    #[test]
    fn perspective_matches_matplotlib_ndc() {
        // Numeric check against Axes3D.get_proj for elev=30, azim=-55 on unit data cube.
        let proj = Projection::from_camera(Camera {
            elev: 30.0,
            azim: -55.0,
        });
        let ranges = ((0.0, 1.0), (0.0, 1.0), (0.0, 1.0));
        let p = proj.project_data(Point3::new(1.0, 0.0, 0.0), ranges.0, ranges.1, ranges.2);
        // From matplotlib reference run in this session:
        assert!((p.x - 0.015378404066198009).abs() < 1e-6, "x={}", p.x);
        assert!((p.y - (-0.08428173010311531)).abs() < 1e-6, "y={}", p.y);
    }

    #[test]
    fn normalize_works() {
        assert!((normalize(5.0, 0.0, 10.0) - 0.5).abs() < 1e-12);
        assert!((normalize(0.0, 0.0, 10.0)).abs() < 1e-12);
        assert!((normalize(10.0, 0.0, 10.0) - 1.0).abs() < 1e-12);
    }
}
