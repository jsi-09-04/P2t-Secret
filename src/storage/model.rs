//! storage/model.rs - Modèle de données sérialisable du coffre-fort P2TSecret.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::crypto::{SecretString, TotpConfig};

/// Version actuelle du schéma de coffre-fort
pub const VAULT_SCHEMA_VERSION: u32 = 1;

/// Données déchiffrées du coffre-fort stockées en mémoire vive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    pub schema_version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Configuration 2FA Microsoft Authenticator & codes de secours
    pub totp_config: TotpConfig,
    /// Liste des identifiants enregistrés
    pub entries: Vec<CredentialEntry>,
    /// Journal d'audit local
    pub audit_logs: Vec<AuditEntry>,
}

impl Vault {
    pub fn new(account_name: &str) -> Result<Self, String> {
        let now = Utc::now();
        let totp_config = TotpConfig::new(account_name)?;
        Ok(Self {
            schema_version: VAULT_SCHEMA_VERSION,
            created_at: now,
            updated_at: now,
            totp_config,
            entries: Vec::new(),
            audit_logs: vec![AuditEntry {
                timestamp: now,
                action: "VAULT_CREATED".to_string(),
                target: account_name.to_string(),
            }],
        })
    }

    pub fn log_action(&mut self, action: impl Into<String>, target: impl Into<String>) {
        self.audit_logs.push(AuditEntry {
            timestamp: Utc::now(),
            action: action.into(),
            target: target.into(),
        });
        if self.audit_logs.len() > 200 {
            self.audit_logs.remove(0);
        }
    }
}

/// Identifiant sécurisé dans le coffre-fort.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub username: String,
    /// Mot de passe confidentiel avec effacement Zeroize
    pub password: SecretString,
    pub category: String,
    pub tags: Vec<String>,
    /// Notes secrètes avec effacement Zeroize
    pub notes: SecretString,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CredentialEntry {
    pub fn new(
        title: impl Into<String>,
        url: impl Into<String>,
        username: impl Into<String>,
        password: SecretString,
        category: impl Into<String>,
        tags: Vec<String>,
        notes: SecretString,
    ) -> Self {
        let now = Utc::now();
        let id = format!("{:x}", rand::random::<u128>());
        Self {
            id,
            title: title.into(),
            url: url.into(),
            username: username.into(),
            password,
            category: category.into(),
            tags,
            notes,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Entrée du journal d'audit local.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub target: String,
}
