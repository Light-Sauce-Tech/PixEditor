use eframe::egui::{Vec2, Pos2};
use crate::editor::Footage;

pub trait CanvasUtils {
    fn init_with_size(&mut self, width: usize, height: usize);
    fn init_with_footage(&mut self, footage: Footage);
    fn draw_on_canvas(&mut self, pos: (usize, usize));
    fn center_canvas(&mut self, available_size: Vec2);
    fn handle_zoom_and_pan(&mut self, ctx: &egui::Context, response: &egui::Response);
}