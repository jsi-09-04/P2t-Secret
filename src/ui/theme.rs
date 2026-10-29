//! ui/theme.rs - Thème clair moderne, lisible et ergonomique pour egui.

use egui::{
    Color32, FontFamily, FontId, Margin, Rounding, Stroke, TextStyle, Vec2, Visuals,
};

// Palette Thème Clair Moderne & Contrasté (Inspiré de macOS Sequoia & Linear App)
pub const COLOR_BG: Color32 = Color32::from_rgb(248, 250, 252);        // Fond principal #f8fafc
pub const COLOR_SURFACE: Color32 = Color32::from_rgb(255, 255, 255);   // Cartes & Panneaux #ffffff
pub const COLOR_SURFACE_ALT: Color32 = Color32::from_rgb(241, 245, 249);// Conteneurs champs #f1f5f9
pub const COLOR_SIDEBAR: Color32 = Color32::from_rgb(248, 250, 252);    // Barre latérale
pub const COLOR_BORDER: Color32 = Color32::from_rgb(226, 232, 240);    // Bordures nettes #e2e8f0
pub const COLOR_BORDER_FOCUS: Color32 = Color32::from_rgb(59, 130, 246);// Focus #3b82f6

pub const COLOR_TEXT_MAIN: Color32 = Color32::from_rgb(15, 23, 42);    // Titres & Textes clés #0f172a (très lisible)
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(71, 85, 105);  // Textes secondaires #475569
pub const COLOR_TEXT_LIGHT: Color32 = Color32::from_rgb(148, 163, 184);// Placeholder & Hints #94a3b8

pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(37, 99, 235);     // Bleu Primaire #2563eb
pub const COLOR_PRIMARY_LIGHT: Color32 = Color32::from_rgb(239, 246, 255); // Bleu très clair #eff6ff
pub const COLOR_EMERALD: Color32 = Color32::from_rgb(5, 150, 105);     // Vert Succès / 2FA #059669
pub const COLOR_EMERALD_BG: Color32 = Color32::from_rgb(236, 253, 245); // Vert clair
pub const COLOR_ROSE: Color32 = Color32::from_rgb(220, 38, 38);        // Rouge Danger / Delete #dc2626
pub const COLOR_ROSE_BG: Color32 = Color32::from_rgb(254, 242, 242);   // Rouge clair
pub const COLOR_AMBER: Color32 = Color32::from_rgb(217, 119, 6);       // Orange Attention #d97706

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::light();
    visuals.override_text_color = Some(COLOR_TEXT_MAIN);
    visuals.panel_fill = COLOR_BG;
    visuals.window_fill = COLOR_SURFACE;
    visuals.window_rounding = Rounding::same(14.0);
    visuals.window_stroke = Stroke::new(1.0_f32, COLOR_BORDER);

    // Style des widgets inactifs (boutons secondaires, champs de saisie)
    visuals.widgets.inactive.bg_fill = COLOR_SURFACE;
    visuals.widgets.inactive.rounding = Rounding::same(8.0);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_MAIN);

    // Style des widgets survolés (hover)
    visuals.widgets.hovered.bg_fill = COLOR_SURFACE_ALT;
    visuals.widgets.hovered.rounding = Rounding::same(8.0);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, COLOR_PRIMARY);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, COLOR_PRIMARY);

    // Style des widgets cliqués / actifs
    visuals.widgets.active.bg_fill = Color32::from_rgb(224, 231, 255);
    visuals.widgets.active.rounding = Rounding::same(8.0);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, COLOR_PRIMARY);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, COLOR_PRIMARY);

    // Style des champs de texte
    visuals.widgets.open.bg_fill = COLOR_SURFACE;
    visuals.widgets.open.rounding = Rounding::same(8.0);
    visuals.widgets.open.bg_stroke = Stroke::new(1.5_f32, COLOR_PRIMARY);

    visuals.extreme_bg_color = COLOR_SURFACE; // Fond des TextEdit

    ctx.set_visuals(visuals);

    // Ajustement de la typographie, espacements et taille des boutons
    let mut style = (*ctx.style()).clone();
    
    style.text_styles = [
        (TextStyle::Heading, FontId::new(22.0, FontFamily::Proportional)),
        (TextStyle::Name("Subheading".into()), FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
    ].into();

    style.spacing.item_spacing = Vec2::new(12.0, 12.0);
    style.spacing.button_padding = Vec2::new(18.0, 10.0); // Boutons généreux et confortables
    style.spacing.window_margin = Margin::same(20.0);
    
    ctx.set_style(style);
}
