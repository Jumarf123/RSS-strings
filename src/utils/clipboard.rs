use egui::Context;

pub fn copy_to_clipboard(ctx: &Context, text: String) {
    ctx.copy_text(text);
}
