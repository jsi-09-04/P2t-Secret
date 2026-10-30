//! ui/app.rs - Application principale egui avec thème clair moderne, haute lisibilité et boutons confortables.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{Align, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, TextEdit, Ui, Vec2};

use crate::crypto::{
    generate_password, generate_qr_matrix, PasswordOptions, SecretString,
};
use crate::storage::{
    default_vault_path, load_vault, parse_csv_file, save_vault, CredentialEntry, Vault,
};
use crate::ui::clipboard::copy_and_auto_clear;
use crate::ui::qr_widget::render_qr_code;
use crate::ui::theme::{
    apply_theme, COLOR_AMBER, COLOR_BG, COLOR_BORDER, COLOR_EMERALD, COLOR_EMERALD_BG,
    COLOR_PRIMARY, COLOR_PRIMARY_LIGHT, COLOR_ROSE, COLOR_ROSE_BG,
    COLOR_SURFACE, COLOR_SURFACE_ALT, COLOR_TEXT_LIGHT, COLOR_TEXT_MAIN, COLOR_TEXT_MUTED,
};

/// État global de l'application
#[derive(Clone, Debug, PartialEq)]
pub enum AppScreen {
    InitPassword,
    Init2FASetup,
    InitBackupCodes,
    UnlockPassword,
    Unlock2FA,
    Dashboard,
}

pub struct P2TSecretApp {
    pub screen: AppScreen,
    pub vault_path: PathBuf,
    pub vault: Option<Vault>,

    // Champs de saisie
    pub input_master_password: String,
    pub input_master_password_confirm: String,
    pub input_account_name: String,
    pub input_totp_code: String,
    pub input_recovery_code: String,
    pub is_recovery_mode: bool,

    // 2FA Setup
    pub setup_qr_size: usize,
    pub setup_qr_modules: Vec<bool>,
    pub setup_secret_key: String,
    pub setup_backup_codes: Vec<String>,

    // Dashboard
    pub search_query: String,
    pub selected_category: String,
    pub revealed_passwords: std::collections::HashSet<String>,

    // Modale Ajouter / Modifier
    pub show_entry_modal: bool,
    pub editing_entry_id: Option<String>,
    pub modal_title: String,
    pub modal_url: String,
    pub modal_username: String,
    pub modal_password: String,
    pub modal_category: String,
    pub modal_tags: String,
    pub modal_notes: String,

    // Modale Générateur
    pub show_generator_modal: bool,
    pub gen_options: PasswordOptions,
    pub gen_current_password: String,
    pub gen_entropy_text: String,

    // Modale Importation CSV
    pub show_import_modal: bool,
    pub import_file_path: String,
    pub import_parsed_entries: Option<Vec<CredentialEntry>>,
    pub import_override_existing: bool,

    // Modale Paramètres (Catégories, Tags, Mot de passe maître)
    pub show_settings_modal: bool,
    pub settings_tab: usize,
    pub new_cat_input: String,
    pub new_tag_input: String,
    pub change_pwd_current: String,
    pub change_pwd_new: String,
    pub change_pwd_confirm: String,

    // Notifications Toast
    pub toast_message: Option<(String, Instant, bool)>, // (message, expiry, is_error)
}

impl Default for P2TSecretApp {
    fn default() -> Self {
        let vault_path = default_vault_path();
        let exists = vault_path.exists();

        let mut app = Self {
            screen: if exists { AppScreen::UnlockPassword } else { AppScreen::InitPassword },
            vault_path,
            vault: None,

            input_master_password: String::new(),
            input_master_password_confirm: String::new(),
            input_account_name: "admin@projet2team.com".to_string(),
            input_totp_code: String::new(),
            input_recovery_code: String::new(),
            is_recovery_mode: false,

            setup_qr_size: 0,
            setup_qr_modules: Vec::new(),
            setup_secret_key: String::new(),
            setup_backup_codes: Vec::new(),

            search_query: String::new(),
            selected_category: "Toutes".to_string(),
            revealed_passwords: std::collections::HashSet::new(),

            show_entry_modal: false,
            editing_entry_id: None,
            modal_title: String::new(),
            modal_url: String::new(),
            modal_username: String::new(),
            modal_password: String::new(),
            modal_category: "Général".to_string(),
            modal_tags: String::new(),
            modal_notes: String::new(),

            show_generator_modal: false,
            gen_options: PasswordOptions::default(),
            gen_current_password: String::new(),
            gen_entropy_text: String::new(),

            show_import_modal: false,
            import_file_path: "/Users/james/02-Interne/P2T Applications/P2TSecret/memory_bank/import/comptes_p2tsecret.csv".to_string(),
            import_parsed_entries: None,
            import_override_existing: false,

            show_settings_modal: false,
            settings_tab: 0,
            new_cat_input: String::new(),
            new_tag_input: String::new(),
            change_pwd_current: String::new(),
            change_pwd_new: String::new(),
            change_pwd_confirm: String::new(),

            toast_message: None,
        };

        app.regenerate_password();
        app
    }
}

impl P2TSecretApp {
    pub fn show_toast(&mut self, msg: impl Into<String>, is_error: bool) {
        self.toast_message = Some((msg.into(), Instant::now() + Duration::from_secs(4), is_error));
    }

    pub fn regenerate_password(&mut self) {
        let res = generate_password(&self.gen_options);
        self.gen_current_password = res.password;
        self.gen_entropy_text = format!("{} bits — {}", res.entropy_bits, res.strength_label);
    }

    pub fn lock_vault(&mut self) {
        self.vault = None;
        self.input_master_password.clear();
        self.input_totp_code.clear();
        self.revealed_passwords.clear();
        self.screen = AppScreen::UnlockPassword;
        self.show_toast("Coffre-fort verrouillé en toute sécurité.", false);
    }

    pub fn save_current_vault(&mut self) -> Result<(), String> {
        if let Some(vault) = &self.vault {
            save_vault(vault, &self.input_master_password, &self.vault_path)?;
            Ok(())
        } else {
            Err("Aucun coffre actif.".to_string())
        }
    }
}

impl eframe::App for P2TSecretApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Applique le thème clair à chaque frame pour garantir la consistance
        apply_theme(ctx);

        // 1. Barre Supérieure (Header Épuré, Lisible et Haut Contrasté)
        egui::TopBottomPanel::top("top_header")
            .frame(
                Frame::none()
                    .fill(COLOR_SURFACE)
                    .inner_margin(Margin::symmetric(24.0, 14.0))
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo & Titre (Fort contraste et typographie nette)
                    ui.label(RichText::new("🔐 P2T").size(22.0).strong().color(COLOR_PRIMARY));
                    ui.label(RichText::new("Secret").size(22.0).strong().color(COLOR_TEXT_MAIN));
                    ui.label(RichText::new("— Coffre-Fort Sécurisé").size(14.0).color(COLOR_TEXT_MUTED));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.screen == AppScreen::Dashboard {
                            // Bouton Verrouiller rouge clair bien proportionné
                            let lock_btn = egui::Button::new(
                                RichText::new("🔒 Verrouiller")
                                    .color(COLOR_ROSE)
                                    .strong()
                                    .size(13.0),
                            )
                            .fill(COLOR_ROSE_BG)
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(254, 202, 202)))
                            .rounding(Rounding::same(8.0))
                            .min_size(Vec2::new(120.0, 34.0));

                            if ui.add(lock_btn).on_hover_text("Verrouiller immédiatement le coffre").clicked() {
                                self.lock_vault();
                            }

                            ui.add_space(10.0);

                            // Badge 2FA Microsoft Actif avec pastille douce
                            Frame::none()
                                .fill(COLOR_EMERALD_BG)
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(167, 243, 208)))
                                .rounding(Rounding::same(12.0))
                                .inner_margin(Margin::symmetric(10.0, 4.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("🛡️ 2FA Microsoft Actif")
                                            .color(COLOR_EMERALD)
                                            .strong()
                                            .size(12.0),
                                    );
                                });

                            ui.add_space(6.0);

                            // Badge AES-256-GCM avec pastille bleue douce
                            Frame::none()
                                .fill(COLOR_PRIMARY_LIGHT)
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(191, 219, 254)))
                                .rounding(Rounding::same(12.0))
                                .inner_margin(Margin::symmetric(10.0, 4.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("🔒 AES-256-GCM")
                                            .color(COLOR_PRIMARY)
                                            .strong()
                                            .size(12.0),
                                    );
                                });
                        }
                    });
                });
            });

        // 2. Barre Latérale Gauche pour le Dashboard (Organisation professionnelle)
        if self.screen == AppScreen::Dashboard {
            egui::SidePanel::left("dashboard_sidebar")
                .exact_width(245.0)
                .resizable(false)
                .frame(
                    Frame::none()
                        .fill(COLOR_SURFACE)
                        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                        .inner_margin(Margin::symmetric(18.0, 20.0)),
                )
                .show(ctx, |ui| {
                    self.ui_sidebar(ui);
                });
        }

        // 3. Conteneur Principal
        egui::CentralPanel::default()
            .frame(Frame::none().fill(COLOR_BG).inner_margin(Margin::same(24.0)))
            .show(ctx, |ui| {
                // Affichage Notification Toast (Bannière contrastée)
                if let Some((msg, expiry, is_error)) = &self.toast_message {
                    if Instant::now() < *expiry {
                        let (bg_color, stroke_color, text_color, icon) = if *is_error {
                            (COLOR_ROSE_BG, Color32::from_rgb(248, 113, 113), COLOR_ROSE, "⚠️")
                        } else {
                            (COLOR_EMERALD_BG, Color32::from_rgb(110, 231, 183), COLOR_EMERALD, "✅")
                        };

                        Frame::none()
                            .fill(bg_color)
                            .stroke(Stroke::new(1.0_f32, stroke_color))
                            .rounding(Rounding::same(10.0))
                            .inner_margin(Margin::symmetric(16.0, 10.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(icon).size(18.0));
                                    ui.label(RichText::new(msg).color(text_color).strong().size(14.0));
                                });
                            });
                        ui.add_space(16.0);
                    }
                }

                match self.screen {
                    AppScreen::InitPassword => self.ui_init_password(ui),
                    AppScreen::Init2FASetup => self.ui_init_2fa_setup(ui),
                    AppScreen::InitBackupCodes => self.ui_init_backup_codes(ui),
                    AppScreen::UnlockPassword => self.ui_unlock_password(ui),
                    AppScreen::Unlock2FA => self.ui_unlock_2fa(ui),
                    AppScreen::Dashboard => self.ui_dashboard(ui),
                }
            });

        // Modales
        if self.show_entry_modal {
            self.ui_entry_modal(ctx);
        }
        if self.show_generator_modal {
            self.ui_generator_modal(ctx);
        }
        if self.show_import_modal {
            self.ui_import_modal(ctx);
        }
        if self.show_settings_modal {
            self.ui_settings_modal(ctx);
        }
    }
}

// =========================================================================
// Écrans d'Initialisation (Setup)
// =========================================================================
impl P2TSecretApp {
    fn ui_init_password(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);

            Frame::none()
                .fill(COLOR_SURFACE)
                .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                .rounding(Rounding::same(16.0))
                .inner_margin(Margin::same(32.0))
                .show(ui, |ui| {
                    ui.set_max_width(480.0);

                    ui.label(RichText::new("🔐 Création du Coffre-Fort").size(24.0).strong().color(COLOR_TEXT_MAIN));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Définissez votre compte et mot de passe maître pour chiffrer vos identifiants.")
                            .color(COLOR_TEXT_MUTED)
                            .size(14.0),
                    );
                    ui.add_space(24.0);

                    ui.vertical_centered_justified(|ui| {
                        ui.label(RichText::new("Compte / Email :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                        ui.add(TextEdit::singleline(&mut self.input_account_name).margin(Margin::same(10.0)));
                        ui.add_space(12.0);

                        ui.label(RichText::new("Mot de passe maître :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                        ui.add(TextEdit::singleline(&mut self.input_master_password).password(true).margin(Margin::same(10.0)));
                        ui.add_space(12.0);

                        ui.label(RichText::new("Confirmer le mot de passe maître :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                        ui.add(TextEdit::singleline(&mut self.input_master_password_confirm).password(true).margin(Margin::same(10.0)));
                        ui.add_space(24.0);

                        let continue_btn = egui::Button::new(
                            RichText::new("Continuer vers la configuration 2FA →")
                                .size(15.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_PRIMARY)
                        .rounding(Rounding::same(10.0))
                        .min_size(Vec2::new(f32::INFINITY, 44.0));

                        if ui.add(continue_btn).clicked() {
                            if self.input_master_password.len() < 8 {
                                self.show_toast("Le mot de passe maître doit comporter au moins 8 caractères.", true);
                            } else if self.input_master_password != self.input_master_password_confirm {
                                self.show_toast("Les deux mots de passe ne correspondent pas.", true);
                            } else {
                                match Vault::new(&self.input_account_name) {
                                    Ok(v) => match v.totp_config.get_url() {
                                        Ok(url) => {
                                            if let Ok((size, modules)) = generate_qr_matrix(&url) {
                                                self.setup_qr_size = size;
                                                self.setup_qr_modules = modules;
                                                self.setup_secret_key = v.totp_config.secret_base32.clone();
                                                self.vault = Some(v);
                                                self.screen = AppScreen::Init2FASetup;
                                            }
                                        }
                                        Err(e) => self.show_toast(e, true),
                                    },
                                    Err(e) => self.show_toast(e, true),
                                }
                            }
                        }
                    });
                });
        });
    }

    fn ui_init_2fa_setup(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            Frame::none()
                .fill(COLOR_SURFACE)
                .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                .rounding(Rounding::same(16.0))
                .inner_margin(Margin::same(28.0))
                .show(ui, |ui| {
                    ui.set_max_width(520.0);

                    ui.label(RichText::new("🛡️ Configuration 2FA Microsoft Authenticator").size(22.0).strong().color(COLOR_TEXT_MAIN));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Scannez ce QR Code avec l'application Microsoft Authenticator sur votre smartphone.")
                            .color(COLOR_TEXT_MUTED)
                            .size(14.0),
                    );
                    ui.add_space(18.0);

                    // Rendu du QR Code avec cadre blanc doux
                    Frame::none()
                        .fill(Color32::WHITE)
                        .stroke(Stroke::new(1.5_f32, COLOR_BORDER))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::same(12.0))
                        .show(ui, |ui| {
                            render_qr_code(ui, self.setup_qr_size, &self.setup_qr_modules, 220.0);
                        });

                    ui.add_space(14.0);
                    ui.label(
                        RichText::new(format!("Clé manuelle : {}", self.setup_secret_key))
                            .color(COLOR_PRIMARY)
                            .strong()
                            .size(13.0)
                            .monospace(),
                    );
                    ui.add_space(20.0);

                    ui.label(RichText::new("Code dynamique à 6 chiffres :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                    let input = ui.add(
                        TextEdit::singleline(&mut self.input_totp_code)
                            .desired_width(220.0)
                            .font(egui::FontId::monospace(26.0))
                            .margin(Margin::same(8.0)),
                    );
                    input.request_focus();

                    ui.add_space(18.0);

                    let val_btn = egui::Button::new(
                        RichText::new("Valider et Activer la 2FA 🛡️")
                            .size(15.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(COLOR_EMERALD)
                    .rounding(Rounding::same(10.0))
                    .min_size(Vec2::new(260.0, 44.0));

                    if ui.add(val_btn).clicked() || (input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        let mut valid = false;
                        let mut backup_codes = Vec::new();

                        if let Some(v) = &mut self.vault {
                            if v.totp_config.verify_code(&self.input_totp_code) {
                                valid = true;
                                backup_codes = v.totp_config.generate_backup_codes();
                            }
                        }

                        if valid {
                            self.setup_backup_codes = backup_codes;
                            self.screen = AppScreen::InitBackupCodes;
                            self.show_toast("2FA Microsoft Authenticator validé avec succès !", false);
                        } else {
                            self.show_toast("Code incorrect ou expiré. Vérifiez Microsoft Authenticator.", true);
                        }
                    }
                });
        });
    }

    fn ui_init_backup_codes(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);

            Frame::none()
                .fill(COLOR_SURFACE)
                .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                .rounding(Rounding::same(16.0))
                .inner_margin(Margin::same(32.0))
                .show(ui, |ui| {
                    ui.set_max_width(520.0);

                    ui.label(RichText::new("🔑 Codes de Secours à Usage Unique").size(22.0).strong().color(COLOR_TEXT_MAIN));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Conservez précieusement ces codes. Ils permettent de déverrouiller votre coffre si vous perdez votre smartphone.")
                            .color(COLOR_TEXT_MUTED)
                            .size(13.0),
                    );
                    ui.add_space(20.0);

                    // Grille des codes dans un cadre dédié
                    Frame::none()
                        .fill(COLOR_SURFACE_ALT)
                        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::same(16.0))
                        .show(ui, |ui| {
                            egui::Grid::new("backup_codes_grid").spacing([40.0, 12.0]).show(ui, |ui| {
                                for (i, code) in self.setup_backup_codes.iter().enumerate() {
                                    ui.label(RichText::new(code).monospace().size(17.0).color(COLOR_PRIMARY).strong());
                                    if (i + 1) % 2 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                        });

                    ui.add_space(24.0);

                    ui.horizontal(|ui| {
                        let copy_btn = egui::Button::new(RichText::new("📋 Copier tous les codes").size(14.0).strong())
                            .min_size(Vec2::new(180.0, 42.0));
                        if ui.add(copy_btn).clicked() {
                            let text = self.setup_backup_codes.join("\n");
                            let _ = copy_and_auto_clear(text, 60);
                            self.show_toast("Codes de secours copiés dans le presse-papier.", false);
                        }

                        ui.add_space(10.0);

                        let finish_btn = egui::Button::new(
                            RichText::new("Accéder au Coffre →")
                                .size(14.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_PRIMARY)
                        .min_size(Vec2::new(180.0, 42.0));

                        if ui.add(finish_btn).clicked() {
                            match self.save_current_vault() {
                                Ok(()) => {
                                    self.screen = AppScreen::Dashboard;
                                    self.show_toast("Coffre-fort initialisé avec succès !", false);
                                }
                                Err(e) => self.show_toast(e, true),
                            }
                        }
                    });
                });
        });
    }
}

// =========================================================================
// Écrans de Déverrouillage (Unlock)
// =========================================================================
impl P2TSecretApp {
    fn ui_unlock_password(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);

            Frame::none()
                .fill(COLOR_SURFACE)
                .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                .rounding(Rounding::same(16.0))
                .inner_margin(Margin::same(36.0))
                .show(ui, |ui| {
                    ui.set_max_width(440.0);

                    // Badge Étape 1/2
                    Frame::none()
                        .fill(COLOR_PRIMARY_LIGHT)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(191, 219, 254)))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::symmetric(14.0, 5.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("ÉTAPE 1 SUR 2 • MOT DE PASSE MAÎTRE").color(COLOR_PRIMARY).strong().size(11.5));
                        });

                    ui.add_space(14.0);
                    ui.label(RichText::new("🔓 Déverrouillage du Coffre").size(24.0).strong().color(COLOR_TEXT_MAIN));
                    ui.add_space(4.0);
                    ui.label(RichText::new("Saisissez votre mot de passe maître pour déchiffrer votre coffre.").color(COLOR_TEXT_MUTED).size(13.5));
                    ui.add_space(24.0);

                    ui.vertical_centered_justified(|ui| {
                        ui.label(RichText::new("Mot de passe maître :").strong().size(13.5).color(COLOR_TEXT_MAIN));
                        let input = ui.add(
                            TextEdit::singleline(&mut self.input_master_password)
                                .password(true)
                                .margin(Margin::symmetric(14.0, 10.0))
                        );
                        input.request_focus();

                        ui.add_space(24.0);

                        let unlock_btn = egui::Button::new(
                            RichText::new("Continuer vers la 2FA (Étape 2) →")
                                .size(15.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_PRIMARY)
                        .rounding(Rounding::same(10.0))
                        .min_size(Vec2::new(f32::INFINITY, 46.0));

                        if ui.add(unlock_btn).clicked() || (input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            match load_vault(&self.vault_path, &self.input_master_password) {
                                Ok(v) => {
                                    self.vault = Some(v);
                                    self.input_totp_code.clear();
                                    self.screen = AppScreen::Unlock2FA;
                                }
                                Err(e) => self.show_toast(e, true),
                            }
                        }
                    });
                });
        });
    }

    fn ui_unlock_2fa(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);

            Frame::none()
                .fill(COLOR_SURFACE)
                .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                .rounding(Rounding::same(16.0))
                .inner_margin(Margin::same(36.0))
                .show(ui, |ui| {
                    ui.set_max_width(460.0);

                    // Badge Étape 2/2
                    Frame::none()
                        .fill(COLOR_EMERALD_BG)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(167, 243, 208)))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(Margin::symmetric(14.0, 5.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("ÉTAPE 2 SUR 2 • MICROSOFT AUTHENTICATOR").color(COLOR_EMERALD).strong().size(11.5));
                        });

                    ui.add_space(14.0);
                    ui.label(RichText::new("🛡️ Validation Double Facteur").size(24.0).strong().color(COLOR_PRIMARY));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Consultez Microsoft Authenticator sur votre smartphone et saisissez le code à 6 chiffres.")
                            .color(COLOR_TEXT_MUTED)
                            .size(13.5),
                    );
                    ui.add_space(24.0);

                    if !self.is_recovery_mode {
                        ui.label(RichText::new("Code dynamique Microsoft Authenticator :").strong().size(13.5).color(COLOR_TEXT_MAIN));
                        let input = ui.add(
                            TextEdit::singleline(&mut self.input_totp_code)
                                .desired_width(220.0)
                                .font(egui::FontId::monospace(26.0))
                                .margin(Margin::same(10.0)),
                        );
                        input.request_focus();

                        ui.add_space(24.0);

                        let val_btn = egui::Button::new(
                            RichText::new("Valider et Ouvrir le Coffre 🛡️")
                                .size(15.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_EMERALD)
                        .rounding(Rounding::same(10.0))
                        .min_size(Vec2::new(260.0, 46.0));

                        if ui.add(val_btn).clicked() || (input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            let mut ok = false;
                            if let Some(v) = &self.vault {
                                ok = v.totp_config.verify_code(&self.input_totp_code);
                            }
                            if ok {
                                self.screen = AppScreen::Dashboard;
                                self.show_toast("Double facteur validé. Bienvenue dans votre coffre !", false);
                            } else {
                                self.show_toast("Code TOTP incorrect ou expiré.", true);
                            }
                        }

                        ui.add_space(18.0);
                        let recovery_link = egui::Button::new(
                            RichText::new("Perte de smartphone ? Utiliser un code de secours")
                                .size(13.0)
                                .color(COLOR_PRIMARY),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE);

                        if ui.add(recovery_link).clicked() {
                            self.is_recovery_mode = true;
                        }
                    } else {
                        ui.label(RichText::new("Code de secours (XXXX-XXXX) :").strong().size(13.5).color(COLOR_TEXT_MAIN));
                        let input = ui.add(
                            TextEdit::singleline(&mut self.input_recovery_code)
                                .desired_width(220.0)
                                .font(egui::FontId::monospace(22.0))
                                .margin(Margin::same(10.0)),
                        );
                        input.request_focus();

                        ui.add_space(24.0);

                        let rec_btn = egui::Button::new(
                            RichText::new("Valider le code de secours")
                                .size(14.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_PRIMARY)
                        .rounding(Rounding::same(10.0))
                        .min_size(Vec2::new(260.0, 46.0));

                        if ui.add(rec_btn).clicked() {
                            let mut ok = false;
                            if let Some(v) = &mut self.vault {
                                ok = v.totp_config.consume_backup_code(&self.input_recovery_code);
                            }
                            if ok {
                                let _ = self.save_current_vault();
                                self.screen = AppScreen::Dashboard;
                                self.show_toast("Connexion de secours réussie (code consommé).", false);
                            } else {
                                self.show_toast("Code de secours invalide ou déjà consommé.", true);
                            }
                        }

                        ui.add_space(14.0);
                        let back_link = egui::Button::new(
                            RichText::new("← Revenir à Microsoft Authenticator")
                                .size(13.0)
                                .color(COLOR_TEXT_MUTED),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE);

                        if ui.add(back_link).clicked() {
                            self.is_recovery_mode = false;
                        }
                    }
                });
        });
    }
}

// =========================================================================
// Barre Latérale du Coffre (Sidebar - Navigation & Actions Rapides)
// =========================================================================
impl P2TSecretApp {
    fn ui_sidebar(&mut self, ui: &mut Ui) {
        // 1. Bouton Action Principal "+ Nouvel Identifiant" (Grand et proéminent)
        let add_btn = egui::Button::new(
            RichText::new("➕ Nouvel Identifiant")
                .size(14.0)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(COLOR_PRIMARY)
        .rounding(Rounding::same(8.0))
        .min_size(Vec2::new(ui.available_width(), 44.0));

        if ui.add(add_btn).on_hover_text("Enregistrer un nouveau secret chiffré").clicked() {
            self.open_new_entry_modal();
        }

        ui.add_space(8.0);

        // 2. Boutons Outils : Générateur & Import CSV
        let gen_btn = egui::Button::new(
            RichText::new("⚡ Générateur MDP")
                .size(13.5)
                .strong()
                .color(COLOR_TEXT_MAIN),
        )
        .fill(COLOR_SURFACE_ALT)
        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
        .rounding(Rounding::same(8.0))
        .min_size(Vec2::new(ui.available_width(), 38.0));

        if ui.add(gen_btn).on_hover_text("Ouvrir le générateur de mot de passe fort").clicked() {
            self.show_generator_modal = true;
        }

        ui.add_space(6.0);

        let import_btn = egui::Button::new(
            RichText::new("📥 Importer CSV")
                .size(13.5)
                .strong()
                .color(COLOR_TEXT_MAIN),
        )
        .fill(COLOR_SURFACE_ALT)
        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
        .rounding(Rounding::same(8.0))
        .min_size(Vec2::new(ui.available_width(), 38.0));

        if ui.add(import_btn).on_hover_text("Importer des comptes depuis un fichier CSV").clicked() {
            self.show_import_modal = true;
            self.import_parsed_entries = None;
        }

        ui.add_space(6.0);

        let settings_btn = egui::Button::new(
            RichText::new("⚙️ Paramètres")
                .size(13.5)
                .strong()
                .color(COLOR_TEXT_MAIN),
        )
        .fill(COLOR_SURFACE_ALT)
        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
        .rounding(Rounding::same(8.0))
        .min_size(Vec2::new(ui.available_width(), 38.0));

        if ui.add(settings_btn).on_hover_text("Personnaliser catégories, tags et sécurité").clicked() {
            self.show_settings_modal = true;
        }

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);

        // 3. Navigation par Catégories
        ui.label(RichText::new("CATÉGORIES").size(11.0).strong().color(COLOR_TEXT_LIGHT));
        ui.add_space(6.0);

        // Extraction des catégories et calcul des totaux
        let mut cat_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let (total_count, sorted_cats) = if let Some(v) = &self.vault {
            for e in &v.entries {
                let cat = if e.category.is_empty() { "Général".to_string() } else { e.category.clone() };
                *cat_counts.entry(cat).or_insert(0) += 1;
            }
            (v.entries.len(), v.get_categories())
        } else {
            (0, Vec::new())
        };

        // Bouton "Tous les secrets"
        let is_all_selected = self.selected_category == "Toutes";
        let (fill_all, text_col_all, stroke_all) = if is_all_selected {
            (COLOR_PRIMARY_LIGHT, COLOR_PRIMARY, Stroke::new(1.0_f32, COLOR_PRIMARY))
        } else {
            (Color32::TRANSPARENT, COLOR_TEXT_MAIN, Stroke::NONE)
        };

        let all_btn = egui::Button::new(
            RichText::new(format!("📂 Tous les secrets ({})", total_count))
                .size(13.5)
                .strong()
                .color(text_col_all),
        )
        .fill(fill_all)
        .stroke(stroke_all)
        .rounding(Rounding::same(8.0))
        .min_size(Vec2::new(ui.available_width(), 36.0));

        if ui.add(all_btn).clicked() {
            self.selected_category = "Toutes".to_string();
        }

        ui.add_space(4.0);

        for cat in sorted_cats {
            let count = cat_counts.get(&cat).copied().unwrap_or(0);
            let is_sel = self.selected_category == cat;
            let (fill, text_col, stroke) = if is_sel {
                (COLOR_PRIMARY_LIGHT, COLOR_PRIMARY, Stroke::new(1.0_f32, COLOR_PRIMARY))
            } else {
                (Color32::TRANSPARENT, COLOR_TEXT_MUTED, Stroke::NONE)
            };

            let cat_icon = match cat.as_str() {
                "Cloud" | "AWS" | "Azure" => "☁️",
                "Serveurs" | "Serveur" | "SSH" | "Linux" => "💻",
                "Finance" | "Banque" => "🏦",
                "Personnel" => "👤",
                "Pro" | "Professionnel" => "💼",
                "Email" | "Messagerie" => "✉️",
                _ => "📁",
            };

            let btn = egui::Button::new(
                RichText::new(format!("{} {} ({})", cat_icon, cat, count))
                    .size(13.0)
                    .color(text_col),
            )
            .fill(fill)
            .stroke(stroke)
            .rounding(Rounding::same(8.0))
            .min_size(Vec2::new(ui.available_width(), 34.0));

            if ui.add(btn).clicked() {
                self.selected_category = cat.clone();
            }
            ui.add_space(2.0);
        }

        // 4. Cartouche d'informations Sécurité en bas de la barre latérale
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(8.0);
            Frame::none()
                .fill(COLOR_SURFACE_ALT)
                .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new("🔒 Coffre Local Sécurisé").size(12.0).strong().color(COLOR_TEXT_MAIN));
                    ui.add_space(2.0);
                    if let Some(v) = &self.vault {
                        ui.label(RichText::new(&v.totp_config.account_name).size(11.0).color(COLOR_TEXT_MUTED));
                    }
                    ui.label(RichText::new("Argon2id + AES-256-GCM").size(10.0).color(COLOR_TEXT_LIGHT));
                });
        });
    }
}

// =========================================================================
// Tableau de bord du Coffre (Dashboard - Zone Centrale Aérée et Ergonomique)
// =========================================================================
impl P2TSecretApp {
    fn ui_dashboard(&mut self, ui: &mut Ui) {
        // Filtrage des Entrées
        let query = self.search_query.to_lowercase();
        let cat_filter = self.selected_category.clone();

        let entries = if let Some(v) = &self.vault {
            v.entries
                .iter()
                .filter(|e| {
                    let matches_cat = cat_filter == "Toutes" || e.category == cat_filter;
                    let matches_query = query.is_empty()
                        || e.title.to_lowercase().contains(&query)
                        || e.username.to_lowercase().contains(&query)
                        || e.url.to_lowercase().contains(&query)
                        || e.category.to_lowercase().contains(&query)
                        || e.tags.iter().any(|t| t.to_lowercase().contains(&query));
                    matches_cat && matches_query
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // 1. Barre de Recherche et Compteur
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(18.0));
            let search_edit = TextEdit::singleline(&mut self.search_query)
                .hint_text("Rechercher par titre, login, URL, tag...")
                .desired_width(ui.available_width() - 50.0)
                .margin(Margin::symmetric(12.0, 9.0));
            ui.add(search_edit);

            if !self.search_query.is_empty() {
                if ui.button(RichText::new("✕").size(13.0).strong()).on_hover_text("Effacer la recherche").clicked() {
                    self.search_query.clear();
                }
            }
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let label_count = format!("{} identifiant(s) affiché(s)", entries.len());
            ui.label(RichText::new(label_count).size(13.0).color(COLOR_TEXT_MUTED));

            if self.selected_category != "Toutes" {
                ui.label(RichText::new(format!("• Catégorie : {}", self.selected_category)).size(13.0).strong().color(COLOR_PRIMARY));
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // 2. État Vide (Empty State)
        let total_vault_entries = self.vault.as_ref().map(|v| v.entries.len()).unwrap_or(0);
        if entries.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);

                Frame::none()
                    .fill(COLOR_SURFACE)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                    .rounding(Rounding::same(16.0))
                    .inner_margin(Margin::same(40.0))
                    .show(ui, |ui| {
                        ui.set_max_width(480.0);

                        if total_vault_entries == 0 {
                            ui.label(RichText::new("🛡️").size(48.0));
                            ui.add_space(8.0);
                            ui.label(RichText::new("Votre Coffre-Fort est vide").size(22.0).strong().color(COLOR_TEXT_MAIN));
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new("Enregistrez vos identifiants, mots de passe et clés d'accès chiffrés en AES-256-GCM.")
                                    .color(COLOR_TEXT_MUTED)
                                    .size(14.0),
                            );
                            ui.add_space(24.0);

                            let empty_add_btn = egui::Button::new(
                                RichText::new("➕ Ajouter mon premier identifiant")
                                    .size(15.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(COLOR_PRIMARY)
                            .rounding(Rounding::same(10.0))
                            .min_size(Vec2::new(280.0, 46.0));

                            if ui.add(empty_add_btn).clicked() {
                                self.open_new_entry_modal();
                            }
                        } else {
                            ui.label(RichText::new("🔍").size(42.0));
                            ui.add_space(8.0);
                            ui.label(RichText::new("Aucun identifiant trouvé").size(20.0).strong().color(COLOR_TEXT_MAIN));
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new("Aucun secret ne correspond à vos critères de recherche actuels.")
                                    .color(COLOR_TEXT_MUTED)
                                    .size(14.0),
                            );
                            ui.add_space(20.0);

                            let reset_btn = egui::Button::new(RichText::new("Réinitialiser les filtres").size(14.0).strong())
                                .min_size(Vec2::new(200.0, 38.0));

                            if ui.add(reset_btn).clicked() {
                                self.search_query.clear();
                                self.selected_category = "Toutes".to_string();
                            }
                        }
                    });
            });
            return;
        }

        // 3. Liste Déroulante des Identifiants
        ScrollArea::vertical().show(ui, |ui| {
            let mut to_delete: Option<String> = None;
            let mut to_edit: Option<CredentialEntry> = None;
            let mut to_copy_pwd: Option<String> = None;
            let mut to_copy_login: Option<String> = None;

            for entry in &entries {
                Frame::none()
                    .fill(COLOR_SURFACE)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                    .rounding(Rounding::same(12.0))
                    .inner_margin(Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // En-tête de carte : Titre et URL
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&entry.title).size(18.0).strong().color(COLOR_TEXT_MAIN));
                                if !entry.url.is_empty() {
                                    ui.label(RichText::new(format!("🔗 {}", entry.url)).color(COLOR_PRIMARY).size(12.5));
                                }
                            });

                            // Actions rapides en haut à droite
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                let del_btn = egui::Button::new(RichText::new("🗑️").color(COLOR_ROSE).size(14.0))
                                    .min_size(Vec2::new(36.0, 32.0));
                                if ui.add(del_btn).on_hover_text("Supprimer cet identifiant").clicked() {
                                    to_delete = Some(entry.id.clone());
                                }

                                let edit_btn = egui::Button::new(RichText::new("✏️ Modifier").size(12.5).strong())
                                    .min_size(Vec2::new(84.0, 32.0));
                                if ui.add(edit_btn).on_hover_text("Modifier cet identifiant").clicked() {
                                    to_edit = Some(entry.clone());
                                }

                                ui.add_space(4.0);

                                // Badge de catégorie stylisé
                                Frame::none()
                                    .fill(COLOR_SURFACE_ALT)
                                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                                    .rounding(Rounding::same(12.0))
                                    .inner_margin(Margin::symmetric(10.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(&entry.category)
                                                .size(11.5)
                                                .strong()
                                                .color(COLOR_TEXT_MUTED),
                                        );
                                    });
                            });
                        });

                        ui.add_space(10.0);

                        // Conteneur Champs (Login + Mot de passe)
                        Frame::none()
                            .fill(COLOR_SURFACE_ALT)
                            .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                            .rounding(Rounding::same(8.0))
                            .inner_margin(Margin::symmetric(14.0, 10.0))
                            .show(ui, |ui| {
                                // Ligne Login
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Identifiant :").color(COLOR_TEXT_MUTED).size(13.0));
                                    ui.label(RichText::new(&entry.username).strong().size(14.0).color(COLOR_TEXT_MAIN));

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        let copy_btn = egui::Button::new(RichText::new("📋 Copier").size(12.5).strong())
                                            .min_size(Vec2::new(90.0, 32.0));
                                        if ui.add(copy_btn).clicked() {
                                            to_copy_login = Some(entry.username.clone());
                                        }
                                    });
                                });

                                ui.add_space(6.0);
                                ui.separator();
                                ui.add_space(6.0);

                                // Ligne Mot de passe
                                let is_revealed = self.revealed_passwords.contains(&entry.id);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Mot de passe :").color(COLOR_TEXT_MUTED).size(13.0));

                                    if is_revealed {
                                        ui.label(
                                            RichText::new(entry.password.as_str())
                                                .monospace()
                                                .strong()
                                                .size(14.5)
                                                .color(COLOR_EMERALD),
                                        );
                                    } else {
                                        ui.label(RichText::new("••••••••••••").monospace().strong().size(14.5));
                                    }

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        let copy_pwd_btn = egui::Button::new(
                                            RichText::new("📋 Copier (30s)")
                                                .size(12.5)
                                                .strong()
                                                .color(Color32::WHITE),
                                        )
                                        .fill(COLOR_PRIMARY)
                                        .min_size(Vec2::new(125.0, 32.0));

                                        if ui.add(copy_pwd_btn).on_hover_text("Copier avec effacement auto dans 30s").clicked() {
                                            to_copy_pwd = Some(entry.password.as_str().to_string());
                                        }

                                        let eye_icon = if is_revealed { "🙈 Masquer" } else { "👁️ Afficher" };
                                        let eye_btn = egui::Button::new(RichText::new(eye_icon).size(12.5))
                                            .min_size(Vec2::new(98.0, 32.0));

                                        if ui.add(eye_btn).clicked() {
                                            if is_revealed {
                                                self.revealed_passwords.remove(&entry.id);
                                            } else {
                                                self.revealed_passwords.insert(entry.id.clone());
                                            }
                                        }
                                    });
                                });
                            });

                        if !entry.notes.as_str().is_empty() {
                            ui.add_space(8.0);
                            Frame::none()
                                .fill(Color32::from_rgb(254, 252, 232)) // Jaune ambré très doux
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(254, 240, 138)))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(12.0, 8.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(format!("📝 Notes : {}", entry.notes.as_str()))
                                            .color(COLOR_AMBER)
                                            .size(12.5),
                                    );
                                });
                        }
                    });

                ui.add_space(10.0);
            }

            // Gestion des actions
            if let Some(login) = to_copy_login {
                let _ = copy_and_auto_clear(login, 60);
                self.show_toast("Identifiant copié dans le presse-papier !", false);
            }
            if let Some(pwd) = to_copy_pwd {
                let _ = copy_and_auto_clear(pwd, 30);
                self.show_toast("Mot de passe copié ! Effacement auto dans 30s.", false);
            }
            if let Some(entry) = to_edit {
                self.open_edit_entry_modal(entry);
            }
            if let Some(id) = to_delete {
                if let Some(v) = &mut self.vault {
                    v.entries.retain(|e| e.id != id);
                    v.log_action("CREDENTIAL_DELETED", id);
                }
                let _ = self.save_current_vault();
                self.show_toast("Identifiant supprimé.", false);
            }
        });
    }

    fn open_new_entry_modal(&mut self) {
        self.editing_entry_id = None;
        self.modal_title.clear();
        self.modal_url.clear();
        self.modal_username.clear();
        self.modal_password.clear();
        self.modal_category = "Général".to_string();
        self.modal_tags.clear();
        self.modal_notes.clear();
        self.show_entry_modal = true;
    }

    fn open_edit_entry_modal(&mut self, entry: CredentialEntry) {
        self.editing_entry_id = Some(entry.id);
        self.modal_title = entry.title;
        self.modal_url = entry.url;
        self.modal_username = entry.username;
        self.modal_password = entry.password.as_str().to_string();
        self.modal_category = entry.category;
        self.modal_tags = entry.tags.join(", ");
        self.modal_notes = entry.notes.as_str().to_string();
        self.show_entry_modal = true;
    }

    fn ui_entry_modal(&mut self, ctx: &egui::Context) {
        let is_edit = self.editing_entry_id.is_some();
        let title = if is_edit { "✏️ Modifier l'identifiant" } else { "➕ Nouvel identifiant sécurisé" };

        let mut close_modal = false;
        let mut save_entry = false;

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .frame(
                Frame::none()
                    .fill(COLOR_SURFACE)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                    .rounding(Rounding::same(14.0))
                    .inner_margin(Margin::same(24.0)),
            )
            .show(ctx, |ui| {
                ui.set_width(460.0);

                ui.label(RichText::new("Titre / Nom du service * :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                ui.add(TextEdit::singleline(&mut self.modal_title).desired_width(f32::INFINITY).margin(Margin::same(8.0)));
                ui.add_space(8.0);

                ui.label(RichText::new("URL d'accès :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                ui.add(TextEdit::singleline(&mut self.modal_url).desired_width(f32::INFINITY).margin(Margin::same(8.0)));
                ui.add_space(8.0);

                ui.label(RichText::new("Login / Email * :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                ui.add(TextEdit::singleline(&mut self.modal_username).desired_width(f32::INFINITY).margin(Margin::same(8.0)));
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Mot de passe secret * :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let gen_btn = egui::Button::new(RichText::new("⚡ Générer fort").size(12.0).strong().color(COLOR_PRIMARY));
                        if ui.add(gen_btn).clicked() {
                            let res = generate_password(&self.gen_options);
                            self.modal_password = res.password;
                        }
                    });
                });
                ui.add(TextEdit::singleline(&mut self.modal_password).desired_width(f32::INFINITY).margin(Margin::same(8.0)));
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Catégorie :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                        let mut cats = if let Some(v) = &self.vault {
                            v.get_categories()
                        } else {
                            vec!["Général".to_string()]
                        };
                        if !cats.contains(&self.modal_category) && !self.modal_category.is_empty() {
                            cats.push(self.modal_category.clone());
                        }

                        egui::ComboBox::from_id_salt("modal_cat_combo")
                            .selected_text(&self.modal_category)
                            .width(210.0)
                            .show_ui(ui, |ui| {
                                for c in &cats {
                                    ui.selectable_value(&mut self.modal_category, c.clone(), c);
                                }
                            });
                    });
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Tags (séparés par virgules) :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                        ui.add(TextEdit::singleline(&mut self.modal_tags).desired_width(210.0).margin(Margin::same(8.0)));
                    });
                });

                // Suggestions de tags cliquables
                if let Some(v) = &self.vault {
                    let available_tags = v.get_all_tags();
                    if !available_tags.is_empty() {
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Suggestions :").size(11.0).color(COLOR_TEXT_MUTED));
                            for t in &available_tags {
                                let is_present = self.modal_tags.split(',').any(|item| item.trim().eq_ignore_ascii_case(t));
                                let text_col = if is_present { COLOR_PRIMARY } else { COLOR_TEXT_MUTED };
                                if ui.small_button(RichText::new(format!("+ {}", t)).size(11.0).color(text_col)).clicked() {
                                    if !is_present {
                                        if !self.modal_tags.trim().is_empty() && !self.modal_tags.trim().ends_with(',') {
                                            self.modal_tags.push_str(", ");
                                        }
                                        self.modal_tags.push_str(t);
                                    }
                                }
                            }
                        });
                    }
                }
                ui.add_space(8.0);

                ui.label(RichText::new("Notes confidentielles :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                ui.add(TextEdit::multiline(&mut self.modal_notes).desired_rows(3).desired_width(f32::INFINITY).margin(Margin::same(8.0)));

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    let cancel_btn = egui::Button::new(RichText::new("Annuler").size(14.0)).min_size(Vec2::new(120.0, 42.0));
                    if ui.add(cancel_btn).clicked() {
                        close_modal = true;
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let save_btn = egui::Button::new(
                            RichText::new("Chiffrer & Sauvegarder")
                                .size(14.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_PRIMARY)
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(210.0, 42.0));

                        if ui.add(save_btn).clicked() {
                            if self.modal_title.trim().is_empty() || self.modal_username.trim().is_empty() || self.modal_password.trim().is_empty() {
                                self.show_toast("Veuillez renseigner le titre, le login et le mot de passe.", true);
                            } else {
                                save_entry = true;
                                close_modal = true;
                            }
                        }
                    });
                });
            });

        if save_entry {
            let tags: Vec<String> = self.modal_tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if let Some(v) = &mut self.vault {
                if let Some(id) = &self.editing_entry_id {
                    if let Some(e) = v.entries.iter_mut().find(|e| &e.id == id) {
                        e.title = self.modal_title.clone();
                        e.url = self.modal_url.clone();
                        e.username = self.modal_username.clone();
                        e.password = SecretString::new(&self.modal_password);
                        e.category = self.modal_category.clone();
                        e.tags = tags;
                        e.notes = SecretString::new(&self.modal_notes);
                        e.updated_at = chrono::Utc::now();
                    }
                    v.log_action("CREDENTIAL_UPDATED", &self.modal_title);
                } else {
                    let entry = CredentialEntry::new(
                        &self.modal_title,
                        &self.modal_url,
                        &self.modal_username,
                        SecretString::new(&self.modal_password),
                        &self.modal_category,
                        tags,
                        SecretString::new(&self.modal_notes),
                    );
                    v.entries.push(entry);
                    v.log_action("CREDENTIAL_CREATED", &self.modal_title);
                }
            }
            let _ = self.save_current_vault();
            self.show_toast("Identifiant chiffré et sauvegardé avec succès.", false);
        }

        if close_modal {
            self.show_entry_modal = false;
        }
    }

    fn ui_generator_modal(&mut self, ctx: &egui::Context) {
        let mut close = false;

        egui::Window::new("⚡ Générateur de Mot de Passe Cryptographique")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .frame(
                Frame::none()
                    .fill(COLOR_SURFACE)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                    .rounding(Rounding::same(14.0))
                    .inner_margin(Margin::same(24.0)),
            )
            .show(ctx, |ui| {
                ui.set_width(440.0);

                ui.label(RichText::new("Mot de passe généré :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                ui.add_space(4.0);

                // Boîte d'affichage du mot de passe généré
                Frame::none()
                    .fill(COLOR_SURFACE_ALT)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&self.gen_current_password)
                                    .monospace()
                                    .size(17.0)
                                    .color(COLOR_PRIMARY)
                                    .strong(),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let copy_btn = egui::Button::new(RichText::new("📋 Copier").size(12.5).strong()).min_size(Vec2::new(88.0, 32.0));
                                if ui.add(copy_btn).clicked() {
                                    let _ = copy_and_auto_clear(self.gen_current_password.clone(), 30);
                                    self.show_toast("Mot de passe copié !", false);
                                }

                                let regen_btn = egui::Button::new(RichText::new("🔄").size(13.0)).min_size(Vec2::new(36.0, 32.0));
                                if ui.add(regen_btn).on_hover_text("Régénérer").clicked() {
                                    self.regenerate_password();
                                }
                            });
                        });
                    });

                ui.add_space(8.0);
                ui.label(RichText::new(&self.gen_entropy_text).color(COLOR_EMERALD).strong().size(13.0));
                ui.add_space(14.0);

                ui.label(RichText::new(format!("Longueur : {} caractères", self.gen_options.length)).strong().size(13.0));
                if ui.add(egui::Slider::new(&mut self.gen_options.length, 8..=64)).changed() {
                    self.regenerate_password();
                }

                ui.add_space(10.0);
                if ui.checkbox(&mut self.gen_options.use_upper, "Majuscules (A-Z)").changed() {
                    self.regenerate_password();
                }
                if ui.checkbox(&mut self.gen_options.use_lower, "Minuscules (a-z)").changed() {
                    self.regenerate_password();
                }
                if ui.checkbox(&mut self.gen_options.use_digits, "Chiffres (0-9)").changed() {
                    self.regenerate_password();
                }
                if ui.checkbox(&mut self.gen_options.use_symbols, "Symboles (!@#$...)").changed() {
                    self.regenerate_password();
                }
                if ui.checkbox(&mut self.gen_options.exclude_ambiguous, "Exclure caractères ambigus (0, O, l, 1, I)").changed() {
                    self.regenerate_password();
                }

                ui.add_space(20.0);
                let close_btn = egui::Button::new(RichText::new("Fermer").size(14.0)).min_size(Vec2::new(110.0, 40.0));
                if ui.add(close_btn).clicked() {
                    close = true;
                }
            });

        if close {
            self.show_generator_modal = false;
        }
    }

    fn ui_import_modal(&mut self, ctx: &egui::Context) {
        let mut close = false;
        let mut do_import = false;

        egui::Window::new("📥 Importer des identifiants (CSV)")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .frame(
                Frame::none()
                    .fill(COLOR_SURFACE)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                    .rounding(Rounding::same(14.0))
                    .inner_margin(Margin::same(24.0)),
            )
            .show(ctx, |ui| {
                ui.set_width(520.0);

                ui.label(RichText::new("Chemin du fichier CSV à importer :").strong().size(13.5).color(COLOR_TEXT_MAIN));
                ui.add_space(4.0);
                ui.add(TextEdit::singleline(&mut self.import_file_path).desired_width(f32::INFINITY).margin(Margin::same(8.0)));
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    let parse_btn = egui::Button::new(RichText::new("🔍 Examiner le fichier").size(13.0).strong())
                        .min_size(Vec2::new(160.0, 34.0));
                    if ui.add(parse_btn).clicked() {
                        let path = std::path::Path::new(&self.import_file_path);
                        match parse_csv_file(path) {
                            Ok(entries) => {
                                let count = entries.len();
                                self.import_parsed_entries = Some(entries);
                                self.show_toast(format!("{} identifiants valides trouvés !", count), false);
                            }
                            Err(e) => {
                                self.import_parsed_entries = None;
                                self.show_toast(format!("Erreur : {}", e), true);
                            }
                        }
                    }

                    if self.import_parsed_entries.is_none() {
                        ui.label(RichText::new("Format standard : nom, login, mot de passe (colonnes 1, 2, 3)").size(12.0).color(COLOR_TEXT_MUTED));
                    }
                });

                if let Some(entries) = &self.import_parsed_entries {
                    ui.add_space(14.0);
                    Frame::none()
                        .fill(COLOR_EMERALD_BG)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(167, 243, 208)))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("✅ {} identifiant(s) détecté(s) prêt(s) pour l'intégration.", entries.len()))
                                    .color(COLOR_EMERALD)
                                    .strong()
                                    .size(13.5),
                            );
                        });

                    ui.add_space(10.0);
                    ui.checkbox(&mut self.import_override_existing, "Écraser les entrées existantes en cas de doublon (même titre et login)");
                }

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    let cancel_btn = egui::Button::new(RichText::new("Annuler").size(14.0)).min_size(Vec2::new(110.0, 40.0));
                    if ui.add(cancel_btn).clicked() {
                        close = true;
                    }

                    if let Some(entries) = &self.import_parsed_entries {
                        if !entries.is_empty() {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let import_btn = egui::Button::new(
                                    RichText::new(format!("Importer dans le coffre ({})", entries.len()))
                                        .size(14.5)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(COLOR_PRIMARY)
                                .rounding(Rounding::same(8.0))
                                .min_size(Vec2::new(240.0, 42.0));

                                if ui.add(import_btn).clicked() {
                                    do_import = true;
                                    close = true;
                                }
                            });
                        }
                    }
                });
            });

        if do_import {
            if let Some(new_entries) = self.import_parsed_entries.take() {
                let mut imported = 0;
                let mut updated = 0;
                if let Some(v) = &mut self.vault {
                    for entry in new_entries {
                        let existing_idx = v.entries.iter().position(|e| {
                            e.title.eq_ignore_ascii_case(&entry.title) && e.username.eq_ignore_ascii_case(&entry.username)
                        });

                        if let Some(idx) = existing_idx {
                            if self.import_override_existing {
                                v.entries[idx] = entry;
                                updated += 1;
                            }
                        } else {
                            v.entries.push(entry);
                            imported += 1;
                        }
                    }
                    v.log_action("CREDENTIALS_IMPORTED", format!("Importés: {}, Mis à jour: {}", imported, updated));
                }
                let _ = self.save_current_vault();
                self.show_toast(format!("Import réussi : {} ajouté(s), {} mis à jour !", imported, updated), false);
            }
        }

        if close {
            self.show_import_modal = false;
        }
    }

    fn ui_settings_modal(&mut self, ctx: &egui::Context) {
        let mut close = false;
        let mut save_needed = false;

        egui::Window::new("⚙️ Paramètres & Personnalisation")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .frame(
                Frame::none()
                    .fill(COLOR_SURFACE)
                    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                    .rounding(Rounding::same(14.0))
                    .inner_margin(Margin::same(24.0)),
            )
            .show(ctx, |ui| {
                ui.set_width(540.0);

                // Onglets en haut de la fenêtre
                ui.horizontal(|ui| {
                    let tabs = [
                        ("🏷️ Catégories & Tags", 0),
                        ("🔑 Mot de passe maître", 1),
                        ("🛡️ 2FA & Sécurité", 2),
                    ];
                    for (name, idx) in tabs {
                        let is_active = self.settings_tab == idx;
                        let (bg, text_color, stroke) = if is_active {
                            (COLOR_PRIMARY_LIGHT, COLOR_PRIMARY, Stroke::new(1.5_f32, COLOR_PRIMARY))
                        } else {
                            (COLOR_SURFACE_ALT, COLOR_TEXT_MUTED, Stroke::new(1.0_f32, COLOR_BORDER))
                        };

                        let btn = egui::Button::new(RichText::new(name).size(13.0).strong().color(text_color))
                            .fill(bg)
                            .stroke(stroke)
                            .rounding(Rounding::same(8.0))
                            .min_size(Vec2::new(160.0, 34.0));

                        if ui.add(btn).clicked() {
                            self.settings_tab = idx;
                        }
                    }
                });

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(14.0);

                match self.settings_tab {
                    0 => {
                        // === ONGLET 1 : CATÉGORIES & TAGS ===
                        ui.label(RichText::new("📁 Gestion des Catégories").size(15.0).strong().color(COLOR_TEXT_MAIN));
                        ui.add_space(4.0);
                        ui.label(RichText::new("Ajoutez ou supprimez vos catégories personnalisées.").size(12.5).color(COLOR_TEXT_MUTED));
                        ui.add_space(8.0);

                        // Formulaire ajout catégorie
                        ui.horizontal(|ui| {
                            ui.add(TextEdit::singleline(&mut self.new_cat_input).hint_text("Nouvelle catégorie...").desired_width(260.0).margin(Margin::same(8.0)));
                            let add_cat_btn = egui::Button::new(RichText::new("➕ Ajouter").size(13.0).strong().color(Color32::WHITE))
                                .fill(COLOR_PRIMARY)
                                .min_size(Vec2::new(100.0, 34.0));
                            if ui.add(add_cat_btn).clicked() {
                                if !self.new_cat_input.trim().is_empty() {
                                    if let Some(v) = &mut self.vault {
                                        v.add_category(&self.new_cat_input);
                                        save_needed = true;
                                    }
                                    self.new_cat_input.clear();
                                    self.show_toast("Catégorie ajoutée avec succès !", false);
                                }
                            }
                        });

                        ui.add_space(10.0);

                        // Liste des catégories existantes
                        let mut to_remove_cat: Option<String> = None;
                        if let Some(v) = &self.vault {
                            let cats = v.get_categories();
                            egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                                for cat in &cats {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("• {}", cat)).size(13.5).strong().color(COLOR_TEXT_MAIN));
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if cat != "Général" {
                                                let del_btn = egui::Button::new(RichText::new("🗑️").size(12.0).color(COLOR_ROSE))
                                                    .min_size(Vec2::new(32.0, 26.0));
                                                if ui.add(del_btn).on_hover_text("Supprimer cette catégorie").clicked() {
                                                    to_remove_cat = Some(cat.clone());
                                                }
                                            }
                                        });
                                    });
                                    ui.add_space(4.0);
                                }
                            });
                        }
                        if let Some(c) = to_remove_cat {
                            if let Some(v) = &mut self.vault {
                                v.remove_category(&c);
                                save_needed = true;
                                self.show_toast(format!("Catégorie '{}' supprimée.", c), false);
                            }
                        }

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(12.0);

                        // Gestion des tags
                        ui.label(RichText::new("🏷️ Gestion des Tags Globaux").size(15.0).strong().color(COLOR_TEXT_MAIN));
                        ui.add_space(4.0);
                        ui.label(RichText::new("Définissez des étiquettes pour classer rapidement vos identifiants.").size(12.5).color(COLOR_TEXT_MUTED));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.add(TextEdit::singleline(&mut self.new_tag_input).hint_text("Nouveau tag...").desired_width(260.0).margin(Margin::same(8.0)));
                            let add_tag_btn = egui::Button::new(RichText::new("➕ Ajouter").size(13.0).strong().color(Color32::WHITE))
                                .fill(COLOR_PRIMARY)
                                .min_size(Vec2::new(100.0, 34.0));
                            if ui.add(add_tag_btn).clicked() {
                                if !self.new_tag_input.trim().is_empty() {
                                    if let Some(v) = &mut self.vault {
                                        v.add_tag(&self.new_tag_input);
                                        save_needed = true;
                                    }
                                    self.new_tag_input.clear();
                                    self.show_toast("Tag ajouté !", false);
                                }
                            }
                        });

                        ui.add_space(10.0);
                        let mut to_remove_tag: Option<String> = None;
                        if let Some(v) = &self.vault {
                            let tags = v.get_all_tags();
                            ui.horizontal_wrapped(|ui| {
                                for tag in &tags {
                                    Frame::none()
                                        .fill(COLOR_SURFACE_ALT)
                                        .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                                        .rounding(Rounding::same(12.0))
                                        .inner_margin(Margin::symmetric(10.0, 5.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(tag).size(12.5).strong().color(COLOR_TEXT_MAIN));
                                                if ui.button(RichText::new("✕").size(11.0).color(COLOR_TEXT_MUTED)).clicked() {
                                                    to_remove_tag = Some(tag.clone());
                                                }
                                            });
                                        });
                                }
                            });
                        }
                        if let Some(t) = to_remove_tag {
                            if let Some(v) = &mut self.vault {
                                v.remove_tag(&t);
                                save_needed = true;
                                self.show_toast(format!("Tag '{}' supprimé.", t), false);
                            }
                        }
                    }
                    1 => {
                        // === ONGLET 2 : MOT DE PASSE MAÎTRE ===
                        ui.label(RichText::new("🔑 Modifier le Mot de Passe Maître").size(15.0).strong().color(COLOR_TEXT_MAIN));
                        ui.add_space(4.0);
                        ui.label(RichText::new("Le coffre sera re-chiffré en AES-256-GCM avec une nouvelle clé Argon2id.").size(12.5).color(COLOR_TEXT_MUTED));
                        ui.add_space(16.0);

                        ui.label(RichText::new("Mot de passe maître actuel * :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                        ui.add(TextEdit::singleline(&mut self.change_pwd_current).password(true).desired_width(f32::INFINITY).margin(Margin::same(8.0)));
                        ui.add_space(10.0);

                        ui.label(RichText::new("Nouveau mot de passe maître * (min. 8 caractères) :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                        ui.add(TextEdit::singleline(&mut self.change_pwd_new).password(true).desired_width(f32::INFINITY).margin(Margin::same(8.0)));
                        ui.add_space(10.0);

                        ui.label(RichText::new("Confirmer le nouveau mot de passe maître * :").strong().size(13.0).color(COLOR_TEXT_MAIN));
                        ui.add(TextEdit::singleline(&mut self.change_pwd_confirm).password(true).desired_width(f32::INFINITY).margin(Margin::same(8.0)));
                        ui.add_space(18.0);

                        let update_pwd_btn = egui::Button::new(
                            RichText::new("Re-chiffrer & Mettre à jour le mot de passe maître")
                                .size(14.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(COLOR_PRIMARY)
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(f32::INFINITY, 44.0));

                        if ui.add(update_pwd_btn).clicked() {
                            if self.change_pwd_current != self.input_master_password {
                                self.show_toast("Le mot de passe maître actuel est incorrect.", true);
                            } else if self.change_pwd_new.len() < 8 {
                                self.show_toast("Le nouveau mot de passe doit comporter au moins 8 caractères.", true);
                            } else if self.change_pwd_new != self.change_pwd_confirm {
                                self.show_toast("Les nouveaux mots de passe ne correspondent pas.", true);
                            } else {
                                if let Some(v) = &mut self.vault {
                                    let account_name = v.totp_config.account_name.clone();
                                    v.log_action("MASTER_PASSWORD_CHANGED", &account_name);
                                    match save_vault(v, &self.change_pwd_new, &self.vault_path) {
                                        Ok(()) => {
                                            self.input_master_password = self.change_pwd_new.clone();
                                            self.change_pwd_current.clear();
                                            self.change_pwd_new.clear();
                                            self.change_pwd_confirm.clear();
                                            self.show_toast("Mot de passe maître mis à jour et coffre re-chiffré avec succès !", false);
                                            close = true;
                                        }
                                        Err(e) => {
                                            self.show_toast(format!("Erreur lors du re-chiffrement : {}", e), true);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // === ONGLET 3 : SÉCURITÉ & AUDIT LOGS ===
                        ui.label(RichText::new("🛡️ État de la Sécurité").size(15.0).strong().color(COLOR_TEXT_MAIN));
                        ui.add_space(8.0);

                        Frame::none()
                            .fill(COLOR_SURFACE_ALT)
                            .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
                            .rounding(Rounding::same(10.0))
                            .inner_margin(Margin::same(14.0))
                            .show(ui, |ui| {
                                if let Some(v) = &self.vault {
                                    ui.label(RichText::new(format!("Compte enregistré : {}", v.totp_config.account_name)).strong().size(13.5).color(COLOR_TEXT_MAIN));
                                    ui.add_space(4.0);
                                    ui.label(RichText::new("Chiffrement : AES-256-GCM + Argon2id (Zeroize)").size(13.0).color(COLOR_PRIMARY));
                                    ui.add_space(4.0);
                                    ui.label(RichText::new(format!("Codes de secours restants : {}", v.totp_config.hashed_backup_codes.len())).size(13.0).color(COLOR_EMERALD));
                                }
                            });

                        ui.add_space(14.0);
                        ui.label(RichText::new("📜 Journal d'audit des actions récentes :").strong().size(13.5).color(COLOR_TEXT_MAIN));
                        ui.add_space(6.0);

                        if let Some(v) = &self.vault {
                            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                for log in v.audit_logs.iter().rev().take(15) {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(log.timestamp.format("%d/%m/%Y %H:%M").to_string()).size(11.5).color(COLOR_TEXT_MUTED));
                                        ui.label(RichText::new(&log.action).strong().size(12.0).color(COLOR_PRIMARY));
                                        ui.label(RichText::new(&log.target).size(12.0).color(COLOR_TEXT_MAIN));
                                    });
                                    ui.add_space(2.0);
                                }
                            });
                        }
                    }
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(12.0);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let close_btn = egui::Button::new(RichText::new("Fermer").size(14.0)).min_size(Vec2::new(110.0, 38.0));
                    if ui.add(close_btn).clicked() {
                        close = true;
                    }
                });
            });

        if save_needed {
            let _ = self.save_current_vault();
        }

        if close {
            self.show_settings_modal = false;
        }
    }
}
