use std::io::{self, Write};

use glam::Vec3;
use particles::{resolution::Resolution, rng::Rng};
use tiny_skia::Pixmap;

struct Cloud {
    positions: Vec<Vec3>,
}

fn sample_vec3(rng: &mut Rng, center: Vec3, size: Vec3) -> Vec3 {
    center + Vec3::new(rng.next_f32(), rng.next_f32(), rng.next_f32()) * size
}

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let resolution = Resolution::new(720, 720);

    let pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();

    let n = 1024;
    let mut rng = Rng::new(0x1234_5678);
    let _cloud = Cloud {
        positions: (0..n)
            .map(|_| sample_vec3(&mut rng, Vec3::ZERO, Vec3::ONE))
            .collect(),
    };

    output.write_all(pixmap.data())?;
    output.flush()?;
    Ok(())
}
