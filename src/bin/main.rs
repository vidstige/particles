use std::{
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec3};
use particles::{
    bitmap::Bitmap,
    color::Color,
    color::Rgba8,
    depth_field::{DepthField, Render, Theme},
    distribution::{collect, Uniform3},
    env::{resolution, DEFAULT_RESOLUTION},
    glitter::{glitter_colors, glitter_particles, view_direction, Glitter},
    projection::project_cloud,
    resolution::Resolution,
    rng::Rng,
    simplex::SimplexNoise,
};

const CAMERA_ANGULAR_VELOCITY: f32 = 0.125;
const GLITTER_TUMBLE_SPEED: f32 = 8.0;

fn simplex_field() -> [SimplexNoise; 3] {
    [
        SimplexNoise::new(0x1f2e_3d4c, 1.4, 1.0),
        SimplexNoise::new(0x2a39_4857, 1.4, 1.0),
        SimplexNoise::new(0x6574_8392, 1.4, 1.0),
    ]
}

fn simplex_offset(field: &[SimplexNoise; 3], point: Vec3, w: f32) -> Vec3 {
    Vec3::new(
        field[0].sample(point.extend(w)),
        field[1].sample(point.extend(w)),
        field[2].sample(point.extend(w)),
    )
}

fn simplex_view(angle: f32) -> Mat4 {
    let radius = 2.0_f32.sqrt();
    let eye = Vec3::new(radius * angle.cos(), 2.0, radius * angle.sin());
    Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y)
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let resolution = resolution()?;
    let theme = Theme {
        background: Rgba8::from_rgb(14, 14, 18),
        foreground: Color::from_rgb8(112, 48, 132),
    };
    let fps = 30.0;
    let duration = 24.0;
    let simplex_speed = 0.125;
    let frame_count = (duration * fps) as usize;
    let mut rng = Rng::new(0x1234_5678);
    let n = 8 * 1024;
    let rest_positions = collect(&mut Uniform3::new(), n, &mut rng);
    let field = simplex_field();
    let base_colors = vec![theme.foreground; n];
    let glitter_params = glitter_particles(&mut rng, n);
    let glitter = Glitter {
        falloff_power: 16.0,
        tumble_speed: GLITTER_TUMBLE_SPEED,
    };
    let projection = projection(&resolution);
    let depth_field = DepthField {
        focus_depth: 2.0,
        blur: 2.0,
        particle_radius: resolution.area_scale(&DEFAULT_RESOLUTION),
    };
    let mut bitmap = Bitmap::new(resolution);

    for frame in 0..frame_count {
        bitmap.fill(theme.background);
        let time = frame as f32 / fps;
        let view = simplex_view(time * CAMERA_ANGULAR_VELOCITY);
        let view_direction = view_direction(view);
        let w = time * simplex_speed;
        let positions = rest_positions
            .iter()
            .map(|rest_position| *rest_position + simplex_offset(&field, *rest_position, w) * 0.45)
            .collect::<Vec<_>>();
        let projected = project_cloud(&bitmap, &positions, projection, view);
        let colors = glitter_colors(&base_colors, &glitter_params, view_direction, glitter, time);
        depth_field.render(&mut bitmap, &projected, &colors);
        output.write_all(bitmap.data())?;
        output.flush()?;
    }

    Ok(())
}
