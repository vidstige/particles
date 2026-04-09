use std::{
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec2, Vec3, Vec4};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    depth_field::{DepthField, Render, Theme},
    env::{fps, resolution, DEFAULT_RESOLUTION},
    field::{project_incompressible, Field},
    projection::project_cloud,
    resolution::Resolution,
    rng::Rng,
    simplex::SimplexNoise,
};

const DURATION: f32 = 24.0;
const FIELD_RESOLUTION: Resolution = Resolution::new(128, 128);
const FIELD_SIZE: Vec2 = Vec2::new(3.2, 3.2);
const PRESSURE_ITERATIONS: usize = 160;
const PARTICLE_COUNT: usize = 8 * 1024;
const MEAN_SPEED: f32 = 0.35;

fn wrap(value: f32, size: f32) -> f32 {
    value.rem_euclid(size)
}

fn wrap_point(point: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(wrap(point.x, size.x), wrap(point.y, size.y))
}

fn from_simplex(resolution: Resolution, size: Vec2) -> Field<Vec2> {
    let width = resolution.width as usize;
    let height = resolution.height as usize;
    let mut field = Field::new(resolution, size, Vec2::ZERO);
    let x_noise = SimplexNoise::new(0x1f2e_3d4c, 1.3, 1.0);
    let y_noise = SimplexNoise::new(0x5a69_7887, 1.3, 1.0);

    for y in 0..height {
        for x in 0..width {
            let point = field.sample(x, y) / size;
            field.set(
                x,
                y,
                Vec2::new(
                    x_noise.sample(Vec4::new(point.x, point.y, 0.17, 0.0)),
                    y_noise.sample(Vec4::new(point.x, point.y, 3.41, 0.0)),
                ),
            );
        }
    }

    field
}

struct SwirlScene {
    field: Field<Vec2>,
    positions: Vec<Vec2>,
}

impl SwirlScene {
    fn new() -> Self {
        let mut rng = Rng::new(0x1234_5678);
        let mut field = from_simplex(FIELD_RESOLUTION, FIELD_SIZE);
        project_incompressible(&mut field, PRESSURE_ITERATIONS);
        field *= MEAN_SPEED / field.mean_length();
        let positions = (0..PARTICLE_COUNT)
            .map(|_| {
                Vec2::new(
                    rng.next_f32_in(0.0, FIELD_SIZE.x),
                    rng.next_f32_in(0.0, FIELD_SIZE.y),
                )
            })
            .collect();

        Self { field, positions }
    }

    fn advance(&mut self, dt: f32) {
        for position in &mut self.positions {
            *position = wrap_point(
                *position + self.field.interpolate(*position) * dt,
                self.field.size(),
            );
        }
    }

    fn cloud(&self) -> Vec<Vec3> {
        let offset = self.field.size() * 0.5;
        self.positions
            .iter()
            .map(|position| {
                let position = *position - offset;
                Vec3::new(position.x, 0.0, position.y)
            })
            .collect()
    }
}

fn camera_eye() -> Vec3 {
    Vec3::new(0.0, 2.35, 2.2)
}

fn view() -> Mat4 {
    Mat4::look_at_rh(camera_eye(), Vec3::ZERO, Vec3::Y)
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn theme() -> Theme {
    Theme {
        background: Rgba8::from_rgb(10, 12, 18),
        foreground: Color::from_rgb8(242, 208, 92),
    }
}

fn depth_field(resolution: &Resolution) -> DepthField {
    DepthField {
        focus_depth: camera_eye().length(),
        blur: 1.1,
        particle_radius: 0.75 * resolution.area_scale(&DEFAULT_RESOLUTION),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let fps = fps()?;
    let dt = 1.0 / fps;
    let resolution = resolution()?;
    let mut bitmap = Bitmap::new(resolution);
    let theme = theme();
    let depth_field = depth_field(bitmap.resolution());
    let view = view();
    let projection = projection(bitmap.resolution());
    let colors = vec![theme.foreground; PARTICLE_COUNT];
    let frame_count = (DURATION * fps) as usize;
    let mut scene = SwirlScene::new();

    for _ in 0..frame_count {
        bitmap.fill(theme.background);
        let positions = scene.cloud();
        let projected = project_cloud(&bitmap, &positions, projection, view);
        depth_field.render(&mut bitmap, &projected, &colors);
        output.write_all(bitmap.data())?;
        output.flush()?;
        scene.advance(dt);
    }

    Ok(())
}
