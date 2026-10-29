//! storage/model.rs - Modèle de données sérialisable du coffre-fort P2TSecret.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::crypto::{SecretString, TotpConfig};

/// Version actuelle du schéma de coffre-fort
pub const VAULT_SCHEMA_VERSION: u32 = 1;

pub fn default_categories() -> Vec<String> {
    vec![
        "Général".to_string(),
        "Messagerie".to_string(),
        "Cloud".to_string(),
        "Serveurs".to_string(),
        "Finance".to_string(),
        "Pro".to_string(),
        "Personnel".to_string(),
    ]
}

/// Données déchiffrées du coffre-fort stockées en mémoire vive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    pub schema_version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Configuration 2FA Microsoft Authenticator & codes de secours
    pub totp_config: TotpConfig,
    /// Liste des catégories personnalisées
    #[serde(default = "default_categories")]
    pub custom_categories: Vec<String>,
    /// Liste des tags globaux réutilisables
    #[serde(default)]
    pub custom_tags: Vec<String>,
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
            custom_categories: default_categories(),
            custom_tags: Vec::new(),
            entries: Vec::new(),
            audit_logs: vec![AuditEntry {
                timestamp: now,
                action: "VAULT_CREATED".to_string(),
                target: account_name.to_string(),
            }],
        })
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut cats = self.custom_categories.clone();
        for e in &self.entries {
            if !e.category.is_empty() && !cats.contains(&e.category) {
                cats.push(e.category.clone());
            }
        }
        cats.sort();
        cats
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags = self.custom_tags.clone();
        for e in &self.entries {
            for t in &e.tags {
                if !t.is_empty() && !tags.contains(t) {
                    tags.push(t.clone());
                }
            }
        }
        tags.sort();
        tags
    }

    pub fn add_category(&mut self, cat: &str) {
        let c = cat.trim().to_string();
        if !c.is_empty() && !self.custom_categories.contains(&c) {
            self.custom_categories.push(c);
            self.log_action("CATEGORY_ADDED", cat);
        }
    }

    pub fn remove_category(&mut self, cat: &str) {
        self.custom_categories.retain(|c| c != cat);
        for e in &mut self.entries {
            if e.category == cat {
                e.category = "Général".to_string();
            }
        }
        self.log_action("CATEGORY_REMOVED", cat);
    }

    pub fn add_tag(&mut self, tag: &str) {
        let t = tag.trim().to_string();
        if !t.is_empty() && !self.custom_tags.contains(&t) {
            self.custom_tags.push(t);
            self.log_action("TAG_ADDED", tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.custom_tags.retain(|t| t != tag);
        self.log_action("TAG_REMOVED", tag);
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
