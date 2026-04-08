use eframe::egui::{self, TextureHandle, TextureOptions};
use particles::{
    bitmap::Bitmap,
    env::DEFAULT_RESOLUTION,
    glitter_scene::{animated_view, GlitterScene, GlitterSceneSettings},
};

struct TweakApp {
    scene: GlitterScene,
    settings: GlitterSceneSettings,
    bitmap: Bitmap,
    texture: Option<TextureHandle>,
}

impl eframe::App for TweakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let time = ctx.input(|input| input.time as f32);
        self.scene
            .render(&mut self.bitmap, time, self.settings, animated_view(time));

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
            }))
        }),
    )
}
