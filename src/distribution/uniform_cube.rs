use glam::Vec3;

use super::Distribution3;
use crate::rng::Rng;

#[derive(Debug)]
pub struct UniformCube;

impl UniformCube {
    pub fn new() -> Self {
        Self
    }
}

impl Distribution3 for UniformCube {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        Vec3::new(rng.next_f32(), rng.next_f32(), rng.next_f32()) * 2.0 - Vec3::ONE
    }
}

#[cfg(test)]
mod tests {
    use super::UniformCube;
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn uniform_cube_samples_stay_within_unit_cube() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(&mut UniformCube::new(), 32, &mut rng) {
            assert!(point.max_element() <= 1.0);
            assert!(point.min_element() >= -1.0);
        }
    }
}
