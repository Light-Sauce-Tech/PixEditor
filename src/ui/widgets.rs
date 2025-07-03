use eframe::egui;
use crate::editor::{PixelArtEditor, Tool};

/// Виджет цветового круга
pub fn color_wheel(editor: &mut PixelArtEditor, ui: &mut egui::Ui) {
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
}

/// Виджет редактора цвета
pub fn color_editor(editor: &mut PixelArtEditor, ui: &mut egui::Ui) {
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
}

/// Виджет управления слоями
pub fn layers_panel(editor: &mut PixelArtEditor, ui: &mut egui::Ui) {
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
            editor.layers.push(crate::editor::Layer::new(
                editor.canvas_width,
                editor.canvas_height
            ));
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

/// Виджет инструментов
pub fn tools_panel(editor: &mut PixelArtEditor, ui: &mut egui::Ui) {
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
}

/// Виджет управления масштабом
pub fn zoom_controls(editor: &mut PixelArtEditor, ui: &mut egui::Ui) {
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
}

/// Виджет превью шаблона холста
pub fn footage_preview(
    ui: &mut egui::Ui,
    footage: &crate::editor::Footage,
    preview_size: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::Vec2::new(preview_size, preview_size),
        egui::Sense::click()
    );
    
    let painter = ui.painter();
    painter.rect_filled(rect, 5.0, egui::Color32::from_gray(30));
    
    let scale_x = preview_size / footage.width as f32;
    let scale_y = preview_size / footage.height as f32;
    
    for x in 0..footage.width {
        for y in 0..footage.height {
            let color = footage.preview[x][y];
            if color != egui::Color32::TRANSPARENT {
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(
                        rect.min.x + x as f32 * scale_x,
                        rect.min.y + y as f32 * scale_y,
                    ),
                    egui::Vec2::new(scale_x, scale_y),
                );
                painter.rect_filled(rect, 0.0, color);
            }
        }
    }
    
    painter.rect_stroke(rect, 5.0, (1.0, egui::Color32::from_gray(80)));
    response
}