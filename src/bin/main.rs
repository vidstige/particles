use std::io::{self, Write};

use glam::{Mat4, UVec3, Vec3};
use particles::{
    assignment::auction_assignment,
    distribution::{
        collect, Cube, Gaussian, Grid3d, Gyroid, Icosahedron, Lissajous, Sphere, Tetrahedron,
        TorusSurface, UniformCube,
    },
    render::{default_theme, render_cloud},
    resolution::Resolution,
    rng::Rng,
};

fn cost_matrix(source: &[Vec3], target: &[Vec3]) -> Vec<f32> {
    source
        .iter()
        .flat_map(|from| target.iter().map(move |to| from.distance_squared(*to)))
        .collect()
}

fn interpolate_cloud(source: &[Vec3], target: &[Vec3], t: f32) -> Vec<Vec3> {
    source
        .iter()
        .zip(target)
        .map(|(from, to)| from.lerp(*to, t))
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

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let resolution = Resolution::new(720, 720);
    let mut rng = Rng::new(0x1234_5678);
    let point_count = 512;
    let epsilon = 0.01;
    let frame_count = 32;
    let theme = default_theme();
    let mut clouds = vec![
        collect(&mut UniformCube::new(), point_count, &mut rng),
        collect(&mut Cube::new(0.9), point_count, &mut rng),
        collect(
            &mut Grid3d::new(UVec3::splat(8), Vec3::splat(1.26)),
            point_count,
            &mut rng,
        ),
        collect(&mut Sphere::new(0.95), point_count, &mut rng),
        collect(&mut Tetrahedron::new(0.95), point_count, &mut rng),
        collect(&mut TorusSurface::new(0.75, 0.25), point_count, &mut rng),
        collect(&mut Icosahedron::new(0.95), point_count, &mut rng),
        collect(&mut Lissajous::new(point_count, 0.9), point_count, &mut rng),
        collect(&mut Gyroid::new(1.1, 0.08), point_count, &mut rng),
        collect(&mut Gaussian::new(0.35), point_count, &mut rng),
    ];

    for index in 1..clouds.len() {
        clouds[index] = match_positions(&clouds[index - 1], &clouds[index], epsilon);
    }

    let radius = max_radius(&clouds).max(1.0);
    let eye = Vec3::new(0.0, 0.0, radius * 3.5);
    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
    let projection = Mat4::perspective_rh_gl(
        50.0_f32.to_radians(),
        resolution.aspect_ratio(),
        0.1,
        eye.z + radius * 2.0,
    );

    for (index, pair) in clouds.windows(2).enumerate() {
        let source = &pair[0];
        let target = &pair[1];

        for frame in usize::from(index > 0)..frame_count {
            let t = frame as f32 / (frame_count as f32 - 1.0);
            let cloud = interpolate_cloud(source, target, t);
            let pixmap = render_cloud(&cloud, &resolution, projection, view, &theme);
            output.write_all(pixmap.data())?;
            output.flush()?;
        }
    }

    Ok(())
}
