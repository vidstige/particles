use std::{
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec2, Vec3, Vec4};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    depth_field::DepthField,
    downscaled::Downscaled,
    env::{fps, resolution, DEFAULT_RESOLUTION},
    field::Field,
    fluid::{advect, project_incompressible},
    glow::Glow,
    glitter::{
        glitter_colors, glitter_normals, rotate_normals, tumble_rotation, view_direction, Glitter,
    },
    projection::project_cloud,
    render::Render,
    resolution::Resolution,
    rng::Rng,
    simplex::SimplexNoise,
};

const DURATION: f32 = 30.0;
const FIELD_RESOLUTION: Resolution = Resolution::new(220, 120);
const FIELD_SIZE: Vec2 = Vec2::new(5.5, 3.0);
const PRESSURE_ITERATIONS: usize = 200;
const PARTICLE_COUNT: usize = 16 * 1024;
const MEAN_SPEED: f32 = 0.6;
const Z_SPREAD: f32 = 1.5;
// Fractional speed remaining after 1 second: 0.7 means 70% after 1s, ~10% after 7s
const VISCOUS_DECAY_PER_SECOND: f32 = 0.85;

fn wrap(value: f32, size: f32) -> f32 {
    value.rem_euclid(size)
}

fn wrap_point(point: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(wrap(point.x, size.x), wrap(point.y, size.y))
}

fn initial_field(resolution: Resolution, size: Vec2) -> Field<Vec2> {
    let width = resolution.width as usize;
    let height = resolution.height as usize;
    let mut field = Field::new(resolution, size, Vec2::ZERO);
    let x_noise = SimplexNoise::new(0xdead_beef, 1.8, 1.0);
    let y_noise = SimplexNoise::new(0xcafe_babe, 1.8, 1.0);
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

struct GelScene {
    field: Field<Vec2>,
    positions: Vec<Vec3>,
}

impl GelScene {
    fn new() -> Self {
        let mut rng = Rng::new(0xabcd_ef01);
        let mut field = initial_field(FIELD_RESOLUTION, FIELD_SIZE);
        project_incompressible(&mut field, PRESSURE_ITERATIONS);
        field *= MEAN_SPEED / field.mean_length();

        let positions = (0..PARTICLE_COUNT)
            .map(|_| {
                Vec3::new(
                    rng.next_f32_in(0.0, FIELD_SIZE.x),
                    rng.next_f32_in(0.0, FIELD_SIZE.y),
                    rng.next_f32_in(-Z_SPREAD, Z_SPREAD),
                )
            })
            .collect();

        Self { field, positions }
    }

    fn advance(&mut self, dt: f32) {
        // Evolve the velocity field (semi-Lagrangian advection)
        self.field = advect(&self.field, dt);
        project_incompressible(&mut self.field, 40);
        // Viscous decay: the gel "sets" over time, naturally freezing the pattern
        self.field *= VISCOUS_DECAY_PER_SECOND.powf(dt);

        for position in &mut self.positions {
            let xy = Vec2::new(position.x, position.y);
            let new_xy = wrap_point(xy + self.field.interpolate(xy) * dt, self.field.size());
            position.x = new_xy.x;
            position.y = new_xy.y;
        }
    }

    fn cloud(&self) -> Vec<Vec3> {
        let offset = self.field.size() * 0.5;
        self.positions
            .iter()
            .map(|p| {
                let xy = Vec2::new(p.x, p.y) - offset;
                Vec3::new(xy.x, p.z, xy.y)
            })
            .collect()
    }
}

fn camera_eye() -> Vec3 {
    Vec3::new(0.0, 3.5, 0.5)
}

fn view() -> Mat4 {
    Mat4::look_at_rh(camera_eye(), Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0))
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let fps = fps()?;
    let dt = 1.0 / fps;
    let resolution = resolution()?;
    let mut bitmap = Bitmap::new(resolution.clone());

    let background = Rgba8::from_rgb(5, 4, 16);
    let base_color = Color::from_rgb8(70, 25, 145);

    let camera = camera_eye();
    let depth_field = DepthField {
        focus_depth: camera.length(),
        blur: 4.0,
        particle_radius: 0.8 * resolution.area_scale(&DEFAULT_RESOLUTION),
    };

    let glow = Downscaled {
        inner: Glow {
            softener: 0.3,
            radius: 4.0,
        },
        scale: 4,
    };

    let glitter = Glitter {
        falloff_power: 24.0,
        tumble_speed: 1.5,
        tumble_axis: Vec3::new(0.4, 1.0, 0.3),
        precession_axis: Vec3::new(0.2, 0.4, 1.0),
        precession_speed: 0.5,
    };

    let view = view();
    let projection = projection(&resolution);
    let vdir = view_direction(view);

    let mut rng = Rng::new(0x9988_7766);
    let normals = glitter_normals(&mut rng, PARTICLE_COUNT);

    let mut scene = GelScene::new();

    // Advance to inspection time (mid-gel, swirls formed but still slowing)
    let inspect_time = 12.0;
    let inspect_frames = (inspect_time * fps) as usize;
    for _ in 0..inspect_frames {
        scene.advance(dt);
    }

    let time = inspect_time;
    let rotation = tumble_rotation(time, glitter);
    let rotated = rotate_normals(&normals, rotation);
    let colors = glitter_colors(base_color, &rotated, vdir, glitter);

    bitmap.fill(background);
    let positions = scene.cloud();
    let projected = project_cloud(&bitmap, &positions, projection, view);
    glow.render(&mut bitmap, &projected, &colors);
    depth_field.render(&mut bitmap, &projected, &colors);

    output.write_all(bitmap.data())?;

    Ok(())
}
