use std::{
    env,
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec3};
use particles::{
    distribution::{collect, Uniform3},
    projection::project_cloud,
    render::{render_cloud, DepthField, Theme},
    resolution::Resolution,
    rng::Rng,
    simplex::SimplexNoise,
};
use tiny_skia::{Color, Pixmap};

const DEFAULT_RESOLUTION: Resolution = Resolution::new(512, 288);

fn simplex_field() -> [SimplexNoise; 3] {
    [
        SimplexNoise::new(0x1f2e_3d4c, 1.4, 1.0),
        SimplexNoise::new(0x2a39_4857, 1.4, 1.0),
        SimplexNoise::new(0x6574_8392, 1.4, 1.0),
    ]
}

fn simplex_offset(field: &[SimplexNoise; 3], point: Vec3, time: f32) -> Vec3 {
    Vec3::new(
        field[0].sample(point.extend(time)),
        field[1].sample(point.extend(time + 7.0)),
        field[2].sample(point.extend(time + 13.0)),
    )
}

fn simplex_view() -> Mat4 {
    let eye = Vec3::new(2.0, 2.0, 2.0);
    let center = Vec3::ZERO;
    Mat4::look_at_rh(eye, center, Vec3::Y)
}

fn resolution() -> Result<Resolution, Box<dyn Error>> {
    let resolution = match env::var("RESOLUTION") {
        Ok(value) => value.parse::<Resolution>()?,
        Err(env::VarError::NotPresent) => DEFAULT_RESOLUTION,
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let resolution = resolution()?;
    let theme = Theme {
        background: Color::from_rgba8(14, 14, 18, 255),
        foreground: Color::from_rgba8(214, 92, 255, 255),
    };
    let fps = 30.0;
    let duration = 12.0;
    let frame_count = (duration * fps) as usize;
    let mut rng = Rng::new(0x1234_5678);
    let n = 1024;
    let rest_positions = collect(&mut Uniform3::new(), n, &mut rng);
    let field = simplex_field();
    let colors = vec![theme.foreground; n];
    let projection = projection(&resolution);
    let view = simplex_view();
    let depth_field = DepthField {
        focus_depth: 2.0,
        blur: 1.0,
    };
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();

    for frame in 0..frame_count {
        pixmap.fill(theme.background);
        let time = frame as f32 / fps;
        let positions = rest_positions
            .iter()
            .map(|rest_position| *rest_position + simplex_offset(&field, *rest_position, time) * 0.45)
            .collect::<Vec<_>>();
        let projected = project_cloud(&pixmap, &positions, projection, view);
        render_cloud(&mut pixmap, &projected, &colors, depth_field);
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
