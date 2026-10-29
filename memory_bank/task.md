# Backlog & Suivi Global — P2TSecret

> ⚠️ **Ce fichier est généré automatiquement depuis `backlog.json`. Ne pas modifier manuellement.**
> Dernière synchronisation : 2026-10-19 21:31:10 | Progression : **100%** (16/16 tâches)

---

## Sommaire

| Epic | Nom | Statut | Tickets | Tâches | Progression |
|------|-----|--------|---------|--------|-------------|
| **EPIC-01** | Cadrage, Spécifications & Architecture Rust + egui | ✅ `completed` | 1 | 3 | 100% |
| **EPIC-02** | Noyau Cryptographique Rust & Sécurité Mémoire | ✅ `completed` | 1 | 4 | 100% |
| **EPIC-03** | Format de Données & Stockage Atomique du Coffre | ✅ `completed` | 1 | 2 | 100% |
| **EPIC-04** | Interface Graphique Native egui / eframe | ✅ `completed` | 1 | 5 | 100% |
| **EPIC-05** | Suite de Tests Automatisés Rust & Validation TDD | ✅ `completed` | 1 | 2 | 100% |

---

## EPIC-01 — Cadrage, Spécifications & Architecture Rust + egui
*Spécification fonctionnelle et technique pour une application de bureau 100% Rust et egui, multi-OS.*

### 📌 TICKET-EPIC-01-01 : Cadrage, Spécifications & Architecture Rust + egui

- [x] **T-P2TSECRET-EPIC-01-T01** [us] : Initialisation du dépôt, configuration Git et raccordement Agentic Factory
- [x] **T-P2TSECRET-EPIC-01-T02** [us] : Rédaction de la spécification fonctionnelle complète (SF-01 à SF-06)
- [x] **T-P2TSECRET-EPIC-01-T03** [us] : Rédaction de la spécification technique et d'architecture 100% Rust + egui

## EPIC-02 — Noyau Cryptographique Rust & Sécurité Mémoire
*Dérivation Argon2id, chiffrement AES-256-GCM, destruction de la RAM avec Zeroize et TOTP RFC 6238 pour Microsoft Authenticator.*

### 📌 TICKET-EPIC-02-01 : Noyau Cryptographique Rust & Sécurité Mémoire

- [x] **T-P2TSECRET-EPIC-02-T01** [us] : Configuration Cargo.toml avec dépendances Rust sécurisées (aes-gcm, argon2, zeroize, totp-rs, qrcodegen)
- [x] **T-P2TSECRET-EPIC-02-T02** [us] : Module KDF Argon2id & Chiffrement authentifié AES-256-GCM avec Zeroize
- [x] **T-P2TSECRET-EPIC-02-T03** [us] : Module TOTP RFC 6238 Microsoft Authenticator & Génération matrice QR Code
- [x] **T-P2TSECRET-EPIC-02-T04** [us] : Générateur cryptographique de mots de passe aléatoires avec calcul d'entropie

## EPIC-03 — Format de Données & Stockage Atomique du Coffre
*Structure de données Vault, écriture atomique sur disque et protection contre la corruption.*

### 📌 TICKET-EPIC-03-01 : Format de Données & Stockage Atomique du Coffre

- [x] **T-P2TSECRET-EPIC-03-T01** [us] : Modélisation des entrées VaultEntry (Service, URL, Login, Password, Notes, Tags)
- [x] **T-P2TSECRET-EPIC-03-T02** [us] : Gestionnaire de persistance atomique avec en-tête magique P2TV01

## EPIC-04 — Interface Graphique Native egui / eframe
*Application de bureau autonome avec rendu GPU, thème sombre cyber, QR code natif et presse-papier sécurisé.*

### 📌 TICKET-EPIC-04-01 : Interface Graphique Native egui / eframe

- [x] **T-P2TSECRET-EPIC-04-T01** [us] : Point d'entrée eframe::App et gestionnaire d'états d'écrans (Init, Unlock, Dashboard)
- [x] **T-P2TSECRET-EPIC-04-T02** [us] : Composant d'affichage de QR Code natif egui pour Microsoft Authenticator
- [x] **T-P2TSECRET-EPIC-04-T03** [us] : Tableau de bord du coffre : liste filtrable, démasquage temporaire et copie sécurisée avec purge auto
- [x] **T-P2TSECRET-EPIC-04-T04** [us] : Modales d'ajout/édition avec intégration du générateur de mot de passe
- [x] **T-P2TSECRET-EPIC-04-T05** [us] : Moteur d'importation CSV universel (détection délimiteur/colonnes, dédoublonnage) et modale UI

## EPIC-05 — Suite de Tests Automatisés Rust & Validation TDD
*Validation unitaire et d'intégration de la cryptographie, du 2FA et de la persistance.*

### 📌 TICKET-EPIC-05-01 : Suite de Tests Automatisés Rust & Validation TDD

- [x] **T-P2TSECRET-EPIC-05-T01** [us] : Tests unitaires du chiffrement AES-256-GCM, Argon2id et Zeroize
- [x] **T-P2TSECRET-EPIC-05-T02** [us] : Tests d'intégration TOTP RFC 6238, synchronisation Microsoft Authenticator et codes de secours
