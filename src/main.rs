use eframe::egui;
use egui::Color32;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

const CANVAS_SIZE: usize = 32;
const PIXEL_SIZE: f32 = 15.0;
const MAX_BRUSH_RADIUS: usize = 5;

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
    pending_save_path: Option<PathBuf>, // Добавляем временное поле для хранения пути
}

#[derive(Default, PartialEq)]
enum Tool {
    #[default]
    Pencil,
    Eraser,
    Fill,
}

#[derive(Default)]
struct Layer {
    pixels: [[Color32; CANVAS_SIZE]; CANVAS_SIZE],
    visible: bool,
}

impl PixelArtEditor {
    fn new() -> Self {
        Self {
            current_color: Color32::BLACK,
            current_tool: Tool::Pencil,
            layers: vec![Layer::default()],
            active_layer: 0,
            brush_radius: 1,
            show_brush_settings: false,
            save_path: None,
            show_save_dialog: false,
            pending_save_path: None,
        }
    }

    fn draw_on_canvas(&mut self, pos: (usize, usize)) {
        if pos.0 >= CANVAS_SIZE || pos.1 >= CANVAS_SIZE {
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
                if nx < CANVAS_SIZE && ny < CANVAS_SIZE {
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
            if x >= CANVAS_SIZE || y >= CANVAS_SIZE || self.layers[self.active_layer].pixels[x][y] != old_color {
                continue;
            }

            self.layers[self.active_layer].pixels[x][y] = new_color;

            if x > 0 { stack.push((x - 1, y)); }
            if x < CANVAS_SIZE - 1 { stack.push((x + 1, y)); }
            if y > 0 { stack.push((x, y - 1)); }
            if y < CANVAS_SIZE - 1 { stack.push((x, y + 1)); }
        }
    }

    fn save_to_png(&self, filename: &PathBuf) -> Result<(), png::EncodingError> {
        let file = File::create(filename)?;
        let w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, CANVAS_SIZE as u32, CANVAS_SIZE as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;

        let mut data = vec![0; CANVAS_SIZE * CANVAS_SIZE * 4];
        
        for x in 0..CANVAS_SIZE {
            for y in 0..CANVAS_SIZE {
                let mut color = Color32::TRANSPARENT;
                for layer in &self.layers {
                    if layer.visible && layer.pixels[x][y] != Color32::TRANSPARENT {
                        color = layer.pixels[x][y];
                    }
                }
                
                let idx = (y * CANVAS_SIZE + x) * 4;
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
}

impl eframe::App for PixelArtEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Меню настроек кисти
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

        // Верхняя панель инструментов
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Кнопки инструментов
                let pencil_btn = ui.selectable_value(&mut self.current_tool, Tool::Pencil, "✏️ Карандаш");
                if pencil_btn.secondary_clicked() {
                    self.show_brush_settings = true;
                }
                ui.selectable_value(&mut self.current_tool, Tool::Eraser, "🧽 Ластик");
                ui.selectable_value(&mut self.current_tool, Tool::Fill, "🎨 Заливка");
                
                ui.separator();
                
                // Выбор цвета
                ui.label("Цвет:");
                ui.color_edit_button_srgba(&mut self.current_color);
                
                ui.separator();
                
                // Управление слоями
                if ui.button("➕ Слой").clicked() {
                    self.layers.push(Layer::default());
                    self.active_layer = self.layers.len() - 1;
                }
                if ui.button("🗑️ Удалить").clicked() && self.layers.len() > 1 {
                    self.layers.remove(self.active_layer);
                    if self.active_layer >= self.layers.len() {
                        self.active_layer = self.layers.len() - 1;
                    }
                }

                // Кнопка сохранения
                if ui.button("💾 Сохранить PNG").clicked() {
                    self.show_save_dialog = true;
                }
            });
        });

        // Основное рабочее пространство
        egui::CentralPanel::default().show(ctx, |ui| {
            let canvas_response = ui.allocate_response(
                egui::Vec2::new(CANVAS_SIZE as f32 * PIXEL_SIZE, CANVAS_SIZE as f32 * PIXEL_SIZE),
                egui::Sense::click_and_drag()
            );

            // Обработка рисования
            if canvas_response.dragged() || canvas_response.clicked() {
                if let Some(pos) = canvas_response.interact_pointer_pos() {
                    let x = ((pos.x - canvas_response.rect.min.x) / PIXEL_SIZE).floor() as usize;
                    let y = ((pos.y - canvas_response.rect.min.y) / PIXEL_SIZE).floor() as usize;
                    self.draw_on_canvas((x, y));
                }
            }

            // Отрисовка холста
            let painter = ui.painter_at(canvas_response.rect);
            for x in 0..CANVAS_SIZE {
                for y in 0..CANVAS_SIZE {
                    let mut color = Color32::TRANSPARENT;
                    for layer in &self.layers {
                        if layer.visible && layer.pixels[x][y] != Color32::TRANSPARENT {
                            color = layer.pixels[x][y];
                        }
                    }

                    let rect = egui::Rect::from_min_size(
                        egui::Pos2::new(
                            canvas_response.rect.min.x + x as f32 * PIXEL_SIZE,
                            canvas_response.rect.min.y + y as f32 * PIXEL_SIZE,
                        ),
                        egui::Vec2::splat(PIXEL_SIZE),
                    );
                    
                    painter.rect_filled(rect, 0.0, color);
                    painter.rect_stroke(rect, 0.0, (1.0, Color32::from_gray(100)));
                }
            }
        });

        // Боковая панель (слои)
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

        // Показываем диалог сохранения если нужно
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