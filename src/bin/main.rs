use std::io::{self, Write};

use glam::Vec3;
use particles::resolution::Resolution;
use tiny_skia::Pixmap;

struct Particle {
    position: Vec3,
    velocity: Vec3,
}

struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        (self.next() as f32) / (u32::MAX as f32)
    }
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
    let _particles: Vec<_> = (0..n)
        .map(|_| {
            Particle {
                position: sample_vec3(&mut rng, Vec3::ZERO, Vec3::ONE),
                velocity: Vec3::ZERO,
            }
        })
        .collect();

    output.write_all(pixmap.data())?;
    output.flush()?;
    Ok(())
}
