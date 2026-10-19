# Registre des Décisions d'Architecture (ADR) — P2TSecret

## ADR-001 : Initialisation de l'espace de travail et raccordement Agentic Factory
- **Date** : 2026-10-19
- **Statut** : Accepté
- **Contexte** : Démarrage du projet P2TSecret dans l'écosystème P2T Applications avec pilotage par l'Agentic Factory.
- **Décision** :
  - Création du repository local avec branche par défaut `main` et `.gitignore` strict.
  - Raccordement au Cockpit Studio de l'Agentic Factory via le setting MySQL `working_directory_p2tsecret`.
  - Déploiement de la Memory Bank complète (`productContext.md`, `activeContext.md`, `progress.md`, `systemPatterns.md`, `decisionLog.md`, `backlog.json`, `task.md`, `specs/`, `plans/`).
  - Déploiement de la charte de développement `AGENTS.md`.
- **Conséquences** : Le projet est immédiatement reconnu par l'Agentic Factory et prêt pour l'exécution autonome des tâches par les agents.

---

## ADR-002 : Choix du protocole 2FA — TOTP RFC 6238
- **Date** : 2026-10-19
- **Statut** : Accepté
- **Contexte** : L'utilisateur demande une authentification à deux facteurs compatible avec **Microsoft Authenticator** (ou similaire type Google Authenticator).
- **Décision** : Adoption du standard ouvert **TOTP (Time-based One-Time Password)** défini par la RFC 6238, avec un intervalle de temps de 30 secondes, un hachage SHA-1 standard (exigé par Microsoft Authenticator et Google Authenticator pour une compatibilité universelle) et une clé secrète encodée en Base32 (160 bits).
- **Conséquences** :
  - Interopérabilité immédiate avec Microsoft Authenticator, Google Authenticator, Bitwarden, 1Password, Aegis.
  - Génération d'URIs `otpauth://totp/...` sous forme de QR Code scannable directement par les applications mobiles.
  - Zéro dépendance envers une API tierce cloud propriétaire : souveraineté totale et fonctionnement hors-ligne garanti.

---

## ADR-003 : Chiffrement du coffre-fort — AES-256-GCM et dérivation de clé
- **Date** : 2026-10-19
- **Statut** : Accepté
- **Contexte** : Les logins et mots de passe stockés doivent être protégés contre toute extraction directe de la base de données.
- **Décision** :
  - Utilisation de l'algorithme symétrique **AES-256-GCM** (Galois/Counter Mode).
  - Génération d'un vecteur d'initialisation (IV / Nonce) aléatoire de 96 bits pour chaque secret.
  - Intégrité vérifiée grâce au tag d'authentification de 128 bits produit par le mode GCM (évite toute attaque par modification de ciphertext).
  - Dérivation de la clé de chiffrement à partir d'un sel aléatoire cryptographique et d'un mot de passe maître ou d'une clé maîtresse sécurisée.
- **Conséquences** : Confidentialité et intégrité maximales au repos. Même en cas de dump de la BDD, les données sont illisibles.

---

## ADR-004 : Choix initial de prototypage
- **Date** : 2026-10-19
- **Statut** : Remplacé par ADR-005
- **Contexte** : Prototypage rapide du modèle de données et des routes d'authentification.

---

## ADR-005 : Architecture Définitive — 100% Rust + egui (eframe)
- **Date** : 2026-10-19
- **Statut** : Accepté
- **Contexte** : L'application doit s'exécuter nativement et de manière autonome sur les postes de travail (Linux, Windows, macOS). L'utilisation d'une pile Web ou d'un interpréteur Python pose des contraintes de sécurité (impossibilité de garantir le vidage de la RAM, dépendances WebKit/Chromium lourdes, packaging multi-OS complexe).
- **Décision** :
  - Refonte complète en **100% Rust** avec le framework graphique natif **`egui`** (via `eframe`).
  - Utilisation de **`zeroize`** pour l'effacement physique garanti des secrets en mémoire vive.
  - Dérivation de clé par **Argon2id** (`argon2`) et chiffrement **AES-256-GCM** (`aes-gcm`).
  - Implémentation certifiée **2FA TOTP RFC 6238** (`totp-rs`) avec rendu direct du QR Code vectoriel pour **Microsoft Authenticator** via `qrcodegen`.
  - Gestion sécurisée du presse-papier système avec purge automatique via `arboard`.
- **Conséquences** :
  - Binaire autonome ultra-compact (~5 à 8 Mo), démarrage instantané (< 20 ms), zéro dépendance runtime.
  - Sécurité mémoire maximale sans surface d'attaque Web.
  - Portabilité native Linux (X11/Wayland), Windows et macOS.

