use glam::Vec3;

use crate::{bitmap::Bitmap, circle_rasterizer::draw_disk, color::Color, color::Rgba8};

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Rgba8,
    pub foreground: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct DepthField {
    pub focus_depth: f32,
    pub blur: f32,
    pub particle_radius: f32,
}

pub trait Render {
    fn render(&self, target: &mut Bitmap, positions: &[Option<Vec3>], colors: &[Color]);
}

fn circle_area(radius: f32) -> f32 {
    std::f32::consts::PI * radius.max(f32::MIN_POSITIVE).powi(2)
}

fn overflow_scale(color: Color) -> f32 {
    color.red.max(color.green).max(color.blue).max(1.0)
}

fn render_cloud(
    target: &mut Bitmap,
    positions: &[Option<Vec3>],
    colors: &[Color],
    depth_field: DepthField,
) {
    assert_eq!(positions.len(), colors.len());

    for (particle, color) in positions.iter().copied().zip(colors.iter().copied()) {
        let Some(particle) = particle else {
            continue;
        };
        let focal_distance = (particle.z - depth_field.focus_depth).abs();
        let radius = depth_field.particle_radius + depth_field.blur * focal_distance;
        let alpha =
            circle_area(depth_field.particle_radius) / circle_area(radius) * overflow_scale(color);
        draw_disk(target, particle.truncate(), radius, color.to_rgba8(alpha));
    }
}

impl Render for DepthField {
    fn render(&self, target: &mut Bitmap, positions: &[Option<Vec3>], colors: &[Color]) {
        render_cloud(target, positions, colors, *self);
    }
}
