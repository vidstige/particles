use std::{
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec2, Vec3};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    depth_field::DepthField,
    env::{fps, resolution, DEFAULT_RESOLUTION},
    gerstner::{displaced_position, surface_grid, GerstnerWave},
    glitter::{glitter_colors, glitter_normals, rotate_normals, tumble_rotation, view_direction, Glitter},
    projection::project_cloud,
    render::Render,
    resolution::Resolution,
    rng::Rng,
};

const GRID_SIZE: Vec2 = Vec2::new(8.0, 8.0);
const GERSTNER_SPEED: f32 = 0.12;
const DURATION: f32 = 30.0;

fn waves() -> [GerstnerWave; 5] {
    [
        GerstnerWave::new(Vec2::new( 1.0,  0.1), 0.11, 2.8, 0.55, 0.75, 0.0),
        GerstnerWave::new(Vec2::new( 0.2,  1.0), 0.08, 1.9, 0.80, 0.70, 0.8),
        GerstnerWave::new(Vec2::new(-0.9,  0.4), 0.05, 1.1, 1.10, 0.55, 1.7),
        GerstnerWave::new(Vec2::new( 0.7, -0.6), 0.04, 0.75, 1.4, 0.45, 2.2),
        GerstnerWave::new(Vec2::new(-0.3, -1.0), 0.03, 0.5, 1.80, 0.35, 0.5),
    ]
}

fn view() -> Mat4 {
    Mat4::look_at_rh(
        Vec3::new(3.19, 2.11, 4.05),
        Vec3::new(0.67, -1.37, 0.54),
        Vec3::Y,
    )
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 20.0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let fps = fps()?;
    let resolution = resolution()?;
    let mut bitmap = Bitmap::new(resolution.clone());

    let background = Rgba8::from_rgb(14, 14, 18);
    let foreground = Color::from_hex("#703084");

    let depth_field = DepthField {
        focus_depth: 3.5,
        blur: 2.9,
        particle_radius: 0.9 * resolution.area_scale(&DEFAULT_RESOLUTION),
    };

    let glitter = Glitter {
        falloff_power: 19.2,
        tumble_speed: 1.0,
        tumble_axis: Vec3::new(0.4, 0.8, 0.2).normalize(),
        precession_axis: Vec3::new(-0.3, 0.1, 0.9).normalize(),
        precession_speed: 0.75,
    };

    let view = view();
    let projection = projection(&resolution);
    let vdir = view_direction(view);

    let mut rng = Rng::new(0x9988_7766);
    let normals = glitter_normals(&mut rng, 128 * 128);
    let rest_positions = surface_grid(128, 128, GRID_SIZE);
    let waves = waves();

    for frame in 0..(DURATION * fps) as usize {
        let time = frame as f32 / fps;

        let positions: Vec<Vec3> = rest_positions
            .iter()
            .map(|&rest| displaced_position(rest, &waves, time * GERSTNER_SPEED))
            .collect();

        let rotation = tumble_rotation(time, glitter);
        let rotated = rotate_normals(&normals, rotation);
        let colors = glitter_colors(&vec![foreground; positions.len()], &rotated, vdir, glitter);

        bitmap.fill(background);
        let projected = project_cloud(&bitmap, &positions, projection, view);
        depth_field.render(&mut bitmap, &projected, &colors);

        output.write_all(bitmap.data())?;
        output.flush()?;
    }

    Ok(())
}
