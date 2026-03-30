use std::{error::Error, io::Write};

use glam::{Mat4, Vec3};
use tiny_skia::Pixmap;

use crate::{
    assignment::auction_assignment,
    color::Color,
    cubic_hermite3::CubicHermite3,
    distribution::{
        collect, Add, Cube, Distribution3, Gaussian, Gyroid, Icosahedron, Lissajous, Sphere,
        Tetrahedron, TorusSurface, UniformCube,
    },
    render::{render_cloud, Theme},
    resolution::Resolution,
    rng::Rng,
};

fn cost_matrix(source: &[Vec3], target: &[Vec3]) -> Vec<f32> {
    source
        .iter()
        .flat_map(|from| target.iter().map(move |to| from.distance_squared(*to)))
        .collect()
}

fn interpolate_cloud(curves: &[CubicHermite3], t: f32) -> Vec<Vec3> {
    curves.iter().map(|curve| curve.sample(t)).collect()
}

fn linger(t: f32, power: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let toward_start = t.powf(power);
    let toward_end = (1.0 - t).powf(power);
    toward_start / (toward_start + toward_end)
}

fn tangents(clouds: &[Vec<Vec3>], index: usize) -> Vec<Vec3> {
    let current = &clouds[index];

    match (
        index.checked_sub(1).map(|prev| &clouds[prev]),
        clouds.get(index + 1),
    ) {
        (Some(previous), Some(next)) => previous
            .iter()
            .zip(next)
            .map(|(previous, next)| (*next - *previous) * 0.5)
            .collect(),
        (None, Some(next)) => current
            .iter()
            .zip(next)
            .map(|(current, next)| *next - *current)
            .collect(),
        (Some(previous), None) => previous
            .iter()
            .zip(current)
            .map(|(previous, current)| *current - *previous)
            .collect(),
        (None, None) => vec![Vec3::ZERO; current.len()],
    }
}

fn curves(
    source: &[Vec3],
    source_tangents: &[Vec3],
    target: &[Vec3],
    target_tangents: &[Vec3],
) -> Vec<CubicHermite3> {
    source
        .iter()
        .zip(source_tangents)
        .zip(target)
        .zip(target_tangents)
        .map(|(((source, source_tangent), target), target_tangent)| {
            CubicHermite3::new(*source, *source_tangent, *target, *target_tangent)
        })
        .collect()
}

fn max_radius(clouds: &[Vec<Vec3>]) -> f32 {
    clouds
        .iter()
        .flat_map(|cloud| cloud.iter())
        .map(|point| point.length())
        .fold(0.0, f32::max)
}

fn match_positions(source: &[Vec3], target: &[Vec3], epsilon: f32) -> Vec<Vec3> {
    assert_eq!(source.len(), target.len());

    let assignment = auction_assignment(&cost_matrix(source, target), source.len(), epsilon);
    assignment.into_iter().map(|index| target[index]).collect()
}

fn noisy<D: Distribution3>(distribution: D, scale: f32) -> Add<D, Gaussian> {
    Add::new(distribution, Gaussian::new(scale))
}

fn clouds(rng: &mut Rng, point_count: usize, noise_scale: f32) -> Vec<Vec<Vec3>> {
    vec![
        collect(
            &mut noisy(UniformCube::new(), noise_scale),
            point_count,
            rng,
        ),
        collect(&mut noisy(Cube::new(0.9), noise_scale), point_count, rng),
        collect(&mut noisy(Sphere::new(0.95), noise_scale), point_count, rng),
        collect(
            &mut noisy(Tetrahedron::new(0.95), noise_scale),
            point_count,
            rng,
        ),
        collect(
            &mut noisy(TorusSurface::new(0.75, 0.25), noise_scale),
            point_count,
            rng,
        ),
        collect(
            &mut noisy(Icosahedron::new(0.95), noise_scale),
            point_count,
            rng,
        ),
        collect(
            &mut noisy(Lissajous::new(point_count, 0.9), noise_scale),
            point_count,
            rng,
        ),
        collect(
            &mut noisy(Gyroid::new(1.1, 0.08), noise_scale),
            point_count,
            rng,
        ),
        collect(
            &mut noisy(Gaussian::new(0.35), noise_scale),
            point_count,
            rng,
        ),
    ]
}

fn camera(clouds: &[Vec<Vec3>], resolution: &Resolution) -> (Mat4, Mat4) {
    let radius = max_radius(clouds).max(1.0);
    let eye = Vec3::new(0.0, 0.0, radius * 2.0);
    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
    let projection = Mat4::perspective_rh_gl(
        50.0_f32.to_radians(),
        resolution.aspect_ratio(),
        0.1,
        eye.z + radius * 2.0,
    );
    (projection, view)
}

pub fn render(
    output: &mut impl Write,
    resolution: &Resolution,
    theme: &Theme,
) -> Result<(), Box<dyn Error>> {
    let mut rng = Rng::new(0x1234_5678);
    let point_count = 1024;
    let noise_scale = 0.03;
    let epsilon = 0.1;
    let segment_frames = 32;
    let linger_power = 2.5;
    let mut clouds = clouds(&mut rng, point_count, noise_scale);
    let base_colors = vec![Color::from_tiny_color(theme.foreground); point_count];

    for index in 1..clouds.len() {
        clouds[index] = match_positions(&clouds[index - 1], &clouds[index], epsilon);
    }
    let tangents: Vec<_> = (0..clouds.len())
        .map(|index| tangents(&clouds, index))
        .collect();
    let (projection, view) = camera(&clouds, resolution);

    for (index, pair) in clouds.windows(2).enumerate() {
        let source = &pair[0];
        let target = &pair[1];
        let segment = curves(source, &tangents[index], target, &tangents[index + 1]);

        for frame in 0..segment_frames {
            let phase = frame as f32 / segment_frames as f32;
            let t = linger(phase, linger_power);
            let cloud = interpolate_cloud(&segment, t);
            let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
            pixmap.fill(theme.background);
            render_cloud(&mut pixmap, &cloud, &base_colors, projection, view);
            output.write_all(pixmap.data())?;
            output.flush()?;
        }
    }

    Ok(())
}
