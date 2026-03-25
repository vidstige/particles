use glam::Vec3;

use super::Distribution3;
use crate::rng::Rng;

#[derive(Debug)]
pub struct Gaussian {
    scale: f32,
}

impl Gaussian {
    pub fn new(scale: f32) -> Self {
        Self { scale }
    }
}

impl Distribution3 for Gaussian {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        Vec3::new(
            rng.next_gaussian(),
            rng.next_gaussian(),
            rng.next_gaussian(),
        ) * self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::Gaussian;
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn gaussian_samples_stay_finite() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(&mut Gaussian::new(0.35), 32, &mut rng) {
            assert!(point.is_finite());
        }
    }
}
