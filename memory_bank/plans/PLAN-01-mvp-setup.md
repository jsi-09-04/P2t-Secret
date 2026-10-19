# Plan d'Implémentation Technique : PLAN-01 — Socle de Sécurité & MVP P2TSecret

## 1. Contexte & Dépendances
- **Projet** : P2TSecret
- **Spec associée** : [SPEC-01](file:///Users/james/02-Interne/P2T%20Applications/P2TSecret/memory_bank/specs/SPEC-01-auth-2fa-totp.md) & [SPEC-02](file:///Users/james/02-Interne/P2T%20Applications/P2TSecret/memory_bank/specs/SPEC-02-vault-encryption.md)
- **Objectif** : Initialisation complète de l'application sécurisée avec backend FastAPI, modules cryptographiques, enrôlement 2FA Microsoft Authenticator, coffre-fort d'identifiants et interface Web moderne.

## 2. Découpage Technique

### Phase 1 : Socle Technique & Cryptographie (EPIC-01 & EPIC-02)
- [x] Initialisation du dépôt Git, configuration `.gitignore` et déclaration dans l'Agentic Factory.
- [x] Spécifications techniques et Memory Bank complète.
- [ ] Module `backend/core/security.py` : hachage de mot de passe maître (PBKDF2-HMAC-SHA256), génération de tokens JWT signés.
- [ ] Module `backend/core/totp_service.py` : génération Base32, URI `otpauth://`, validation TOTP RFC 6238, QR code SVG natif.
- [ ] Module `backend/core/crypto_vault.py` : chiffrement et déchiffrement AES-256-GCM avec IV/Nonce de 96 bits et vérification du tag d'intégrité.
- [ ] Modélisation et persistance des utilisateurs et des entrées de coffre dans SQLite chiffrable (`backend/models/database.py`).

### Phase 2 : API REST FastAPI
- [ ] Routeur `backend/routers/auth.py` :
  - `POST /register`
  - `POST /login`
  - `POST /2fa/setup` (génère le QR Code Microsoft Authenticator)
  - `POST /2fa/confirm`
  - `POST /2fa/verify`
- [ ] Routeur `backend/routers/vault.py` :
  - `GET /vault` (liste avec métadonnées masquées)
  - `POST /vault` (ajout chiffré)
  - `GET /vault/{id}` (déchiffrement sécurisé)
  - `PUT /vault/{id}` (mise à jour)
  - `DELETE /vault/{id}` (suppression)
  - `POST /vault/generate-password` (générateur de mot de passe aléatoire cryptographique)

### Phase 3 : Tests Automatisés (TDD)
- [ ] `tests/test_crypto.py` : test d'intégrité AES-256-GCM, rejet des ciphertexts altérés.
- [ ] `tests/test_totp.py` : test de génération TOTP, synchronisation avec Microsoft Authenticator, rejet des codes expirés ou rejoués.
- [ ] `tests/test_api_auth.py` : cycle complet inscription -> 2FA setup -> vérification -> connexion.
- [ ] `tests/test_api_vault.py` : cycle CRUD sur les identifiants chiffrés.

### Phase 4 : Interface Web Moderne
- [ ] Design System Vanilla CSS (dark mode haute fidélité, accents cyber-cyan/violet, glassmorphism, responsive).
- [ ] Écran d'authentification et modal 2FA avec scan direct Microsoft Authenticator.
- [ ] Tableau de bord avec recherche, démasquage temporaire, bouton copier avec auto-effacement, et modale d'ajout/génération de secret.
