use crate::{color::Color, rng::Rng};
use glam::{Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Glitter {
    pub falloff_power: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct GlitterParams {
    pub normal: Vec3,
}

fn random_normal(rng: &mut Rng) -> Vec3 {
    Vec3::new(
        rng.next_gaussian(),
        rng.next_gaussian(),
        rng.next_gaussian(),
    )
    .normalize()
}

fn glitter_amount(particle: GlitterParams, view_direction: Vec3, glitter: Glitter) -> f32 {
    particle
        .normal
        .dot(view_direction)
        .clamp(0.0, 1.0)
        .powf(glitter.falloff_power)
}

fn lerp(left: f32, right: f32, t: f32) -> f32 {
    left * (1.0 - t) + right * t
}

fn lerp_color(left: Color, right: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        lerp(left.red, right.red, t),
        lerp(left.green, right.green, t),
        lerp(left.blue, right.blue, t),
    )
}

pub fn view_direction(view: Mat4) -> Vec3 {
    view.inverse().transform_vector3(-Vec3::Z).normalize()
}

pub fn glitter_particles(rng: &mut Rng, count: usize) -> Vec<GlitterParams> {
    (0..count)
        .map(|_| GlitterParams {
            normal: random_normal(rng),
        })
        .collect()
}

pub fn glitter_colors(
    base_colors: &[Color],
    particles: &[GlitterParams],
    view_direction: Vec3,
    glitter: Glitter,
) -> Vec<Color> {
    assert_eq!(base_colors.len(), particles.len());

    let glitter_tint = Color::new(8.0, 8.0, 8.0);
    base_colors
        .iter()
        .zip(particles)
        .map(|(base_color, particle)| {
            let amount = glitter_amount(*particle, view_direction, glitter);
            lerp_color(*base_color, glitter_tint, amount)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{glitter_colors, view_direction, Glitter, GlitterParams};
    use crate::color::Color;
    use glam::{Mat4, Vec3};

    #[test]
    fn view_direction_matches_look_at_forward_axis() {
        let eye = Vec3::new(2.0, 2.0, 2.0);
        let center = Vec3::ZERO;
        let view = Mat4::look_at_rh(eye, center, Vec3::Y);

        let direction = view_direction(view);

        assert!(direction.abs_diff_eq((center - eye).normalize(), 1e-6));
    }

    #[test]
    fn glitter_colors_brightens_particles_facing_the_camera() {
        let base_color = Color::new(0.25, 0.5, 0.75);
        let view_direction = Vec3::new(0.0, 0.0, -1.0);
        let glitter = Glitter {
            falloff_power: 16.0,
        };
        let particles = [
            GlitterParams {
                normal: view_direction,
            },
            GlitterParams { normal: Vec3::X },
            GlitterParams {
                normal: -view_direction,
            },
        ];

        let colors = glitter_colors(&[base_color; 3], &particles, view_direction, glitter);

        assert_eq!(colors[0], Color::new(1.0, 1.0, 1.0));
        assert_eq!(colors[1], base_color);
        assert_eq!(colors[2], base_color);
    }
}
