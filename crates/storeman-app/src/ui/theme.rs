use eframe::egui::{self, Color32, Stroke, Vec2, Visuals};

pub const SIDEBAR_BG: Color32 = Color32::from_rgb(45, 55, 35);
pub const CONTENT_BG: Color32 = Color32::from_rgb(40, 40, 40);
pub const ACCENT_RED: Color32 = Color32::from_rgb(180, 30, 30);
pub const TEXT_COLOR: Color32 = Color32::from_rgb(220, 220, 200);
pub const BUTTON_COLOR: Color32 = Color32::from_rgb(75, 90, 55);
pub const BUTTON_HOVER: Color32 = Color32::from_rgb(95, 115, 70);
pub const PANEL_BG: Color32 = Color32::from_rgb(50, 52, 48);
pub const HEADER_BG: Color32 = Color32::from_rgb(35, 45, 25);
pub const WARNING_COLOR: Color32 = Color32::from_rgb(200, 150, 30);
pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(60, 160, 60);

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = Visuals::dark();
    style.visuals.panel_fill = CONTENT_BG;
    style.visuals.window_fill = CONTENT_BG;
    style.visuals.extreme_bg_color = Color32::from_rgb(30, 30, 28);
    style.visuals.faint_bg_color = Color32::from_rgb(45, 47, 43);
    style.visuals.widgets.inactive.bg_fill = BUTTON_COLOR;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_COLOR);
    style.visuals.widgets.hovered.bg_fill = BUTTON_HOVER;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(110, 130, 80);
    style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
    style.visuals.override_text_color = Some(TEXT_COLOR);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_COLOR);
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.button_padding = Vec2::new(12.0, 6.0);
    ctx.set_style(style);
}
