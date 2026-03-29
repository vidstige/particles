use std::f32::consts::TAU;

use tiny_skia::Color as TinyColor;

use crate::{color::Color, rng::Rng};

#[derive(Clone, Copy, Debug)]
pub struct GlitterParams {
    pub density: f32,
    pub speed: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct GlitterParticle {
    phase: f32,
    seed: f32,
}

fn unit_noise(seed: f32) -> f32 {
    (seed.sin() * 43_758.547).rem_euclid(1.0)
}

fn mix_color(left: Color, right: Color, t: f32) -> Color {
    left * (1.0 - t) + right * t
}

fn glitter_amount(particle: GlitterParticle, time: f32, params: GlitterParams) -> f32 {
    if particle.seed >= params.density {
        return 0.0;
    }

    (time * params.speed + particle.phase)
        .sin()
        .max(0.0)
        .powi(8)
}

pub fn default_glitter_params() -> GlitterParams {
    GlitterParams {
        density: 0.06,
        speed: 0.5,
    }
}

pub fn glitter_particles(rng: &mut Rng, count: usize) -> Vec<GlitterParticle> {
    (0..count)
        .map(|_| GlitterParticle {
            phase: rng.next_f32_in(0.0, TAU),
            seed: unit_noise(rng.next_f32()),
        })
        .collect()
}

pub fn glitter_colors(
    base_colors: &[Color],
    particles: &[GlitterParticle],
    time: f32,
    params: GlitterParams,
) -> Vec<Color> {
    assert_eq!(base_colors.len(), particles.len());

    let glitter_tint = Color::from_tiny_color(TinyColor::WHITE);
    base_colors
        .iter()
        .zip(particles)
        .map(|(base_color, particle)| {
            let glitter = glitter_amount(*particle, time, params);
            mix_color(*base_color, glitter_tint, glitter)
        })
        .collect()
}
