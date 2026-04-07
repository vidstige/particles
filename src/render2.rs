use glam::Vec3;

use crate::{
    bitmap::Bitmap,
    color::Color,
    render::{render_cloud_with_alpha_scale, DepthField},
};

fn overflow_scale(color: Color) -> f32 {
    color.red.max(color.green).max(color.blue).max(1.0)
}

pub fn render_cloud(
    bitmap: &mut Bitmap,
    positions: &[Option<Vec3>],
    colors: &[Color],
    depth_field: DepthField,
) {
    render_cloud_with_alpha_scale(bitmap, positions, colors, depth_field, overflow_scale);
}
