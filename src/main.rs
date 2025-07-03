mod editor;
mod ui;

use eframe::egui;
use editor::PixelArtEditor;

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Pixel Art Editor",
        options,
        Box::new(|cc| Box::new(PixelArtEditor::new(cc))),
    );
}