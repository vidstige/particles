use eframe::egui::{self, TextureHandle, TextureOptions};
use glam::{Mat4, Vec2, Vec3, Vec4};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    depth_field::DepthField,
    downscaled::Downscaled,
    env::DEFAULT_RESOLUTION,
    field::Field,
    fluid::{advect, advect_scalar, project_incompressible},
    glow::Glow,
    glitter::{glitter_colors, glitter_normals, rotate_normals, tumble_rotation, view_direction, Glitter},
    projection::project_cloud,
    render::Render,
    resolution::Resolution,
    rng::Rng,
    simplex::SimplexNoise,
    themes,
};

const DURATION: f32 = 24.0;
const PARTICLE_COUNT: usize = 16 * 1024;
const GLITTER_TUMBLE_SPEED: f32 = 2.0;
const GLITTER_PRECESSION_SPEED: f32 = 1.5;
const FLOW_FIELD_RESOLUTION: Resolution = Resolution::new(128, 128);
const FLOW_FIELD_SIZE: Vec2 = Vec2::new(8.0, 8.0);

fn format_time(seconds: f32) -> String {
    format!("{seconds:05.2}s")
}

fn image_size(bitmap: &Bitmap, available: egui::Vec2) -> egui::Vec2 {
    let size = egui::Vec2::new(bitmap.width() as f32, bitmap.height() as f32);
    let scale = (available.x / size.x).min(available.y / size.y);
    size * scale
}

// Gaussian vortex velocity at a point, derived from stream function ψ = strength·exp(-r²/2R²).
fn vortex_vel(delta: Vec2, strength: f32, radius: f32) -> Vec2 {
    let exp_falloff = (-delta.length_squared() / (2.0 * radius * radius)).exp();
    Vec2::new(
        -strength * delta.y / (radius * radius) * exp_falloff,
         strength * delta.x / (radius * radius) * exp_falloff,
    )
}

fn flow_field_with_vortex() -> Field<Vec2> {
    let mut field = Field::new(FLOW_FIELD_RESOLUTION, FLOW_FIELD_SIZE, Vec2::ZERO);
    let center = FLOW_FIELD_SIZE * 0.25;
    let perp_dir = Vec2::new(-1.0, 1.0);
    let spacing = 0.6;
    let center1 = center + perp_dir * spacing;
    let center2 = center - perp_dir * spacing;
    for y in 0..FLOW_FIELD_RESOLUTION.height as usize {
        for x in 0..FLOW_FIELD_RESOLUTION.width as usize {
            let pos = field.sample(x, y);
            let vel = vortex_vel(pos - center1,  1.0, 0.5)
                    + vortex_vel(pos - center2, -1.0, 0.5);
            field.set(x, y, vel);
        }
    }
    field
}

fn flow_field_from_simplex() -> Field<Vec2> {
    let width = FLOW_FIELD_RESOLUTION.width as usize;
    let height = FLOW_FIELD_RESOLUTION.height as usize;
    let mut field = Field::new(FLOW_FIELD_RESOLUTION, FLOW_FIELD_SIZE, Vec2::ZERO);
    let x_noise = SimplexNoise::new(0x1f2e_3d4c, 1.3, 1.0);
    let y_noise = SimplexNoise::new(0x5a69_7887, 1.3, 1.0);
    for y in 0..height {
        for x in 0..width {
            let point = field.sample(x, y) / FLOW_FIELD_SIZE;
            field.set(x, y, Vec2::new(
                x_noise.sample(Vec4::new(point.x, point.y, 0.17, 0.0)),
                y_noise.sample(Vec4::new(point.x, point.y, 3.41, 0.0)),
            ));
        }
    }
    let mut pressure = Field::new(FLOW_FIELD_RESOLUTION, FLOW_FIELD_SIZE, 0.0f32);
    project_incompressible(&mut field, &mut pressure, 160);
    field
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

#[derive(Clone, Copy, Debug)]
struct Settings {
    background: Rgba8,
    depth_field: DepthField,
    glitter: Glitter,
    glow: Glow,
}

impl Settings {
    fn for_resolution(resolution: &Resolution) -> Self {
        Self {
            background: Rgba8::from_rgb(16, 16, 48),
            depth_field: DepthField {
                focus_depth: 7.0,
                blur: 1.0,
                particle_radius: resolution.area_scale(&DEFAULT_RESOLUTION),
            },
            glitter: Glitter {
                falloff_power: 14.0,
                tumble_speed: GLITTER_TUMBLE_SPEED,
                tumble_axis: Vec3::new(0.4, 1.0, 0.3).normalize(),
                precession_axis: Vec3::new(0.2, 0.4, 1.0).normalize(),
                precession_speed: GLITTER_PRECESSION_SPEED,
            },
            glow: Glow {
                softener: 0.40,
                radius: 4.0,
            },
        }
    }

    fn glitter_speed(&self) -> f32 {
        self.glitter.tumble_speed
    }

    fn set_glitter_speed(&mut self, speed: f32) {
        self.glitter.tumble_speed = speed;
        self.glitter.precession_speed = speed * GLITTER_PRECESSION_SPEED / GLITTER_TUMBLE_SPEED;
    }
}

fn density_field() -> Field<Color> {
    let mut field = Field::new(FLOW_FIELD_RESOLUTION, FLOW_FIELD_SIZE, Color::BLACK);
    for y in 0..FLOW_FIELD_RESOLUTION.height as usize {
        for x in 0..FLOW_FIELD_RESOLUTION.width as usize {
            let pos = field.sample(x, y);
            field.set(x, y, themes::aurora(pos / FLOW_FIELD_SIZE));
        }
    }
    field
}

struct Scene {
    normals: Vec<glam::Vec3>,
    flow_field: Field<Vec2>,
    pressure: Field<f32>,
    density: Field<Color>,
    density_positions: Vec<Vec3>,
    flow_positions: Vec<Vec2>,
    flow_colors: Vec<Color>,
}

impl Scene {
    fn new() -> Self {
        let mut rng = Rng::new(0x1234_5678);
        let normals = glitter_normals(&mut rng, PARTICLE_COUNT);
        let flow_field = flow_field_with_vortex();
        let pressure = Field::new(FLOW_FIELD_RESOLUTION, FLOW_FIELD_SIZE, 0.0f32);
        let density = density_field();
        let offset = FLOW_FIELD_SIZE * 0.5;
        let density_positions: Vec<Vec3> = (0..FLOW_FIELD_RESOLUTION.height as usize)
            .flat_map(|y| (0..FLOW_FIELD_RESOLUTION.width as usize).map(move |x| {
                let pos = Vec2::new(
                    x as f32 * FLOW_FIELD_SIZE.x / FLOW_FIELD_RESOLUTION.width as f32,
                    y as f32 * FLOW_FIELD_SIZE.y / FLOW_FIELD_RESOLUTION.height as f32,
                ) - offset;
                Vec3::new(pos.x, 0.0, pos.y)
            }))
            .collect();
        let flow_positions: Vec<Vec2> = (0..PARTICLE_COUNT)
            .map(|_| Vec2::new(
                rng.next_f32_in(0.0, FLOW_FIELD_SIZE.x),
                rng.next_f32_in(0.0, FLOW_FIELD_SIZE.y),
            ))
            .collect();
        let flow_colors = flow_positions
            .iter()
            .map(|p| themes::aurora(*p / FLOW_FIELD_SIZE))
            .collect();

        Self { normals, flow_field, pressure, density, density_positions, flow_positions, flow_colors }
    }

    fn advance(&mut self, dt: f32) {
        self.flow_field = advect(&self.flow_field, dt);
        project_incompressible(&mut self.flow_field, &mut self.pressure, 20);
        self.density = advect_scalar(&self.density, &self.flow_field, dt);
        for position in &mut self.flow_positions {
            let next = *position + self.flow_field.interpolate(*position) * dt;
            *position = Vec2::new(
                next.x.rem_euclid(FLOW_FIELD_SIZE.x),
                next.y.rem_euclid(FLOW_FIELD_SIZE.y),
            );
        }
    }

    fn render(&self, bitmap: &mut Bitmap, time: f32, settings: Settings, view: Mat4) {
        bitmap.fill(settings.background);

        let offset = FLOW_FIELD_SIZE * 0.5;
        let bloom = Downscaled { inner: settings.glow, scale: 4 };

        let density_colors = self.density.values.clone();
        let density_projected = project_cloud(bitmap, &self.density_positions, projection(bitmap.resolution()), view);
        bloom.render(bitmap, &density_projected, &density_colors);

        let positions: Vec<Vec3> = self.flow_positions
            .iter()
            .map(|p| { let p = *p - offset; Vec3::new(p.x, 0.0, p.y) })
            .collect();
        let projected = project_cloud(bitmap, &positions, projection(bitmap.resolution()), view);
        let rotated_normals = rotate_normals(&self.normals, tumble_rotation(time, settings.glitter));
        let vdir = view_direction(view);
        let colors = glitter_colors(&self.flow_colors, &rotated_normals, vdir, settings.glitter);
        bloom.render(bitmap, &projected, &self.flow_colors);
        settings.depth_field.render(bitmap, &projected, &colors);
    }
}

struct Camera {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Camera {
    fn new(eye: Vec3, target: Vec3) -> Self {
        let offset = eye - target;
        let distance = offset.length();
        Self {
            target,
            yaw: offset.z.atan2(offset.x),
            pitch: (offset.y / distance).asin(),
            distance,
        }
    }

    fn eye(&self) -> Vec3 {
        let orbit = Vec3::new(
            self.pitch.cos() * self.yaw.cos(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.sin(),
        );
        self.target + orbit * self.distance
    }

    fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }

    fn orbit(&mut self, delta: egui::Vec2) {
        self.yaw -= delta.x * 0.01;
        self.pitch = (self.pitch + delta.y * 0.01).clamp(-1.4, 1.4);
    }

    fn pan(&mut self, delta: egui::Vec2, viewport: egui::Rect) {
        let forward = (self.target - self.eye()).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward).normalize();
        let scale = self.distance / viewport.height().max(1.0);
        self.target += (-delta.x * right + delta.y * up) * scale;
    }

    fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance * (-delta * 0.001).exp()).clamp(0.5, 12.0);
    }
}

struct TweakApp {
    scene: Scene,
    settings: Settings,
    bitmap: Bitmap,
    texture: Option<TextureHandle>,
    camera: Camera,
    time: f32,
    playing: bool,
    last_ui_time: Option<f64>,
}

impl eframe::App for TweakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|input| input.time);
        if let Some(last_ui_time) = self.last_ui_time {
            if self.playing {
                let dt = (now - last_ui_time) as f32;
                self.time = (self.time + dt).rem_euclid(DURATION);
                self.scene.advance(dt);
                ctx.request_repaint();
            }
        }
        self.last_ui_time = Some(now);

        egui::TopBottomPanel::top("transport").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let label = if self.playing { "Pause" } else { "Play" };
                if ui.button(label).clicked() {
                    self.playing = !self.playing;
                    self.last_ui_time = Some(now);
                    if self.playing {
                        ctx.request_repaint();
                    }
                }
                ui.add(
                    egui::Slider::new(&mut self.time, 0.0..=DURATION)
                        .show_value(false)
                        .clamping(egui::SliderClamping::Always),
                );
                ui.label(format!("{} / {}", format_time(self.time), format_time(DURATION)));
            });
        });

        egui::SidePanel::left("tweaks")
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Render");
                ui.add(
                    egui::Slider::new(&mut self.settings.depth_field.blur, 0.0..=12.0)
                        .text("Blur"),
                );
                ui.add(
                    egui::Slider::new(&mut self.settings.depth_field.focus_depth, 0.1..=12.0)
                        .text("Focus depth"),
                );
                ui.add(
                    egui::Slider::new(&mut self.settings.depth_field.particle_radius, 0.25..=8.0)
                        .text("Particle radius"),
                );
                ui.add(
                    egui::Slider::new(&mut self.settings.glitter.falloff_power, 1.0..=32.0)
                        .text("Glitter fall-off"),
                );
                let mut glitter_speed = self.settings.glitter_speed();
                if ui
                    .add(egui::Slider::new(&mut glitter_speed, 0.0..=16.0).text("Glitter speed"))
                    .changed()
                {
                    self.settings.set_glitter_speed(glitter_speed);
                }

                ui.separator();
                ui.heading("Glow");
                ui.add(
                    egui::Slider::new(&mut self.settings.glow.softener, 0.0..=1.0)
                        .text("Softener"),
                );
                ui.add(
                    egui::Slider::new(&mut self.settings.glow.radius, 0.25..=16.0)
                        .text("Radius"),
                );

                ui.separator();
                ui.heading("Camera");
                let eye = self.camera.eye();
                let t = self.camera.target;
                let fmt = |v: Vec3| format!("Vec3::new({:.2}, {:.2}, {:.2})", v.x, v.y, v.z);
                ui.add(egui::Label::new(egui::RichText::new(fmt(eye)).monospace()).selectable(true));
                ui.add(egui::Label::new(egui::RichText::new(fmt(t)).monospace().weak()).selectable(true));
            });

        self.scene.render(&mut self.bitmap, self.time, self.settings, self.camera.view());

        let image = egui::ColorImage::from_rgba_unmultiplied(
            [self.bitmap.width() as usize, self.bitmap.height() as usize],
            self.bitmap.data(),
        );
        if let Some(texture) = &mut self.texture {
            texture.set(image, TextureOptions::LINEAR);
        } else {
            self.texture = Some(ctx.load_texture("particles", image, TextureOptions::LINEAR));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                let size = image_size(&self.bitmap, ui.available_size());
                let response = ui
                    .centered_and_justified(|ui| {
                        ui.add(
                            egui::Image::new((texture.id(), size))
                                .sense(egui::Sense::click_and_drag()),
                        )
                    })
                    .inner;
                let (pointer_delta, modified) =
                    ctx.input(|input| (input.pointer.delta(), input.modifiers.any()));
                let pan_with_primary =
                    response.dragged_by(egui::PointerButton::Primary) && modified;

                if response.dragged_by(egui::PointerButton::Primary) && !pan_with_primary {
                    self.camera.orbit(pointer_delta);
                    ctx.request_repaint();
                }
                if pan_with_primary
                    || response.dragged_by(egui::PointerButton::Secondary)
                    || response.dragged_by(egui::PointerButton::Middle)
                {
                    self.camera.pan(pointer_delta, response.rect);
                    ctx.request_repaint();
                }

                let (scroll_delta, modified) = ctx.input(|input| {
                    if response.hovered() {
                        (input.smooth_scroll_delta, input.modifiers.any())
                    } else {
                        (egui::Vec2::ZERO, false)
                    }
                });
                if scroll_delta != egui::Vec2::ZERO {
                    if modified {
                        self.camera.zoom(scroll_delta.y);
                    } else {
                        self.camera.pan(scroll_delta, response.rect);
                    }
                    ctx.request_repaint();
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    let resolution = DEFAULT_RESOLUTION;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([
            resolution.width as f32 + 240.0,
            resolution.height as f32 + 56.0,
        ]),
        ..Default::default()
    };
    eframe::run_native(
        "gel tweak",
        options,
        Box::new(|_cc| {
            Ok(Box::new(TweakApp {
                scene: Scene::new(),
                settings: Settings::for_resolution(&resolution),
                bitmap: Bitmap::new(resolution),
                texture: None,
                camera: Camera::new(Vec3::new(0.0, 7.0, 0.5), Vec3::ZERO),
                time: 0.0,
                playing: true,
                last_ui_time: None,
            }))
        }),
    )
}
