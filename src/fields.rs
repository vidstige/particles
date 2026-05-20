use glam::Vec2;

use crate::{field::Field, resolution::Resolution, rng::Rng};

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
