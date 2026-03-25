mod cube;
mod distribution3;
mod gaussian;
mod grid3d;
mod gyroid;
mod lissajous;
mod sphere;
mod tetrahedron;
mod torus_surface;
mod uniform_cube;

use glam::Vec3;

use crate::rng::Rng;

pub use cube::Cube;
pub use distribution3::{collect, Distribution3};
pub use gaussian::Gaussian;
pub use grid3d::Grid3d;
pub use gyroid::Gyroid;
pub use lissajous::Lissajous;
pub use sphere::Sphere;
pub use tetrahedron::Tetrahedron;
pub use torus_surface::TorusSurface;
pub use uniform_cube::UniformCube;

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

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::{collect, icosahedron_faces, icosahedron_vertices, Icosahedron};
    use crate::rng::Rng;

    fn point_is_on_any_face(point: Vec3, vertices: &[Vec3], faces: &[[usize; 3]]) -> bool {
        faces.iter().any(|[a, b, c]| {
            let normal = (vertices[*b] - vertices[*a])
                .cross(vertices[*c] - vertices[*a])
                .normalize();
            normal.dot(point - vertices[*a]).abs() < 1e-5
        })
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
