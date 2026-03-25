use glam::Vec3;

use super::{cube_face_point, Distribution3};
use crate::rng::Rng;

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

#[cfg(test)]
mod tests {
    use super::Cube;
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn cube_points_stay_on_cube_surface() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(&mut Cube::new(0.7), 32, &mut rng) {
            assert!(point.max_element() <= 0.7);
            assert!(point.min_element() >= -0.7);
            assert!((point.abs().max_element() - 0.7).abs() < 1e-5);
        }
    }
}
