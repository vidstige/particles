use glam::Vec2;

use crate::{bitmap::Bitmap, color::Rgba8};

fn coverage(
    distance_squared: f32,
    radius: f32,
    has_inner_radius: bool,
    inner_radius_squared: f32,
    outer_radius_squared: f32,
) -> f32 {
    if has_inner_radius && distance_squared <= inner_radius_squared {
        return 1.0;
    }
    if distance_squared >= outer_radius_squared {
        return 0.0;
    }
    (radius + 0.5 - distance_squared.sqrt()).clamp(0.0, 1.0)
}

pub fn draw_disk(bitmap: &mut Bitmap, center: Vec2, radius: f32, color: Rgba8) {
    let width = bitmap.width() as i32;
    let height = bitmap.height() as i32;
    let radius_plus_half = radius + 0.5;
    let has_inner_radius = radius >= 0.5;
    let inner_radius = (radius - 0.5).max(0.0);
    let inner_radius_squared = inner_radius * inner_radius;
    let outer_radius_squared = radius_plus_half * radius_plus_half;
    let min_x = (center.x - radius - 0.5).floor().max(0.0) as i32;
    let max_x = (center.x + radius + 0.5).ceil().min((width - 1) as f32) as i32;
    let min_y = (center.y - radius - 0.5).floor().max(0.0) as i32;
    let max_y = (center.y + radius + 0.5).ceil().min((height - 1) as f32) as i32;
    let data = bitmap.data_mut();

    for y in min_y..=max_y {
        let sample_y = y as f32 + 0.5 - center.y;
        let sample_y_squared = sample_y * sample_y;
        for x in min_x..=max_x {
            let sample_x = x as f32 + 0.5 - center.x;
            let distance_squared = sample_x * sample_x + sample_y_squared;
            let alpha = coverage(
                distance_squared,
                radius,
                has_inner_radius,
                inner_radius_squared,
                outer_radius_squared,
            );
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
    use crate::{bitmap::Bitmap, color::Rgba8, resolution::Resolution};
    use glam::Vec2;

    #[test]
    fn draw_disk_antialiases_edges() {
        let mut bitmap = Bitmap::new(Resolution::new(5, 5));
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
        let mut bitmap = Bitmap::new(Resolution::new(3, 3));
        let color = Rgba8::new(191, 191, 191, 191);

        draw_disk(&mut bitmap, Vec2::new(1.5, 1.5), 0.5, color);
        draw_disk(&mut bitmap, Vec2::new(1.5, 1.5), 0.5, color);

        let pixel = bitmap.pixel(1, 1).unwrap();
        assert_eq!(pixel.red, 255);
        assert_eq!(pixel.green, 255);
        assert_eq!(pixel.blue, 255);
        assert_eq!(pixel.alpha, 255);
    }

    #[test]
    fn draw_disk_keeps_small_disks_partially_transparent() {
        let mut bitmap = Bitmap::new(Resolution::new(3, 3));

        draw_disk(
            &mut bitmap,
            Vec2::new(1.5, 1.5),
            0.25,
            Rgba8::from_rgb(255, 255, 255),
        );

        let pixel = bitmap.pixel(1, 1).unwrap();
        assert_eq!(pixel.alpha, 191);
    }
}
