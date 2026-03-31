use glam::{Vec2, Vec3};

use crate::{color::Color, resolution::Resolution};
use tiny_skia::{BlendMode, Color as TinyColor, FilterQuality, Pixmap, PixmapPaint, Transform};

const FOREGROUND_ALPHA: u8 = 96;
const GLOW_DOWNSAMPLE: u32 = 2;
const PARTICLE_RADIUS: f32 = 1.0;
const GLOW_INTENSITY: f32 = 0.85;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: TinyColor,
    pub foreground: TinyColor,
}

#[derive(Clone, Copy, Debug)]
pub struct DepthField {
    pub focus_depth: f32,
    pub blur: f32,
}

fn area(radius: f32) -> f32 {
    std::f32::consts::PI * radius.max(f32::MIN_POSITIVE).powi(2)
}

fn from_pixmap(pixmap: &Pixmap) -> Resolution {
    Resolution::new(pixmap.width(), pixmap.height())
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
            (color * weight).add_to_pixel::<WRITE_ALPHA>(&mut pixels[index..index + 4]);
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

pub fn default_theme() -> Theme {
    Theme {
        background: TinyColor::from_rgba8(14, 14, 18, 255),
        foreground: TinyColor::from_rgba8(214, 92, 255, FOREGROUND_ALPHA),
    }
}

pub fn render_cloud(
    pixmap: &mut Pixmap,
    positions: &[Option<Vec3>],
    colors: &[Color],
    depth_field: DepthField,
) {
    assert_eq!(positions.len(), colors.len());

    let resolution = from_pixmap(pixmap);
    let blur = depth_field.blur;
    let (glow_width, glow_height) = glow_dimensions(&resolution);
    let mut glow = Pixmap::new(glow_width, glow_height).unwrap();
    glow.fill(TinyColor::TRANSPARENT);
    let pixels = pixmap.data_mut();

    for (particle, color) in positions.iter().copied().zip(colors.iter().copied()) {
        let Some(particle) = particle else {
            continue;
        };
        let focal_distance = (particle.z - depth_field.focus_depth).abs();
        let radius = PARTICLE_RADIUS + blur * focal_distance;
        let color = color * (area(PARTICLE_RADIUS) / area(radius));

        if radius < GLOW_DOWNSAMPLE as f32 {
            splat_sharp(
                pixels,
                resolution.width,
                resolution.height,
                particle.truncate(),
                radius,
                color,
            );
        } else {
            splat_glow(
                &mut glow,
                particle.truncate() / GLOW_DOWNSAMPLE as f32,
                radius / GLOW_DOWNSAMPLE as f32,
                color,
            );
        }
    }

    composite_glow(pixmap, &glow);
}
