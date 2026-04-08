use eframe::egui;

#[derive(Default)]
struct TweakApp;

impl eframe::App for TweakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("particles tweak");
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "particles tweak",
        options,
        Box::new(|_cc| Ok(Box::new(TweakApp))),
    )
}
