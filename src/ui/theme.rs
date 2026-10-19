//! ui/theme.rs - Thème cyber sombre et style visuel pour egui.

use egui::{Color32, Stroke, Rounding, Visuals};

pub const COLOR_BG_DARK: Color32 = Color32::from_rgb(10, 14, 23);
pub const COLOR_CARD_BG: Color32 = Color32::from_rgba_premultiplied(18, 25, 42, 230);
pub const COLOR_CYAN: Color32 = Color32::from_rgb(0, 242, 254);
pub const COLOR_BLUE: Color32 = Color32::from_rgb(79, 172, 254);
pub const COLOR_EMERALD: Color32 = Color32::from_rgb(44, 182, 125);
pub const COLOR_ROSE: Color32 = Color32::from_rgb(244, 63, 94);
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(Color32::from_rgb(248, 250, 252));
    visuals.panel_fill = COLOR_BG_DARK;
    visuals.window_fill = COLOR_CARD_BG;
    visuals.window_rounding = Rounding::same(12.0);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 20));

    // Widgets normaux
    visuals.widgets.inactive.bg_fill = Color32::from_rgba_premultiplied(20, 28, 48, 200);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 15));

    // Widgets hover
    visuals.widgets.hovered.bg_fill = Color32::from_rgba_premultiplied(30, 42, 70, 240);
    visuals.widgets.hovered.rounding = Rounding::same(8.0);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_CYAN);

    // Widgets actifs / cliqués
    visuals.widgets.active.bg_fill = Color32::from_rgba_premultiplied(0, 242, 254, 50);
    visuals.widgets.active.rounding = Rounding::same(8.0);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, COLOR_CYAN);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    ctx.set_style(style);
}
