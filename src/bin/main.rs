use std::{
    env,
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec2, Vec3};
use particles::{
    color::Color,
    gerstner::{surface_grid, update_positions, GerstnerWave},
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

fn water_surface(columns: usize, rows: usize, size: Vec2) -> Vec<Vec3> {
    surface_grid(columns, rows, size)
}

fn water_waves() -> [GerstnerWave; 5] {
    [
        GerstnerWave::new(Vec2::new(1.0, 0.1), 0.11, 2.8, 0.55, 0.75, 0.0),
        GerstnerWave::new(Vec2::new(0.2, 1.0), 0.08, 1.9, 0.8, 0.7, 0.8),
        GerstnerWave::new(Vec2::new(-0.9, 0.4), 0.05, 1.1, 1.1, 0.55, 1.7),
        GerstnerWave::new(Vec2::new(0.7, -0.6), 0.04, 0.75, 1.4, 0.45, 2.2),
        GerstnerWave::new(Vec2::new(-0.3, -1.0), 0.03, 0.5, 1.8, 0.35, 0.5),
    ]
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn view(angle: f32, radius: f32, height: f32) -> Mat4 {
    let eye = Vec3::new(radius * angle.cos(), height, radius * angle.sin());
    Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let resolution = resolution()?;
    let theme = default_theme();
    let seconds_per_frame = 1.0 / 60.0;
    let frame_count = 512;
    let theta = std::f32::consts::TAU / 2048.0;
    let rest_positions = water_surface(64, 64, Vec2::new(8.0, 8.0));
    let mut positions = vec![Vec3::ZERO; rest_positions.len()];
    let waves = water_waves();
    let mut rng = Rng::new(0x9abc_def0);
    let glitter_particles = glitter_particles(&mut rng, rest_positions.len());
    let base_colors = vec![Color::from_tiny_color(theme.foreground); rest_positions.len()];
    let glitter = Glitter {
        glint_time: 0.2,
        pause_time: 1.0,
    };
    let radius = 4.0;
    let height = 1.5;
    let projection = projection(&resolution);
    let focus_depth = Vec2::new(radius, height).length();
    let depth_field = DepthField {
        focus_depth,
        blur: 8.0,
    };

    for frame in 0..frame_count {
        let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
        pixmap.fill(theme.background);
        let angle = frame as f32 * theta;
        let time = frame as f32 * seconds_per_frame;
        update_positions(&mut positions, &rest_positions, &waves, time);
        let colors: Vec<_> = glitter_colors(&base_colors, &glitter_particles, time, glitter)
            .into_iter()
            .map(|color| {
                tiny_skia::Color::from_rgba8(
                    color.red.clamp(0.0, 255.0) as u8,
                    color.green.clamp(0.0, 255.0) as u8,
                    color.blue.clamp(0.0, 255.0) as u8,
                    255,
                )
            })
            .collect();
        let projected = project_cloud(&pixmap, &positions, projection, view(angle, radius, height));
        render_cloud(&mut pixmap, &projected, &colors, depth_field);
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
