use std::ops::MulAssign;

use glam::Vec2;

pub struct Field<T> {
    size: usize,
    bounds: f32,
    cell_size: f32,
    values: Vec<T>,
}

fn wrap(value: f32, bounds: f32) -> f32 {
    let span = bounds * 2.0;
    (value + bounds).rem_euclid(span) - bounds
}

fn wrap_point(point: Vec2, bounds: f32) -> Vec2 {
    Vec2::new(wrap(point.x, bounds), wrap(point.y, bounds))
}

pub fn divergence_at(field: &Field<Vec2>, x: usize, y: usize) -> f32 {
    let x = x as isize;
    let y = y as isize;
    let vx = field.values[field.index(x, y)].x - field.values[field.index(x - 1, y)].x;
    let vy = field.values[field.index(x, y)].y - field.values[field.index(x, y - 1)].y;
    (vx + vy) / field.cell_size
}

pub fn project_divergence_free(field: &mut Field<Vec2>, iterations: usize) {
    let mut divergence = vec![0.0; field.values.len()];
    for y in 0..field.size {
        for x in 0..field.size {
            let index = field.index(x as isize, y as isize);
            divergence[index] = divergence_at(field, x, y);
        }
    }
    let mut pressure = vec![0.0; field.values.len()];
    let mut next_pressure = vec![0.0; field.values.len()];
    let cell_size_squared = field.cell_size * field.cell_size;

    for _ in 0..iterations {
        for y in 0..field.size {
            for x in 0..field.size {
                let index = field.index(x as isize, y as isize);
                let left = pressure[field.index(x as isize - 1, y as isize)];
                let right = pressure[field.index(x as isize + 1, y as isize)];
                let down = pressure[field.index(x as isize, y as isize - 1)];
                let up = pressure[field.index(x as isize, y as isize + 1)];
                next_pressure[index] =
                    (left + right + down + up - cell_size_squared * divergence[index]) * 0.25;
            }
        }
        std::mem::swap(&mut pressure, &mut next_pressure);
    }

    for y in 0..field.size {
        for x in 0..field.size {
            let index = field.index(x as isize, y as isize);
            let grad_x = (pressure[field.index(x as isize + 1, y as isize)] - pressure[index])
                / field.cell_size;
            let grad_y = (pressure[field.index(x as isize, y as isize + 1)] - pressure[index])
                / field.cell_size;
            field.values[index] -= Vec2::new(grad_x, grad_y);
        }
    }
}

impl<T: Clone> Field<T> {
    pub fn new(size: usize, bounds: f32, value: T) -> Self {
        Self {
            size,
            bounds,
            cell_size: bounds * 2.0 / size as f32,
            values: vec![value; size * size],
        }
    }
}

impl<T> Field<T> {
    fn index(&self, x: isize, y: isize) -> usize {
        let size = self.size as isize;
        let x = x.rem_euclid(size) as usize;
        let y = y.rem_euclid(size) as usize;
        y * self.size + x
    }

    pub fn bounds(&self) -> f32 {
        self.bounds
    }

    pub fn cell_center(&self, x: usize, y: usize) -> Vec2 {
        let min = -self.bounds + self.cell_size * 0.5;
        Vec2::new(
            min + x as f32 * self.cell_size,
            min + y as f32 * self.cell_size,
        )
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        let index = self.index(x as isize, y as isize);
        self.values[index] = value;
    }
}

impl Field<Vec2> {
    fn sample_cell(&self, x: isize, y: isize) -> Vec2 {
        self.values[self.index(x, y)]
    }

    pub fn mean_length(&self) -> f32 {
        self.values.iter().map(|value| value.length()).sum::<f32>() / self.values.len() as f32
    }

    pub fn sample(&self, point: Vec2) -> Vec2 {
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
}

impl MulAssign<f32> for Field<Vec2> {
    fn mul_assign(&mut self, scale: f32) {
        for value in &mut self.values {
            *value *= scale;
        }
    }
}

#[cfg(test)]
fn divergence_rms(field: &Field<Vec2>) -> f32 {
    let mut mean_square = 0.0;
    for y in 0..field.size {
        for x in 0..field.size {
            mean_square += divergence_at(field, x, y).powi(2);
        }
    }
    mean_square /= field.values.len() as f32;
    mean_square.sqrt()
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::{divergence_rms, project_divergence_free, Field};

    #[test]
    fn mean_length_scales_with_uniform_field_scaling() {
        let mut field = Field::new(4, 1.0, Vec2::new(3.0, 4.0));

        assert_eq!(field.mean_length(), 5.0);

        field *= 0.5;

        assert_eq!(field.mean_length(), 2.5);
    }

    #[test]
    fn projection_reduces_field_divergence() {
        let mut field = Field::new(32, 1.0, Vec2::ZERO);

        for y in 0..field.size {
            for x in 0..field.size {
                let index = field.index(x as isize, y as isize);
                let point = field.cell_center(x, y);
                field.values[index] = point * 0.5;
            }
        }

        let before = divergence_rms(&field);
        project_divergence_free(&mut field, 80);
        let after = divergence_rms(&field);

        assert!(after < before * 0.2, "{before} -> {after}");
    }
}
