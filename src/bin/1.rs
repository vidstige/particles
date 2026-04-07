use std::{
    error::Error,
    io::{self, Write},
};

use glam::Mat4;
use particles::{
    bitmap::Bitmap,
    color::Color,
    color::Rgba8,
    env::resolution,
    projection::project_cloud,
    render::{render_cloud, DepthField, Theme},
    resolution::Resolution,
    timeline::Timeline,
};

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let resolution = resolution()?;
    let theme = Theme {
        background: Rgba8::from_rgb(14, 14, 18),
        foreground: Color::from_rgb8(214, 92, 255),
    };
    let fps = 30.0;
    let duration = 44.0;
    let frame_count = (duration * fps) as usize;
    let timeline = Timeline::new();
    let colors = vec![theme.foreground; timeline.particle_count()];
    let projection = projection(&resolution);
    let depth_field = DepthField {
        focus_depth: 4.0,
        blur: 8.0,
    };
    let mut bitmap = Bitmap::new(resolution.width, resolution.height);

    for frame in 0..frame_count {
        bitmap.fill(theme.background);
        let time = frame as f32 / fps;
        let positions = timeline.particles(time);
        let view = timeline.view(time);
        let projected = project_cloud(&resolution, &positions, projection, view);
        render_cloud(&mut bitmap, &projected, &colors, depth_field);
        output.write_all(bitmap.data())?;
        output.flush()?;
    }

    Ok(())
}
