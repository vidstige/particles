use std::{
    error::Error,
    io::{self, Write},
};

use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    data::Dat,
    npy::read_npz,
    depth_field::DepthField,
    render::Render,
    env::{fps, resolution, DEFAULT_RESOLUTION},
    field::Field,
    projection::project_cloud,
    resolution::Resolution,
    vec3_fmt::DatVec3,
};

const DURATION: f32 = 24.0;

fn camera_eye() -> Vec3 {
    Vec3::new(0.0, 2.35, 2.2)
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

fn draw_density_texture(bitmap: &mut Bitmap, density: &Field<Color>, projection: Mat4, view: Mat4) {
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

    let fields_path = std::env::args().nth(2).unwrap_or_else(|| "fields.npz".to_string());
    let npz = read_npz(&fields_path)?;

    let ws = npz.get("world_size").ok_or("world_size missing from fields.npz")?;
    let size = Vec2::new(ws.data[0], ws.data[1]);

    let vel = npz.get("velocity").ok_or("velocity missing from fields.npz")?;
    let h = vel.shape[0] as u32;
    let w = vel.shape[1] as u32;
    let mut field = Field::new(Resolution::new(w, h), size, Vec2::ZERO);
    for (v, chunk) in field.values.iter_mut().zip(vel.data.chunks(2)) {
        *v = Vec2::new(chunk[0], chunk[1]);
    }

    let mut positions: Vec<Vec2> = npz.get("positions")
        .ok_or("positions missing from fields.npz")?
        .data.chunks(2).map(|c| Vec2::new(c[0], c[1])).collect();

    let background = Rgba8::from_rgb(10, 12, 18);
    let colors: Vec<Color> = npz.get("colors")
        .ok_or("colors missing from fields.npz")?
        .data.chunks(3).map(|c| Color::new(c[0], c[1], c[2])).collect();

    let density: Field<Color> = {
        let den = npz.get("density").ok_or("density missing from fields.npz")?;
        let h = den.shape[0] as u32;
        let w = den.shape[1] as u32;
        let mut f = Field::new(Resolution::new(w, h), size, Color::BLACK);
        for (c, chunk) in f.values.iter_mut().zip(den.data.chunks(3)) {
            *c = Color::new(chunk[0], chunk[1], chunk[2]);
        }
        f
    };

    for _ in 0..frame_count {
        for position in &mut positions {
            let next = *position + field.interpolate(*position) * dt;
            *position = Vec2::new(next.x.rem_euclid(size.x), next.y.rem_euclid(size.y));
        }
        let offset = size * 0.5;
        let cloud: Vec<Vec3> = positions.iter()
            .map(|p| { let p = *p - offset; Vec3::new(p.x, 0.0, p.y) })
            .collect();

        bitmap.fill(background);
        draw_density_texture(&mut bitmap, &density, projection, view);
        let projected = project_cloud(&bitmap, &cloud, projection, view);
        depth_field.render(&mut bitmap, &projected, &colors);
        output.write_all(bitmap.data())?;
        output.flush()?;
    }

    Ok(())
}
