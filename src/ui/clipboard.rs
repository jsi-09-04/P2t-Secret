//! ui/clipboard.rs - Gestion sécurisée du presse-papier avec temporisation d'effacement.

use std::time::Duration;

pub fn copy_and_auto_clear(text: String, timeout_secs: u64) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| format!("Erreur accès presse-papier: {}", e))?;

    clipboard.set_text(&text)
        .map_err(|e| format!("Erreur copie dans le presse-papier: {}", e))?;

    // Lancer un thread d'effacement automatique
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(timeout_secs));
        if let Ok(mut clip) = arboard::Clipboard::new() {
            if let Ok(current) = clip.get_text() {
                if current == text {
                    let _ = clip.set_text("");
                }
            }
        }
    });

    Ok(())
}
