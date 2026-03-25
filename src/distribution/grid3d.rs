use glam::{UVec3, Vec3};

use super::Distribution3;
use crate::rng::Rng;

fn grid_axis_point(index: u32, resolution: u32, size: f32) -> f32 {
    if resolution <= 1 {
        0.0
    } else {
        index as f32 * size / (resolution - 1) as f32 - size * 0.5
    }
}

#[derive(Debug)]
pub struct Grid3d {
    resolution: UVec3,
    size: Vec3,
}

impl Grid3d {
    pub fn new(resolution: UVec3, size: Vec3) -> Self {
        Self { resolution, size }
    }
}

impl Distribution3 for Grid3d {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        let mut point = Vec3::new(
            rng.next_f32_in(-self.size.x * 0.5, self.size.x * 0.5),
            rng.next_f32_in(-self.size.y * 0.5, self.size.y * 0.5),
            rng.next_f32_in(-self.size.z * 0.5, self.size.z * 0.5),
        );

        match rng.next_index(3) {
            0 => {
                let plane = rng.next_index(self.resolution.x as usize) as u32;
                point.x = grid_axis_point(plane, self.resolution.x, self.size.x);
            }
            1 => {
                let plane = rng.next_index(self.resolution.y as usize) as u32;
                point.y = grid_axis_point(plane, self.resolution.y, self.size.y);
            }
            _ => {
                let plane = rng.next_index(self.resolution.z as usize) as u32;
                point.z = grid_axis_point(plane, self.resolution.z, self.size.z);
            }
        }

        point
    }
}

#[cfg(test)]
mod tests {
    use super::{grid_axis_point, Grid3d};
    use crate::{distribution::collect, rng::Rng};
    use glam::{UVec3, Vec3};

    fn is_on_grid_plane(coordinate: f32, resolution: u32, size: f32) -> bool {
        (0..resolution)
            .any(|index| (coordinate - grid_axis_point(index, resolution, size)).abs() < 1e-5)
    }

    #[test]
    fn grid_3d_points_stay_within_bounds_and_on_grid_planes() {
        let mut rng = Rng::new(0x1234_5678);

        for point in collect(
            &mut Grid3d::new(UVec3::new(3, 2, 4), Vec3::new(2.0, 4.0, 6.0)),
            64,
            &mut rng,
        ) {
            assert!(point.x.abs() <= 1.0);
            assert!(point.y.abs() <= 2.0);
            assert!(point.z.abs() <= 3.0);
            assert!(
                is_on_grid_plane(point.x, 3, 2.0)
                    || is_on_grid_plane(point.y, 2, 4.0)
                    || is_on_grid_plane(point.z, 4, 6.0)
            );
        }
    }
}
