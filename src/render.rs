use glam::{Mat4, Vec3, Vec4};

use crate::resolution::Resolution;
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

const FOREGROUND_ALPHA: u8 = 96;
const PARTICLE_SIZE: f32 = 3.0;

fn particle_paint(theme: &Theme) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.anti_alias = false;
    paint.set_color(theme.foreground);
    paint
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
}

pub fn default_theme() -> Theme {
    Theme {
        background: Color::from_rgba8(14, 14, 18, 255),
        foreground: Color::from_rgba8(214, 92, 255, FOREGROUND_ALPHA),
    }
}

fn draw_square(pixmap: &mut Pixmap, x: f32, y: f32, paint: &Paint) {
    let half_size = PARTICLE_SIZE * 0.5;
    let rect = Rect::from_xywh(x - half_size, y - half_size, PARTICLE_SIZE, PARTICLE_SIZE).unwrap();
    pixmap.fill_rect(rect, paint, Transform::identity(), None);
}

fn project(
    point: Vec3,
    resolution: &Resolution,
    projection: Mat4,
    view: Mat4,
) -> Option<(f32, f32)> {
    let clip = projection * view * Vec4::new(point.x, point.y, point.z, 1.0);
    if clip.w <= 0.0 {
        return None;
    }

    let ndc = clip.truncate() / clip.w;
    if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 || !(-1.0..=1.0).contains(&ndc.z) {
        return None;
    }

    let x = ((ndc.x + 1.0) * 0.5 * (resolution.width - 1) as f32).round();
    let y = ((1.0 - (ndc.y + 1.0) * 0.5) * (resolution.height - 1) as f32).round();
    Some((x, y))
}

pub fn render_background(resolution: &Resolution, theme: &Theme) -> Pixmap {
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    pixmap.fill(theme.background);
    pixmap
}

pub fn render_cloud(
    positions: &[Vec3],
    resolution: &Resolution,
    projection: Mat4,
    view: Mat4,
    theme: &Theme,
) -> Pixmap {
    let mut pixmap = render_background(resolution, theme);
    let paint = particle_paint(theme);

    for point in positions {
        let Some((x, y)) = project(*point, resolution, projection, view) else {
            continue;
        };
        draw_square(&mut pixmap, x, y, &paint);
    }

    pixmap
}
