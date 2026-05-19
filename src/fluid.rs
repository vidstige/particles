use std::ops::{Add, Mul, Sub};

use glam::Vec2;

use crate::{
    field::{divergence_at, subtract, Field},
    poisson::solve_poisson_gauss_seidel,
    resolution::Resolution,
    rng::Rng,
};

pub fn flow_field_from_bezier(rng: &mut Rng, resolution: Resolution, size: Vec2) -> Field<Vec2> {
    let p0 = Vec2::new(rng.next_f32_in(0.0, size.x), rng.next_f32_in(0.0, size.y));
    let p1 = Vec2::new(rng.next_f32_in(0.0, size.x), rng.next_f32_in(0.0, size.y));
    let p2 = Vec2::new(rng.next_f32_in(0.0, size.x), rng.next_f32_in(0.0, size.y));

    let width = resolution.width as usize;
    let height = resolution.height as usize;
    let mut field = Field::new(resolution, size, Vec2::ZERO);
    let radius = 2.0_f32;
    let steps = 400;

    for y in 0..height {
        for x in 0..width {
            let pos = field.sample(x, y);
            let mut best_dist = f32::MAX;
            let mut best_tangent = Vec2::ZERO;
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let mt = 1.0 - t;
                let curve_pos = p0 * (mt * mt) + p1 * (2.0 * mt * t) + p2 * (t * t);
                let dist = (pos - curve_pos).length();
                if dist < best_dist {
                    best_dist = dist;
                    best_tangent = ((p1 - p0) * (2.0 * mt) + (p2 - p1) * (2.0 * t)).normalize_or_zero();
                }
            }
            if best_dist < radius {
                field.set(x, y, best_tangent);
            }
        }
    }
    field
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
    let mut pressure = field.new_like(0.0f32);
    solve_poisson_gauss_seidel(&divergence(field), &mut pressure, iterations);
    subtract(field, &gradient(&pressure));
}

pub fn advect_scalar<T>(density: &Field<T>, velocity: &Field<Vec2>, dt: f32) -> Field<T>
where
    T: Clone + Default + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
{
    let mut result = density.new_like(T::default());
    for y in 0..density.height() {
        for x in 0..density.width() {
            let pos = density.sample(x, y);
            let prev_pos = pos - velocity.interpolate(pos) * dt;
            let index = density.index(x as isize, y as isize);
            result.set_index(index, density.interpolate(prev_pos));
        }
    }
    result
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
