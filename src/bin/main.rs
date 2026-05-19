use std::{
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec2, Vec3};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    data::Dat,
    texture::draw_texture,
    depth_field::DepthField,
    render::Render,
    env::{fps, resolution, DEFAULT_RESOLUTION},
    field::Field,
    glitter::{glitter_colors, glitter_normals, rotate_normals, view_direction, Glitter},
    glitter_io::load_glitter,
    projection::project_cloud,
    resolution::Resolution,
    rng::Rng,
    themes::{self, Cosmos, Sample},
    vec3_fmt::DatVec3,
};

const PARTICLE_COUNT: usize = 32 * 1024;
const FLOW_FIELD_RESOLUTION: Resolution = Resolution::new(128, 128);
const FLOW_FIELD_SIZE: Vec2 = Vec2::new(8.0, 8.0);

fn flow_field_from_bezier(rng: &mut Rng) -> Field<Vec2> {
    let p0 = Vec2::new(rng.next_f32_in(0.0, FLOW_FIELD_SIZE.x), rng.next_f32_in(0.0, FLOW_FIELD_SIZE.y));
    let p1 = Vec2::new(rng.next_f32_in(0.0, FLOW_FIELD_SIZE.x), rng.next_f32_in(0.0, FLOW_FIELD_SIZE.y));
    let p2 = Vec2::new(rng.next_f32_in(0.0, FLOW_FIELD_SIZE.x), rng.next_f32_in(0.0, FLOW_FIELD_SIZE.y));

    let mut field = Field::new(FLOW_FIELD_RESOLUTION, FLOW_FIELD_SIZE, Vec2::ZERO);
    let radius = 2.0_f32;
    let steps = 400;

    for y in 0..FLOW_FIELD_RESOLUTION.height as usize {
        for x in 0..FLOW_FIELD_RESOLUTION.width as usize {
            let pos = field.sample(x, y);
            let mut best_dist = f32::MAX;
            let mut best_tangent = Vec2::ZERO;
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let mt = 1.0 - t;
                let curve_pos = p0 * (mt * mt) + p1 * (2.0 * mt * t) + p2 * (t * t);
                let dist = (pos - curve_pos).length();
                if dist < best_dist {
                    best_dist = dist;
                    best_tangent = ((p1 - p0) * (2.0 * mt) + (p2 - p1) * (2.0 * t)).normalize_or_zero();
                }
            }
            if best_dist < radius {
                field.set(x, y, best_tangent);
            }
        }
    }
    field
}

const DURATION: f32 = 24.0;

fn camera_eye() -> Vec3 {
    Vec3::new(0.0, 2.35, 2.2)
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn default_depth_field(resolution: &Resolution) -> DepthField {
    DepthField {
        focus_depth: camera_eye().length(),
        blur: 1.1,
        particle_radius: 0.75 * resolution.area_scale(&DEFAULT_RESOLUTION),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let fps = fps()?;
    let dt = 1.0 / fps;
    let resolution = resolution()?;
    let mut bitmap = Bitmap::new(resolution);
    let projection = projection(bitmap.resolution());
    let frame_count = (DURATION * fps) as usize;

    let dat_path = std::env::args().nth(1).unwrap_or_else(|| "tweaks.dat".to_string());
    let dat = Dat::read(&dat_path).ok();

    let eye = dat.as_ref()
        .and_then(|d| d.get("camera", "eye"))
        .and_then(|s| s.parse::<DatVec3>().ok())
        .map(|v| v.0)
        .unwrap_or_else(camera_eye);
    let target = dat.as_ref()
        .and_then(|d| d.get("camera", "target"))
        .and_then(|s| s.parse::<DatVec3>().ok())
        .map(|v| v.0)
        .unwrap_or(Vec3::ZERO);
    let view = Mat4::look_at_rh(eye, target, Vec3::Y);

    let mut depth_field = default_depth_field(bitmap.resolution());
    if let Some(d) = &dat {
        if let Some(v) = d.get("depth_field", "focus_depth").and_then(|s| s.parse().ok()) { depth_field.focus_depth = v; }
        if let Some(v) = d.get("depth_field", "blur").and_then(|s| s.parse().ok()) { depth_field.blur = v; }
        if let Some(v) = d.get("depth_field", "particle_radius").and_then(|s| s.parse().ok()) { depth_field.particle_radius = v; }
    }

    let size = FLOW_FIELD_SIZE;
    let mut rng = Rng::new(0x1234_5678);
    let normals = glitter_normals(&mut rng, PARTICLE_COUNT);
    let field = flow_field_from_bezier(&mut rng);
    let density = themes::sample_at_resolution(Cosmos, FLOW_FIELD_RESOLUTION, size);
    let mut positions: Vec<Vec2> = (0..PARTICLE_COUNT)
        .map(|_| Vec2::new(rng.next_f32_in(0.0, size.x), rng.next_f32_in(0.0, size.y)))
        .collect();
    let background = Rgba8::from_rgb(10, 12, 18);
    let colors: Vec<Color> = positions.iter()
        .map(|p| Cosmos.sample(*p / size))
        .collect();
    let mut tumble_times = vec![0.0f32; PARTICLE_COUNT];

    let glitter = {
        let default = Glitter {
            falloff_power: 14.0,
            axis0_speed: 2.0,
            axis0: Vec3::new(0.4, 1.0, 0.3).normalize(),
            axis1: Vec3::new(0.2, 0.4, 1.0).normalize(),
            axis1_speed: 1.5,
        };
        dat.as_ref().map_or(default, |d| load_glitter(d, default))
    };
    let vdir = view_direction(view);

    for _ in 0..frame_count {
        for (position, tumble_time) in positions.iter_mut().zip(&mut tumble_times) {
            let velocity = field.interpolate(*position);
            let next = *position + velocity * dt;
            *position = Vec2::new(next.x.rem_euclid(size.x), next.y.rem_euclid(size.y));
            *tumble_time += (velocity * dt).length();
        }
        let offset = size * 0.5;
        let cloud: Vec<Vec3> = positions.iter()
            .map(|p| { let p = *p - offset; Vec3::new(p.x, 0.0, p.y) })
            .collect();

        let rotated = rotate_normals(&normals, &tumble_times, glitter);
        let render_colors = glitter_colors(&colors, &rotated, vdir, glitter);

        bitmap.fill(background);
        draw_texture(&mut bitmap, &density, projection, view);
        let projected = project_cloud(&bitmap, &cloud, projection, view);
        depth_field.render(&mut bitmap, &projected, &render_colors);
        output.write_all(bitmap.data())?;
        output.flush()?;
    }

    Ok(())
}
