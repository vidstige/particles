use glam::Vec3;

use super::Distribution3;
use crate::rng::Rng;

#[derive(Debug)]
pub struct TorusSurface {
    major_radius: f32,
    minor_radius: f32,
}

impl TorusSurface {
    pub fn new(major_radius: f32, minor_radius: f32) -> Self {
        Self {
            major_radius,
            minor_radius,
        }
    }
}

impl Distribution3 for TorusSurface {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        let u = std::f32::consts::TAU * rng.next_f32();
        let v = std::f32::consts::TAU * rng.next_f32();
        let ring = self.major_radius + self.minor_radius * v.cos();
        Vec3::new(ring * u.cos(), ring * u.sin(), self.minor_radius * v.sin())
    }
}

#[cfg(test)]
mod tests {
    use super::TorusSurface;
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn torus_surface_points_stay_on_torus() {
        let mut rng = Rng::new(0x1234_5678);
        let major_radius = 0.8;
        let minor_radius = 0.25;

        for point in collect(
            &mut TorusSurface::new(major_radius, minor_radius),
            32,
            &mut rng,
        ) {
            let ring = point.truncate().length();
            let torus_distance = (ring - major_radius).powi(2) + point.z.powi(2);
            assert!((torus_distance - minor_radius.powi(2)).abs() < 1e-5);
        }
    }
}
