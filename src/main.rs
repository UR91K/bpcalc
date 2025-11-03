use crate::app::HarmonicApp;
use eframe::egui;

mod app;
mod calculation;
mod color;
mod visualizer;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([706.0, 678.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Harmonic Anti-Node Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(HarmonicApp::default()))),
    )
}
