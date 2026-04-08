use glam::{Mat4, Vec3};

use crate::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    depth_field::{DepthField, Render, Theme},
    distribution::{collect, Uniform3},
    env::DEFAULT_RESOLUTION,
    glitter::{
        glitter_colors, glitter_normals, rotate_normals, tumble_rotation, view_direction, Glitter,
    },
    projection::project_cloud,
    resolution::Resolution,
    rng::Rng,
    simplex::SimplexNoise,
};

pub const DEFAULT_DURATION: f32 = 24.0;
pub const DEFAULT_FPS: f32 = 30.0;

const CAMERA_ANGULAR_VELOCITY: f32 = 0.125;
const DEFAULT_PARTICLE_COUNT: usize = 8 * 1024;
const DEFAULT_SIMPLEX_SCALE: f32 = 0.45;
const DEFAULT_SIMPLEX_SPEED: f32 = 0.125;
const DEFAULT_GLITTER_TUMBLE_SPEED: f32 = 8.0;
const DEFAULT_GLITTER_PRECESSION_SPEED: f32 = 1.5;

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

pub fn animated_view(time: f32) -> Mat4 {
    let radius = 2.0_f32.sqrt();
    let angle = time * CAMERA_ANGULAR_VELOCITY;
    let eye = Vec3::new(radius * angle.cos(), 2.0, radius * angle.sin());
    Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y)
}

pub fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

#[derive(Clone, Copy, Debug)]
pub struct GlitterSceneSettings {
    pub theme: Theme,
    pub depth_field: DepthField,
    pub glitter: Glitter,
    pub simplex_scale: f32,
    pub simplex_speed: f32,
}

impl GlitterSceneSettings {
    pub fn for_resolution(resolution: &Resolution) -> Self {
        Self {
            theme: Theme {
                background: Rgba8::from_rgb(14, 14, 18),
                foreground: Color::from_rgb8(112, 48, 132),
            },
            depth_field: DepthField {
                focus_depth: 2.0,
                blur: 2.0,
                particle_radius: resolution.area_scale(&DEFAULT_RESOLUTION),
            },
            glitter: Glitter {
                falloff_power: 16.0,
                tumble_speed: DEFAULT_GLITTER_TUMBLE_SPEED,
                tumble_axis: Vec3::new(0.4, 0.8, 0.2).normalize(),
                precession_axis: Vec3::new(-0.3, 0.1, 0.9).normalize(),
                precession_speed: DEFAULT_GLITTER_PRECESSION_SPEED,
            },
            simplex_scale: DEFAULT_SIMPLEX_SCALE,
            simplex_speed: DEFAULT_SIMPLEX_SPEED,
        }
    }

    pub fn glitter_speed(&self) -> f32 {
        self.glitter.tumble_speed
    }

    pub fn set_glitter_speed(&mut self, speed: f32) {
        self.glitter.tumble_speed = speed;
        self.glitter.precession_speed =
            speed * DEFAULT_GLITTER_PRECESSION_SPEED / DEFAULT_GLITTER_TUMBLE_SPEED;
    }
}

pub struct GlitterScene {
    rest_positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    field: [SimplexNoise; 3],
}

impl GlitterScene {
    pub fn new() -> Self {
        let mut rng = Rng::new(0x1234_5678);
        let rest_positions = collect(&mut Uniform3::new(), DEFAULT_PARTICLE_COUNT, &mut rng);
        let normals = glitter_normals(&mut rng, DEFAULT_PARTICLE_COUNT);

        Self {
            rest_positions,
            normals,
            field: simplex_field(),
        }
    }

    pub fn render(
        &self,
        bitmap: &mut Bitmap,
        time: f32,
        settings: GlitterSceneSettings,
        view: Mat4,
    ) {
        bitmap.fill(settings.theme.background);

        let w = time * settings.simplex_speed;
        let positions = self
            .rest_positions
            .iter()
            .map(|rest_position| {
                *rest_position
                    + simplex_offset(&self.field, *rest_position, w) * settings.simplex_scale
            })
            .collect::<Vec<_>>();
        let projected = project_cloud(bitmap, &positions, projection(bitmap.resolution()), view);
        let rotated_normals =
            rotate_normals(&self.normals, tumble_rotation(time, settings.glitter));
        let colors = glitter_colors(
            settings.theme.foreground,
            &rotated_normals,
            view_direction(view),
            settings.glitter,
        );
        settings.depth_field.render(bitmap, &projected, &colors);
    }
}
