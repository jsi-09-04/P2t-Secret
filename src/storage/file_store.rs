//! storage/file_store.rs - Enregistrement et chargement atomique du fichier de coffre-fort chiffré.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::crypto::{
    derive_master_key, generate_salt, decrypt_payload, encrypt_payload, EncryptedBlob
};
use crate::storage::model::Vault;

pub const MAGIC_HEADER: &[u8; 6] = b"P2TV01";

/// Détermine le chemin standard du coffre-fort sur la machine.
pub fn default_vault_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".p2tsecret").join("vault.p2t")
}

/// Sauvegarde de manière atomique le coffre-fort chiffré.
pub fn save_vault(vault: &Vault, password: &str, file_path: &Path) -> Result<(), String> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Impossible de créer le dossier: {}", e))?;
    }

    // 1. Sérialisation JSON du coffre
    let json_bytes = serde_json::to_vec(vault)
        .map_err(|e| format!("Erreur sérialisation coffre: {}", e))?;

    // 2. Génération d'un sel Argon2id
    let salt = generate_salt();

    // 3. Dérivation de la clé maîtresse
    let master_key = derive_master_key(password, &salt)?;

    // 4. Chiffrement AES-256-GCM
    let encrypted_blob = encrypt_payload(&json_bytes, &master_key.0)?;

    // 5. Assemblage du format binaire
    // [MAGIC (6)] + [SALT (16)] + [NONCE (12)] + [CIPHERTEXT]
    let mut buffer = Vec::with_capacity(6 + 16 + 12 + encrypted_blob.ciphertext.len());
    buffer.extend_from_slice(MAGIC_HEADER);
    buffer.extend_from_slice(&salt);
    buffer.extend_from_slice(&encrypted_blob.nonce);
    buffer.extend_from_slice(&encrypted_blob.ciphertext);

    // 6. Écriture atomique via fichier temporaire
    let tmp_path = file_path.with_extension("tmp");
    {
        let mut tmp_file = File::create(&tmp_path)
            .map_err(|e| format!("Erreur création fichier temporaire: {}", e))?;
        tmp_file.write_all(&buffer)
            .map_err(|e| format!("Erreur écriture fichier temporaire: {}", e))?;
        tmp_file.sync_all()
            .map_err(|e| format!("Erreur sync disque: {}", e))?;
    }

    fs::rename(&tmp_path, file_path)
        .map_err(|e| format!("Erreur écriture atomique du coffre: {}", e))?;

    Ok(())
}

/// Charge et déchiffre le coffre-fort depuis le disque.
pub fn load_vault(file_path: &Path, password: &str) -> Result<Vault, String> {
    if !file_path.exists() {
        return Err("Fichier de coffre-fort introuvable.".to_string());
    }

    let mut file = File::open(file_path)
        .map_err(|e| format!("Impossible d'ouvrir le fichier: {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Erreur lecture fichier: {}", e))?;

    if buffer.len() < (6 + 16 + 12 + 16) {
        return Err("Fichier de coffre corrompu ou incomplet.".to_string());
    }

    // Vérification de l'en-tête magique
    if &buffer[0..6] != MAGIC_HEADER {
        return Err("Format de fichier non reconnu ou incompatible.".to_string());
    }

    let mut salt = [0u8; 16];
    salt.copy_from_slice(&buffer[6..22]);

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&buffer[22..34]);

    let ciphertext = buffer[34..].to_vec();

    // Dérivation de la clé maîtresse
    let master_key = derive_master_key(password, &salt)?;

    // Déchiffrement AES-256-GCM
    let blob = EncryptedBlob { nonce, ciphertext };
    let json_bytes = decrypt_payload(&blob, &master_key.0)?;

    // Désérialisation du Vault
    let vault: Vault = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("Erreur désérialisation du coffre: {}", e))?;

    Ok(vault)
}
