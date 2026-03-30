use std::{
    env,
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec3};
use particles::{
    color::Color,
    distribution::{collect, Gaussian},
    glitter::{glitter_colors, glitter_particles, Glitter},
    projection::project_cloud,
    render::{default_theme, render_cloud, DepthField},
    resolution::Resolution,
    rng::Rng,
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

fn gaussian_cloud(point_count: usize, scale: f32) -> Vec<Vec3> {
    collect(
        &mut Gaussian::new(scale),
        point_count,
        &mut Rng::new(0x1234_5678),
    )
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn view(angle: f32, radius: f32) -> Mat4 {
    let eye = Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin());
    Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let resolution = resolution()?;
    let theme = default_theme();
    let seconds_per_frame = 1.0 / 60.0;
    let frame_count = 512;
    let theta = std::f32::consts::TAU / 1024.0;
    let cloud = gaussian_cloud(4096, 0.42);
    let mut rng = Rng::new(0x9abc_def0);
    let glitter_particles = glitter_particles(&mut rng, cloud.len());
    let base_colors = vec![Color::from_tiny_color(theme.foreground); cloud.len()];
    let glitter = Glitter {
        glint_time: 0.2,
        pause_time: 1.0,
    };
    let radius = 4.0;
    let projection = projection(&resolution);
    let depth_field = DepthField {
        focus_depth: radius,
        blur_depth: 1.0,
    };

    for frame in 0..frame_count {
        let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
        pixmap.fill(theme.background);
        let angle = frame as f32 * theta;
        let time = frame as f32 * seconds_per_frame;
        let colors = glitter_colors(&base_colors, &glitter_particles, time, glitter);
        let positions = project_cloud(&pixmap, &cloud, projection, view(angle, radius));
        render_cloud(&mut pixmap, &positions, &colors, depth_field);
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
