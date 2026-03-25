use glam::Vec3;

use super::{sphere_point, Distribution3};
use crate::rng::Rng;

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

#[cfg(test)]
mod tests {
    use super::Sphere;
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn sphere_points_stay_on_requested_radius() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(&mut Sphere::new(0.8), 32, &mut rng) {
            assert!((point.length() - 0.8).abs() < 1e-5);
        }
    }
}
