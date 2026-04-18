use glam::{Vec2, Vec3};

use crate::{bitmap::Bitmap, color::Color, render::Render};

#[derive(Clone, Copy, Debug)]
pub struct Glow {
    pub background: Color,
    pub softener: f32,
    pub radius: f32,
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
    )
}

fn draw_glow(bitmap: &mut Bitmap, center: Vec2, color: Color, glow: Glow) {
    let width = bitmap.width() as i32;
    let height = bitmap.height() as i32;
    let radius = glow.radius;
    let min_x = (center.x - radius).floor().max(0.0) as i32;
    let max_x = (center.x + radius).ceil().min((width - 1) as f32) as i32;
    let min_y = (center.y - radius).floor().max(0.0) as i32;
    let max_y = (center.y + radius).ceil().min((height - 1) as f32) as i32;

    let softened = Color::new(
        color.red * glow.softener,
        color.green * glow.softener,
        color.blue * glow.softener,
    );

    let data = bitmap.data_mut();
    for y in min_y..=max_y {
        let dy = y as f32 + 0.5 - center.y;
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - center.x;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance >= radius {
                continue;
            }
            let t = smoothstep(0.0, radius, distance);
            let glow_color = lerp_color(softened, glow.background, t);
            let index = (y as usize * width as usize + x as usize) * 4;
            let r = (glow_color.red.clamp(0.0, 1.0) * 255.0) as u8;
            let g = (glow_color.green.clamp(0.0, 1.0) * 255.0) as u8;
            let b = (glow_color.blue.clamp(0.0, 1.0) * 255.0) as u8;
            data[index] = data[index].max(r);
            data[index + 1] = data[index + 1].max(g);
            data[index + 2] = data[index + 2].max(b);
            data[index + 3] = 255;
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
