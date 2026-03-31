use std::f32::consts::PI;

use tiny_skia::Color as TinyColor;

use crate::rng::Rng;

#[derive(Clone, Copy, Debug)]
pub struct Glitter {
    pub glint_time: f32,
    pub pause_time: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct GlitterParams {
    phase: f32,
}

fn glitter_amount(particle: GlitterParams, time: f32, glitter: Glitter) -> f32 {
    let cycle_time = (glitter.glint_time + glitter.pause_time).max(f32::MIN_POSITIVE);
    let glint_time = glitter.glint_time.max(f32::MIN_POSITIVE);
    let cycle_phase = (time / cycle_time + particle.phase).fract() * cycle_time;
    if cycle_phase >= glitter.glint_time {
        return 0.0;
    }

    let glint_phase = (cycle_phase / glint_time).clamp(0.0, 1.0);
    (PI * glint_phase).sin().max(0.0).powi(8)
}

fn lerp(left: f32, right: f32, t: f32) -> f32 {
    left * (1.0 - t) + right * t
}

fn lerp_color(left: TinyColor, right: TinyColor, t: f32) -> TinyColor {
    let t = t.clamp(0.0, 1.0);
    TinyColor::from_rgba(
        lerp(left.red(), right.red(), t),
        lerp(left.green(), right.green(), t),
        lerp(left.blue(), right.blue(), t),
        lerp(left.alpha(), right.alpha(), t),
    )
    .unwrap()
}

pub fn glitter_particles(rng: &mut Rng, count: usize) -> Vec<GlitterParams> {
    (0..count)
        .map(|_| GlitterParams {
            phase: rng.next_f32(),
        })
        .collect()
}

pub fn glitter_colors(
    base_colors: &[TinyColor],
    particles: &[GlitterParams],
    time: f32,
    glitter: Glitter,
) -> Vec<TinyColor> {
    assert_eq!(base_colors.len(), particles.len());

    let glitter_tint = TinyColor::WHITE;
    base_colors
        .iter()
        .zip(particles)
        .map(|(base_color, particle)| {
            let amount = glitter_amount(*particle, time, glitter);
            lerp_color(*base_color, glitter_tint, amount)
        })
        .collect()
}
