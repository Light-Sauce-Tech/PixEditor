use eframe::egui::{Color32, Pos2, Rect};

pub trait ColorUtils {
    fn update_color_edit_rgba(&mut self);
    fn update_current_color_from_rgba(&mut self);
    fn rgb_to_hsv(&self, r: u8, g: u8, b: u8) -> (f32, f32, f32);
    fn get_color_wheel_marker_pos(&self, rect: Rect) -> Option<Pos2>;
    fn update_color_from_wheel(&mut self, pos: Pos2, rect: Rect);
}