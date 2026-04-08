use eframe::egui::{self, TextureHandle, TextureOptions};
use particles::{
    bitmap::Bitmap,
    env::DEFAULT_RESOLUTION,
    glitter_scene::{animated_view, GlitterScene, GlitterSceneSettings, DEFAULT_DURATION},
};

fn format_time(seconds: f32) -> String {
    format!("{seconds:05.2}s")
}

fn image_size(bitmap: &Bitmap, available: egui::Vec2) -> egui::Vec2 {
    let size = egui::Vec2::new(bitmap.width() as f32, bitmap.height() as f32);
    let scale = (available.x / size.x).min(available.y / size.y);
    size * scale
}

struct TweakApp {
    scene: GlitterScene,
    settings: GlitterSceneSettings,
    bitmap: Bitmap,
    texture: Option<TextureHandle>,
    time: f32,
    playing: bool,
    last_ui_time: Option<f64>,
}

impl eframe::App for TweakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|input| input.time);
        if let Some(last_ui_time) = self.last_ui_time {
            if self.playing {
                self.time = (self.time + (now - last_ui_time) as f32).rem_euclid(DEFAULT_DURATION);
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
                    egui::Slider::new(&mut self.time, 0.0..=DEFAULT_DURATION)
                        .show_value(false)
                        .clamping(egui::SliderClamping::Always),
                );
                ui.label(format!(
                    "{} / {}",
                    format_time(self.time),
                    format_time(DEFAULT_DURATION)
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
            });

        self.scene.render(
            &mut self.bitmap,
            self.time,
            self.settings,
            animated_view(self.time),
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
                ui.centered_and_justified(|ui| {
                    ui.image((texture.id(), size));
                });
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
                time: 0.0,
                playing: true,
                last_ui_time: None,
            }))
        }),
    )
}
