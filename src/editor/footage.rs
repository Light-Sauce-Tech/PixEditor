use eframe::egui::Color32;
use std::fmt;

/// Шаблон холста с предустановленными параметрами
#[derive(Debug, Clone)]
pub struct Footage {
    /// Название шаблона
    pub name: String,
    /// Ширина в пикселях
    pub width: usize,
    /// Высота в пикселях
    pub height: usize,
    /// Превью шаблона (цвета пикселей)
    pub preview: Vec<Vec<Color32>>,
    /// Категория шаблона
    pub category: FootageCategory,
}

/// Категории шаблонов
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootageCategory {
    Square,
    Horizontal,
    Vertical,
    Character,
    Tile,
    Custom,
}

impl fmt::Display for FootageCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FootageCategory::Square => write!(f, "🔲 Квадратные"),
            FootageCategory::Horizontal => write!(f, "🖥️ Горизонтальные"),
            FootageCategory::Vertical => write!(f, "📱 Вертикальные"),
            FootageCategory::Character => write!(f, "🧑 Персонажи"),
            FootageCategory::Tile => write!(f, "🧱 Тайлы"),
            FootageCategory::Custom => write!(f, "⚙️ Пользовательские"),
        }
    }
}

impl Footage {
    /// Создает новый шаблон
    pub fn new(
        name: impl Into<String>,
        width: usize,
        height: usize,
        category: FootageCategory,
    ) -> Self {
        let name = name.into();
        let preview = Self::create_preview(width, height, Self::category_color(category));
        Self {
            name,
            width,
            height,
            preview,
            category,
        }
    }

    /// Создает шаблон персонажа
    pub fn new_character(width: usize, height: usize) -> Self {
        let name = format!("Персонаж {}x{}", width, height);
        let preview = Self::create_character_template(width, height);
        Self {
            name,
            width,
            height,
            preview,
            category: FootageCategory::Character,
        }
    }

    /// Создает шаблон тайла
    pub fn new_tile(width: usize, height: usize) -> Self {
        let name = format!("Тайл {}x{}", width, height);
        let preview = Self::create_tile_template(width, height);
        Self {
            name,
            width,
            height,
            preview,
            category: FootageCategory::Tile,
        }
    }

    /// Цвет по умолчанию для категории
    fn category_color(category: FootageCategory) -> Color32 {
        match category {
            FootageCategory::Square => Color32::from_rgb(100, 200, 100),
            FootageCategory::Horizontal => Color32::from_rgb(200, 150, 50),
            FootageCategory::Vertical => Color32::from_rgb(200, 50, 150),
            FootageCategory::Character => Color32::from_rgb(100, 100, 200),
            FootageCategory::Tile => Color32::from_rgb(150, 150, 50),
            FootageCategory::Custom => Color32::from_rgb(100, 100, 100),
        }
    }

    /// Создает превью шаблона
    pub fn create_preview(width: usize, height: usize, color: Color32) -> Vec<Vec<Color32>> {
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

    /// Создает шаблон персонажа
    pub fn create_character_template(width: usize, height: usize) -> Vec<Vec<Color32>> {
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

    /// Создает шаблон тайла
    pub fn create_tile_template(width: usize, height: usize) -> Vec<Vec<Color32>> {
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

    /// Возвращает стандартные шаблоны
    pub fn default_footages() -> Vec<Self> {
        vec![
            Self::new("16x16 (Квадрат)", 16, 16, FootageCategory::Square),
            Self::new("32x32 (Квадрат)", 32, 32, FootageCategory::Square),
            Self::new("64x64 (Квадрат)", 64, 64, FootageCategory::Square),
            Self::new("16:9 (HD)", 32, 18, FootageCategory::Horizontal),
            Self::new("32:18 (Full HD)", 64, 36, FootageCategory::Horizontal),
            Self::new("4:3 (Классический)", 32, 24, FootageCategory::Horizontal),
            Self::new("9:16 (Вертикальный)", 18, 32, FootageCategory::Vertical),
            Self::new("3:4 (Портрет)", 24, 32, FootageCategory::Vertical),
            Self::new_character(16, 32),
            Self::new_character(32, 64),
            Self::new_tile(16, 16),
            Self::new_tile(32, 32),
        ]
    }
}