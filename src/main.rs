use eframe::egui;
use egui::{Color32, Vec2, Pos2, Rect};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

const PIXEL_SIZE: f32 = 15.0;
const MAX_BRUSH_RADIUS: usize = 5;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 5.0;
const ZOOM_STEP: f32 = 0.1;

#[derive(Default)]
struct PixelArtEditor {
    current_color: Color32,
    current_tool: Tool,
    layers: Vec<Layer>,
    active_layer: usize,
    brush_radius: usize,
    show_brush_settings: bool,
    save_path: Option<PathBuf>,
    show_save_dialog: bool,
    pending_save_path: Option<PathBuf>,
    canvas_width: usize,
    canvas_height: usize,
    show_canvas_selector: bool,
    custom_width: String,
    custom_height: String,
    zoom: f32,
    canvas_offset: Vec2,
    is_panning: bool,
    last_pan_pos: Pos2,
}

#[derive(Default, PartialEq)]
enum Tool {
    #[default]
    Pencil,
    Eraser,
    Fill,
}

struct Layer {
    pixels: Vec<Vec<Color32>>,
    visible: bool,
}

impl Layer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![vec![Color32::TRANSPARENT; height]; width],
            visible: true,
        }
    }
}

#[derive(Clone)]
struct Footage {
    name: String,
    width: usize,
    height: usize,
    preview: Vec<Vec<Color32>>,
}

impl Footage {
    fn create_preview(width: usize, height: usize, color: Color32) -> Vec<Vec<Color32>> {
        let mut preview = vec![vec![Color32::TRANSPARENT; height]; width];
        
        for x in 0..width {
            for y in 0..height {
                if (x + y) % 4 == 0 {
                    preview[x][y] = color;
                }
            }
        }
        
        preview
    }
}

impl PixelArtEditor {
    fn new() -> Self {
        Self {
            current_color: Color32::BLACK,
            current_tool: Tool::Pencil,
            layers: vec![Layer::new(32, 32)],
            active_layer: 0,
            brush_radius: 1,
            show_brush_settings: false,
            save_path: None,
            show_save_dialog: false,
            pending_save_path: None,
            canvas_width: 32,
            canvas_height: 32,
            show_canvas_selector: true,
            custom_width: "32".to_string(),
            custom_height: "32".to_string(),
            zoom: 1.0,
            canvas_offset: Vec2::ZERO,
            is_panning: false,
            last_pan_pos: Pos2::ZERO,
        }
    }

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

    fn get_footages() -> Vec<Footage> {
        vec![
            Footage {
                name: "16x16 (Квадрат)".to_string(),
                width: 16,
                height: 16,
                preview: Footage::create_preview(16, 16, Color32::from_rgb(100, 200, 100)),
            },
            Footage {
                name: "32x32 (Квадрат)".to_string(),
                width: 32,
                height: 32,
                preview: Footage::create_preview(32, 32, Color32::from_rgb(200, 100, 100)),
            },
            Footage {
                name: "64x64 (Квадрат)".to_string(),
                width: 64,
                height: 64,
                preview: Footage::create_preview(64, 64, Color32::from_rgb(100, 100, 200)),
            },
            Footage {
                name: "16:9 (HD)".to_string(),
                width: 32,
                height: 18,
                preview: Footage::create_preview(32, 18, Color32::from_rgb(200, 150, 50)),
            },
            Footage {
                name: "32:18 (Full HD)".to_string(),
                width: 64,
                height: 36,
                preview: Footage::create_preview(64, 36, Color32::from_rgb(150, 50, 200)),
            },
            Footage {
                name: "4:3 (Классический)".to_string(),
                width: 32,
                height: 24,
                preview: Footage::create_preview(32, 24, Color32::from_rgb(50, 200, 150)),
            },
            Footage {
                name: "9:16 (Вертикальный)".to_string(),
                width: 18,
                height: 32,
                preview: Footage::create_preview(18, 32, Color32::from_rgb(200, 50, 150)),
            },
            Footage {
                name: "3:4 (Портрет)".to_string(),
                width: 24,
                height: 32,
                preview: Footage::create_preview(24, 32, Color32::from_rgb(50, 150, 200)),
            },
            Footage {
                name: "Персонаж 16x32".to_string(),
                width: 16,
                height: 32,
                preview: Self::create_character_template(16, 32),
            },
            Footage {
                name: "Персонаж 32x64".to_string(),
                width: 32,
                height: 64,
                preview: Self::create_character_template(32, 64),
            },
            Footage {
                name: "Тайл 16x16".to_string(),
                width: 16,
                height: 16,
                preview: Self::create_tile_template(16, 16),
            },
            Footage {
                name: "Тайл 32x32".to_string(),
                width: 32,
                height: 32,
                preview: Self::create_tile_template(32, 32),
            },
        ]
    }

    fn create_character_template(width: usize, height: usize) -> Vec<Vec<Color32>> {
        let mut template = vec![vec![Color32::TRANSPARENT; height]; width];
        
        let head_height = height / 4;
        for x in width/4..width*3/4 {
            for y in 0..head_height {
                template[x][y] = Color32::from_rgb(100, 100, 200);
            }
        }
        
        for x in width/3..width*2/3 {
            for y in head_height..height {
                template[x][y] = Color32::from_rgb(200, 100, 100);
            }
        }
        
        template
    }

    fn create_tile_template(width: usize, height: usize) -> Vec<Vec<Color32>> {
        let mut template = vec![vec![Color32::TRANSPARENT; height]; width];
        
        for x in 0..width {
            for y in 0..height {
                if (x + y) % 4 == 0 {
                    template[x][y] = Color32::from_rgb(150, 150, 50);
                }
            }
        }
        
        template
    }

    fn draw_on_canvas(&mut self, pos: (usize, usize)) {
        if pos.0 >= self.canvas_width || pos.1 >= self.canvas_height {
            return;
        }

        match self.current_tool {
            Tool::Pencil => self.draw_square(pos.0, pos.1, self.current_color),
            Tool::Eraser => self.draw_square(pos.0, pos.1, Color32::TRANSPARENT),
            Tool::Fill => self.flood_fill(pos.0, pos.1, self.current_color),
        }
    }

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

    fn save_to_png(&self, filename: &PathBuf) -> Result<(), png::EncodingError> {
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

    fn show_save_dialog(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        let mut should_save = false;
        
        egui::Window::new("Сохранить изображение")
            .collapsible(false)
            .resizable(false)
            .open(&mut self.show_save_dialog)
            .show(ctx, |ui| {
                ui.label("Выберите место для сохранения:");
                
                if ui.button("Выбрать папку").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("pixel_art.png")
                        .save_file()
                    {
                        self.pending_save_path = Some(path);
                    }
                }
                
                if let Some(path) = &self.pending_save_path {
                    ui.label(format!("Будет сохранено в: {}", path.display()));
                    
                    if ui.button("Сохранить").clicked() {
                        should_save = true;
                        should_close = true;
                    }
                }

                if ui.button("Отмена").clicked() {
                    should_close = true;
                }
            });

        if should_close {
            self.show_save_dialog = false;
        }

        if should_save {
            if let Some(path) = &self.pending_save_path {
                if let Err(e) = self.save_to_png(path) {
                    eprintln!("Ошибка сохранения: {}", e);
                } else {
                    println!("Изображение сохранено как {}", path.display());
                    self.save_path = self.pending_save_path.take();
                }
            }
        }
    }

    fn show_canvas_selector(&mut self, ctx: &egui::Context) {
        egui::Area::new("canvas_selector_area")
            .fixed_pos(egui::pos2(0.0, 0.0))
            .movable(false)
            .interactable(true)
            .show(ctx, |ui| {
                let screen_rect = ui.max_rect();
                let mut mesh = egui::Mesh::default();
                mesh.colored_vertex(screen_rect.left_top(), Color32::from_rgb(30, 30, 50));
                mesh.colored_vertex(screen_rect.right_top(), Color32::from_rgb(50, 30, 50));
                mesh.colored_vertex(screen_rect.right_bottom(), Color32::from_rgb(20, 10, 30));
                mesh.colored_vertex(screen_rect.left_bottom(), Color32::from_rgb(10, 20, 30));
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(2, 3, 0);
                ui.painter().add(mesh);

                egui::Window::new("🎨 Pixel Art Editor - Создание нового холста")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(egui::Frame::window(&ctx.style()).inner_margin(20.0))
                    .show(ctx, |ui| {
                        ui.heading("Создать новый холст");
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            ui.label("Ширина:");
                            if ui.text_edit_singleline(&mut self.custom_width).changed() {
                                if let Ok(num) = self.custom_width.parse::<usize>() {
                                    if num > 0 && num <= 256 {
                                        self.canvas_width = num;
                                    }
                                }
                            }
                            
                            ui.label("Высота:");
                            if ui.text_edit_singleline(&mut self.custom_height).changed() {
                                if let Ok(num) = self.custom_height.parse::<usize>() {
                                    if num > 0 && num <= 256 {
                                        self.canvas_height = num;
                                    }
                                }
                            }
                            
                            if ui.button("Создать").clicked() {
                                self.init_with_size(self.canvas_width, self.canvas_height);
                            }
                        });
                        
                        ui.separator();
                        ui.heading("Или выберите шаблон:");
                        
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.collapsing("🔲 Квадратные", |ui| {
                                self.show_footage_grid(ui, &[0, 1, 2]);
                            });
                            
                            ui.collapsing("🖥️ Горизонтальные", |ui| {
                                self.show_footage_grid(ui, &[3, 4, 5]);
                            });
                            
                            ui.collapsing("📱 Вертикальные", |ui| {
                                self.show_footage_grid(ui, &[6, 7]);
                            });
                            
                            ui.collapsing("🧑 Персонажи", |ui| {
                                self.show_footage_grid(ui, &[8, 9]);
                            });
                            
                            ui.collapsing("🧱 Тайлы", |ui| {
                                self.show_footage_grid(ui, &[10, 11]);
                            });
                        });
                    });
            });
    }
    
    fn show_footage_grid(&mut self, ui: &mut egui::Ui, indices: &[usize]) {
        let footages = Self::get_footages();
        egui::Grid::new("footages_grid")
            .num_columns(3)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                for &i in indices {
                    if i >= footages.len() { continue; }
                    
                    let footage = &footages[i];
                    ui.vertical(|ui| {
                        let preview_size = 100.0;
                        let (rect, _) = ui.allocate_exact_size(
                            Vec2::new(preview_size, preview_size),
                            egui::Sense::click()
                        );
                        
                        let painter = ui.painter();
                        painter.rect_filled(rect, 5.0, Color32::from_gray(30));
                        
                        let scale_x = preview_size / footage.width as f32;
                        let scale_y = preview_size / footage.height as f32;
                        
                        for x in 0..footage.width {
                            for y in 0..footage.height {
                                let color = footage.preview[x][y];
                                if color != Color32::TRANSPARENT {
                                    let rect = egui::Rect::from_min_size(
                                        egui::Pos2::new(
                                            rect.min.x + x as f32 * scale_x,
                                            rect.min.y + y as f32 * scale_y,
                                        ),
                                        Vec2::new(scale_x, scale_y),
                                    );
                                    painter.rect_filled(rect, 0.0, color);
                                }
                            }
                        }
                        
                        painter.rect_stroke(rect, 5.0, (1.0, Color32::from_gray(80)));
                        ui.label(&footage.name);
                        
                        if ui.button("Выбрать").clicked() {
                            self.init_with_footage(footage.clone());
                        }
                    });
                }
                ui.end_row();
            });
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

    fn center_canvas(&mut self, available_size: Vec2) {
        let canvas_size = Vec2::new(
            self.canvas_width as f32 * PIXEL_SIZE * self.zoom,
            self.canvas_height as f32 * PIXEL_SIZE * self.zoom,
        );

        if canvas_size.x < available_size.x && canvas_size.y < available_size.y {
            self.canvas_offset = (available_size - canvas_size) / 2.0;
        }
    }
}

impl eframe::App for PixelArtEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.show_canvas_selector {
            self.show_canvas_selector(ctx);
            return;
        }

        if self.show_brush_settings {
            egui::Window::new("Настройки кисти")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Размер: {}x{}", self.brush_radius, self.brush_radius));
                    ui.add(egui::Slider::new(&mut self.brush_radius, 1..=MAX_BRUSH_RADIUS));
                    if ui.button("Закрыть").clicked() {
                        self.show_brush_settings = false;
                    }
                });
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let pencil_btn = ui.selectable_value(&mut self.current_tool, Tool::Pencil, "✏️ Карандаш");
                if pencil_btn.secondary_clicked() {
                    self.show_brush_settings = true;
                }
                ui.selectable_value(&mut self.current_tool, Tool::Eraser, "🧽 Ластик");
                ui.selectable_value(&mut self.current_tool, Tool::Fill, "🎨 Заливка");
                
                ui.separator();
                
                ui.label("Цвет:");
                ui.color_edit_button_srgba(&mut self.current_color);
                
                ui.separator();
                
                if ui.button("➕ Слой").clicked() {
                    self.layers.push(Layer::new(self.canvas_width, self.canvas_height));
                    self.active_layer = self.layers.len() - 1;
                }
                if ui.button("🗑️ Удалить").clicked() && self.layers.len() > 1 {
                    self.layers.remove(self.active_layer);
                    if self.active_layer >= self.layers.len() {
                        self.active_layer = self.layers.len() - 1;
                    }
                }

                ui.separator();

                if ui.button("-").clicked() {
                    self.zoom = (self.zoom - ZOOM_STEP).max(MIN_ZOOM);
                }
                ui.label(format!("{:.0}%", self.zoom * 100.0));
                if ui.button("+").clicked() {
                    self.zoom = (self.zoom + ZOOM_STEP).min(MAX_ZOOM);
                }
                if ui.button("Сброс").clicked() {
                    self.zoom = 1.0;
                    self.center_canvas(ui.available_size());
                }

                ui.separator();

                if ui.button("💾 Сохранить PNG").clicked() {
                    self.show_save_dialog = true;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            self.center_canvas(available_size);

            let canvas_size = Vec2::new(
                self.canvas_width as f32 * PIXEL_SIZE * self.zoom,
                self.canvas_height as f32 * PIXEL_SIZE * self.zoom,
            );

            let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            self.handle_zoom_and_pan(ctx, &response);

            let canvas_rect = Rect::from_min_size(
                response.rect.min + self.canvas_offset,
                canvas_size,
            );

            painter.rect_filled(canvas_rect, 0.0, Color32::from_gray(30));

            for x in 0..self.canvas_width {
                for y in 0..self.canvas_height {
                    let mut color = Color32::TRANSPARENT;
                    for layer in &self.layers {
                        if layer.visible && layer.pixels[x][y] != Color32::TRANSPARENT {
                            color = layer.pixels[x][y];
                        }
                    }

                    let rect = Rect::from_min_size(
                        Pos2::new(
                            canvas_rect.min.x + x as f32 * PIXEL_SIZE * self.zoom,
                            canvas_rect.min.y + y as f32 * PIXEL_SIZE * self.zoom,
                        ),
                        Vec2::splat(PIXEL_SIZE * self.zoom),
                    );
                    
                    painter.rect_filled(rect, 0.0, color);
                    painter.rect_stroke(rect, 0.0, (1.0, Color32::from_gray(100)));
                }
            }

            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let x = ((pos.x - canvas_rect.min.x) / (PIXEL_SIZE * self.zoom)).floor() as usize;
                    let y = ((pos.y - canvas_rect.min.y) / (PIXEL_SIZE * self.zoom)).floor() as usize;
                    if x < self.canvas_width && y < self.canvas_height {
                        self.draw_on_canvas((x, y));
                    }
                }
            }
        });

        egui::SidePanel::right("layers_panel").show(ctx, |ui| {
            ui.label("📚 Слои");
            ui.separator();
            
            for (i, layer) in self.layers.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut layer.visible, "");
                    if ui.selectable_label(i == self.active_layer, &format!("Слой {}", i + 1)).clicked() {
                        self.active_layer = i;
                    }
                });
            }
        });

        if self.show_save_dialog {
            self.show_save_dialog(ctx);
        }
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Pixel Art Editor",
        options,
        Box::new(|_cc| Box::new(PixelArtEditor::new())),
    );
}