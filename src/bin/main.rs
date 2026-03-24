use std::io::{self, Write};

use particles::{cloud::Cloud, render::{render_cloud, View}, resolution::Resolution, rng::Rng};

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let resolution = Resolution::new(720, 720);
    let mut rng = Rng::new(0x1234_5678);
    let cloud = Cloud::gaussian_sphere(1024, &mut rng);
    let view = View::fit(&resolution, &[&cloud]);
    let pixmap = render_cloud(&cloud, &resolution, &view);

    output.write_all(pixmap.data())?;
    output.flush()?;
    Ok(())
}
