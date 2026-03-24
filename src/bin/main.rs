use std::io::{self, Write};

use particles::resolution::Resolution;
use tiny_skia::Pixmap;


fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let resolution = Resolution::new(720, 720);

    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    output.write_all(pixmap.data())?;
    output.flush()?;
    Ok(())
}
