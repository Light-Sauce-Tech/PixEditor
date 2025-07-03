use eframe::egui::Color32;

/// Перечисление доступных инструментов редактора
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Eraser,
    Brush,
    Fill,
    Eyedropper,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Pencil
    }
}

impl Tool {
    /// Возвращает иконку для инструмента
    pub fn icon(&self) -> &'static str {
        match self {
            Tool::Pencil => "✏️",
            Tool::Eraser => "🧽",
            Tool::Brush => "🖌️",
            Tool::Fill => "🌊",
            Tool::Eyedropper => "👁️",
        }
    }

    /// Возвращает название инструмента
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Pencil => "Карандаш",
            Tool::Eraser => "Ластик",
            Tool::Brush => "Кисть",
            Tool::Fill => "Заливка",
            Tool::Eyedropper => "Пипетка",
        }
    }

    /// Возвращает описание инструмента
    pub fn description(&self) -> &'static str {
        match self {
            Tool::Pencil => "Рисование отдельных пикселей",
            Tool::Eraser => "Стирание пикселей",
            Tool::Brush => "Рисование с заданным размером кисти",
            Tool::Fill => "Заливка области одним цветом",
            Tool::Eyedropper => "Выбор цвета с холста",
        }
    }

    /// Определяет, требует ли инструмент настройки размера кисти
    pub fn needs_brush_size(&self) -> bool {
        matches!(self, Tool::Pencil | Tool::Brush | Tool::Eraser)
    }

    /// Применяет инструмент к пикселю
    pub fn apply(
        &self,
        pixels: &mut Vec<Vec<Color32>>,
        x: usize,
        y: usize,
        color: Color32,
        brush_size: usize,
    ) {
        match self {
            Tool::Pencil | Tool::Brush | Tool::Eraser => {
                let actual_color = match self {
                    Tool::Eraser => Color32::TRANSPARENT,
                    _ => color,
                };

                for dy in 0..brush_size {
                    for dx in 0..brush_size {
                        let nx = x.saturating_add(dx);
                        let ny = y.saturating_add(dy);
                        if nx < pixels.len() && ny < pixels[0].len() {
                            pixels[nx][ny] = actual_color;
                        }
                    }
                }
            }
            Tool::Fill => {
                Self::flood_fill(pixels, x, y, color);
            }
            Tool::Eyedropper => {} // Обработка пипетки происходит отдельно
        }
    }

    /// Алгоритм заливки области
    fn flood_fill(pixels: &mut Vec<Vec<Color32>>, x: usize, y: usize, new_color: Color32) {
        let old_color = pixels[x][y];
        if old_color == new_color {
            return;
        }

        let width = pixels.len();
        let height = if width > 0 { pixels[0].len() } else { 0 };

        let mut stack = vec![(x, y)];
        while let Some((x, y)) = stack.pop() {
            if x >= width || y >= height || pixels[x][y] != old_color {
                continue;
            }

            pixels[x][y] = new_color;

            if x > 0 {
                stack.push((x - 1, y));
            }
            if x < width - 1 {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y < height - 1 {
                stack.push((x, y + 1));
            }
        }
    }
}