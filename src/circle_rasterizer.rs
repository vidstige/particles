use glam::Vec2;

use crate::{bitmap::Bitmap, color::Rgba8};

fn coverage(center: Vec2, radius: f32, x: i32, y: i32) -> f32 {
    let sample_x = x as f32 + 0.5 - center.x;
    let sample_y = y as f32 + 0.5 - center.y;
    (radius + 0.5 - sample_x.hypot(sample_y)).clamp(0.0, 1.0)
}

pub fn draw_disk(bitmap: &mut Bitmap, center: Vec2, radius: f32, color: Rgba8) {
    let width = bitmap.width() as i32;
    let height = bitmap.height() as i32;
    let min_x = (center.x - radius - 0.5).floor().max(0.0) as i32;
    let max_x = (center.x + radius + 0.5).ceil().min((width - 1) as f32) as i32;
    let min_y = (center.y - radius - 0.5).floor().max(0.0) as i32;
    let max_y = (center.y + radius + 0.5).ceil().min((height - 1) as f32) as i32;
    let data = bitmap.data_mut();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let alpha = coverage(center, radius, x, y);
            if alpha == 0.0 {
                continue;
            }

            let color = color.scale(alpha);
            let index = (y as usize * width as usize + x as usize) * 4;
            data[index] = data[index].saturating_add(color.red);
            data[index + 1] = data[index + 1].saturating_add(color.green);
            data[index + 2] = data[index + 2].saturating_add(color.blue);
            data[index + 3] = data[index + 3].saturating_add(color.alpha);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::draw_disk;
    use crate::{bitmap::Bitmap, color::Rgba8};
    use glam::Vec2;

    #[test]
    fn draw_disk_antialiases_edges() {
        let mut bitmap = Bitmap::new(5, 5);
        draw_disk(
            &mut bitmap,
            Vec2::new(2.5, 2.5),
            1.0,
            Rgba8::from_rgb(255, 255, 255),
        );

        let center = bitmap.pixel(2, 2).unwrap();
        let edge = bitmap.pixel(3, 2).unwrap();
        let outside = bitmap.pixel(4, 2).unwrap();

        assert_eq!(center.alpha, 255);
        assert!(edge.alpha > 0);
        assert!(edge.alpha < 255);
        assert_eq!(outside.alpha, 0);
    }

    #[test]
    fn draw_disk_adds_colors_with_saturation() {
        let mut bitmap = Bitmap::new(3, 3);
        let color = Rgba8::new(191, 191, 191, 191);

        draw_disk(&mut bitmap, Vec2::new(1.5, 1.5), 0.5, color);
        draw_disk(&mut bitmap, Vec2::new(1.5, 1.5), 0.5, color);

        let pixel = bitmap.pixel(1, 1).unwrap();
        assert_eq!(pixel.red, 255);
        assert_eq!(pixel.green, 255);
        assert_eq!(pixel.blue, 255);
        assert_eq!(pixel.alpha, 255);
    }
}
