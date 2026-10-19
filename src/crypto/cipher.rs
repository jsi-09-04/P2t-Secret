//! crypto/cipher.rs - Chiffrement et déchiffrement authentifié AES-256-GCM.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Structure contenant les données chiffrées et le nonce unique de 96 bits.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EncryptedBlob {
    /// Nonce aléatoire unique de 12 octets (96 bits)
    pub nonce: [u8; 12],
    /// Données chiffrées incluant le tag d'authentification de 16 octets à la fin
    pub ciphertext: Vec<u8>,
}

/// Chaîne de caractères sensible effacée de la mémoire lors du drop.
#[derive(Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SecretString(pub String);

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretString(••••••••)")
    }
}

impl SecretString {
    pub fn new(val: impl Into<String>) -> Self {
        Self(val.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Chiffre un payload binaire avec AES-256-GCM et une clé de 32 octets.
pub fn encrypt_payload(data: &[u8], key: &[u8; 32]) -> Result<EncryptedBlob, String> {
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Erreur initialisation AES-GCM: {}", e))?;

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Erreur chiffrement AES-GCM: {}", e))?;

    Ok(EncryptedBlob {
        nonce: nonce_bytes,
        ciphertext,
    })
}

/// Déchiffre un payload binaire et valide le tag d'intégrité de 16 octets.
pub fn decrypt_payload(blob: &EncryptedBlob, key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let nonce = Nonce::from_slice(&blob.nonce);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Erreur initialisation AES-GCM: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, blob.ciphertext.as_ref())
        .map_err(|_| "Échec de déchiffrement : mot de passe incorrect ou données corrompues.".to_string())?;

    Ok(plaintext)
}
