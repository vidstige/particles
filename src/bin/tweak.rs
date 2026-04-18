use eframe::egui::{self, TextureHandle, TextureOptions};
use glam::{Mat4, Vec2, Vec3, Vec4};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    depth_field::{DepthField, Theme},
    glow::Glow,
    render::Render,
    distribution::{collect, Uniform3},
    env::DEFAULT_RESOLUTION,
    field::Field,
    fluid::{advect, project_incompressible},
    gerstner::{displaced_position, surface_grid, GerstnerWave},
    glitter::{
        glitter_colors, glitter_normals, rotate_normals, tumble_rotation, view_direction, Glitter,
    },
    projection::project_cloud,
    resolution::Resolution,
    rng::Rng,
    simplex::SimplexNoise,
};

const DURATION: f32 = 24.0;
const PARTICLE_COUNT: usize = 8 * 1024;
const SIMPLEX_SCALE: f32 = 0.45;
const SIMPLEX_SPEED: f32 = 0.125;
const GLITTER_TUMBLE_SPEED: f32 = 2.0;
const GLITTER_PRECESSION_SPEED: f32 = 1.5;
const FLOW_FIELD_RESOLUTION: Resolution = Resolution::new(128, 128);
const FLOW_FIELD_SIZE: Vec2 = Vec2::new(8.0, 8.0);
const FLOW_MEAN_SPEED: f32 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq)]
enum FieldMode {
    Simplex,
    Incompressible,
    Gerstner,
}

fn format_time(seconds: f32) -> String {
    format!("{seconds:05.2}s")
}

fn image_size(bitmap: &Bitmap, available: egui::Vec2) -> egui::Vec2 {
    let size = egui::Vec2::new(bitmap.width() as f32, bitmap.height() as f32);
    let scale = (available.x / size.x).min(available.y / size.y);
    size * scale
}

fn simplex_field() -> [SimplexNoise; 3] {
    [
        SimplexNoise::new(0x1f2e_3d4c, 1.4, 1.0),
        SimplexNoise::new(0x2a39_4857, 1.4, 1.0),
        SimplexNoise::new(0x6574_8392, 1.4, 1.0),
    ]
}

fn simplex_offset(field: &[SimplexNoise; 3], point: Vec3, w: f32) -> Vec3 {
    Vec3::new(
        field[0].sample(point.extend(w)),
        field[1].sample(point.extend(w)),
        field[2].sample(point.extend(w)),
    )
}

fn water_waves() -> [GerstnerWave; 5] {
    [
        GerstnerWave::new(Vec2::new(1.0, 0.1), 0.11, 2.8, 0.55, 0.75, 0.0),
        GerstnerWave::new(Vec2::new(0.2, 1.0), 0.08, 1.9, 0.8, 0.7, 0.8),
        GerstnerWave::new(Vec2::new(-0.9, 0.4), 0.05, 1.1, 1.1, 0.55, 1.7),
        GerstnerWave::new(Vec2::new(0.7, -0.6), 0.04, 0.75, 1.4, 0.45, 2.2),
        GerstnerWave::new(Vec2::new(-0.3, -1.0), 0.03, 0.5, 1.8, 0.35, 0.5),
    ]
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
            field.set(
                x,
                y,
                Vec2::new(
                    x_noise.sample(Vec4::new(point.x, point.y, 0.17, 0.0)),
                    y_noise.sample(Vec4::new(point.x, point.y, 3.41, 0.0)),
                ),
            );
        }
    }
    project_incompressible(&mut field, 160);
    field *= FLOW_MEAN_SPEED / field.mean_length();
    field
}

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

#[derive(Clone, Copy, Debug)]
struct Settings {
    theme: Theme,
    depth_field: DepthField,
    glitter: Glitter,
    glow: Glow,
    simplex_scale: f32,
    simplex_speed: f32,
    gerstner_speed: f32,
}

impl Settings {
    fn for_resolution(resolution: &Resolution) -> Self {
        Self {
            theme: Theme {
                background: Rgba8::from_rgb(14, 14, 18),
                foreground: Color::from_rgb8(112, 48, 132),
            },
            depth_field: DepthField {
                focus_depth: 2.0,
                blur: 2.0,
                particle_radius: resolution.area_scale(&DEFAULT_RESOLUTION),
            },
            glitter: Glitter {
                falloff_power: 16.0,
                tumble_speed: GLITTER_TUMBLE_SPEED,
                tumble_axis: Vec3::new(0.4, 0.8, 0.2).normalize(),
                precession_axis: Vec3::new(-0.3, 0.1, 0.9).normalize(),
                precession_speed: GLITTER_PRECESSION_SPEED,
            },
            glow: Glow {
                background: Color::from_rgb8(14, 14, 18),
                softener: 0.5,
                radius: 8.0,
            },
            simplex_scale: SIMPLEX_SCALE,
            simplex_speed: SIMPLEX_SPEED,
            gerstner_speed: 0.25,
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

struct Scene {
    rest_positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    field: [SimplexNoise; 3],
    flow_field: Field<Vec2>,
    flow_positions: Vec<Vec2>,
    gerstner_rest_positions: Vec<Vec3>,
    gerstner_waves: [GerstnerWave; 5],
}

impl Scene {
    fn new() -> Self {
        let mut rng = Rng::new(0x1234_5678);
        let rest_positions = collect(&mut Uniform3::new(), PARTICLE_COUNT, &mut rng);
        let normals = glitter_normals(&mut rng, PARTICLE_COUNT);
        let flow_field = flow_field_from_simplex();
        let flow_positions = (0..PARTICLE_COUNT)
            .map(|_| {
                Vec2::new(
                    rng.next_f32_in(0.0, FLOW_FIELD_SIZE.x),
                    rng.next_f32_in(0.0, FLOW_FIELD_SIZE.y),
                )
            })
            .collect();
        // 128 * 64 == PARTICLE_COUNT, so normals align without padding
        let gerstner_rest_positions = surface_grid(128, 64, Vec2::new(8.0, 4.0));

        Self {
            rest_positions,
            normals,
            field: simplex_field(),
            flow_field,
            flow_positions,
            gerstner_rest_positions,
            gerstner_waves: water_waves(),
        }
    }

    fn advance(&mut self, dt: f32, mode: FieldMode) {
        if mode == FieldMode::Incompressible {
            self.flow_field = advect(&self.flow_field, dt);
            project_incompressible(&mut self.flow_field, 40);
            //let mean = self.flow_field.mean_length();
            //assert!(mean > 0.0);
            //self.flow_field *= FLOW_MEAN_SPEED / mean;
            for position in &mut self.flow_positions {
                let next = *position + self.flow_field.interpolate(*position) * dt;
                *position = Vec2::new(
                    next.x.rem_euclid(FLOW_FIELD_SIZE.x),
                    next.y.rem_euclid(FLOW_FIELD_SIZE.y),
                );
            }
        }
    }

    fn render(
        &self,
        bitmap: &mut Bitmap,
        time: f32,
        settings: Settings,
        view: Mat4,
        mode: FieldMode,
    ) {
        bitmap.fill(settings.theme.background);

        let positions: Vec<Vec3> = match mode {
            FieldMode::Simplex => {
                let w = time * settings.simplex_speed;
                self.rest_positions
                    .iter()
                    .map(|rest| {
                        *rest + simplex_offset(&self.field, *rest, w) * settings.simplex_scale
                    })
                    .collect()
            }
            FieldMode::Incompressible => {
                let offset = FLOW_FIELD_SIZE * 0.5;
                self.flow_positions
                    .iter()
                    .map(|p| {
                        let p = *p - offset;
                        Vec3::new(p.x, 0.0, p.y)
                    })
                    .collect()
            }
            FieldMode::Gerstner => self
                .gerstner_rest_positions
                .iter()
                .map(|rest| {
                    displaced_position(*rest, &self.gerstner_waves, time * settings.gerstner_speed)
                })
                .collect(),
        };

        let projected = project_cloud(bitmap, &positions, projection(bitmap.resolution()), view);
        let rotated_normals =
            rotate_normals(&self.normals, tumble_rotation(time, settings.glitter));
        let colors = glitter_colors(
            settings.theme.foreground,
            &rotated_normals,
            view_direction(view),
            settings.glitter,
        );
        settings.glow.render(bitmap, &projected, &colors);
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
    field_mode: FieldMode,
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
                self.scene.advance(dt, self.field_mode);
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
                ui.label(format!(
                    "{} / {}",
                    format_time(self.time),
                    format_time(DURATION)
                ));
            });
        });

        egui::SidePanel::left("tweaks")
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Fields");
                ui.radio_value(&mut self.field_mode, FieldMode::Simplex, "Simplex noise");
                ui.radio_value(
                    &mut self.field_mode,
                    FieldMode::Incompressible,
                    "Incompressible 2D",
                );
                ui.radio_value(&mut self.field_mode, FieldMode::Gerstner, "Gerstner waves");
                if self.field_mode == FieldMode::Gerstner {
                    ui.add(
                        egui::Slider::new(&mut self.settings.gerstner_speed, 0.05..=2.0)
                            .text("Speed"),
                    );
                }

                ui.separator();
                ui.heading("Render");
                ui.add(
                    egui::Slider::new(&mut self.settings.depth_field.blur, 0.0..=12.0).text("Blur"),
                );
                ui.add(
                    egui::Slider::new(&mut self.settings.depth_field.focus_depth, 0.1..=8.0)
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
                ui.heading("Colors");

                let mut foreground = [
                    (self.settings.theme.foreground.red * 255.0).round() as u8,
                    (self.settings.theme.foreground.green * 255.0).round() as u8,
                    (self.settings.theme.foreground.blue * 255.0).round() as u8,
                ];
                ui.horizontal(|ui| {
                    ui.label("Foreground");
                    if ui.color_edit_button_srgb(&mut foreground).changed() {
                        self.settings.theme.foreground =
                            Color::from_rgb8(foreground[0], foreground[1], foreground[2]);
                    }
                });

                let mut background = [
                    self.settings.theme.background.red,
                    self.settings.theme.background.green,
                    self.settings.theme.background.blue,
                ];
                ui.horizontal(|ui| {
                    ui.label("Background");
                    if ui.color_edit_button_srgb(&mut background).changed() {
                        self.settings.theme.background =
                            Rgba8::from_rgb(background[0], background[1], background[2]);
                    }
                });

                ui.separator();
                ui.heading("Glow");
                ui.add(
                    egui::Slider::new(&mut self.settings.glow.softener, 0.0..=1.0)
                        .text("Softener"),
                );
                ui.add(
                    egui::Slider::new(&mut self.settings.glow.radius, 1.0..=64.0)
                        .text("Radius"),
                );
                let mut glow_background = [
                    (self.settings.glow.background.red * 255.0).round() as u8,
                    (self.settings.glow.background.green * 255.0).round() as u8,
                    (self.settings.glow.background.blue * 255.0).round() as u8,
                ];
                ui.horizontal(|ui| {
                    ui.label("Glow bg");
                    if ui.color_edit_button_srgb(&mut glow_background).changed() {
                        self.settings.glow.background = Color::from_rgb8(
                            glow_background[0],
                            glow_background[1],
                            glow_background[2],
                        );
                    }
                });
            });

        self.scene.render(
            &mut self.bitmap,
            self.time,
            self.settings,
            self.camera.view(),
            self.field_mode,
        );

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

                let scroll = ctx.input(|input| {
                    if response.hovered() && input.modifiers.any() {
                        input.smooth_scroll_delta.y
                    } else {
                        0.0
                    }
                });
                if scroll != 0.0 {
                    self.camera.zoom(scroll);
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
        "particles tweak",
        options,
        Box::new(|_cc| {
            Ok(Box::new(TweakApp {
                scene: Scene::new(),
                settings: Settings::for_resolution(&resolution),
                field_mode: FieldMode::Incompressible,
                bitmap: Bitmap::new(resolution),
                texture: None,
                camera: Camera::new(Vec3::new(2.0_f32.sqrt(), 2.0, 0.0), Vec3::ZERO),
                time: 0.0,
                playing: true,
                last_ui_time: None,
            }))
        }),
    )
}
