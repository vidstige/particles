use glam::Vec3;

use super::{gyroid_value, Distribution3};
use crate::rng::Rng;

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

#[cfg(test)]
mod tests {
    use super::{gyroid_value, Gyroid};
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn gyroid_points_stay_close_to_implicit_surface() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(&mut Gyroid::new(1.2, 0.08), 32, &mut rng) {
            let unscaled = point * (std::f32::consts::PI / 1.2);
            assert!(gyroid_value(unscaled).abs() <= 0.08);
        }
    }
}
