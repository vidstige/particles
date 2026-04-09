use eframe::egui::{self, TextureHandle, TextureOptions};
use glam::{Mat4, Vec3};
use particles::{
    bitmap::Bitmap,
    color::{Color, Rgba8},
    depth_field::{DepthField, Render, Theme},
    distribution::{collect, Uniform3},
    env::DEFAULT_RESOLUTION,
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
const GLITTER_TUMBLE_SPEED: f32 = 8.0;
const GLITTER_PRECESSION_SPEED: f32 = 1.5;

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

fn projection(resolution: &Resolution) -> Mat4 {
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), resolution.aspect_ratio(), 0.1, 12.0)
}

#[derive(Clone, Copy, Debug)]
struct GlitterSceneSettings {
    theme: Theme,
    depth_field: DepthField,
    glitter: Glitter,
    simplex_scale: f32,
    simplex_speed: f32,
}

impl GlitterSceneSettings {
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
            simplex_scale: SIMPLEX_SCALE,
            simplex_speed: SIMPLEX_SPEED,
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

struct GlitterScene {
    rest_positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    field: [SimplexNoise; 3],
}

impl GlitterScene {
    fn new() -> Self {
        let mut rng = Rng::new(0x1234_5678);
        let rest_positions = collect(&mut Uniform3::new(), PARTICLE_COUNT, &mut rng);
        let normals = glitter_normals(&mut rng, PARTICLE_COUNT);

        Self {
            rest_positions,
            normals,
            field: simplex_field(),
        }
    }

    fn render(&self, bitmap: &mut Bitmap, time: f32, settings: GlitterSceneSettings, view: Mat4) {
        bitmap.fill(settings.theme.background);

        let w = time * settings.simplex_speed;
        let positions = self
            .rest_positions
            .iter()
            .map(|rest_position| {
                *rest_position
                    + simplex_offset(&self.field, *rest_position, w) * settings.simplex_scale
            })
            .collect::<Vec<_>>();
        let projected = project_cloud(bitmap, &positions, projection(bitmap.resolution()), view);
        let rotated_normals =
            rotate_normals(&self.normals, tumble_rotation(time, settings.glitter));
        let colors = glitter_colors(
            settings.theme.foreground,
            &rotated_normals,
            view_direction(view),
            settings.glitter,
        );
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
        self.pitch = (self.pitch - delta.y * 0.01).clamp(-1.4, 1.4);
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
    scene: GlitterScene,
    settings: GlitterSceneSettings,
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
                self.time = (self.time + (now - last_ui_time) as f32).rem_euclid(DURATION);
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
            });

        self.scene.render(
            &mut self.bitmap,
            self.time,
            self.settings,
            self.camera.view(),
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
                scene: GlitterScene::new(),
                settings: GlitterSceneSettings::for_resolution(&resolution),
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
