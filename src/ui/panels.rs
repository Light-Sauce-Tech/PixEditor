use eframe::egui;
use crate::editor::{PixelArtEditor, Tool};

/// Рендеринг главного меню
pub fn render_main_menu(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Меню "Файл"
            if ui.selectable_label(editor.show_file_menu, "Файл").clicked() {
                editor.show_file_menu = !editor.show_file_menu;
            }

            // Другие пункты меню
            ui.selectable_value(&mut (), (), "Главная");
            ui.selectable_value(&mut (), (), "Палитра");
            ui.selectable_value(&mut (), (), "Анимация");
            ui.selectable_value(&mut (), (), "Холст");
        });
    });

    // Выпадающее меню файла
    if editor.show_file_menu {
        render_file_menu(editor, ctx);
    }
}

/// Рендеринг меню файла
fn render_file_menu(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    egui::Window::new("Файл")
        .collapsible(false)
        .resizable(false)
        .fixed_pos(egui::pos2(10.0, 25.0))
        .show(ctx, |ui| {
            if ui.button("Новый").clicked() {
                editor.show_canvas_selector = true;
                editor.show_file_menu = false;
            }
            if ui.button("Открыть").clicked() {
                // TODO: Реализовать открытие файла
                editor.show_file_menu = false;
            }
            if ui.button("Сохранить").clicked() {
                editor.show_save_dialog = true;
                editor.show_file_menu = false;
            }
            if ui.button("Экспорт").clicked() {
                // TODO: Реализовать экспорт
                editor.show_file_menu = false;
            }
        });
}

/// Рендеринг панели инструментов
pub fn render_tools_panel(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    egui::TopBottomPanel::top("tools_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            for tool in [
                Tool::Pencil,
                Tool::Eraser,
                Tool::Brush,
                Tool::Fill,
                Tool::Eyedropper,
            ] {
                let is_selected = editor.current_tool == tool;
                let label = format!("{} {}", tool.icon(), tool.name());
                
                if ui.selectable_label(is_selected, &label).clicked() {
                    editor.current_tool = tool;
                    if tool.needs_brush_size() {
                        editor.show_brush_settings = true;
                    }
                }
            }
        });
    });

    // Окно настроек кисти
    if editor.show_brush_settings {
        render_brush_settings(editor, ctx);
    }
}

/// Рендеринг настроек кисти
fn render_brush_settings(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    egui::Window::new("Настройки кисти")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(format!("Размер: {}x{}", editor.brush_radius, editor.brush_radius));
            ui.add(egui::Slider::new(&mut editor.brush_radius, 1..=super::MAX_BRUSH_RADIUS));
            if ui.button("Закрыть").clicked() {
                editor.show_brush_settings = false;
            }
        });
}

/// Рендеринг правой панели
pub fn render_right_panel(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    egui::SidePanel::right("right_panel").show(ctx, |ui| {
        ui.vertical(|ui| {
            render_zoom_controls(editor, ui);
            render_color_picker(editor, ui);
            render_layers_panel(editor, ui);
        });
    });
}

/// Рендеринг управления масштабом
fn render_zoom_controls(editor: &mut PixelArtEditor, ui: &mut egui::Ui) {
    ui.label("Масштаб");
    ui.horizontal(|ui| {
        if ui.button("-").clicked() {
            editor.zoom = (editor.zoom - super::ZOOM_STEP).max(super::MIN_ZOOM);
        }
        ui.label(format!("{:.0}%", editor.zoom * 100.0));
        if ui.button("+").clicked() {
            editor.zoom = (editor.zoom + super::ZOOM_STEP).min(super::MAX_ZOOM);
        }
    });
    if ui.button("Сброс").clicked() {
        editor.zoom = 1.0;
        editor.center_canvas(ui.available_size());
    }
    ui.separator();
}

/// Рендеринг палитры цветов
fn render_color_picker(editor: &mut PixelArtEditor, ui: &mut egui::Ui) {
    ui.label("RGB круг");
    if let Some(texture) = &editor.color_wheel_texture {
        let size = 150.0;
        let response = ui.add(
            egui::Image::new(texture, egui::Vec2::new(size, size))
                .sense(egui::Sense::drag())
        );
        
        let rect = response.rect;
        
        // Обработка выбора цвета
        if response.drag_started() {
            editor.is_color_picking = true;
        }
        
        if editor.is_color_picking {
            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                editor.update_color_from_wheel(pos, rect);
            }
        }
        
        if response.drag_released() {
            editor.is_color_picking = false;
        }
        
        // Маркер текущего цвета
        if let Some(marker_pos) = editor.get_color_wheel_marker_pos(rect) {
            ui.painter().circle(
                marker_pos,
                5.0,
                egui::Color32::WHITE,
                egui::Stroke::new(2.0, egui::Color32::BLACK),
            );
        }
    }

    // Редактирование цвета
    ui.separator();
    ui.label("Редактирование цвета");
    
    // Превью цвета
    ui.horizontal(|ui| {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                ui.cursor().min,
                egui::Vec2::new(30.0, 30.0)
            ),
            0.0,
            editor.current_color
        );
        
        ui.vertical(|ui| {
            ui.label(format!("R: {}", editor.current_color.r()));
            ui.label(format!("G: {}", editor.current_color.g()));
            ui.label(format!("B: {}", editor.current_color.b()));
            ui.label(format!("A: {}", editor.current_color.a()));
        });
    });
    
    // RGB ползунки
    if ui.add(egui::Slider::new(&mut editor.color_edit_rgba[0], 0.0..=1.0).text("R")).changed() {
        editor.update_current_color_from_rgba();
    }
    if ui.add(egui::Slider::new(&mut editor.color_edit_rgba[1], 0.0..=1.0).text("G")).changed() {
        editor.update_current_color_from_rgba();
    }
    if ui.add(egui::Slider::new(&mut editor.color_edit_rgba[2], 0.0..=1.0).text("B")).changed() {
        editor.update_current_color_from_rgba();
    }
    if ui.add(egui::Slider::new(&mut editor.color_edit_rgba[3], 0.0..=1.0).text("A")).changed() {
        editor.update_current_color_from_rgba();
    }
    
    ui.separator();
}

/// Рендеринг панели слоев
fn render_layers_panel(editor: &mut PixelArtEditor, ui: &mut egui::Ui) {
    ui.label("Слои");
    
    for (i, layer) in editor.layers.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.checkbox(&mut layer.visible, "");
            if ui.selectable_label(i == editor.active_layer, &format!("Слой {}", i + 1)).clicked() {
                editor.active_layer = i;
            }
        });
    }
    
    // Кнопки управления слоями
    ui.horizontal(|ui| {
        if ui.button("+").clicked() {
            editor.layers.push(super::Layer::new(editor.canvas_width, editor.canvas_height));
            editor.active_layer = editor.layers.len() - 1;
        }
        if ui.button("-").clicked() && editor.layers.len() > 1 {
            editor.layers.remove(editor.active_layer);
            if editor.active_layer >= editor.layers.len() {
                editor.active_layer = editor.layers.len() - 1;
            }
        }
    });
}

/// Рендеринг холста
pub fn render_canvas(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let available_size = ui.available_size();
        editor.center_canvas(available_size);

        let canvas_size = egui::Vec2::new(
            editor.canvas_width as f32 * super::PIXEL_SIZE * editor.zoom,
            editor.canvas_height as f32 * super::PIXEL_SIZE * editor.zoom,
        );

        let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
        editor.handle_zoom_and_pan(ctx, &response);

        let canvas_rect = egui::Rect::from_min_size(
            response.rect.min + editor.canvas_offset,
            canvas_size,
        );

        // Фон холста
        painter.rect_filled(canvas_rect, 0.0, egui::Color32::from_gray(30));

        // Отрисовка пикселей
        for x in 0..editor.canvas_width {
            for y in 0..editor.canvas_height {
                let mut color = egui::Color32::TRANSPARENT;
                for layer in &editor.layers {
                    if layer.visible && layer.pixels[x][y] != egui::Color32::TRANSPARENT {
                        color = layer.pixels[x][y];
                    }
                }

                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(
                        canvas_rect.min.x + x as f32 * super::PIXEL_SIZE * editor.zoom,
                        canvas_rect.min.y + y as f32 * super::PIXEL_SIZE * editor.zoom,
                    ),
                    egui::Vec2::splat(super::PIXEL_SIZE * editor.zoom),
                );
                
                painter.rect_filled(rect, 0.0, color);
                painter.rect_stroke(rect, 0.0, (1.0, egui::Color32::from_gray(100)));
            }
        }

        // Обработка рисования
        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let x = ((pos.x - canvas_rect.min.x) / (super::PIXEL_SIZE * editor.zoom)).floor() as usize;
                let y = ((pos.y - canvas_rect.min.y) / (super::PIXEL_SIZE * editor.zoom)).floor() as usize;
                if x < editor.canvas_width && y < editor.canvas_height {
                    editor.draw_on_canvas((x, y));
                }
            }
        }
    });
}