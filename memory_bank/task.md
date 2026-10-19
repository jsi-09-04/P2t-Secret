# Backlog & Suivi Global — P2TSecret (Rust + egui)

> ⚠️ **Ce fichier est généré en miroir de `backlog.json`. Source de vérité unique : `memory_bank/backlog.json`.**
> Dernière synchronisation : 2026-10-19 | Progression globale : **100%** (15/15 tâches terminées)

---

## Sommaire des Epics

| Epic | Nom | Statut | Priorité | Tâches | Progression |
|---|---|---|---|---|---|
| **EPIC-01** | Cadrage, Spécifications & Architecture Rust + egui | `completed` | Must | 3 | 100% |
| **EPIC-02** | Noyau Cryptographique Rust & Sécurité Mémoire (Zeroize) | `completed` | Must | 4 | 100% |
| **EPIC-03** | Format de Données & Stockage Atomique du Coffre | `completed` | Must | 2 | 100% |
| **EPIC-04** | Interface Graphique Native egui / eframe | `completed` | Must | 4 | 100% |
| **EPIC-05** | Suite de Tests Automatisés Rust & Validation TDD | `completed` | Must | 2 | 100% |

---

## [EPIC-01] Cadrage, Spécifications & Architecture Rust + egui
*Spécification fonctionnelle et technique pour une application de bureau 100% Rust et egui, multi-OS.*
- **Priorité** : `Must`
- **Statut** : `completed`

- [x] **T-P2TSECRET-EPIC-01-T01** - Initialisation du dépôt, configuration Git et raccordement Agentic Factory `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-01-T02** - Rédaction de la spécification fonctionnelle complète (SF-01 à SF-06) `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-01-T03** - Rédaction de la spécification technique et d'architecture 100% Rust + egui `[sdlc]` `[Must]`

---

## [EPIC-02] Noyau Cryptographique Rust & Sécurité Mémoire (Zeroize)
*Dérivation Argon2id, chiffrement AES-256-GCM, destruction de la RAM avec Zeroize et TOTP RFC 6238 pour Microsoft Authenticator.*
- **Priorité** : `Must`
- **Statut** : `completed`

- [x] **T-P2TSECRET-EPIC-02-T01** - Configuration Cargo.toml avec dépendances Rust sécurisées (aes-gcm, argon2, zeroize, totp-rs, qrcodegen) `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-02-T02** - Module KDF Argon2id & Chiffrement authentifié AES-256-GCM avec Zeroize `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-02-T03** - Module TOTP RFC 6238 Microsoft Authenticator & Génération matrice QR Code `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-02-T04** - Générateur cryptographique de mots de passe aléatoires avec calcul d'entropie `[sdlc]` `[Must]`

---

## [EPIC-03] Format de Données & Stockage Atomique du Coffre
*Structure de données Vault, écriture atomique sur disque et protection contre la corruption.*
- **Priorité** : `Must`
- **Statut** : `completed`

- [x] **T-P2TSECRET-EPIC-03-T01** - Modélisation des entrées VaultEntry (Service, URL, Login, Password, Notes, Tags) `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-03-T02** - Gestionnaire de persistance atomique avec en-tête magique P2TV01 `[sdlc]` `[Must]`

---

## [EPIC-04] Interface Graphique Native egui / eframe
*Application de bureau autonome avec rendu GPU, thème sombre cyber, QR code natif et presse-papier sécurisé.*
- **Priorité** : `Must`
- **Statut** : `completed`

- [x] **T-P2TSECRET-EPIC-04-T01** - Point d'entrée eframe::App et gestionnaire d'états d'écrans (Init, Unlock, Dashboard) `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-04-T02** - Composant d'affichage de QR Code natif egui pour Microsoft Authenticator `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-04-T03** - Tableau de bord du coffre : liste filtrable, démasquage temporaire et copie sécurisée avec purge auto `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-04-T04** - Modales d'ajout/édition avec intégration du générateur de mot de passe `[sdlc]` `[Must]`

---

## [EPIC-05] Suite de Tests Automatisés Rust & Validation TDD
*Validation unitaire et d'intégration de la cryptographie, du 2FA et de la persistance.*
- **Priorité** : `Must`
- **Statut** : `completed`

- [x] **T-P2TSECRET-EPIC-05-T01** - Tests unitaires du chiffrement AES-256-GCM, Argon2id et Zeroize `[sdlc]` `[Must]`
- [x] **T-P2TSECRET-EPIC-05-T02** - Tests d'intégration TOTP RFC 6238, synchronisation Microsoft Authenticator et codes de secours `[sdlc]` `[Must]`
