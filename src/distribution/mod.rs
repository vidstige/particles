mod distribution3;
mod gaussian;
mod uniform_cube;

use glam::{UVec3, Vec3};

use crate::rng::Rng;

pub use distribution3::{collect, Distribution3};
pub use gaussian::Gaussian;
pub use uniform_cube::UniformCube;

fn grid_axis_point(index: u32, resolution: u32, size: f32) -> f32 {
    if resolution <= 1 {
        0.0
    } else {
        index as f32 * size / (resolution - 1) as f32 - size * 0.5
    }
}

fn sphere_point(radius: f32, z: f32, angle: f32) -> Vec3 {
    let ring = (1.0 - z * z).sqrt() * radius;
    Vec3::new(ring * angle.cos(), ring * angle.sin(), z * radius)
}

fn lissajous_point(t: f32, scale: f32) -> Vec3 {
    Vec3::new((3.0 * t).sin(), (5.0 * t).sin(), (7.0 * t).sin()) * scale
}

fn gyroid_value(point: Vec3) -> f32 {
    point.x.sin() * point.y.cos() + point.y.sin() * point.z.cos() + point.z.sin() * point.x.cos()
}

fn cube_face_point(half_extent: f32, face: usize, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(-half_extent, u, v),
        1 => Vec3::new(half_extent, u, v),
        2 => Vec3::new(u, -half_extent, v),
        3 => Vec3::new(u, half_extent, v),
        4 => Vec3::new(u, v, -half_extent),
        _ => Vec3::new(u, v, half_extent),
    }
}

fn triangle_point(a: Vec3, b: Vec3, c: Vec3, rng: &mut Rng) -> Vec3 {
    let u = rng.next_f32().sqrt();
    let v = rng.next_f32();
    a * (1.0 - u) + b * (u * (1.0 - v)) + c * (u * v)
}

fn tetrahedron_vertices(radius: f32) -> [Vec3; 4] {
    [
        Vec3::new(1.0, 1.0, 1.0).normalize() * radius,
        Vec3::new(-1.0, -1.0, 1.0).normalize() * radius,
        Vec3::new(-1.0, 1.0, -1.0).normalize() * radius,
        Vec3::new(1.0, -1.0, -1.0).normalize() * radius,
    ]
}

fn tetrahedron_faces() -> [[usize; 3]; 4] {
    [[0, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]]
}

fn icosahedron_vertices(radius: f32) -> [Vec3; 12] {
    let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
    [
        Vec3::new(-1.0, phi, 0.0).normalize() * radius,
        Vec3::new(1.0, phi, 0.0).normalize() * radius,
        Vec3::new(-1.0, -phi, 0.0).normalize() * radius,
        Vec3::new(1.0, -phi, 0.0).normalize() * radius,
        Vec3::new(0.0, -1.0, phi).normalize() * radius,
        Vec3::new(0.0, 1.0, phi).normalize() * radius,
        Vec3::new(0.0, -1.0, -phi).normalize() * radius,
        Vec3::new(0.0, 1.0, -phi).normalize() * radius,
        Vec3::new(phi, 0.0, -1.0).normalize() * radius,
        Vec3::new(phi, 0.0, 1.0).normalize() * radius,
        Vec3::new(-phi, 0.0, -1.0).normalize() * radius,
        Vec3::new(-phi, 0.0, 1.0).normalize() * radius,
    ]
}

fn icosahedron_faces() -> [[usize; 3]; 20] {
    [
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ]
}

#[derive(Debug)]
pub struct Sphere {
    radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Distribution3 for Sphere {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        sphere_point(
            self.radius,
            rng.next_f32_in(-1.0, 1.0),
            std::f32::consts::TAU * rng.next_f32(),
        )
    }
}

#[derive(Debug)]
pub struct Lissajous {
    phase: f32,
    scale: f32,
    step: f32,
}

impl Lissajous {
    pub fn new(count: usize, scale: f32) -> Self {
        Self {
            phase: 0.0,
            scale,
            step: std::f32::consts::TAU / count as f32,
        }
    }
}

impl Distribution3 for Lissajous {
    fn sample(&mut self, _rng: &mut Rng) -> Vec3 {
        let point = lissajous_point(self.phase, self.scale);
        self.phase += self.step;
        point
    }
}

#[derive(Debug)]
pub struct Gyroid {
    scale: f32,
    thickness: f32,
}

impl Gyroid {
    pub fn new(scale: f32, thickness: f32) -> Self {
        Self { scale, thickness }
    }
}

impl Distribution3 for Gyroid {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        loop {
            let point = Vec3::new(
                rng.next_f32_in(-std::f32::consts::PI, std::f32::consts::PI),
                rng.next_f32_in(-std::f32::consts::PI, std::f32::consts::PI),
                rng.next_f32_in(-std::f32::consts::PI, std::f32::consts::PI),
            );
            if gyroid_value(point).abs() <= self.thickness {
                return point * (self.scale / std::f32::consts::PI);
            }
        }
    }
}

#[derive(Debug)]
pub struct Cube {
    half_extent: f32,
}

impl Cube {
    pub fn new(half_extent: f32) -> Self {
        Self { half_extent }
    }
}

impl Distribution3 for Cube {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        cube_face_point(
            self.half_extent,
            rng.next_index(6),
            rng.next_f32_in(-self.half_extent, self.half_extent),
            rng.next_f32_in(-self.half_extent, self.half_extent),
        )
    }
}

#[derive(Debug)]
pub struct Tetrahedron {
    radius: f32,
}

impl Tetrahedron {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Distribution3 for Tetrahedron {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        let vertices = tetrahedron_vertices(self.radius);
        let [a, b, c] = tetrahedron_faces()[rng.next_index(4)];
        triangle_point(vertices[a], vertices[b], vertices[c], rng)
    }
}

#[derive(Debug)]
pub struct Icosahedron {
    radius: f32,
}

impl Icosahedron {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Distribution3 for Icosahedron {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        let vertices = icosahedron_vertices(self.radius);
        let [a, b, c] = icosahedron_faces()[rng.next_index(20)];
        triangle_point(vertices[a], vertices[b], vertices[c], rng)
    }
}

#[derive(Debug)]
pub struct Grid3d {
    resolution: UVec3,
    size: Vec3,
}

impl Grid3d {
    pub fn new(resolution: UVec3, size: Vec3) -> Self {
        Self { resolution, size }
    }
}

impl Distribution3 for Grid3d {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        let mut point = Vec3::new(
            rng.next_f32_in(-self.size.x * 0.5, self.size.x * 0.5),
            rng.next_f32_in(-self.size.y * 0.5, self.size.y * 0.5),
            rng.next_f32_in(-self.size.z * 0.5, self.size.z * 0.5),
        );

        match rng.next_index(3) {
            0 => {
                let plane = rng.next_index(self.resolution.x as usize) as u32;
                point.x = grid_axis_point(plane, self.resolution.x, self.size.x);
            }
            1 => {
                let plane = rng.next_index(self.resolution.y as usize) as u32;
                point.y = grid_axis_point(plane, self.resolution.y, self.size.y);
            }
            _ => {
                let plane = rng.next_index(self.resolution.z as usize) as u32;
                point.z = grid_axis_point(plane, self.resolution.z, self.size.z);
            }
        }

        point
    }
}

#[derive(Debug)]
pub struct TorusSurface {
    major_radius: f32,
    minor_radius: f32,
}

impl TorusSurface {
    pub fn new(major_radius: f32, minor_radius: f32) -> Self {
        Self {
            major_radius,
            minor_radius,
        }
    }
}

impl Distribution3 for TorusSurface {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        let u = std::f32::consts::TAU * rng.next_f32();
        let v = std::f32::consts::TAU * rng.next_f32();
        let ring = self.major_radius + self.minor_radius * v.cos();
        Vec3::new(ring * u.cos(), ring * u.sin(), self.minor_radius * v.sin())
    }
}

#[cfg(test)]
mod tests {
    use glam::{UVec3, Vec3};

    use super::{
        collect, gyroid_value, icosahedron_faces, icosahedron_vertices, tetrahedron_faces,
        tetrahedron_vertices, Cube, Grid3d, Gyroid, Icosahedron, Lissajous, Sphere, Tetrahedron,
        TorusSurface,
    };
    use crate::rng::Rng;

    fn point_is_on_any_face(point: Vec3, vertices: &[Vec3], faces: &[[usize; 3]]) -> bool {
        faces.iter().any(|[a, b, c]| {
            let normal = (vertices[*b] - vertices[*a])
                .cross(vertices[*c] - vertices[*a])
                .normalize();
            normal.dot(point - vertices[*a]).abs() < 1e-5
        })
    }

    fn is_on_grid_plane(coordinate: f32, resolution: u32, size: f32) -> bool {
        (0..resolution).any(|index| {
            (coordinate - super::grid_axis_point(index, resolution, size)).abs() < 1e-5
        })
    }

    #[test]
    fn grid_3d_points_stay_within_bounds_and_on_grid_planes() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(
            &mut Grid3d::new(UVec3::new(3, 2, 4), Vec3::new(2.0, 4.0, 6.0)),
            64,
            &mut rng,
        ) {
            assert!(point.x.abs() <= 1.0);
            assert!(point.y.abs() <= 2.0);
            assert!(point.z.abs() <= 3.0);
            assert!(
                is_on_grid_plane(point.x, 3, 2.0)
                    || is_on_grid_plane(point.y, 2, 4.0)
                    || is_on_grid_plane(point.z, 4, 6.0)
            );
        }
    }

    #[test]
    fn torus_surface_points_stay_on_torus() {
        let mut rng = Rng::new(0x1234_5678);
        let major_radius = 0.8;
        let minor_radius = 0.25;

        for point in collect(
            &mut TorusSurface::new(major_radius, minor_radius),
            32,
            &mut rng,
        ) {
            let ring = point.truncate().length();
            let torus_distance = (ring - major_radius).powi(2) + point.z.powi(2);
            assert!((torus_distance - minor_radius.powi(2)).abs() < 1e-5);
        }
    }

    #[test]
    fn sphere_points_stay_on_requested_radius() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(&mut Sphere::new(0.8), 32, &mut rng) {
            assert!((point.length() - 0.8).abs() < 1e-5);
        }
    }

    #[test]
    fn lissajous_is_antipodal_half_a_cycle_later() {
        let points = collect(
            &mut Lissajous::new(16, 0.75),
            16,
            &mut Rng::new(0x1234_5678),
        );

        for index in 0..8 {
            assert!((points[index] + points[index + 8]).length() < 1e-5);
        }
    }

    #[test]
    fn gyroid_points_stay_close_to_implicit_surface() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(&mut Gyroid::new(1.2, 0.08), 32, &mut rng) {
            let unscaled = point * (std::f32::consts::PI / 1.2);
            assert!(gyroid_value(unscaled).abs() <= 0.08);
        }
    }

    #[test]
    fn cube_points_stay_on_cube_surface() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(&mut Cube::new(0.7), 32, &mut rng) {
            assert!(point.max_element() <= 0.7);
            assert!(point.min_element() >= -0.7);
            assert!((point.abs().max_element() - 0.7).abs() < 1e-5);
        }
    }

    #[test]
    fn tetrahedron_points_stay_on_tetrahedron_faces() {
        let mut rng = Rng::new(0x1234_5678);
        let vertices = tetrahedron_vertices(0.9);
        let faces = tetrahedron_faces();

        for point in collect(&mut Tetrahedron::new(0.9), 32, &mut rng) {
            assert!(point_is_on_any_face(point, &vertices, &faces));
        }
    }

    #[test]
    fn icosahedron_points_stay_on_icosahedron_faces() {
        let mut rng = Rng::new(0x1234_5678);
        let vertices = icosahedron_vertices(0.95);
        let faces = icosahedron_faces();

        for point in collect(&mut Icosahedron::new(0.95), 32, &mut rng) {
            assert!(point_is_on_any_face(point, &vertices, &faces));
        }
    }
}
