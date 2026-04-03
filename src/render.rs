use glam::{Vec2, Vec3};

use crate::color::Color;
use tiny_skia::{BlendMode, Color as TinyColor, FillRule, Paint, PathBuilder, Pixmap, Transform};

const PARTICLE_RADIUS: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: TinyColor,
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

fn with_alpha(color: TinyColor, alpha: f32) -> TinyColor {
    let mut tmp = color.clone();
    tmp.set_alpha(alpha);
    tmp
}

fn draw_disk(pixmap: &mut Pixmap, center: Vec2, radius: f32, color: TinyColor) {
    let Some(path) = PathBuilder::from_circle(center.x, center.y, radius) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.blend_mode = BlendMode::Plus;
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
}

pub fn render_cloud(
    pixmap: &mut Pixmap,
    positions: &[Option<Vec3>],
    colors: &[Color],
    depth_field: DepthField,
) {
    assert_eq!(positions.len(), colors.len());

    let blur = depth_field.blur;

    for (particle, color) in positions.iter().copied().zip(colors.iter().copied()) {
        let Some(particle) = particle else {
            continue;
        };
        let focal_distance = (particle.z - depth_field.focus_depth).abs();
        let radius = PARTICLE_RADIUS + blur * focal_distance;
        let alpha = circle_area(PARTICLE_RADIUS) / circle_area(radius);
        draw_disk(
            pixmap,
            particle.truncate(),
            radius,
            with_alpha(color.to_tiny_color(), alpha),
        );
    }
}
