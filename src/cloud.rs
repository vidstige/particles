use glam::Vec3;

use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct Cloud {
    pub positions: Vec<Vec3>,
}

impl Cloud {
    pub fn uniform_cube(count: usize, rng: &mut Rng) -> Self {
        let positions = (0..count)
            .map(|_| Vec3::new(rng.next_f32(), rng.next_f32(), rng.next_f32()) * 2.0 - Vec3::ONE)
            .collect();
        Self { positions }
    }

    pub fn gaussian_sphere(count: usize, rng: &mut Rng) -> Self {
        let positions = (0..count)
            .map(|_| {
                Vec3::new(
                    rng.next_gaussian(),
                    rng.next_gaussian(),
                    rng.next_gaussian(),
                ) * 0.35
            })
            .collect();
        Self { positions }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::Cloud;
    use crate::rng::Rng;

    #[test]
    fn uniform_cube_stays_inside_bounds() {
        let mut rng = Rng::new(1);
        let cloud = Cloud::uniform_cube(4_096, &mut rng);
        for point in cloud.positions {
            assert!(point.cmpge(Vec3::splat(-1.0)).all());
            assert!(point.cmple(Vec3::splat(1.0)).all());
        }
    }

    #[test]
    fn gaussian_sphere_is_more_concentrated_than_uniform_cube() {
        let mut uniform_rng = Rng::new(2);
        let mut gaussian_rng = Rng::new(2);
        let uniform = Cloud::uniform_cube(4_096, &mut uniform_rng);
        let gaussian = Cloud::gaussian_sphere(4_096, &mut gaussian_rng);

        let uniform_mean = mean_length(&uniform);
        let gaussian_mean = mean_length(&gaussian);
        let gaussian_center = mean_position(&gaussian);

        assert!(gaussian_mean < uniform_mean * 0.7);
        assert!(gaussian_center.length() < 0.05);
    }

    fn mean_length(cloud: &Cloud) -> f32 {
        cloud.positions.iter().map(|point| point.length()).sum::<f32>() / cloud.positions.len() as f32
    }

    fn mean_position(cloud: &Cloud) -> Vec3 {
        cloud.positions.iter().copied().sum::<Vec3>() / cloud.positions.len() as f32
    }
}
