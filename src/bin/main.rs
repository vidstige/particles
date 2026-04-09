use std::{
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec2, Vec3, Vec4};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    depth_field::{DepthField, Render, Theme},
    env::{resolution, DEFAULT_RESOLUTION},
    projection::project_cloud,
    resolution::Resolution,
    rng::Rng,
    simplex::SimplexNoise,
};

const DURATION: f32 = 24.0;
const FPS: f32 = 30.0;
const DT: f32 = 1.0 / FPS;
const FIELD_SIZE: usize = 128;
const FIELD_BOUNDS: f32 = 1.6;
const PRESSURE_ITERATIONS: usize = 160;
const PARTICLE_COUNT: usize = 8 * 1024;
const MEAN_SPEED: f32 = 0.35;

fn wrap(value: f32, bounds: f32) -> f32 {
    let span = bounds * 2.0;
    (value + bounds).rem_euclid(span) - bounds
}

fn wrap_point(point: Vec2, bounds: f32) -> Vec2 {
    Vec2::new(wrap(point.x, bounds), wrap(point.y, bounds))
}

struct VectorField {
    size: usize,
    bounds: f32,
    cell_size: f32,
    values: Vec<Vec2>,
}

impl VectorField {
    fn index(&self, x: isize, y: isize) -> usize {
        let size = self.size as isize;
        let x = x.rem_euclid(size) as usize;
        let y = y.rem_euclid(size) as usize;
        y * self.size + x
    }

    fn cell_center(&self, x: usize, y: usize) -> Vec2 {
        let min = -self.bounds + self.cell_size * 0.5;
        Vec2::new(
            min + x as f32 * self.cell_size,
            min + y as f32 * self.cell_size,
        )
    }

    fn divergence_at(&self, x: usize, y: usize) -> f32 {
        let x = x as isize;
        let y = y as isize;
        let vx = self.values[self.index(x, y)].x - self.values[self.index(x - 1, y)].x;
        let vy = self.values[self.index(x, y)].y - self.values[self.index(x, y - 1)].y;
        (vx + vy) / self.cell_size
    }

    fn sample_cell(&self, x: isize, y: isize) -> Vec2 {
        self.values[self.index(x, y)]
    }

    fn new(size: usize, bounds: f32) -> Self {
        Self {
            size,
            bounds,
            cell_size: bounds * 2.0 / size as f32,
            values: vec![Vec2::ZERO; size * size],
        }
    }

    fn from_simplex(size: usize, bounds: f32) -> Self {
        let mut field = Self::new(size, bounds);
        let x_noise = SimplexNoise::new(0x1f2e_3d4c, 1.3, 1.0);
        let y_noise = SimplexNoise::new(0x5a69_7887, 1.3, 1.0);

        for y in 0..size {
            for x in 0..size {
                let index = field.index(x as isize, y as isize);
                let point = field.cell_center(x, y) / bounds;
                field.values[index] = Vec2::new(
                    x_noise.sample(Vec4::new(point.x, point.y, 0.17, 0.0)),
                    y_noise.sample(Vec4::new(point.x, point.y, 3.41, 0.0)),
                );
            }
        }

        field.project_divergence_free(PRESSURE_ITERATIONS);
        field.scale_mean_speed(MEAN_SPEED);
        field
    }

    fn project_divergence_free(&mut self, iterations: usize) {
        let mut divergence = vec![0.0; self.values.len()];
        for y in 0..self.size {
            for x in 0..self.size {
                let index = self.index(x as isize, y as isize);
                divergence[index] = self.divergence_at(x, y);
            }
        }
        let mut pressure = vec![0.0; self.values.len()];
        let mut next_pressure = vec![0.0; self.values.len()];
        let cell_size_squared = self.cell_size * self.cell_size;

        for _ in 0..iterations {
            for y in 0..self.size {
                for x in 0..self.size {
                    let index = self.index(x as isize, y as isize);
                    let left = pressure[self.index(x as isize - 1, y as isize)];
                    let right = pressure[self.index(x as isize + 1, y as isize)];
                    let down = pressure[self.index(x as isize, y as isize - 1)];
                    let up = pressure[self.index(x as isize, y as isize + 1)];
                    next_pressure[index] =
                        (left + right + down + up - cell_size_squared * divergence[index]) * 0.25;
                }
            }
            std::mem::swap(&mut pressure, &mut next_pressure);
        }

        for y in 0..self.size {
            for x in 0..self.size {
                let index = self.index(x as isize, y as isize);
                let grad_x = (pressure[self.index(x as isize + 1, y as isize)] - pressure[index])
                    / self.cell_size;
                let grad_y = (pressure[self.index(x as isize, y as isize + 1)] - pressure[index])
                    / self.cell_size;
                self.values[index] -= Vec2::new(grad_x, grad_y);
            }
        }
    }

    fn scale_mean_speed(&mut self, target: f32) {
        let mean_speed =
            self.values.iter().map(|value| value.length()).sum::<f32>() / self.values.len() as f32;
        let scale = target / mean_speed.max(f32::MIN_POSITIVE);

        for value in &mut self.values {
            *value *= scale;
        }
    }

    fn sample(&self, point: Vec2) -> Vec2 {
        let point = wrap_point(point, self.bounds);
        let grid = (point + Vec2::splat(self.bounds)) / self.cell_size - Vec2::splat(0.5);
        let base = grid.floor();
        let fraction = grid - base;
        let x = base.x as isize;
        let y = base.y as isize;
        let v00 = self.sample_cell(x, y);
        let v10 = self.sample_cell(x + 1, y);
        let v01 = self.sample_cell(x, y + 1);
        let v11 = self.sample_cell(x + 1, y + 1);
        let low = v00.lerp(v10, fraction.x);
        let high = v01.lerp(v11, fraction.x);
        low.lerp(high, fraction.y)
    }

    #[cfg(test)]
    fn divergence_rms(&self) -> f32 {
        let mut mean_square = 0.0;
        for y in 0..self.size {
            for x in 0..self.size {
                mean_square += self.divergence_at(x, y).powi(2);
            }
        }
        mean_square /= self.values.len() as f32;
        mean_square.sqrt()
    }
}

struct SwirlScene {
    field: VectorField,
    positions: Vec<Vec2>,
}

impl SwirlScene {
    fn new() -> Self {
        let mut rng = Rng::new(0x1234_5678);
        let positions = (0..PARTICLE_COUNT)
            .map(|_| {
                Vec2::new(
                    rng.next_f32_in(-FIELD_BOUNDS, FIELD_BOUNDS),
                    rng.next_f32_in(-FIELD_BOUNDS, FIELD_BOUNDS),
                )
            })
            .collect();

        Self {
            field: VectorField::from_simplex(FIELD_SIZE, FIELD_BOUNDS),
            positions,
        }
    }

    fn advance(&mut self, dt: f32) {
        for position in &mut self.positions {
            *position = wrap_point(
                *position + self.field.sample(*position) * dt,
                self.field.bounds,
            );
        }
    }

    fn cloud(&self) -> Vec<Vec3> {
        self.positions
            .iter()
            .map(|position| Vec3::new(position.x, 0.0, position.y))
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
    let resolution = resolution()?;
    let mut bitmap = Bitmap::new(resolution);
    let theme = theme();
    let depth_field = depth_field(bitmap.resolution());
    let view = view();
    let projection = projection(bitmap.resolution());
    let colors = vec![theme.foreground; PARTICLE_COUNT];
    let frame_count = (DURATION * FPS) as usize;
    let mut scene = SwirlScene::new();

    for _ in 0..frame_count {
        bitmap.fill(theme.background);
        let positions = scene.cloud();
        let projected = project_cloud(&bitmap, &positions, projection, view);
        depth_field.render(&mut bitmap, &projected, &colors);
        output.write_all(bitmap.data())?;
        output.flush()?;
        scene.advance(DT);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::VectorField;

    #[test]
    fn projection_reduces_field_divergence() {
        let mut field = VectorField::new(32, 1.0);

        for y in 0..field.size {
            for x in 0..field.size {
                let index = field.index(x as isize, y as isize);
                let point = field.cell_center(x, y);
                field.values[index] = point * 0.5;
            }
        }

        let before = field.divergence_rms();
        field.project_divergence_free(80);
        let after = field.divergence_rms();

        assert!(after < before * 0.2, "{before} -> {after}");
    }
}
