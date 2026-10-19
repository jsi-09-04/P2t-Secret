//! storage/mod.rs - Racine du module de persistance.

pub mod file_store;
pub mod model;

pub use file_store::{default_vault_path, load_vault, save_vault};
pub use model::{AuditEntry, CredentialEntry, Vault};
