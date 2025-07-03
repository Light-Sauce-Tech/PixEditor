mod panels;
mod dialogs;
mod widgets;

pub use panels::*;
pub use dialogs::*;
pub use widgets::*;

pub fn render(editor: &mut PixelArtEditor, ctx: &egui::Context) {
    // Основной рендеринг интерфейса
    render_main_menu(editor, ctx);
    render_tools_panel(editor, ctx);
    render_canvas(editor, ctx);
    render_right_panel(editor, ctx);
    render_dialogs(editor, ctx);
}