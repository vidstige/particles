use std::io::{self, Write};

use glam::Vec3;
use particles::{
    assignment::match_clouds,
    cloud::Cloud,
    render::{render_cloud, View},
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
    let view = View::fit(&resolution, &[&source, &target]);

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
        let pixmap = render_cloud(&cloud, &resolution, &view);
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
