use glam::Vec2;

use crate::{
    field::{divergence_at, subtract, Field},
    poisson::solve_poisson_jacobi,
};

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

pub fn gradient(field: &Field<f32>) -> Field<Vec2> {
    let mut gradient = field.new_like(Vec2::ZERO);
    let cell_size = field.cell_size();
    for y in 0..field.height() {
        for x in 0..field.width() {
            let index = field.index(x as isize, y as isize);
            let grad_x = (field.values[field.index(x as isize + 1, y as isize)]
                - field.values[index])
                / cell_size.x;
            let grad_y = (field.values[field.index(x as isize, y as isize + 1)]
                - field.values[index])
                / cell_size.y;
            gradient.set_index(index, Vec2::new(grad_x, grad_y));
        }
    }
    gradient
}

pub fn project_incompressible(field: &mut Field<Vec2>, iterations: usize) {
    let pressure = solve_poisson_jacobi(&divergence(field), iterations);
    subtract(field, &gradient(&pressure));
}

pub fn advect(field: &Field<Vec2>, dt: f32) -> Field<Vec2> {
    let mut result = field.new_like(Vec2::ZERO);
    for y in 0..field.height() {
        for x in 0..field.width() {
            let pos = field.sample(x, y);
            let prev_pos = pos - field.interpolate(pos) * dt;
            let index = field.index(x as isize, y as isize);
            result.set_index(index, field.interpolate(prev_pos));
        }
    }
    result
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

    use crate::{field::Field, resolution::Resolution};

    use super::{divergence_rms, project_incompressible};

    fn divergent_field(resolution: Resolution) -> Field<Vec2> {
        let mut field = Field::new(resolution, Vec2::new(2.0, 2.0), Vec2::ZERO);
        for y in 0..field.height() {
            for x in 0..field.width() {
                let point = field.sample(x, y);
                field.set(x, y, point * 0.5);
            }
        }
        field
    }

    #[test]
    fn projection_reduces_field_divergence() {
        let mut field = divergent_field(Resolution::new(32, 24));

        let before = divergence_rms(&field);
        project_incompressible(&mut field, 80);
        let after = divergence_rms(&field);

        assert!(after < before * 0.2, "{before} -> {after}");
    }
}
