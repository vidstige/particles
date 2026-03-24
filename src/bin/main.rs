use std::io::{self, Write};

use particles::{cloud::Cloud, resolution::Resolution, rng::Rng};
use tiny_skia::Pixmap;

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let resolution = Resolution::new(720, 720);

    let pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();

    let n = 1024;
    let mut rng = Rng::new(0x1234_5678);
    let _cloud = Cloud::uniform_cube(n, &mut rng);

    output.write_all(pixmap.data())?;
    output.flush()?;
    Ok(())
}
