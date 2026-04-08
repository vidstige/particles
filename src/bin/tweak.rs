use eframe::egui::{self, TextureHandle, TextureOptions};
use particles::{
    bitmap::Bitmap,
    env::DEFAULT_RESOLUTION,
    glitter_scene::{animated_view, GlitterScene, GlitterSceneSettings, DEFAULT_DURATION},
};

fn format_time(seconds: f32) -> String {
    format!("{seconds:05.2}s")
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
                let size = egui::Vec2::new(self.bitmap.width() as f32, self.bitmap.height() as f32);
                ui.image((texture.id(), size));
            }
        });
    }
}

fn main() -> eframe::Result {
    let resolution = DEFAULT_RESOLUTION;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([resolution.width as f32, resolution.height as f32]),
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
