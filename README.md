# P2TSecret 🔐 — Application Native Multi-OS (Rust + egui)

> **Gestionnaire de logins et mots de passe ultra-sécurisé avec 2FA Microsoft Authenticator.**
> Application de bureau 100% native Rust (Linux, Windows, macOS), zéro dépendance Web, zéro runtime.

---

## 🌟 Atouts & Sécurité Militaire

- 🛡️ **Double Facteur 2FA Microsoft Authenticator (RFC 6238)** : Enrôlement instantané via QR Code vectoriel natif, validation de code à 6 chiffres (+/- 1 intervalle de 30s) et codes de secours à usage unique.
- 🔒 **Chiffrement Authentifié AES-256-GCM** : Tout le coffre est chiffré avec AES-256-GCM, un nonce unique de 96 bits par sauvegarde et un tag d'intégrité de 128 bits.
- 🧠 **Dérivation de Clé Argon2id** : Protection maximale contre les attaques matérielles (GPU/ASIC) avec sel cryptographique de 16 octets.
- 🧹 **Sécurité de la RAM (Zeroize-on-Drop)** : Les structures contenant les clés maîtres et mots de passe sont physiquement écrasées par des zéros en mémoire vive dès leur destruction.
- ⚡ **Zéro Surface d'Attaque Web** : Interface graphique native en mode immédiat avec `egui` (rendu GPU 60 FPS, pas de WebView, pas de XSS, binaire compact).
- 📋 **Presse-Papier Sécurisé Multi-OS** : Copie en 1 clic avec purge et effacement automatique du presse-papier après 30 secondes.
- 🎲 **Générateur de Mots de Passe Forts** : Entropie cryptographique en temps réel, curseur de longueur et options de complexité.

---

## 📂 Architecture du Projet

```
.
├── Cargo.toml                    # Dépendances Rust (eframe, egui, aes-gcm, argon2, zeroize, totp-rs)
├── AGENTS.md                     # Directives de développement de l'Agentic Factory
├── README.md                     # Présentation de l'application
├── memory_bank/                  # Référentiel de vérité de l'Agentic Factory
│   ├── productContext.md         # Vision et cas d'usage
│   ├── activeContext.md          # Focus actuel et historique
│   ├── progress.md               # Avancement réel du projet
│   ├── systemPatterns.md         # Patterns cryptographiques et machine à états egui
│   ├── decisionLog.md            # Registre ADR (ADR-005 : Architecture 100% Rust + egui)
│   ├── backlog.json              # Source de vérité du backlog synchronisé
│   ├── task.md                   # Rendu Markdown du backlog
│   └── specs/                    # Spécifications détaillées
│       ├── SPEC-01-fonctionnelle.md
│       └── SPEC-02-technique-architecture.md
├── src/
│   ├── lib.rs                    # Exposition des modules crypto, storage, ui
│   ├── main.rs                   # Point d'entrée natif eframe::run_native
│   ├── crypto/                   # Argon2id, AES-GCM, TOTP RFC 6238, QR code matrix, générateur
│   ├── storage/                  # Modèle Vault et persistance atomique P2TV01
│   └── ui/                       # Vues egui, thème cyber sombre, QR widget, clipboard
└── tests/                        # Tests d'intégration et unitaires Rust
    ├── crypto_tests.rs
    └── vault_storage_tests.rs
```

---

## 🚀 Lancement & Compilation

### Prérequis
- Compilateur Rust (`cargo` / `rustc` 1.80+)

### 1. Lancer les tests automatisés
```bash
cargo test
```

### 2. Démarrer l'application en mode développement
```bash
cargo run
```

### 3. Compiler pour la production (Binaire optimisé et allégé)
```bash
cargo build --release
```
L'exécutable optimisé se trouve dans `target/release/p2tsecret`.
