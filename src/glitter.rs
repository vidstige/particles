use std::f32::consts::PI;

use tiny_skia::Color as TinyColor;

use crate::{color::Color, rng::Rng};

#[derive(Clone, Copy, Debug)]
pub struct GlitterParams {
    pub glint_time: f32,
    pub pause_time: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct GlitterParticle {
    phase: f32,
}

fn mix_color(left: Color, right: Color, t: f32) -> Color {
    left * (1.0 - t) + right * t
}

fn glitter_amount(particle: GlitterParticle, time: f32, params: GlitterParams) -> f32 {
    let cycle_time = (params.glint_time + params.pause_time).max(f32::MIN_POSITIVE);
    let glint_time = params.glint_time.max(f32::MIN_POSITIVE);
    let cycle_phase = (time / cycle_time + particle.phase).fract() * cycle_time;
    if cycle_phase >= params.glint_time {
        return 0.0;
    }

    let glint_phase = (cycle_phase / glint_time).clamp(0.0, 1.0);
    (PI * glint_phase).sin().max(0.0).powi(8)
}

pub fn default_glitter_params() -> GlitterParams {
    GlitterParams {
        glint_time: 3.0,
        pause_time: 9.0,
    }
}

pub fn glitter_particles(rng: &mut Rng, count: usize) -> Vec<GlitterParticle> {
    (0..count)
        .map(|_| GlitterParticle {
            phase: rng.next_f32(),
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
