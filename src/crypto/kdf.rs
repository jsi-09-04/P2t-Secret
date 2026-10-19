//! crypto/kdf.rs - Dérivation de clé Argon2id avec effacement sécurisé en mémoire (Zeroize).

use argon2::{
    Argon2, Algorithm, Version, Params
};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Clé maîtresse AES-256 de 32 octets avec effacement garanti de la RAM lors de sa destruction.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey(pub [u8; 32]);

/// Dérive une clé de 256 bits à partir d'un mot de passe maître et d'un sel cryptographique de 16 octets via Argon2id.
pub fn derive_master_key(password: &str, salt: &[u8; 16]) -> Result<MasterKey, String> {
    // Paramètres recommandés OWASP : m_cost = 64 Mo, t_cost = 3 itérations, p_cost = 4 threads
    let params = Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| format!("Erreur paramètres Argon2: {}", e))?;
    
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let mut key_bytes = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key_bytes)
        .map_err(|e| format!("Erreur dérivation Argon2id: {}", e))?;
    
    Ok(MasterKey(key_bytes))
}

/// Génère un sel cryptographiquement sûr de 16 octets.
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut salt);
    salt
}
