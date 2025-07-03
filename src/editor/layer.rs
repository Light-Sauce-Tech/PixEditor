use eframe::egui::Color32;
use serde::{Serialize, Deserialize};

/// Представляет слой пиксельной графики
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Двумерный массив пикселей (x, y)
    pixels: Vec<Vec<Color32>>,
    /// Видимость слоя
    visible: bool,
    /// Имя слоя (опционально)
    name: Option<String>,
    /// Прозрачность слоя (0.0 - полностью прозрачный, 1.0 - непрозрачный)
    opacity: f32,
    /// Блокировка слоя от редактирования
    locked: bool,
}

impl Layer {
    /// Создает новый прозрачный слой заданного размера
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![vec![Color32::TRANSPARENT; height]; width],
            visible: true,
            name: None,
            opacity: 1.0,
            locked: false,
        }
    }

    /// Возвращает ширину слоя в пикселях
    pub fn width(&self) -> usize {
        self.pixels.len()
    }

    /// Возвращает высоту слоя в пикселях
    pub fn height(&self) -> usize {
        if self.pixels.is_empty() {
            0
        } else {
            self.pixels[0].len()
        }
    }

    /// Проверяет, видим ли слой
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Устанавливает видимость слоя
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Проверяет, заблокирован ли слой для редактирования
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Устанавливает блокировку слоя
    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }

    /// Возвращает прозрачность слоя
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Устанавливает прозрачность слоя
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Возвращает имя слоя (если есть)
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Устанавливает имя слоя
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Получает цвет пикселя с учетом прозрачности слоя
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color32> {
        if x < self.width() && y < self.height() {
            let mut color = self.pixels[x][y];
            if self.opacity < 1.0 {
                color = color.gamma_multiply(self.opacity);
            }
            Some(color)
        } else {
            None
        }
    }

    /// Устанавливает цвет пикселя (если слой не заблокирован)
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color32) -> bool {
        if self.locked || x >= self.width() || y >= self.height() {
            return false;
        }
        self.pixels[x][y] = color;
        true
    }

    /// Заполняет весь слой указанным цветом
    pub fn fill(&mut self, color: Color32) {
        if self.locked {
            return;
        }
        for row in &mut self.pixels {
            for pixel in row {
                *pixel = color;
            }
        }
    }

    /// Обрезает или расширяет слой до нового размера
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        let mut new_pixels = vec![vec![Color32::TRANSPARENT; new_height]; new_width];
        
        let copy_width = self.width().min(new_width);
        let copy_height = self.height().min(new_height);
        
        for x in 0..copy_width {
            for y in 0..copy_height {
                new_pixels[x][y] = self.pixels[x][y];
            }
        }
        
        self.pixels = new_pixels;
    }

    /// Создает копию слоя
    pub fn duplicate(&self) -> Self {
        Self {
            pixels: self.pixels.clone(),
            visible: self.visible,
            name: self.name.as_ref().map(|n| format!("{} (копия)", n)),
            opacity: self.opacity,
            locked: self.locked,
        }
    }

    /// Объединяет текущий слой с другим (поверх него)
    pub fn merge_with(&mut self, other: &Layer) {
        if self.width() != other.width() || self.height() != other.height() {
            return;
        }
        
        for x in 0..self.width() {
            for y in 0..self.height() {
                let other_color = other.pixels[x][y];
                if other_color != Color32::TRANSPARENT {
                    self.pixels[x][y] = other_color;
                }
            }
        }
    }
}