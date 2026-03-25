use glam::Vec3;

use super::{triangle_point, Distribution3};
use crate::rng::Rng;

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

#[cfg(test)]
mod tests {
    use super::{tetrahedron_faces, tetrahedron_vertices, Tetrahedron};
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn tetrahedron_points_stay_on_tetrahedron_faces() {
        let mut rng = Rng::new(0x1234_5678);
        let vertices = tetrahedron_vertices(0.9);
        let faces = tetrahedron_faces();

        for point in collect(&mut Tetrahedron::new(0.9), 32, &mut rng) {
            assert!(faces.iter().any(|[a, b, c]| {
                let normal = (vertices[*b] - vertices[*a])
                    .cross(vertices[*c] - vertices[*a])
                    .normalize();
                normal.dot(point - vertices[*a]).abs() < 1e-5
            }));
        }
    }
}
