use glam::{Mat4, Vec2, Vec4, Vec4Swizzles};

use crate::{bitmap::Bitmap, color::Color, field::Field};

pub fn draw_texture(bitmap: &mut Bitmap, density: &Field<Color>, projection: Mat4, view: Mat4) {
    let inv_vp = (projection * view).inverse();
    let width = bitmap.width() as f32;
    let height = bitmap.height() as f32;
    let offset = density.size() * 0.5;
    for py in 0..bitmap.height() {
        for px in 0..bitmap.width() {
            let x_ndc = (px as f32 + 0.5) / width * 2.0 - 1.0;
            let y_ndc = 1.0 - (py as f32 + 0.5) / height * 2.0;
            let near_h = inv_vp * Vec4::new(x_ndc, y_ndc, -1.0, 1.0);
            let near = near_h.xyz() / near_h.w;
            let far_h = inv_vp * Vec4::new(x_ndc, y_ndc, 1.0, 1.0);
            let far = far_h.xyz() / far_h.w;
            let dir = far - near;
            if dir.y.abs() < 1e-6 { continue; }
            let t = -near.y / dir.y;
            if t < 0.0 { continue; }
            let world = near + dir * t;
            let field_pos = Vec2::new(world.x + offset.x, world.z + offset.y);
            bitmap.set_pixel(px, py, density.interpolate(field_pos).to_rgba8(1.0));
        }
    }
}
