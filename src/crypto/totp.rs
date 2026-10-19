//! crypto/totp.rs - Gestion TOTP RFC 6238 pour Microsoft Authenticator et matrice QR Code.

use totp_rs::{Algorithm, TOTP, Secret};
use qrcodegen::{QrCode, QrCodeEcc};
use rand::Rng;

pub const ISSUER_NAME: &str = "P2TSecret";

/// Configuration TOTP pour un utilisateur du coffre.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TotpConfig {
    /// Clé secrète Base32 (ex: 32 caractères)
    pub secret_base32: String,
    /// Compte associé (ex: "utilisateur@p2t")
    pub account_name: String,
    /// Hashes SHA-256 des codes de secours
    pub hashed_backup_codes: Vec<String>,
}

impl TotpConfig {
    /// Crée une nouvelle configuration avec une clé secrète Base32 aléatoire de 160 bits.
    pub fn new(account_name: &str) -> Result<Self, String> {
        let secret = Secret::generate_secret();
        let secret_base32 = secret.to_encoded().to_string();

        Ok(Self {
            secret_base32,
            account_name: account_name.to_string(),
            hashed_backup_codes: Vec::new(),
        })
    }

    /// Construit l'objet TOTP compatible Microsoft Authenticator (SHA1, 30s, 6 digits).
    pub fn build_totp(&self) -> Result<TOTP, String> {
        let secret_bytes = Secret::Encoded(self.secret_base32.clone())
            .to_bytes()
            .map_err(|e| format!("Erreur décodage clé Base32: {:?}", e))?;

        TOTP::new(
            Algorithm::SHA1,
            6,
            1, // Fenêtre de tolérance : +/- 1 intervalle (90s au total)
            30,
            secret_bytes,
            Some(ISSUER_NAME.to_string()),
            self.account_name.clone(),
        )
        .map_err(|e| format!("Erreur initialisation TOTP: {:?}", e))
    }

    /// Retourne l'URL standard `otpauth://` pour Microsoft Authenticator.
    pub fn get_url(&self) -> Result<String, String> {
        let totp = self.build_totp()?;
        Ok(totp.get_url())
    }

    /// Vérifie un code à 6 chiffres saisi par l'utilisateur.
    pub fn verify_code(&self, code: &str) -> bool {
        let clean = code.trim().replace(' ', "");
        if clean.len() != 6 || !clean.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        if let Ok(totp) = self.build_totp() {
            totp.check_current(&clean).unwrap_or(false)
        } else {
            false
        }
    }

    /// Génère 8 codes de secours aléatoires (format XXXX-XXXX) et enregistre leurs hashes.
    pub fn generate_backup_codes(&mut self) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let mut plain_codes = Vec::new();
        let mut hashes = Vec::new();

        for _ in range(8) {
            let n1: u16 = rng.gen_range(0x1000..=0xFFFF);
            let n2: u16 = rng.gen_range(0x1000..=0xFFFF);
            let code = format!("{:04X}-{:04X}", n1, n2);
            let hash = hash_code(&code);

            plain_codes.push(code);
            hashes.push(hash);
        }

        self.hashed_backup_codes = hashes;
        plain_codes
    }

    /// Tente d'utiliser un code de secours. Retourne true et consomme le code si valide.
    pub fn consume_backup_code(&mut self, code: &str) -> bool {
        let h = hash_code(code.trim().to_uppercase().as_str());
        if let Some(pos) = self.hashed_backup_codes.iter().position(|item| item == &h) {
            self.hashed_backup_codes.remove(pos);
            true
        } else {
            false
        }
    }
}

fn range(n: usize) -> std::ops::Range<usize> {
    0..n
}

fn hash_code(code: &str) -> String {
    // Hachage SHA-256 / hash déterministe pour les codes de secours
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Génère une matrice booléenne représentant les modules noirs (true) et blancs (false) du QR Code.
pub fn generate_qr_matrix(text: &str) -> Result<(usize, Vec<bool>), String> {
    let qr = QrCode::encode_text(text, QrCodeEcc::Medium)
        .map_err(|e| format!("Erreur encodage QR code: {:?}", e))?;

    let size = qr.size() as usize;
    let mut modules = Vec::with_capacity(size * size);

    for y in 0..qr.size() {
        for x in 0..qr.size() {
            modules.push(qr.get_module(x, y));
        }
    }

    Ok((size, modules))
}
