use std::ops::MulAssign;

use glam::Vec2;

use crate::{poisson::solve_poisson_jacobi, resolution::Resolution};

pub struct Field<T> {
    resolution: Resolution,
    // Full world-space size of the field. Stored values live on the grid corners.
    size: Vec2,
    values: Vec<T>,
}

fn wrap(value: f32, size: f32) -> f32 {
    value.rem_euclid(size)
}

fn wrap_point(point: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(wrap(point.x, size.x), wrap(point.y, size.y))
}

pub fn divergence_at(field: &Field<Vec2>, x: usize, y: usize) -> f32 {
    let x = x as isize;
    let y = y as isize;
    let vx = field.values[field.index(x, y)].x - field.values[field.index(x - 1, y)].x;
    let vy = field.values[field.index(x, y)].y - field.values[field.index(x, y - 1)].y;
    let cell_size = field.cell_size();
    vx / cell_size.x + vy / cell_size.y
}

pub fn divergence(field: &Field<Vec2>) -> Field<f32> {
    let mut divergence = field.new_like(0.0);
    for y in 0..field.height() {
        for x in 0..field.width() {
            let index = field.index(x as isize, y as isize);
            divergence.set_index(index, divergence_at(field, x, y));
        }
    }

    divergence
}

pub fn subtract_gradient(field: &mut Field<Vec2>, scalar: &Field<f32>) {
    assert_eq!(field.resolution, scalar.resolution);
    assert_eq!(field.size, scalar.size);

    let cell_size = field.cell_size();
    for y in 0..field.height() {
        for x in 0..field.width() {
            let index = field.index(x as isize, y as isize);
            let grad_x = (scalar.values[field.index(x as isize + 1, y as isize)]
                - scalar.values[index])
                / cell_size.x;
            let grad_y = (scalar.values[field.index(x as isize, y as isize + 1)]
                - scalar.values[index])
                / cell_size.y;
            field.values[index] -= Vec2::new(grad_x, grad_y);
        }
    }
}

pub fn project_incompressible(field: &mut Field<Vec2>, iterations: usize) {
    let pressure = solve_poisson_jacobi(&divergence(field), iterations);
    subtract_gradient(field, &pressure);
}

impl<T: Clone> Field<T> {
    pub fn new(resolution: Resolution, size: Vec2, value: T) -> Self {
        assert!(resolution.area() > 0);
        Self {
            values: vec![value; resolution.area()],
            resolution,
            size,
        }
    }
}

impl<T> Field<T> {
    pub(crate) fn new_like<U: Clone>(&self, value: U) -> Field<U> {
        Field::new(self.resolution.clone(), self.size, value)
    }

    pub(crate) fn index(&self, x: isize, y: isize) -> usize {
        let width = self.resolution.width as isize;
        let height = self.resolution.height as isize;
        let x = x.rem_euclid(width) as usize;
        let y = y.rem_euclid(height) as usize;
        y * self.width() + x
    }

    pub(crate) fn width(&self) -> usize {
        self.resolution.width as usize
    }

    pub(crate) fn height(&self) -> usize {
        self.resolution.height as usize
    }

    pub(crate) fn cell_size(&self) -> Vec2 {
        Vec2::new(
            self.size.x / self.resolution.width as f32,
            self.size.y / self.resolution.height as f32,
        )
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn sample(&self, x: usize, y: usize) -> Vec2 {
        let cell_size = self.cell_size();
        Vec2::new(x as f32 * cell_size.x, y as f32 * cell_size.y)
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        let index = self.index(x as isize, y as isize);
        self.values[index] = value;
    }

    pub(crate) fn set_index(&mut self, index: usize, value: T) {
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

    pub fn interpolate(&self, point: Vec2) -> Vec2 {
        let point = wrap_point(point, self.size);
        let grid = point / self.cell_size();
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

impl Field<f32> {
    pub(crate) fn get(&self, index: usize) -> f32 {
        self.values[index]
    }

    pub(crate) fn get_wrapped(&self, x: isize, y: isize) -> f32 {
        self.values[self.index(x, y)]
    }
}

#[cfg(test)]
fn divergence_rms(field: &Field<Vec2>) -> f32 {
    let mean_square = divergence(field)
        .values
        .into_iter()
        .map(|value| value.powi(2))
        .sum::<f32>()
        / field.values.len() as f32;
    mean_square.sqrt()
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use crate::resolution::Resolution;

    use super::{divergence_rms, project_incompressible, Field};

    #[test]
    fn sample_returns_grid_corner_positions() {
        let field = Field::new(Resolution::new(4, 2), Vec2::new(8.0, 6.0), Vec2::ZERO);

        assert_eq!(field.sample(0, 0), Vec2::ZERO);
        assert_eq!(field.sample(1, 0), Vec2::new(2.0, 0.0));
        assert_eq!(field.sample(0, 1), Vec2::new(0.0, 3.0));
    }

    #[test]
    fn mean_length_scales_with_uniform_field_scaling() {
        let mut field = Field::new(
            Resolution::new(4, 3),
            Vec2::new(2.0, 2.0),
            Vec2::new(3.0, 4.0),
        );

        assert_eq!(field.mean_length(), 5.0);

        field *= 0.5;

        assert_eq!(field.mean_length(), 2.5);
    }

    #[test]
    fn projection_reduces_field_divergence() {
        let mut field = Field::new(Resolution::new(32, 24), Vec2::new(2.0, 2.0), Vec2::ZERO);

        for y in 0..field.height() {
            for x in 0..field.width() {
                let point = field.sample(x, y);
                field.set(x, y, point * 0.5);
            }
        }

        let before = divergence_rms(&field);
        project_incompressible(&mut field, 80);
        let after = divergence_rms(&field);

        assert!(after < before * 0.2, "{before} -> {after}");
    }
}
