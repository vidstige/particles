use glam::{Mat4, Vec3, Vec4};

use crate::resolution::Resolution;
use tiny_skia::{Color, Pixmap, PremultipliedColorU8};

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub foreground: PremultipliedColorU8,
}

pub fn default_theme() -> Theme {
    Theme {
        background: Color::from_rgba8(14, 14, 18, 255),
        foreground: PremultipliedColorU8::from_rgba(214, 92, 255, 255).unwrap(),
    }
}

fn project(point: Vec3, resolution: &Resolution, view_projection: Mat4) -> Option<(u32, u32)> {
    let clip = view_projection * Vec4::new(point.x, point.y, point.z, 1.0);
    if clip.w <= 0.0 {
        return None;
    }

    let ndc = clip.truncate() / clip.w;
    if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 || !(-1.0..=1.0).contains(&ndc.z) {
        return None;
    }

    let x = ((ndc.x + 1.0) * 0.5 * (resolution.width - 1) as f32).round() as u32;
    let y = ((1.0 - (ndc.y + 1.0) * 0.5) * (resolution.height - 1) as f32).round() as u32;
    Some((x, y))
}

pub fn render_cloud(
    positions: &[Vec3],
    resolution: &Resolution,
    projection: Mat4,
    view: Mat4,
    theme: &Theme,
) -> Pixmap {
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    pixmap.fill(theme.background);
    let view_projection = projection * view;

    for point in positions {
        let Some((x, y)) = project(*point, resolution, view_projection) else {
            continue;
        };
        let index = y as usize * resolution.width as usize + x as usize;
        pixmap.pixels_mut()[index] = theme.foreground;
    }

    pixmap
}
