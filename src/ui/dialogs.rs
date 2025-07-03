use eframe::egui;
use std::path::PathBuf;
use crate::editor::PixelArtEditor;

/// Рендеринг всех диалоговых окон
pub fn render_dialogs(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    if editor.show_save_dialog {
        render_save_dialog(editor, ctx);
    }
    
    if editor.show_canvas_selector {
        render_canvas_selector(editor, ctx);
    }
}

/// Диалог сохранения файла
fn render_save_dialog(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    let mut should_close = false;
    let mut should_save = false;
    
    egui::Window::new("Сохранить изображение")
        .collapsible(false)
        .resizable(false)
        .open(&mut editor.show_save_dialog)
        .show(ctx, |ui| {
            ui.label("Выберите место для сохранения:");
            
            if ui.button("Выбрать папку").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("pixel_art.png")
                    .save_file()
                {
                    editor.pending_save_path = Some(path);
                }
            }
            
            if let Some(path) = &editor.pending_save_path {
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
        editor.show_save_dialog = false;
    }

    if should_save {
        if let Some(path) = &editor.pending_save_path {
            if let Err(e) = editor.save_to_png(path) {
                eprintln!("Ошибка сохранения: {}", e);
            } else {
                println!("Изображение сохранено как {}", path.display());
                editor.save_path = editor.pending_save_path.take();
            }
        }
    }
}

/// Диалог выбора холста
fn render_canvas_selector(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    egui::Area::new("canvas_selector_area")
        .fixed_pos(egui::pos2(0.0, 0.0))
        .movable(false)
        .interactable(true)
        .show(ctx, |ui| {
            // Фоновый градиент
            let screen_rect = ui.max_rect();
            let mut mesh = egui::Mesh::default();
            mesh.colored_vertex(screen_rect.left_top(), egui::Color32::from_rgb(30, 30, 50));
            mesh.colored_vertex(screen_rect.right_top(), egui::Color32::from_rgb(50, 30, 50));
            mesh.colored_vertex(screen_rect.right_bottom(), egui::Color32::from_rgb(20, 10, 30));
            mesh.colored_vertex(screen_rect.left_bottom(), egui::Color32::from_rgb(10, 20, 30));
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(2, 3, 0);
            ui.painter().add(mesh);

            // Окно выбора холста
            egui::Window::new("🎨 Pixel Art Editor - Создание нового холста")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(egui::Frame::window(&ctx.style()).inner_margin(20.0))
                .show(ctx, |ui| {
                    ui.heading("Создать новый холст");
                    ui.separator();
                    
                    // Кастомный размер
                    ui.horizontal(|ui| {
                        ui.label("Ширина:");
                        if ui.text_edit_singleline(&mut editor.custom_width).changed() {
                            if let Ok(num) = editor.custom_width.parse::<usize>() {
                                if num > 0 && num <= 256 {
                                    editor.canvas_width = num;
                                }
                            }
                        }
                        
                        ui.label("Высота:");
                        if ui.text_edit_singleline(&mut editor.custom_height).changed() {
                            if let Ok(num) = editor.custom_height.parse::<usize>() {
                                if num > 0 && num <= 256 {
                                    editor.canvas_height = num;
                                }
                            }
                        }
                        
                        if ui.button("Создать").clicked() {
                            editor.init_with_size(editor.canvas_width, editor.canvas_height);
                        }
                    });
                    
                    ui.separator();
                    ui.heading("Или выберите шаблон:");
                    
                    // Список шаблонов
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        render_footage_category(ui, editor, "🔲 Квадратные", &[0, 1, 2]);
                        render_footage_category(ui, editor, "🖥️ Горизонтальные", &[3, 4, 5]);
                        render_footage_category(ui, editor, "📱 Вертикальные", &[6, 7]);
                        render_footage_category(ui, editor, "🧑 Персонажи", &[8, 9]);
                        render_footage_category(ui, editor, "🧱 Тайлы", &[10, 11]);
                    });
                });
        });
}

/// Рендеринг категории шаблонов
fn render_footage_category(
    ui: &mut egui::Ui,
    editor: &mut PixelArtEditor,
    category_name: &str,
    indices: &[usize],
) {
    ui.collapsing(category_name, |ui| {
        let footages = editor.get_footages();
        egui::Grid::new(format!("{}_grid", category_name))
            .num_columns(3)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                for &i in indices {
                    if i >= footages.len() { continue; }
                    
                    let footage = &footages[i];
                    render_footage_preview(ui, editor, footage);
                }
                ui.end_row();
            });
    });
}

/// Рендеринг превью шаблона
fn render_footage_preview(
    ui: &mut egui::Ui,
    editor: &mut PixelArtEditor,
    footage: &crate::editor::Footage,
) {
    ui.vertical(|ui| {
        let preview_size = 100.0;
        let (rect, _) = ui.allocate_exact_size(
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
        ui.label(&footage.name);
        
        if ui.button("Выбрать").clicked() {
            editor.init_with_footage(footage.clone());
        }
    });
}