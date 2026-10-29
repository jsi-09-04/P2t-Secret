# Contexte Actif — P2TSecret (Rust + egui)

## 1. Focus Actuel
- **Phase** : Finalisation de l'application native desktop 100% Rust + egui (eframe 0.30).
- **Objectif immédiat** : Expérience utilisateur (UX/UI) haut de gamme en **thème clair moderne**, haute lisibilité, organisation aérée des écrans et boutons confortables.

## 2. Décisions Récientes
- **ADR-005** : Architecture 100% Rust native avec **egui / eframe 0.30** : aucun webview, binaire statique portable multi-OS (macOS Sonoma/Sequoia, Windows, Linux).
- **ADR-006** : Refonte UX/UI thème clair moderne (palette contrastée `#f8fafc` / `#0f172a` / `#2563eb`), typographie proportionnelle adaptée, barre latérale (Sidebar) de navigation par catégories et boutons d'action agrandis (42-46px pour les actions principales, 32-34px pour les actions secondaires/cartes).
- **Cryptographie** : Argon2id, AES-256-GCM, Zeroize (destruction immédiate en mémoire), TOTP RFC 6238 Microsoft Authenticator avec QR code vectoriel natif, sauvegarde atomique `P2TV01`.

## 3. État d'Avancement & Tests
- **Compilation** : `cargo check` et `cargo build` terminés avec succès (code 0, 0 warning).
- **Tests unitaires et d'intégration** : 8/8 tests réussis (`crypto_tests`, `vault_storage_tests`, `csv_import_tests`).
- **Import CSV & Migration** :
  - Analyse complète du fichier `Liste des comptes de messagerie_v3_2023.xlsx` (10 feuilles analysées).
  - Génération du fichier CSV standardisé : `memory_bank/import/comptes_p2tsecret.csv` (198 identifiants extraits, catégorisés et nettoyés).
  - Module Rust d'importation universelle (`src/storage/csv_import.rs`) avec détection automatique du délimiteur (`,` ou `;`), réconciliation de colonnes et gestion des doublons.
- **Personnalisation & Paramètres** :
  - Gestion dynamique des catégories et des tags personnalisés (`custom_categories` et `custom_tags` dans `VaultData`).
  - Modale native `⚙️ Paramètres & Personnalisation` avec gestion des catégories, nuage de tags globaux, et statistiques de sécurité.
  - Sélecteur de catégorie dynamique `ComboBox` et suggestions de tags en un clic lors de l'ajout/édition d'un identifiant.
- **Changement sécurisé du mot de passe maître** :
  - Vérification du mot de passe maître actuel, validation de longueur min. 8 caractères.
  - Re-dérivation complète de la clé de chiffrement via Argon2id.
  - Re-chiffrement AES-256-GCM et écriture atomique sur le disque (`P2TV01`).
  - Journalisation de l'événement d'audit `MASTER_PASSWORD_CHANGED`.

