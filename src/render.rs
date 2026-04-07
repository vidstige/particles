use glam::Vec3;

use crate::{bitmap::Bitmap, circle_rasterizer::draw_disk, color::Color, color::Rgba8};

const PARTICLE_RADIUS: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Rgba8,
    pub foreground: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct DepthField {
    pub focus_depth: f32,
    pub blur: f32,
}

fn circle_area(radius: f32) -> f32 {
    std::f32::consts::PI * radius.max(f32::MIN_POSITIVE).powi(2)
}

pub(crate) fn render_cloud_with_alpha_scale(
    bitmap: &mut Bitmap,
    positions: &[Option<Vec3>],
    colors: &[Color],
    depth_field: DepthField,
    alpha_scale: impl Fn(Color) -> f32,
) {
    assert_eq!(positions.len(), colors.len());

    for (particle, color) in positions.iter().copied().zip(colors.iter().copied()) {
        let Some(particle) = particle else {
            continue;
        };
        let focal_distance = (particle.z - depth_field.focus_depth).abs();
        let radius = PARTICLE_RADIUS + depth_field.blur * focal_distance;
        let alpha = circle_area(PARTICLE_RADIUS) / circle_area(radius) * alpha_scale(color);
        draw_disk(bitmap, particle.truncate(), radius, color.to_rgba8(alpha));
    }
}

pub fn render_cloud(
    bitmap: &mut Bitmap,
    positions: &[Option<Vec3>],
    colors: &[Color],
    depth_field: DepthField,
) {
    render_cloud_with_alpha_scale(bitmap, positions, colors, depth_field, |_| 1.0);
}
