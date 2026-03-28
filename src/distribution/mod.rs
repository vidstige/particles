mod add;
mod constant;
mod cube;
mod distribution3;
mod gaussian;
mod gyroid;
mod icosahedron;
mod lissajous;
mod sphere;
mod tetrahedron;
mod torus_surface;
mod uniform_cube;

use glam::Vec3;

use crate::rng::Rng;

pub use add::Add;
pub use constant::Constant;
pub use cube::Cube;
pub use distribution3::{collect, Distribution3};
pub use gaussian::Gaussian;
pub use gyroid::Gyroid;
pub use icosahedron::Icosahedron;
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
