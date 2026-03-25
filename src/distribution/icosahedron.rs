use glam::Vec3;

use super::{triangle_point, Distribution3};
use crate::rng::Rng;

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
    use super::{icosahedron_faces, icosahedron_vertices, Icosahedron};
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn icosahedron_points_stay_on_icosahedron_faces() {
        let mut rng = Rng::new(0x1234_5678);
        let vertices = icosahedron_vertices(0.95);
        let faces = icosahedron_faces();

        for point in collect(&mut Icosahedron::new(0.95), 32, &mut rng) {
            assert!(faces.iter().any(|[a, b, c]| {
                let normal = (vertices[*b] - vertices[*a])
                    .cross(vertices[*c] - vertices[*a])
                    .normalize();
                normal.dot(point - vertices[*a]).abs() < 1e-5
            }));
        }
    }
}
