use std::sync::Arc;

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, Style, Stroke, Visuals};

pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();
    let bg = Color32::from_rgb(14, 16, 20);
    let panel = Color32::from_rgb(19, 22, 28);
    let panel_inner = Color32::from_rgb(24, 28, 36);
    let widget = Color32::from_rgb(32, 36, 46);
    let widget_hover = Color32::from_rgb(38, 44, 58);
    let widget_active = Color32::from_rgb(46, 54, 72);
    let border = Color32::from_rgb(46, 54, 68);
    let border_soft = Color32::from_rgb(38, 44, 56);
    let accent = Color32::from_rgb(90, 163, 255);
    let text = Color32::from_rgb(232, 236, 244);

    visuals.window_fill = bg;
    visuals.panel_fill = panel;
    visuals.window_stroke = Stroke::new(1.0, border_soft);
    visuals.faint_bg_color = Color32::from_rgb(22, 26, 34);
    visuals.extreme_bg_color = Color32::from_rgb(16, 18, 24);
    visuals.text_edit_bg_color = Some(panel_inner);
    visuals.code_bg_color = Color32::from_rgb(28, 32, 42);
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = Stroke::new(1.0, accent);
    visuals.hyperlink_color = accent;
    visuals.override_text_color = Some(text);
    visuals.warn_fg_color = Color32::from_rgb(245, 178, 82);
    visuals.error_fg_color = Color32::from_rgb(255, 102, 118);

    visuals.widgets.noninteractive.bg_fill = panel_inner;
    visuals.widgets.noninteractive.weak_bg_fill = panel_inner;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_soft);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill = widget;
    visuals.widgets.inactive.weak_bg_fill = widget;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_soft);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text);

    visuals.widgets.hovered.bg_fill = widget_hover;
    visuals.widgets.hovered.weak_bg_fill = widget_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, border);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text);

    visuals.widgets.active.bg_fill = widget_active;
    visuals.widgets.active.weak_bg_fill = widget_active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, text);

    visuals.widgets.open.bg_fill = widget_active;
    visuals.widgets.open.weak_bg_fill = widget_active;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, text);
    ctx.set_visuals(visuals);

    let mut fonts = FontDefinitions::default();
    // Prefer Segoe UI if available; fallback to built-in fonts.
    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
        fonts
            .font_data
            .insert("segoe-ui".to_owned(), Arc::new(FontData::from_owned(bytes)));
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe-ui".to_owned());
    }
    ctx.set_fonts(fonts);

    let mut style: Style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.interact_size = egui::vec2(36.0, 30.0);
    style.visuals.widgets.noninteractive.corner_radius = 9.0.into();
    style.visuals.widgets.inactive.corner_radius = 9.0.into();
    style.visuals.widgets.active.corner_radius = 9.0.into();
    style.visuals.widgets.hovered.corner_radius = 9.0.into();
    style.visuals.widgets.open.corner_radius = 9.0.into();
    style.visuals.window_corner_radius = 10.0.into();
    style.visuals.menu_corner_radius = 9.0.into();
    style.visuals.striped = true;
    style.visuals.slider_trailing_fill = true;
    ctx.set_style(style);
}
