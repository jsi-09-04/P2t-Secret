//! main.rs - Point d'entrée de l'application native P2TSecret (Rust + egui).

pub mod crypto;
pub mod storage;
pub mod ui;

use ui::{apply_theme, P2TSecretApp};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("P2TSecret — Coffre-Fort Sécurisé 2FA")
            .with_inner_size([1040.0, 720.0])
            .with_min_inner_size([800.0, 560.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "P2TSecret",
        native_options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(P2TSecretApp::default()))
        }),
    )
}
