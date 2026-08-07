use bg2_editor::{app, fonts};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 750.0]),
        ..Default::default()
    };
    eframe::run_native(
        "BG2:EE Save Editor",
        options,
        Box::new(|cc| {
            fonts::setup(&cc.egui_ctx);
            Ok(Box::new(app::Bg2EditorApp::default()))
        }),
    )
}
