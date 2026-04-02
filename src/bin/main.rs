use std::{
    env,
    error::Error,
    io::{self, Write},
};

use glam::Mat4;
use particles::{
    projection::project_cloud,
    render::{default_theme, render_cloud, Theme},
    resolution::Resolution,
    timeline::Timeline,
};
use tiny_skia::Pixmap;

fn default_resolution() -> Resolution {
    Resolution::new(512, 288)
}

fn resolution() -> Result<Resolution, Box<dyn Error>> {
    let resolution = match env::var("RESOLUTION") {
        Ok(value) => value.parse::<Resolution>()?,
        Err(env::VarError::NotPresent) => default_resolution(),
        Err(error) => return Err(error.into()),
    };

    if resolution.area() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "RESOLUTION must have non-zero area",
        )
        .into());
    }

    Ok(resolution)
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn render(
    output: &mut impl Write,
    resolution: &Resolution,
    theme: &Theme,
) -> Result<(), Box<dyn Error>> {
    let seconds_per_frame = 1.0 / 60.0;
    let frame_count = 512;
    let timeline = Timeline::new();
    let colors = vec![theme.foreground; timeline.particle_count()];
    let projection = projection(resolution);
    let depth_field = timeline.depth_field();

    for frame in 0..frame_count {
        let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
        pixmap.fill(theme.background);
        let time = frame as f32 * seconds_per_frame;
        let positions = timeline.particles(time);
        let projected = project_cloud(&pixmap, &positions, projection, timeline.view(time));
        render_cloud(&mut pixmap, &projected, &colors, depth_field);
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let resolution = resolution()?;
    let theme = default_theme();
    render(&mut output, &resolution, &theme)
}
