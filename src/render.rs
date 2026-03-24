use crate::{cloud::Cloud, resolution::Resolution};
use tiny_skia::{Color, Pixmap, PremultipliedColorU8};

pub struct View {
    scale: f32,
    distance: f32,
    focal: f32,
    aspect: f32,
}

impl View {
    pub fn fit(resolution: &Resolution, clouds: &[&Cloud]) -> Self {
        let radius = clouds
            .iter()
            .flat_map(|cloud| cloud.positions.iter())
            .map(|point| point.length())
            .fold(0.0, f32::max)
            .max(1.0);
        let scale = 1.0 / radius;
        let fov_y = 50.0_f32.to_radians();
        let focal = 1.0 / (fov_y * 0.5).tan();
        let framed_radius = 1.1;
        let distance = framed_radius * (1.0 + focal);

        Self {
            scale,
            distance,
            focal,
            aspect: resolution.aspect_ratio(),
        }
    }
}

pub fn render_cloud(cloud: &Cloud, resolution: &Resolution, view: &View) -> Pixmap {
    let mut pixmap = Pixmap::new(resolution.width, resolution.height).unwrap();
    pixmap.fill(Color::from_rgba8(0, 0, 0, 255));

    for point in &cloud.positions {
        let Some((x, y)) = project(*point, resolution, view) else {
            continue;
        };

        let index = y as usize * resolution.width as usize + x as usize;
        pixmap.pixels_mut()[index] = PremultipliedColorU8::from_rgba(255, 255, 255, 255).unwrap();
    }

    pixmap
}

fn project(
    point: glam::Vec3,
    resolution: &Resolution,
    view: &View,
) -> Option<(u32, u32)> {
    let point = point * view.scale;
    let depth = view.distance - point.z;
    if depth <= 0.0 {
        return None;
    }

    let ndc_x = point.x * view.focal / (view.aspect * depth);
    let ndc_y = point.y * view.focal / depth;
    if ndc_x.abs() > 1.0 || ndc_y.abs() > 1.0 {
        return None;
    }

    let x = ((ndc_x + 1.0) * 0.5 * (resolution.width - 1) as f32).round() as u32;
    let y = ((1.0 - (ndc_y + 1.0) * 0.5) * (resolution.height - 1) as f32).round() as u32;
    Some((x, y))
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use tiny_skia::PremultipliedColorU8;

    use super::{render_cloud, View};
    use crate::{cloud::Cloud, resolution::Resolution};

    #[test]
    fn renders_projected_points_into_pixmap() {
        let resolution = Resolution::new(5, 5);
        let cloud = Cloud {
            positions: vec![Vec3::ZERO],
        };

        let view = View::fit(&resolution, &[&cloud]);
        let pixmap = render_cloud(&cloud, &resolution, &view);

        assert_eq!(pixmap.width(), 5);
        assert_eq!(pixmap.height(), 5);
        assert_eq!(
            pixmap.pixel(2, 2),
            Some(PremultipliedColorU8::from_rgba(255, 255, 255, 255).unwrap())
        );
        assert_eq!(
            pixmap.pixel(0, 0),
            Some(PremultipliedColorU8::from_rgba(0, 0, 0, 255).unwrap())
        );
    }
}
