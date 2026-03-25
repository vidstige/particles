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

fn lissajous_point(t: f32, scale: f32) -> Vec3 {
    Vec3::new((3.0 * t).sin(), (5.0 * t).sin(), (7.0 * t).sin()) * scale
}

fn gyroid_value(point: Vec3) -> f32 {
    point.x.sin() * point.y.cos() + point.y.sin() * point.z.cos() + point.z.sin() * point.x.cos()
}

fn cube_face_point(half_extent: f32, face: usize, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(-half_extent, u, v),
        1 => Vec3::new(half_extent, u, v),
        2 => Vec3::new(u, -half_extent, v),
        3 => Vec3::new(u, half_extent, v),
        4 => Vec3::new(u, v, -half_extent),
        _ => Vec3::new(u, v, half_extent),
    }
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

pub fn lissajous(count: usize, scale: f32) -> Vec<Vec3> {
    (0..count)
        .map(|index| lissajous_point(index as f32 * std::f32::consts::TAU / count as f32, scale))
        .collect()
}

pub fn gyroid(count: usize, scale: f32, thickness: f32, rng: &mut Rng) -> Vec<Vec3> {
    let mut positions = Vec::with_capacity(count);

    while positions.len() < count {
        let point = Vec3::new(
            rng.next_f32_in(-std::f32::consts::PI, std::f32::consts::PI),
            rng.next_f32_in(-std::f32::consts::PI, std::f32::consts::PI),
            rng.next_f32_in(-std::f32::consts::PI, std::f32::consts::PI),
        );
        if gyroid_value(point).abs() <= thickness {
            positions.push(point * (scale / std::f32::consts::PI));
        }
    }

    positions
}

pub fn cube(count: usize, half_extent: f32, rng: &mut Rng) -> Vec<Vec3> {
    (0..count)
        .map(|_| {
            cube_face_point(
                half_extent,
                rng.next_index(6),
                rng.next_f32_in(-half_extent, half_extent),
                rng.next_f32_in(-half_extent, half_extent),
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

    use super::{cube, grid_3d, gyroid, gyroid_value, lissajous, sphere, torus_surface};
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

    #[test]
    fn lissajous_is_antipodal_half_a_cycle_later() {
        let points = lissajous(16, 0.75);

        for index in 0..8 {
            assert!((points[index] + points[index + 8]).length() < 1e-5);
        }
    }

    #[test]
    fn gyroid_points_stay_close_to_implicit_surface() {
        let mut rng = Rng::new(0x1234_5678);

        for point in gyroid(32, 1.2, 0.08, &mut rng) {
            let unscaled = point * (std::f32::consts::PI / 1.2);
            assert!(gyroid_value(unscaled).abs() <= 0.08);
        }
    }

    #[test]
    fn cube_points_stay_on_cube_surface() {
        let mut rng = Rng::new(0x1234_5678);

        for point in cube(32, 0.7, &mut rng) {
            assert!(point.max_element() <= 0.7);
            assert!(point.min_element() >= -0.7);
            assert!((point.abs().max_element() - 0.7).abs() < 1e-5);
        }
    }
}
