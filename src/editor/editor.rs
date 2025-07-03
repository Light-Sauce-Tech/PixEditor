use eframe::egui::{self, Color32, Vec2, Pos2, Rect, TextureHandle};
use std::path::PathBuf;
use std::fs::File;
use std::io::BufWriter;

use crate::ui;
use super::{
    tools::Tool,
    layer::Layer,
    footage::Footage,
    color::ColorUtils,
    canvas::CanvasUtils,
};

const PIXEL_SIZE: f32 = 15.0;
const MAX_BRUSH_RADIUS: usize = 5;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 5.0;
const ZOOM_STEP: f32 = 0.1;

pub struct PixelArtEditor {
    // Состояние редактора
    pub current_color: Color32,
    pub current_tool: Tool,
    pub layers: Vec<Layer>,
    pub active_layer: usize,
    pub brush_radius: usize,
    
    // Настройки UI
    pub show_brush_settings: bool,
    pub show_file_menu: bool,
    pub show_canvas_selector: bool,
    pub show_save_dialog: bool,
    
    // Данные холста
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub custom_width: String,
    pub custom_height: String,
    
    // Навигация
    pub zoom: f32,
    pub canvas_offset: Vec2,
    pub is_panning: bool,
    pub last_pan_pos: Pos2,
    
    // Цвет
    pub color_wheel_texture: Option<TextureHandle>,
    pub is_color_picking: bool,
    pub color_edit_rgba: [f32; 4],
    
    // Файлы
    pub save_path: Option<PathBuf>,
    pub pending_save_path: Option<PathBuf>,
}

impl PixelArtEditor {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut editor = Self {
            current_color: Color32::BLACK,
            current_tool: Tool::Pencil,
            layers: vec![Layer::new(32, 32)],
            active_layer: 0,
            brush_radius: 1,
            show_brush_settings: false,
            show_file_menu: false,
            show_canvas_selector: true,
            show_save_dialog: false,
            canvas_width: 32,
            canvas_height: 32,
            custom_width: "32".to_string(),
            custom_height: "32".to_string(),
            zoom: 1.0,
            canvas_offset: Vec2::ZERO,
            is_panning: false,
            last_pan_pos: Pos2::ZERO,
            color_wheel_texture: None,
            is_color_picking: false,
            color_edit_rgba: [0.0, 0.0, 0.0, 1.0],
            save_path: None,
            pending_save_path: None,
        };
        
        editor.create_color_wheel_texture(cc);
        editor.update_color_edit_rgba();
        editor
    }

    fn create_color_wheel_texture(&mut self, cc: &eframe::CreationContext<'_>) {
        let size = 256;
        let mut pixels = vec![Color32::TRANSPARENT; size * size];
        
        let center = (size as f32 / 2.0, size as f32 / 2.0);
        let radius = size as f32 / 2.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center.0;
                let dy = y as f32 - center.1;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance <= radius {
                    let angle = dy.atan2(dx) + std::f32::consts::PI;
                    let normalized_angle = angle / (2.0 * std::f32::consts::PI);
                    let normalized_distance = distance / radius;
                    
                    let h = normalized_angle;
                    let s = normalized_distance;
                    let v = 1.0;
                    
                    let c = v * s;
                    let x_val = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
                    let m = v - c;
                    
                    let (r, g, b) = match (h * 6.0) as i32 {
                        0 => (c, x_val, 0.0),
                        1 => (x_val, c, 0.0),
                        2 => (0.0, c, x_val),
                        3 => (0.0, x_val, c),
                        4 => (x_val, 0.0, c),
                        _ => (c, 0.0, x_val),
                    };
                    
                    pixels[y * size + x] = Color32::from_rgb(
                        ((r + m) * 255.0) as u8,
                        ((g + m) * 255.0) as u8,
                        ((b + m) * 255.0) as u8,
                    );
                }
            }
        }
        
        let image = egui::ColorImage {
            size: [size, size],
            pixels,
        };
        
        self.color_wheel_texture = Some(cc.egui_ctx.load_texture(
            "color_wheel",
            image,
            egui::TextureOptions::LINEAR
        ));
    }

    pub fn save_to_png(&self, filename: &PathBuf) -> Result<(), png::EncodingError> {
        let file = File::create(filename)?;
        let w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.canvas_width as u32, self.canvas_height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;

        let mut data = vec![0; self.canvas_width * self.canvas_height * 4];
        
        for x in 0..self.canvas_width {
            for y in 0..self.canvas_height {
                let mut color = Color32::TRANSPARENT;
                for layer in &self.layers {
                    if layer.visible && layer.pixels[x][y] != Color32::TRANSPARENT {
                        color = layer.pixels[x][y];
                    }
                }
                
                let idx = (y * self.canvas_width + x) * 4;
                data[idx] = color.r();
                data[idx + 1] = color.g();
                data[idx + 2] = color.b();
                data[idx + 3] = color.a();
            }
        }

        writer.write_image_data(&data)?;
        Ok(())
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::render(self, ctx);
    }
}

impl ColorUtils for PixelArtEditor {
    fn update_color_edit_rgba(&mut self) {
        self.color_edit_rgba = [
            self.current_color.r() as f32 / 255.0,
            self.current_color.g() as f32 / 255.0,
            self.current_color.b() as f32 / 255.0,
            self.current_color.a() as f32 / 255.0,
        ];
    }

    fn update_current_color_from_rgba(&mut self) {
        self.current_color = Color32::from_rgba_premultiplied(
            (self.color_edit_rgba[0] * 255.0) as u8,
            (self.color_edit_rgba[1] * 255.0) as u8,
            (self.color_edit_rgba[2] * 255.0) as u8,
            (self.color_edit_rgba[3] * 255.0) as u8,
        );
    }

    fn rgb_to_hsv(&self, r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;
        
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            ((g - b) / delta) % 6.0
        } else if max == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        } * 60.0;
        
        let h = (if h < 0.0 { h + 360.0 } else { h }) / 360.0;
        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;
        
        (h, s, v)
    }

    fn get_color_wheel_marker_pos(&self, rect: Rect) -> Option<Pos2> {
        let (h, s, _) = self.rgb_to_hsv(
            self.current_color.r(),
            self.current_color.g(),
            self.current_color.b(),
        );
        
        let radius = rect.width() / 2.0;
        let angle = h * 2.0 * std::f32::consts::PI - std::f32::consts::PI;
        let distance = s * radius;
        
        Some(Pos2::new(
            rect.center().x + distance * angle.cos(),
            rect.center().y + distance * angle.sin(),
        ))
    }

    fn update_color_from_wheel(&mut self, pos: Pos2, rect: Rect) {
        let dx = pos.x - rect.center().x;
        let dy = pos.y - rect.center().y;
        let distance = (dx * dx + dy * dy).sqrt();
        let radius = rect.width() / 2.0;
        
        if distance <= radius {
            let angle = dy.atan2(dx) + std::f32::consts::PI;
            let normalized_angle = angle / (2.0 * std::f32::consts::PI);
            let normalized_distance = distance / radius;
            
            let h = normalized_angle;
            let s = normalized_distance;
            let v = 1.0;
            
            let c = v * s;
            let x_val = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
            let m = v - c;
            
            let (r, g, b) = match (h * 6.0) as i32 {
                0 => (c, x_val, 0.0),
                1 => (x_val, c, 0.0),
                2 => (0.0, c, x_val),
                3 => (0.0, x_val, c),
                4 => (x_val, 0.0, c),
                _ => (c, 0.0, x_val),
            };
            
            self.current_color = Color32::from_rgb(
                ((r + m) * 255.0) as u8,
                ((g + m) * 255.0) as u8,
                ((b + m) * 255.0) as u8,
            );
            self.update_color_edit_rgba();
        }
    }
}

impl CanvasUtils for PixelArtEditor {
    fn init_with_size(&mut self, width: usize, height: usize) {
        self.canvas_width = width;
        self.canvas_height = height;
        self.layers = vec![Layer::new(width, height)];
        self.active_layer = 0;
        self.show_canvas_selector = false;
        self.zoom = 1.0;
        self.canvas_offset = Vec2::ZERO;
    }

    fn init_with_footage(&mut self, footage: Footage) {
        self.canvas_width = footage.width;
        self.canvas_height = footage.height;
        self.layers = vec![Layer {
            pixels: footage.preview,
            visible: true,
        }];
        self.active_layer = 0;
        self.show_canvas_selector = false;
        self.zoom = 1.0;
        self.canvas_offset = Vec2::ZERO;
    }

    fn draw_on_canvas(&mut self, pos: (usize, usize)) {
        if pos.0 >= self.canvas_width || pos.1 >= self.canvas_height {
            return;
        }

        match self.current_tool {
            Tool::Pencil => self.draw_square(pos.0, pos.1, self.current_color),
            Tool::Eraser => self.draw_square(pos.0, pos.1, Color32::TRANSPARENT),
            Tool::Brush => self.draw_square(pos.0, pos.1, self.current_color),
            Tool::Fill => self.flood_fill(pos.0, pos.1, self.current_color),
            Tool::Eyedropper => {
                if pos.0 < self.canvas_width && pos.1 < self.canvas_height {
                    self.current_color = self.layers[self.active_layer].pixels[pos.0][pos.1];
                    self.update_color_edit_rgba();
                }
            }
        }
    }

    fn center_canvas(&mut self, available_size: Vec2) {
        let canvas_size = Vec2::new(
            self.canvas_width as f32 * PIXEL_SIZE * self.zoom,
            self.canvas_height as f32 * PIXEL_SIZE * self.zoom,
        );

        if canvas_size.x < available_size.x && canvas_size.y < available_size.y {
            self.canvas_offset = (available_size - canvas_size) / 2.0;
        }
    }

    fn handle_zoom_and_pan(&mut self, ctx: &egui::Context, response: &egui::Response) {
        if let Some(mouse_pos) = response.hover_pos() {
            let scroll_delta = ctx.input(|i| i.scroll_delta.y);
            if scroll_delta != 0.0 {
                let old_zoom = self.zoom;
                self.zoom = (self.zoom * (1.0 + scroll_delta * 0.01)).clamp(MIN_ZOOM, MAX_ZOOM);
                
                let mouse_canvas_pos = (mouse_pos.to_vec2() - self.canvas_offset) / old_zoom;
                self.canvas_offset = mouse_pos.to_vec2() - mouse_canvas_pos * self.zoom;
            }
        }

        if response.drag_started_by(egui::PointerButton::Middle) {
            self.is_panning = true;
            self.last_pan_pos = ctx.pointer_latest_pos().unwrap_or_default();
        }

        if self.is_panning {
            if let Some(current_pos) = ctx.pointer_latest_pos() {
                let delta = current_pos.to_vec2() - self.last_pan_pos.to_vec2();
                self.canvas_offset += delta;
                self.last_pan_pos = current_pos;
            }

            if ctx.input(|i| i.pointer.any_released()) {
                self.is_panning = false;
            }
        }
    }
}

impl PixelArtEditor {
    fn draw_square(&mut self, x: usize, y: usize, color: Color32) {
        let radius = self.brush_radius;
        for dy in 0..radius {
            for dx in 0..radius {
                let nx = x.saturating_add(dx);
                let ny = y.saturating_add(dy);
                if nx < self.canvas_width && ny < self.canvas_height {
                    self.layers[self.active_layer].pixels[nx][ny] = color;
                }
            }
        }
    }

    fn flood_fill(&mut self, x: usize, y: usize, new_color: Color32) {
        let old_color = self.layers[self.active_layer].pixels[x][y];
        if old_color == new_color {
            return;
        }

        let mut stack = vec![(x, y)];
        while let Some((x, y)) = stack.pop() {
            if x >= self.canvas_width || y >= self.canvas_height || 
               self.layers[self.active_layer].pixels[x][y] != old_color {
                continue;
            }

            self.layers[self.active_layer].pixels[x][y] = new_color;

            if x > 0 { stack.push((x - 1, y)); }
            if x < self.canvas_width - 1 { stack.push((x + 1, y)); }
            if y > 0 { stack.push((x, y - 1)); }
            if y < self.canvas_height - 1 { stack.push((x, y + 1)); }
        }
    }
}