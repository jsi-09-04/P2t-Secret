# Contexte Actif — P2TSecret

## 1. Focus Actuel
- **Phase** : Initialisation du socle technique, de la gouvernance Agentic Factory et du noyau de sécurité.
- **Objectif immédiat** : Rendre le projet 100% opérationnel au sein de l'Agentic Factory (Studio Orchestrateur, BacklogManager, Wiki) avec le squelette applicatif prêt pour le cycle TDD.

## 2. Décisions Récientes
- **ADR-001** : Initialisation du projet et déploiement de la Memory Bank standardisée. Enregistrement dans les settings MySQL sous la clé `working_directory_p2tsecret`.
- **ADR-002** : Implémentation du standard 2FA **TOTP (RFC 6238)** pour une compatibilité native et universelle avec **Microsoft Authenticator** (ainsi que Google Authenticator).
- **ADR-003** : Chiffrement symétrique **AES-256-GCM** avec vecteur d'initialisation aléatoire 96 bits par secret et dérivation de clé maîtresse par PBKDF2/Argon2id.
- **ADR-004** : Stack technique alliant un backend Python FastAPI (avec Pydantic v2, cryptography, pyotp) et une interface web moderne ultra-soignée avec esthétique cyber-sécurité (dark mode, glassmorphism).

## 3. Prochaines Étapes Immédiates
1. Finalisation des spécifications détaillées dans `memory_bank/specs/` (`SPEC-01-auth-2fa-totp.md`, `SPEC-02-vault-encryption.md`).
2. Implémentation du backend FastAPI avec les modules de sécurité :
   - `backend/core/security.py` : hachage de mot de passe, dérivation de clé, génération de jetons JWT.
   - `backend/core/crypto_vault.py` : moteur AES-256-GCM pour le coffre-fort.
   - `backend/core/totp_service.py` : enrôlement et validation TOTP RFC 6238, génération de QR codes pour Microsoft Authenticator.
3. Tests unitaires et d'intégration validant le flux 2FA et le chiffrement.
4. Interface Web interactive permettant de tester le flux complet d'enrôlement 2FA Microsoft Authenticator et la consultation sécurisée.
