//! ui/app.rs - Application principale egui, machine à états et gestion des vues.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{Align, Color32, Layout, RichText, ScrollArea, TextEdit, Ui};

use crate::crypto::{
    generate_password, generate_qr_matrix, PasswordOptions, SecretString,
};
use crate::storage::{
    default_vault_path, load_vault, save_vault, CredentialEntry, Vault,
};
use crate::ui::clipboard::copy_and_auto_clear;
use crate::ui::qr_widget::render_qr_code;
use crate::ui::theme::{COLOR_CYAN, COLOR_EMERALD, COLOR_ROSE, COLOR_TEXT_MUTED};

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
        self.show_toast("Coffre-fort verrouillé.", false);
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
        // En-tête principal
        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔐 P2T").size(20.0).strong().color(COLOR_CYAN));
                ui.label(RichText::new("Secret").size(20.0).strong().color(Color32::WHITE));
                ui.label(RichText::new("— Coffre-Fort Sécurisé 2FA").size(13.0).color(COLOR_TEXT_MUTED));

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.screen == AppScreen::Dashboard {
                        if ui.button(RichText::new("🔒 Verrouiller").color(COLOR_ROSE)).clicked() {
                            self.lock_vault();
                        }
                        ui.label(RichText::new("🛡️ 2FA Microsoft Actif").color(COLOR_EMERALD).size(12.0));
                        ui.label(RichText::new("🔒 AES-256-GCM").color(COLOR_CYAN).size(12.0));
                    }
                });
            });
            ui.add_space(6.0);
        });

        // Conteneur Central
        egui::CentralPanel::default().show(ctx, |ui| {
            // Affichage Toast Notification
            if let Some((msg, expiry, is_error)) = &self.toast_message {
                if Instant::now() < *expiry {
                    let color = if *is_error { COLOR_ROSE } else { COLOR_EMERALD };
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(if *is_error { "⚠️" } else { "✅" }).size(16.0));
                            ui.label(RichText::new(msg).color(color).strong());
                        });
                    });
                    ui.add_space(10.0);
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

        // Modales éventuelles
        if self.show_entry_modal {
            self.ui_entry_modal(ctx);
        }
        if self.show_generator_modal {
            self.ui_generator_modal(ctx);
        }
    }
}

// =========================================================================
// Écrans d'Initialisation (Setup)
// =========================================================================
impl P2TSecretApp {
    fn ui_init_password(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("Création du Coffre-Fort").size(24.0).strong().color(Color32::WHITE));
            ui.label(RichText::new("Définissez votre mot de passe maître pour chiffrer vos identifiants.").color(COLOR_TEXT_MUTED));
            ui.add_space(20.0);

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_max_width(450.0);
                ui.add_space(10.0);

                ui.label("Compte / Email :");
                ui.add(TextEdit::singleline(&mut self.input_account_name).desired_width(f32::INFINITY));
                ui.add_space(10.0);

                ui.label("Mot de passe maître :");
                ui.add(TextEdit::singleline(&mut self.input_master_password).password(true).desired_width(f32::INFINITY));
                ui.add_space(10.0);

                ui.label("Confirmer le mot de passe maître :");
                ui.add(TextEdit::singleline(&mut self.input_master_password_confirm).password(true).desired_width(f32::INFINITY));
                ui.add_space(20.0);

                if ui.button(RichText::new("Continuer vers la configuration 2FA →").size(15.0).strong().color(Color32::BLACK)).clicked() {
                    if self.input_master_password.len() < 8 {
                        self.show_toast("Le mot de passe maître doit comporter au moins 8 caractères.", true);
                    } else if self.input_master_password != self.input_master_password_confirm {
                        self.show_toast("Les deux mots de passe ne correspondent pas.", true);
                    } else {
                        // Créer le coffre en mémoire et générer le QR code 2FA
                        match Vault::new(&self.input_account_name) {
                            Ok(v) => {
                                match v.totp_config.get_url() {
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
                                }
                            }
                            Err(e) => self.show_toast(e, true),
                        }
                    }
                }
                ui.add_space(10.0);
            });
        });
    }

    fn ui_init_2fa_setup(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("Configuration 2FA Microsoft Authenticator").size(22.0).strong().color(Color32::WHITE));
            ui.label(RichText::new("Scannez ce QR Code avec l'application Microsoft Authenticator sur votre mobile.").color(COLOR_TEXT_MUTED));
            ui.add_space(15.0);

            // Rendu du QR Code vectoriel
            render_qr_code(ui, self.setup_qr_size, &self.setup_qr_modules, 220.0);
            ui.add_space(10.0);

            ui.label(RichText::new(format!("Clé manuelle : {}", self.setup_secret_key)).color(COLOR_CYAN).monospace());
            ui.add_space(15.0);

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_max_width(380.0);
                ui.label("Code à 6 chiffres affiché par Microsoft Authenticator :");
                let response = ui.add(
                    TextEdit::singleline(&mut self.input_totp_code)
                        .desired_width(180.0)
                        .font(egui::FontId::monospace(22.0))
                );
                response.request_focus();

                ui.add_space(15.0);
                if ui.button(RichText::new("Valider et Activer la 2FA 🛡️").size(15.0).strong()).clicked() {
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
                        self.show_toast("Code invalide ou expiré. Vérifiez Microsoft Authenticator.", true);
                    }
                }
            });
        });
    }

    fn ui_init_backup_codes(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(25.0);
            ui.label(RichText::new("Codes de Secours à Usage Unique").size(22.0).strong().color(COLOR_EMERALD));
            ui.label(RichText::new("Conservez précieusement ces codes en lieu sûr. Ils vous permettront de déverrouiller votre coffre si vous perdez votre smartphone.").color(COLOR_TEXT_MUTED));
            ui.add_space(20.0);

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_max_width(450.0);
                egui::Grid::new("backup_codes_grid").spacing([30.0, 10.0]).show(ui, |ui| {
                    for (i, code) in self.setup_backup_codes.iter().enumerate() {
                        ui.label(RichText::new(code).monospace().size(16.0).color(COLOR_CYAN).strong());
                        if (i + 1) % 2 == 0 {
                            ui.end_row();
                        }
                    }
                });

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui.button("📋 Copier tous les codes").clicked() {
                        let text = self.setup_backup_codes.join("\n");
                        let _ = copy_and_auto_clear(text, 60);
                        self.show_toast("Codes de secours copiés dans le presse-papier.", false);
                    }

                    if ui.button(RichText::new("Terminer et Accéder au Coffre →").strong().color(Color32::BLACK)).clicked() {
                        match self.save_current_vault() {
                            Ok(()) => {
                                self.screen = AppScreen::Dashboard;
                                self.show_toast("Coffre-fort initialisé et chiffré sur disque !", false);
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
            ui.add_space(60.0);
            ui.label(RichText::new("Déverrouillage du Coffre-Fort").size(24.0).strong().color(Color32::WHITE));
            ui.label(RichText::new("Entrez votre mot de passe maître pour déchiffrer vos secrets.").color(COLOR_TEXT_MUTED));
            ui.add_space(25.0);

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_max_width(400.0);
                ui.label("Mot de passe maître :");
                let input = ui.add(TextEdit::singleline(&mut self.input_master_password).password(true).desired_width(f32::INFINITY));
                input.request_focus();

                ui.add_space(20.0);
                if ui.button(RichText::new("Déverrouiller (Étape 1/2) →").size(15.0).strong()).clicked()
                    || (input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
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
    }

    fn ui_unlock_2fa(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.label(RichText::new("Authentification Forte 2FA").size(24.0).strong().color(COLOR_CYAN));
            ui.label(RichText::new("Ouvrez Microsoft Authenticator et saisissez votre code dynamique.").color(COLOR_TEXT_MUTED));
            ui.add_space(25.0);

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_max_width(400.0);

                if !self.is_recovery_mode {
                    ui.label("Code à 6 chiffres Microsoft Authenticator :");
                    let input = ui.add(
                        TextEdit::singleline(&mut self.input_totp_code)
                            .desired_width(180.0)
                            .font(egui::FontId::monospace(26.0))
                    );
                    input.request_focus();

                    ui.add_space(20.0);
                    if ui.button(RichText::new("Valider le Code 🛡️").size(15.0).strong()).clicked()
                        || (input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        let mut ok = false;
                        if let Some(v) = &self.vault {
                            ok = v.totp_config.verify_code(&self.input_totp_code);
                        }
                        if ok {
                            self.screen = AppScreen::Dashboard;
                            self.show_toast("Authentification 2FA validée. Bienvenue !", false);
                        } else {
                            self.show_toast("Code TOTP incorrect ou expiré.", true);
                        }
                    }

                    ui.add_space(15.0);
                    if ui.button("Perte de smartphone ? Utiliser un code de secours").clicked() {
                        self.is_recovery_mode = true;
                    }
                } else {
                    ui.label("Code de secours (XXXX-XXXX) :");
                    let input = ui.add(TextEdit::singleline(&mut self.input_recovery_code).desired_width(200.0).font(egui::FontId::monospace(20.0)));
                    input.request_focus();

                    ui.add_space(20.0);
                    if ui.button(RichText::new("Utiliser ce code de secours").size(14.0).strong()).clicked() {
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

                    ui.add_space(10.0);
                    if ui.button("Revenir à Microsoft Authenticator").clicked() {
                        self.is_recovery_mode = false;
                    }
                }
            });
        });
    }
}

// =========================================================================
// Tableau de bord du Coffre (Dashboard)
// =========================================================================
impl P2TSecretApp {
    fn ui_dashboard(&mut self, ui: &mut Ui) {
        // Barre d'outils
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(16.0));
            ui.add(TextEdit::singleline(&mut self.search_query).hint_text("Rechercher un service, login, catégorie, tag...").desired_width(320.0));

            ui.add_space(10.0);
            if ui.button("⚡ Générateur MDP").clicked() {
                self.show_generator_modal = true;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button(RichText::new("+ Nouvel Identifiant").strong().color(Color32::BLACK)).clicked() {
                    self.open_new_entry_modal();
                }
            });
        });

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(10.0);

        // Récupérer les identifiants
        let query = self.search_query.to_lowercase();
        let entries = if let Some(v) = &self.vault {
            v.entries
                .iter()
                .filter(|e| {
                    query.is_empty()
                        || e.title.to_lowercase().contains(&query)
                        || e.username.to_lowercase().contains(&query)
                        || e.url.to_lowercase().contains(&query)
                        || e.category.to_lowercase().contains(&query)
                        || e.tags.iter().any(|t| t.to_lowercase().contains(&query))
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if entries.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(RichText::new("🔒").size(40.0));
                ui.label(RichText::new("Aucun identifiant trouvé").size(18.0).color(COLOR_TEXT_MUTED));
                ui.label("Cliquez sur '+ Nouvel Identifiant' pour enregistrer votre premier secret.");
            });
            return;
        }

        // Grille des cartes
        ScrollArea::vertical().show(ui, |ui| {
            let mut to_delete: Option<String> = None;
            let mut to_edit: Option<CredentialEntry> = None;
            let mut to_copy_pwd: Option<String> = None;
            let mut to_copy_login: Option<String> = None;

            for entry in &entries {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&entry.title).size(16.0).strong().color(COLOR_CYAN));
                            if !entry.url.is_empty() {
                                ui.label(RichText::new(&entry.url).color(COLOR_TEXT_MUTED).size(12.0));
                            }
                        });

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            if ui.button(RichText::new("🗑️").color(COLOR_ROSE)).on_hover_text("Supprimer").clicked() {
                                to_delete = Some(entry.id.clone());
                            }
                            if ui.button("✏️").on_hover_text("Modifier").clicked() {
                                to_edit = Some(entry.clone());
                            }
                            ui.label(RichText::new(&entry.category).size(11.0).color(COLOR_TEXT_MUTED));
                        });
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Ligne Login
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Login :").color(COLOR_TEXT_MUTED).size(13.0));
                        ui.label(RichText::new(&entry.username).strong());
                        if ui.button("📋").on_hover_text("Copier l'identifiant").clicked() {
                            to_copy_login = Some(entry.username.clone());
                        }
                    });

                    // Ligne Mot de passe
                    let is_revealed = self.revealed_passwords.contains(&entry.id);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Mot de passe :").color(COLOR_TEXT_MUTED).size(13.0));
                        if is_revealed {
                            ui.label(RichText::new(entry.password.as_str()).monospace().strong().color(COLOR_EMERALD));
                        } else {
                            ui.label(RichText::new("••••••••••••").monospace().strong());
                        }

                        let eye_icon = if is_revealed { "🙈" } else { "👁️" };
                        if ui.button(eye_icon).on_hover_text("Afficher / Masquer").clicked() {
                            if is_revealed {
                                self.revealed_passwords.remove(&entry.id);
                            } else {
                                self.revealed_passwords.insert(entry.id.clone());
                            }
                        }

                        if ui.button("📋 Copier").on_hover_text("Copier le mot de passe (auto-effacement 30s)").clicked() {
                            to_copy_pwd = Some(entry.password.as_str().to_string());
                        }
                    });

                    if !entry.notes.as_str().is_empty() {
                        ui.add_space(4.0);
                        ui.label(RichText::new(format!("Notes : {}", entry.notes.as_str())).color(COLOR_TEXT_MUTED).size(12.0));
                    }
                });
                ui.add_space(8.0);
            }

            // Actions déclenchées
            if let Some(login) = to_copy_login {
                let _ = copy_and_auto_clear(login, 60);
                self.show_toast("Identifiant copié !", false);
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
        let title = if is_edit { "Modifier l'identifiant" } else { "Nouvel identifiant sécurisé" };

        let mut close_modal = false;
        let mut save_entry = false;

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_width(450.0);

                ui.label("Titre / Service * :");
                ui.add(TextEdit::singleline(&mut self.modal_title).desired_width(f32::INFINITY));

                ui.label("URL d'accès :");
                ui.add(TextEdit::singleline(&mut self.modal_url).desired_width(f32::INFINITY));

                ui.label("Login / Utilisateur * :");
                ui.add(TextEdit::singleline(&mut self.modal_username).desired_width(f32::INFINITY));

                ui.horizontal(|ui| {
                    ui.label("Mot de passe secret * :");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("⚡ Générer").clicked() {
                            let res = generate_password(&self.gen_options);
                            self.modal_password = res.password;
                        }
                    });
                });
                ui.add(TextEdit::singleline(&mut self.modal_password).desired_width(f32::INFINITY));

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Catégorie :");
                        ui.add(TextEdit::singleline(&mut self.modal_category).desired_width(200.0));
                    });
                    ui.vertical(|ui| {
                        ui.label("Tags :");
                        ui.add(TextEdit::singleline(&mut self.modal_tags).desired_width(200.0));
                    });
                });

                ui.label("Notes confidentielles :");
                ui.add(TextEdit::multiline(&mut self.modal_notes).desired_rows(3).desired_width(f32::INFINITY));

                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    if ui.button("Annuler").clicked() {
                        close_modal = true;
                    }
                    if ui.button(RichText::new("Chiffrer & Sauvegarder").strong().color(Color32::BLACK)).clicked() {
                        if self.modal_title.trim().is_empty() || self.modal_username.trim().is_empty() || self.modal_password.trim().is_empty() {
                            self.show_toast("Veuillez renseigner le titre, le login et le mot de passe.", true);
                        } else {
                            save_entry = true;
                            close_modal = true;
                        }
                    }
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
            self.show_toast("Coffre-fort mis à jour avec succès.", false);
        }

        if close_modal {
            self.show_entry_modal = false;
        }
    }

    fn ui_generator_modal(&mut self, ctx: &egui::Context) {
        let mut close = false;

        egui::Window::new("Générateur de Mot de Passe Cryptographique")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_width(400.0);

                ui.label("Mot de passe généré :");
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.gen_current_password).monospace().size(18.0).color(COLOR_CYAN).strong());
                    if ui.button("🔄").clicked() {
                        self.regenerate_password();
                    }
                    if ui.button("📋").clicked() {
                        let _ = copy_and_auto_clear(self.gen_current_password.clone(), 30);
                        self.show_toast("Mot de passe copié !", false);
                    }
                });

                ui.label(RichText::new(&self.gen_entropy_text).color(COLOR_EMERALD).size(12.0));
                ui.add_space(10.0);

                ui.label(format!("Longueur : {} caractères", self.gen_options.length));
                if ui.add(egui::Slider::new(&mut self.gen_options.length, 8..=64)).changed() {
                    self.regenerate_password();
                }

                ui.add_space(5.0);
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

                ui.add_space(15.0);
                if ui.button("Fermer").clicked() {
                    close = true;
                }
            });

        if close {
            self.show_generator_modal = false;
        }
    }
}
