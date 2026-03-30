use glam::{Mat4, Vec2, Vec3, Vec4};
use tiny_skia::Pixmap;

use crate::resolution::Resolution;

fn from_pixmap(pixmap: &Pixmap) -> Resolution {
    Resolution::new(pixmap.width(), pixmap.height())
}

fn project_clip(clip: Vec4, resolution: &Resolution) -> Option<Vec2> {
    if clip.w <= 0.0 {
        return None;
    }

    let ndc = clip.truncate() / clip.w;
    if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 || !(-1.0..=1.0).contains(&ndc.z) {
        return None;
    }

    let x = (ndc.x + 1.0) * 0.5 * (resolution.width - 1) as f32;
    let y = (1.0 - (ndc.y + 1.0) * 0.5) * (resolution.height - 1) as f32;
    Some(Vec2::new(x, y))
}

fn project_position(point: Vec3, resolution: &Resolution, view_projection: Mat4) -> Option<Vec2> {
    project_clip(view_projection * point.extend(1.0), resolution)
}

fn project_particle(
    point: Vec3,
    resolution: &Resolution,
    view_projection: Mat4,
    view: Mat4,
) -> Option<Vec3> {
    let view_point = view.transform_point3(point);
    let screen = project_position(point, resolution, view_projection)?;

    Some(screen.extend(-view_point.z))
}

pub fn project_cloud(
    pixmap: &Pixmap,
    positions: &[Vec3],
    projection: Mat4,
    view: Mat4,
) -> Vec<Option<Vec3>> {
    let resolution = from_pixmap(pixmap);
    let view_projection = projection * view;
    positions
        .iter()
        .copied()
        .map(|point| project_particle(point, &resolution, view_projection, view))
        .collect()
}
