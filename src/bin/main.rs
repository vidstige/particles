use std::io::{self, Write};

use glam::{Mat4, Vec3};
use particles::{
    assignment::match_clouds,
    cloud::Cloud,
    render::render_cloud,
    resolution::Resolution,
    rng::Rng,
};

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let resolution = Resolution::new(720, 720);
    let mut source_rng = Rng::new(0x1234_5678);
    let mut target_rng = Rng::new(0x8765_4321);
    let n = 512;
    let source = uniform_cube(n, &mut source_rng);
    let target = gaussian_sphere(n, &mut target_rng);
    let target = match_clouds(&source, &target);
    let radius = max_radius(&[&source, &target]).max(1.0);
    let eye = Vec3::new(0.0, 0.0, radius * 3.5);
    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
    let projection = Mat4::perspective_rh_gl(
        50.0_f32.to_radians(),
        resolution.aspect_ratio(),
        0.1,
        eye.z + radius * 2.0,
    );

    for frame in 0..90 {
        let t = frame as f32 / 89.0;
        let cloud = Cloud {
            positions: source
                .positions
                .iter()
                .zip(&target.positions)
                .map(|(from, to)| from.lerp(*to, t))
                .collect(),
        };
        let pixmap = render_cloud(&cloud, &resolution, projection, view);
        output.write_all(pixmap.data())?;
    }

    output.flush()?;
    Ok(())
}

fn uniform_cube(count: usize, rng: &mut Rng) -> Cloud {
    let positions = (0..count)
        .map(|_| Vec3::new(rng.next_f32(), rng.next_f32(), rng.next_f32()) * 2.0 - Vec3::ONE)
        .collect();
    Cloud { positions }
}

fn gaussian_sphere(count: usize, rng: &mut Rng) -> Cloud {
    let positions = (0..count)
        .map(|_| {
            Vec3::new(
                rng.next_gaussian(),
                rng.next_gaussian(),
                rng.next_gaussian(),
            ) * 0.35
        })
        .collect();
    Cloud { positions }
}

fn max_radius(clouds: &[&Cloud]) -> f32 {
    clouds
        .iter()
        .flat_map(|cloud| cloud.positions.iter())
        .map(|point| point.length())
        .fold(0.0, f32::max)
}
