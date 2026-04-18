use glam::{Vec2, Vec3};

use crate::{bitmap::Bitmap, color::Color, render::Render};

#[derive(Clone, Copy, Debug)]
pub struct Glow {
    pub softener: f32,
    pub radius: f32,
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn draw_glow(bitmap: &mut Bitmap, center: Vec2, color: Color, glow: Glow) {
    let width = bitmap.width() as i32;
    let height = bitmap.height() as i32;
    let radius = glow.radius;
    let min_x = (center.x - radius).floor().max(0.0) as i32;
    let max_x = (center.x + radius).ceil().min((width - 1) as f32) as i32;
    let min_y = (center.y - radius).floor().max(0.0) as i32;
    let max_y = (center.y + radius).ceil().min((height - 1) as f32) as i32;

    let data = bitmap.data_mut();
    for y in min_y..=max_y {
        let dy = y as f32 + 0.5 - center.y;
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - center.x;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance >= radius {
                continue;
            }
            let intensity = (1.0 - smoothstep(0.0, radius, distance)) * glow.softener;
            let index = (y as usize * width as usize + x as usize) * 4;
            let r = (color.red.clamp(0.0, 1.0) * intensity * 255.0) as u8;
            let g = (color.green.clamp(0.0, 1.0) * intensity * 255.0) as u8;
            let b = (color.blue.clamp(0.0, 1.0) * intensity * 255.0) as u8;
            data[index] = data[index].saturating_add(r);
            data[index + 1] = data[index + 1].saturating_add(g);
            data[index + 2] = data[index + 2].saturating_add(b);
            data[index + 3] = data[index + 3].saturating_add((intensity.clamp(0.0, 1.0) * 255.0) as u8);
        }
    }
}

impl Render for Glow {
    fn render(&self, target: &mut Bitmap, positions: &[Option<Vec3>], colors: &[Color]) {
        for (particle, &color) in positions.iter().zip(colors.iter()) {
            let Some(particle) = particle else {
                continue;
            };
            draw_glow(target, particle.truncate(), color, *self);
        }
    }
}
