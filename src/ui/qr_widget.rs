//! ui/qr_widget.rs - Rendu vectoriel natif de QR Code dans egui.

use egui::{Color32, Pos2, Rect, Rounding, Ui, Vec2};

/// Dessine un QR Code de manière vectorielle nette et instantanée dans egui.
pub fn render_qr_code(ui: &mut Ui, size_modules: usize, modules: &[bool], target_width: f32) {
    if size_modules == 0 || modules.is_empty() {
        return;
    }

    let (response, painter) = ui.allocate_painter(Vec2::splat(target_width), egui::Sense::hover());
    let rect = response.rect;

    // Fond blanc avec bordure légèrement arrondie
    painter.rect_filled(rect, Rounding::same(8.0), Color32::WHITE);

    let padding = 12.0;
    let qr_inner_width = (target_width - padding * 2.0).max(10.0);
    let cell_size = qr_inner_width / (size_modules as f32);

    let origin = Pos2::new(rect.min.x + padding, rect.min.y + padding);

    for y in 0..size_modules {
        for x in 0..size_modules {
            if modules[y * size_modules + x] {
                let cell_rect = Rect::from_min_size(
                    Pos2::new(origin.x + (x as f32) * cell_size, origin.y + (y as f32) * cell_size),
                    Vec2::splat(cell_size + 0.3), // Léger chevauchement pour éviter les micro-interstices
                );
                painter.rect_filled(cell_rect, Rounding::ZERO, Color32::BLACK);
            }
        }
    }
}
