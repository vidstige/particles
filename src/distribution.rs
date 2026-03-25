use glam::Vec3;

use crate::rng::Rng;

fn grid_radius(count: usize) -> i32 {
    let mut radius = 0;
    while ((2 * radius + 1) as usize).pow(3) < count {
        radius += 1;
    }
    radius
}

fn sphere_point(radius: f32, z: f32, angle: f32) -> Vec3 {
    let ring = (1.0 - z * z).sqrt() * radius;
    Vec3::new(ring * angle.cos(), ring * angle.sin(), z * radius)
}

pub fn uniform_cube(count: usize, rng: &mut Rng) -> Vec<Vec3> {
    (0..count)
        .map(|_| Vec3::new(rng.next_f32(), rng.next_f32(), rng.next_f32()) * 2.0 - Vec3::ONE)
        .collect()
}

pub fn gaussian_sphere(count: usize, rng: &mut Rng) -> Vec<Vec3> {
    (0..count)
        .map(|_| {
            Vec3::new(
                rng.next_gaussian(),
                rng.next_gaussian(),
                rng.next_gaussian(),
            ) * 0.35
        })
        .collect()
}

pub fn sphere(count: usize, radius: f32, rng: &mut Rng) -> Vec<Vec3> {
    (0..count)
        .map(|_| {
            sphere_point(
                radius,
                rng.next_f32_in(-1.0, 1.0),
                std::f32::consts::TAU * rng.next_f32(),
            )
        })
        .collect()
}

pub fn grid_3d(count: usize, spacing: Vec3) -> Vec<Vec3> {
    let radius = grid_radius(count);
    let mut positions = (-radius..=radius)
        .flat_map(|z| {
            (-radius..=radius).flat_map(move |y| {
                (-radius..=radius).map(move |x| {
                    Vec3::new(
                        x as f32 * spacing.x,
                        y as f32 * spacing.y,
                        z as f32 * spacing.z,
                    )
                })
            })
        })
        .collect::<Vec<_>>();
    positions.sort_by(|a, b| {
        a.length_squared()
            .total_cmp(&b.length_squared())
            .then(a.x.total_cmp(&b.x))
            .then(a.y.total_cmp(&b.y))
            .then(a.z.total_cmp(&b.z))
    });
    positions.truncate(count);
    positions
}

pub fn torus_surface(
    count: usize,
    major_radius: f32,
    minor_radius: f32,
    rng: &mut Rng,
) -> Vec<Vec3> {
    (0..count)
        .map(|_| {
            let u = std::f32::consts::TAU * rng.next_f32();
            let v = std::f32::consts::TAU * rng.next_f32();
            let ring = major_radius + minor_radius * v.cos();
            Vec3::new(ring * u.cos(), ring * u.sin(), minor_radius * v.sin())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::{grid_3d, sphere, torus_surface};
    use crate::rng::Rng;

    #[test]
    fn grid_3d_returns_centered_points_first() {
        let points = grid_3d(5, Vec3::new(2.0, 4.0, 6.0));

        assert_eq!(
            points,
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(-2.0, 0.0, 0.0),
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(0.0, -4.0, 0.0),
                Vec3::new(0.0, 4.0, 0.0),
            ]
        );
    }

    #[test]
    fn torus_surface_points_stay_on_torus() {
        let mut rng = Rng::new(0x1234_5678);
        let major_radius = 0.8;
        let minor_radius = 0.25;

        for point in torus_surface(32, major_radius, minor_radius, &mut rng) {
            let ring = point.truncate().length();
            let torus_distance = (ring - major_radius).powi(2) + point.z.powi(2);
            assert!((torus_distance - minor_radius.powi(2)).abs() < 1e-5);
        }
    }

    #[test]
    fn sphere_points_stay_on_requested_radius() {
        let mut rng = Rng::new(0x1234_5678);

        for point in sphere(32, 0.8, &mut rng) {
            assert!((point.length() - 0.8).abs() < 1e-5);
        }
    }
}
