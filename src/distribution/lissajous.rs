use glam::Vec3;

use super::{lissajous_point, Distribution3};
use crate::rng::Rng;

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

#[cfg(test)]
mod tests {
    use super::Lissajous;
    use crate::{distribution::collect, rng::Rng};

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
}
