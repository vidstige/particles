use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::{color::Color, resolution::Resolution};
use tiny_skia::{BlendMode, Color as TinyColor, FilterQuality, Pixmap, PixmapPaint, Transform};

const FOREGROUND_ALPHA: u8 = 96;
const GLOW_DOWNSAMPLE: u32 = 2;
const DEPTH_BLUR_SCALE: f32 = 4.0;
const SHARP_RADIUS: f32 = 1.25;
const SHARP_INTENSITY: f32 = 1.35;

const GLOW_RADIUS: f32 = 6.5;
const GLOW_INTENSITY: f32 = 0.65;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: TinyColor,
    pub foreground: TinyColor,
}

fn blur_amount(depth: f32, focus_depth: f32, depth_span: f32) -> f32 {
    (depth - focus_depth).abs() * DEPTH_BLUR_SCALE / depth_span.max(f32::MIN_POSITIVE)
}

fn focus_amount(blur: f32) -> f32 {
    (1.0 - blur).clamp(0.0, 1.0)
}

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

fn glow_dimensions(resolution: &Resolution) -> (u32, u32) {
    let width = resolution.width.div_ceil(GLOW_DOWNSAMPLE).max(1);
    let height = resolution.height.div_ceil(GLOW_DOWNSAMPLE).max(1);
    (width, height)
}

fn pixmap_bounds(width: u32, height: u32, center: Vec2, radius: f32) -> (i32, i32, i32, i32) {
    let min_x = (center.x - radius).floor().max(0.0) as i32;
    let max_x = (center.x + radius)
        .ceil()
        .min(width.saturating_sub(1) as f32) as i32;
    let min_y = (center.y - radius).floor().max(0.0) as i32;
    let max_y = (center.y + radius)
        .ceil()
        .min(height.saturating_sub(1) as f32) as i32;
    (min_x, max_x, min_y, max_y)
}

fn add_color<const WRITE_ALPHA: bool>(pixel: &mut [u8], color: Color) {
    pixel[0] = (pixel[0] as f32 + color.red).clamp(0.0, 255.0) as u8;
    pixel[1] = (pixel[1] as f32 + color.green).clamp(0.0, 255.0) as u8;
    pixel[2] = (pixel[2] as f32 + color.blue).clamp(0.0, 255.0) as u8;
    if WRITE_ALPHA {
        pixel[3] = pixel[0].max(pixel[1]).max(pixel[2]);
    }
}

fn splat<const WRITE_ALPHA: bool>(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    center: Vec2,
    radius: f32,
    color: Color,
) {
    let (min_x, max_x, min_y, max_y) = pixmap_bounds(width, height, center, radius);
    let stride = width as usize * 4;
    let radius2 = radius * radius;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let offset = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - center;
            let weight = (1.0 - offset.length_squared() / radius2.max(f32::MIN_POSITIVE))
                .max(0.0)
                .powi(2);
            let index = y as usize * stride + x as usize * 4;
            add_color::<WRITE_ALPHA>(&mut pixels[index..index + 4], color * weight);
        }
    }
}

fn splat_glow(glow: &mut Pixmap, center: Vec2, radius: f32, color: Color) {
    let width = glow.width();
    let height = glow.height();
    splat::<true>(glow.data_mut(), width, height, center, radius, color);
}

fn splat_sharp(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    center: Vec2,
    radius: f32,
    color: Color,
) {
    splat::<false>(pixels, width, height, center, radius, color);
}

fn composite_glow(pixmap: &mut Pixmap, glow: &Pixmap) {
    let paint = PixmapPaint {
        opacity: GLOW_INTENSITY,
        blend_mode: BlendMode::Plus,
        quality: FilterQuality::Bilinear,
    };

    pixmap.draw_pixmap(
        0,
        0,
        glow.as_ref(),
        &paint,
        Transform::from_scale(GLOW_DOWNSAMPLE as f32, GLOW_DOWNSAMPLE as f32),
        None,
    );
}

fn focus_depth(depth_min: f32, depth_max: f32) -> f32 {
    depth_min + (depth_max - depth_min) * 0.5
}

pub fn default_theme() -> Theme {
    Theme {
        background: TinyColor::from_rgba8(14, 14, 18, 255),
        foreground: TinyColor::from_rgba8(214, 92, 255, FOREGROUND_ALPHA),
    }
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

pub fn render_cloud(pixmap: &mut Pixmap, positions: &[Option<Vec3>], colors: &[Color]) {
    assert_eq!(positions.len(), colors.len());

    let resolution = from_pixmap(pixmap);
    let depths: Vec<_> = positions
        .iter()
        .filter_map(|particle| particle.map(|particle| particle.z))
        .collect();
    if depths.is_empty() {
        return;
    }

    let depth_min = depths.iter().copied().reduce(f32::min).unwrap();
    let depth_max = depths.iter().copied().reduce(f32::max).unwrap();
    let depth_span = (depth_max - depth_min).max(1.0);
    let focus_depth = focus_depth(depth_min, depth_max);
    let (glow_width, glow_height) = glow_dimensions(&resolution);
    let mut glow = Pixmap::new(glow_width, glow_height).unwrap();
    glow.fill(TinyColor::from_rgba8(0, 0, 0, 0));
    let pixels = pixmap.data_mut();

    for (particle, color) in positions.iter().copied().zip(colors.iter().copied()) {
        let Some(particle) = particle else {
            continue;
        };
        let depth_t = (particle.z - depth_min) / depth_span;
        let blur = blur_amount(particle.z, focus_depth, depth_span);
        let focus = focus_amount(blur);
        let near_weight = 1.15 - depth_t * 0.35;
        let sharp_energy = near_weight * focus.powi(2) * SHARP_INTENSITY;
        let glow_energy = near_weight * (0.05 + blur * 0.5);
        let glow_radius = 0.5 + GLOW_RADIUS * blur;

        splat_sharp(
            pixels,
            resolution.width,
            resolution.height,
            particle.truncate(),
            SHARP_RADIUS,
            color * sharp_energy,
        );

        splat_glow(
            &mut glow,
            particle.truncate() / GLOW_DOWNSAMPLE as f32,
            glow_radius / GLOW_DOWNSAMPLE as f32,
            color * glow_energy,
        );
    }

    composite_glow(pixmap, &glow);
}

#[cfg(test)]
mod tests {
    use super::{blur_amount, focus_amount};

    #[test]
    fn blur_amount_is_zero_at_focus_and_symmetric_around_it() {
        assert_eq!(blur_amount(4.0, 4.0, 2.0), 0.0);
        assert_eq!(blur_amount(3.5, 4.0, 2.0), 1.0);
        assert_eq!(blur_amount(4.5, 4.0, 2.0), 1.0);
    }

    #[test]
    fn focus_amount_fades_out_once_blur_reaches_one() {
        assert_eq!(focus_amount(0.0), 1.0);
        assert_eq!(focus_amount(0.25), 0.75);
        assert_eq!(focus_amount(1.0), 0.0);
        assert_eq!(focus_amount(2.0), 0.0);
    }
}
