use std::io::{self, Write};

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
    let source = Cloud::uniform_cube(1024, &mut source_rng);
    let target = Cloud::gaussian_sphere(1024, &mut target_rng);
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
