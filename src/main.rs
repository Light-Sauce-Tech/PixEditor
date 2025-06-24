use eframe::egui;
use egui::Color32;

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
        }
    }

    fn draw_on_canvas(&mut self, pos: (usize, usize)) {
        if pos.0 >= CANVAS_SIZE || pos.1 >= CANVAS_SIZE {
            return;
        }

        match self.current_tool {
            Tool::Pencil => {
                self.draw_circle(pos.0, pos.1, self.brush_radius, self.current_color);
            }
            Tool::Eraser => {
                self.draw_circle(pos.0, pos.1, self.brush_radius, Color32::TRANSPARENT);
            }
            Tool::Fill => {
                self.flood_fill(pos.0, pos.1, self.current_color);
            }
        }
    }

    fn draw_circle(&mut self, x: usize, y: usize, radius: usize, color: Color32) {
        let r_squared = radius * radius;
        for dy in -(radius as i32)..=radius as i32 {
            for dx in -(radius as i32)..=radius as i32 {
                if dx * dx + dy * dy <= r_squared as i32 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && nx < CANVAS_SIZE as i32 && ny < CANVAS_SIZE as i32 {
                        self.layers[self.active_layer].pixels[nx as usize][ny as usize] = color;
                    }
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

            if x > 0 {
                stack.push((x - 1, y));
            }
            if x < CANVAS_SIZE - 1 {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y < CANVAS_SIZE - 1 {
                stack.push((x, y + 1));
            }
        }
    }
}

impl eframe::App for PixelArtEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Контекстное меню для настроек кисти
        if self.show_brush_settings {
            egui::Window::new("Настройки кисти")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Радиус: {}", self.brush_radius));
                    ui.add(egui::Slider::new(&mut self.brush_radius, 1..=MAX_BRUSH_RADIUS));
                    
                    if ui.button("Закрыть").clicked() {
                        self.show_brush_settings = false;
                    }
                });
        }

        // Панель инструментов сверху
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Кнопка карандаша с контекстным меню
                let pencil_btn = ui.selectable_value(&mut self.current_tool, Tool::Pencil, "✏️ Карандаш");
                if pencil_btn.secondary_clicked() {
                    self.show_brush_settings = true;
                }
                
                ui.selectable_value(&mut self.current_tool, Tool::Eraser, "🧽 Ластик");
                ui.selectable_value(&mut self.current_tool, Tool::Fill, "🎨 Заливка");
                
                ui.separator();
                
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
            });
        });

        // Основной интерфейс
        egui::CentralPanel::default().show(ctx, |ui| {
            let canvas_response = ui.allocate_response(
                egui::Vec2::new(
                    CANVAS_SIZE as f32 * PIXEL_SIZE, 
                    CANVAS_SIZE as f32 * PIXEL_SIZE
                ), 
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
                    
                    // Смешиваем цвета всех видимых слоев
                    for layer in &self.layers {
                        if layer.visible {
                            let layer_color = layer.pixels[x][y];
                            if layer_color != Color32::TRANSPARENT {
                                color = layer_color;
                            }
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

        // Правая панель (палитра + слои)
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            // Палитра цветов
            ui.label("🎨 Палитра");
            ui.separator();
            
            let colors = [
                Color32::BLACK, Color32::WHITE,
                Color32::RED, Color32::GREEN, Color32::BLUE,
                Color32::YELLOW, Color32::from_rgb(255, 165, 0), // ORANGE
                Color32::from_rgb(128, 0, 128), // PURPLE
            ];
            
            ui.horizontal_wrapped(|ui| {
                for color in colors {
                    if ui.color_edit_button_srgba(&mut color.to_owned()).clicked() {
                        self.current_color = color;
                    }
                }
            });

            // Слои под палитрой
            ui.add_space(20.0);
            ui.label("📚 Слои");
            ui.separator();
            
            for (i, layer) in self.layers.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut layer.visible, "");
                    
                    let layer_name = format!("Слой {}", i + 1);
                    if ui.selectable_label(i == self.active_layer, &layer_name).clicked() {
                        self.active_layer = i;
                    }
                });
            }
        });
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