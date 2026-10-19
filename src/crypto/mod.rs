//! crypto/mod.rs - Racine du module cryptographique.

pub mod cipher;
pub mod generator;
pub mod kdf;
pub mod totp;

pub use cipher::{decrypt_payload, encrypt_payload, EncryptedBlob, SecretString};
pub use generator::{generate_password, PasswordOptions, PasswordResult};
pub use kdf::{derive_master_key, generate_salt, MasterKey};
pub use totp::{generate_qr_matrix, TotpConfig, ISSUER_NAME};
