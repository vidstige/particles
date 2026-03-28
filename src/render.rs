use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::resolution::Resolution;
use tiny_skia::{Color, Pixmap};

const FOREGROUND_ALPHA: u8 = 96;
const GLOW_DOWNSAMPLE: u32 = 2;
const FOCUS_SWEEP_SPEED: f32 = 0.35;
const FOCUS_RANGE_SCALE: f32 = 0.16;
const FOCUS_RANGE_BIAS: f32 = 0.18;
const GLOW_RADIUS: f32 = 6.5;
const GLOW_INTENSITY: f32 = 0.65;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
}

#[derive(Clone, Copy, Debug)]
struct ProjectedParticle {
    screen: Vec2,
    depth: f32,
}

fn foreground_rgb(theme: &Theme) -> Vec3 {
    let alpha = theme.foreground.alpha() * 255.0;
    Vec3::new(
        theme.foreground.red(),
        theme.foreground.green(),
        theme.foreground.blue(),
    ) * alpha
}

fn circle_of_confusion(depth: f32, focus_depth: f32, focus_range: f32) -> f32 {
    (depth - focus_depth).abs() / focus_range.max(f32::MIN_POSITIVE)
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
) -> Option<ProjectedParticle> {
    let view_point = view.transform_point3(point);
    let screen = project_position(point, resolution, view_projection)?;

    Some(ProjectedParticle {
        screen,
        depth: -view_point.z,
    })
}

fn circular_falloff(offset: Vec2, radius: f32) -> f32 {
    let distance2 = offset.length_squared();
    let radius2 = radius * radius;
    if distance2 >= radius2 {
        return 0.0;
    }

    let weight = 1.0 - distance2 / radius2.max(f32::MIN_POSITIVE);
    weight * weight
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

fn add_rgb(pixel: &mut [u8], color: Vec3) {
    pixel[0] = (pixel[0] as f32 + color.x).clamp(0.0, 255.0) as u8;
    pixel[1] = (pixel[1] as f32 + color.y).clamp(0.0, 255.0) as u8;
    pixel[2] = (pixel[2] as f32 + color.z).clamp(0.0, 255.0) as u8;
}

fn glow_dimensions(resolution: &Resolution) -> (u32, u32) {
    let width = resolution.width.div_ceil(GLOW_DOWNSAMPLE).max(1);
    let height = resolution.height.div_ceil(GLOW_DOWNSAMPLE).max(1);
    (width, height)
}

fn glow_bounds(width: u32, height: u32, center: Vec2, radius: f32) -> (i32, i32, i32, i32) {
    pixmap_bounds(width, height, center, radius)
}

fn splat_glow(glow: &mut [Vec3], width: u32, height: u32, center: Vec2, radius: f32, color: Vec3) {
    let (min_x, max_x, min_y, max_y) = glow_bounds(width, height, center, radius);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let offset = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - center;
            let weight = circular_falloff(offset, radius);
            if weight == 0.0 {
                continue;
            }

            let index = y as usize * width as usize + x as usize;
            glow[index] += color * weight;
        }
    }
}

fn glow_pixel(glow: &[Vec3], width: u32, height: u32, x: i32, y: i32) -> Vec3 {
    let x = x.clamp(0, width as i32 - 1) as usize;
    let y = y.clamp(0, height as i32 - 1) as usize;
    glow[y * width as usize + x]
}

fn sample_glow(glow: &[Vec3], width: u32, height: u32, position: Vec2) -> Vec3 {
    let x0 = position.x.floor() as i32;
    let y0 = position.y.floor() as i32;
    let tx = position.x - x0 as f32;
    let ty = position.y - y0 as f32;
    let top = glow_pixel(glow, width, height, x0, y0)
        .lerp(glow_pixel(glow, width, height, x0 + 1, y0), tx);
    let bottom = glow_pixel(glow, width, height, x0, y0 + 1)
        .lerp(glow_pixel(glow, width, height, x0 + 1, y0 + 1), tx);
    top.lerp(bottom, ty)
}

fn composite_glow(pixmap: &mut Pixmap, glow: &[Vec3], glow_width: u32, glow_height: u32) {
    let width = pixmap.width();
    let height = pixmap.height();
    let pixels = pixmap.data_mut();
    let stride = width as usize * 4;
    let scale = 1.0 / GLOW_DOWNSAMPLE as f32;

    for y in 0..height {
        for x in 0..width {
            let sample = sample_glow(
                glow,
                glow_width,
                glow_height,
                Vec2::new(
                    (x as f32 + 0.5) * scale - 0.5,
                    (y as f32 + 0.5) * scale - 0.5,
                ),
            ) * GLOW_INTENSITY;
            if sample == Vec3::ZERO {
                continue;
            }

            let index = y as usize * stride + x as usize * 4;
            add_rgb(&mut pixels[index..index + 4], sample);
        }
    }
}

fn focus_depth(depth_min: f32, depth_max: f32, time: f32) -> f32 {
    let sweep = 0.5 + 0.25 * (time * FOCUS_SWEEP_SPEED).sin();
    depth_min + (depth_max - depth_min) * sweep
}

pub fn default_theme() -> Theme {
    Theme {
        background: Color::from_rgba8(14, 14, 18, 255),
        foreground: Color::from_rgba8(214, 92, 255, FOREGROUND_ALPHA),
    }
}

pub fn render_cloud(
    pixmap: &mut Pixmap,
    positions: &[Vec3],
    resolution: &Resolution,
    projection: Mat4,
    view: Mat4,
    theme: &Theme,
    time: f32,
) {
    let view_projection = projection * view;
    let mut particles: Vec<_> = positions
        .iter()
        .filter_map(|point| project_particle(*point, resolution, view_projection, view))
        .collect();

    particles.sort_by(|left, right| left.depth.total_cmp(&right.depth));

    let depth_min = particles.first().unwrap().depth;
    let depth_max = particles.last().unwrap().depth;
    let depth_span = (depth_max - depth_min).max(1.0);
    let focus_depth = focus_depth(depth_min, depth_max, time);
    let focus_range = depth_span * FOCUS_RANGE_SCALE + FOCUS_RANGE_BIAS;
    let tint = foreground_rgb(theme);
    let (glow_width, glow_height) = glow_dimensions(resolution);
    let mut glow = vec![Vec3::ZERO; glow_width as usize * glow_height as usize];

    for particle in &particles {
        let depth_t = (particle.depth - depth_min) / depth_span;
        let coc = circle_of_confusion(particle.depth, focus_depth, focus_range);
        let near_weight = 1.15 - depth_t * 0.35;
        let glow_energy = near_weight * (0.2 + coc * 0.5);
        let glow_radius = 1.0 + GLOW_RADIUS * coc;

        splat_glow(
            &mut glow,
            glow_width,
            glow_height,
            particle.screen / GLOW_DOWNSAMPLE as f32,
            glow_radius / GLOW_DOWNSAMPLE as f32,
            tint * glow_energy,
        );
    }

    composite_glow(pixmap, &glow, glow_width, glow_height);
}

#[cfg(test)]
mod tests {
    use super::circle_of_confusion;

    #[test]
    fn circle_of_confusion_is_zero_at_focus_and_symmetric_around_it() {
        assert_eq!(circle_of_confusion(4.0, 4.0, 0.5), 0.0);
        assert_eq!(circle_of_confusion(3.5, 4.0, 0.5), 1.0);
        assert_eq!(circle_of_confusion(4.5, 4.0, 0.5), 1.0);
    }
}
