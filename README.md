# P2TSecret 🔐 — Gestionnaire d'Identifiants Native Multi-OS (Rust + egui)

> **Gestionnaire de logins et mots de passe ultra-sécurisé avec authentification double facteur (2FA) Microsoft Authenticator.**
> Application de bureau 100% native Rust (Linux, Windows, macOS) : zéro dépendance Web, zéro WebView, zéro runtime externe.

---

## 📋 Table des matières

- [Fonctionnalités](#fonctionnalités)
- [Initialisation & première utilisation](#initialisation--première-utilisation)
- [Import d'identifiants (CSV)](#import-didentifiants-csv)
- [Architecture du projet](#architecture-du-projet)
- [Lancement & compilation](#lancement--compilation)
- [Développement](#développement)
- [Licence](#licence)

---

## 🌟 Fonctionnalités

### 🔒 Sécurité & Chiffrement
- **Chiffrement authentifié AES-256-GCM** : tout le coffre est chiffré avec un nonce (IV) unique par sauvegarde et un tag d'intégrité de 128 bits — toute altération est détectée.
- **Dérivation de clé Argon2id** : le mot de passe maître génère la clé via Argon2id (mémoire-sensible, résistant aux GPU/ASIC), avec sel unique par sauvegarde.
- **Zéro mot de passe en clair** : aucun identifiant n'est stocké ou logué en clair.
- **Sécurité de la RAM (Zeroize-on-Drop)** : les structures contenant les clés maîtresses et mots de passe sont écrasées par des zéros en mémoire vive dès leur destruction.
- **Format de fichier fermé `P2TV01`** : format de stockage propriétaire, chiffré de bout en bout.

### 🔐 Authentification Double Facteur (2FA)
- **TOTP RFC 6238** compatible **Microsoft Authenticator** / Google Authenticator.
- Enrôlement instantané via **QR code vectoriel natif** ou saisie manuelle de la clé secrète Base32.
- Validation d'un code à **6 chiffres** (fenêtre de ±1 intervalle de 30 s) avec protection contre le rejeu.
- **Codes de secours à usage unique** générés et téléchargeurs à l'enrôlement.

### 🗂️ Gestion du Coffre-fort
- Ajout, modification, consultation et archivage d'identifiants (nom, login, mot de passe, URL, notes secrètes, tags/catégories).
- **Mots de passe masqués** par défaut, démasquables à la demande ou copiables en 1 clic.
- **Presse-papier sécurisé multi-OS** : copie en 1 clic avec purge et effacement automatique après 30 secondes.
- Tri et filtrage par catégorie/tags, recherche instantanée.

### 🎲 Générateur de Mots de Passe
- Génération paramétrable (longueur, minuscules/majuscules, chiffres, caractères spéciaux).
- Évaluation en temps réel de l'**entropie** et de la robustesse.

### 📊 Audit & Tracabilité
- Journalisation des consultations, tentatives de connexion et modifications d'identifiants.
- Détection des tentatives suspectes (brute-force).

### ⚡ Expérience
- Interface native en **mode immédiat avec `egui`** (rendu GPU ~60 FPS, thème cyber sombre, modales fluides).
- Binaire compact et autonome, aucune installation système requise.

---

## 🔐 Initialisation & première utilisation

### Prérequis
- Compilateur Rust : `cargo` / `rustc` **1.80+** ([installateur Rust](https://www.rust-lang.org/tools/install)).
- Sur **Linux**, les bibliothèques système pour le rendu graphique (GL/XCB) doivent être présentes (fournies par défaut sur la plupart des distributions graphiques).

### 1. Compiler et lancer l'application
```bash
# Développement
cargo run

# Binaire de production optimisé
cargo build --release
./target/release/p2tsecret
```
La première ouverture **crée automatiquement le coffre-fort** et lance l'assistant d'initialisation.

### 2. Assistant d'initialisation (première ouverture)
L'application guide l'utilisateur à travers un enrôlement en 4 étapes :

1. **Mot de passe maître** — Choisir un mot de passe maître (minimum 8 caractères). Il chifffre l'intégralité du coffre ; il est **impossible de récupérer un compte sans lui**.
2. **Enrôlement 2FA** — Scanner le QR code affiché avec **Microsoft Authenticator** (ou saisir la clé Base32 manuellement), puis valider le code à 6 chiffres généré.
3. **Codes de secours** — Télécharger et **sauvegarder impérativement** les codes de secours à usage unique (permettent l'accès en cas de perte de l'appareil 2FA).
4. **Création du coffre** — Le fichier de coffre chiffré est initialisé.

### Où sont stockés les données ?
Le coffre-fort chiffré est enregistré de façon atomique à l'emplacement suivant :

```
$HOME/.p2tsecret/vault.p2t      (format P2TV01)
```

> 💡 **Migration / Sauvegarde** : pour transférer votre coffre vers une autre machine, copiez simplement le fichier `vault.p2t` (ou tout le dossier `.p2tsecret`) vers le même emplacement. Le mot de passe maître et le 2FA sont les seuls éléments hors-ligne nécessaires.

### Utilisation ultérieure (déverrouillage)
À chaque lancement, deux étapes suffisent :
1. Saisie du **mot de passe maître**.
2. Saisie du **code 2FA** (ou d'un **code de secours**) — puis accès au tableau de bord.

---

## 📥 Import d'identifiants (CSV)

Un importeur CSV natif permet d'intégrer rapidement un lot d'identifiants depuis la modal **« 📥 Importer des identifiants »**.

**Format standard pris en charge** (correspondance par position des colonnes) :

| Colonne 1 | Colonne 2 | Colonne 3 |
|-----------|-----------|-----------|
| Nom de l'application | Login | Mot de passe |

- Délimiteur **automatiquement détecté** : virgule `,` ou point-virgule `;`.
- Une **ligne d'en-tête** (ex. `nom,login,mot de passe`) est automatiquement ignorée.
- Les lignes incomplètes (nom ou mot de passe manquant) sont filtrées.

Exemple de fichier CSV compatible :

```csv
Google,admin@example.com,Secr3t!
OVH,host,P@ssw0rd
```

---

## 📂 Architecture du projet

```
.
├── Cargo.toml                    # Dépendances Rust (eframe, egui, aes-gcm, argon2, zeroize, totp-rs, csv...)
├── AGENTS.md                     # Directives de développement de l'Agentic Factory
├── README.md                     # Présentation de l'application
├── LICENSE                       # License Apache 2.0
├── memory_bank/                  # Référentiel de vérité de l'Agentic Factory
│   ├── productContext.md         # Vision et cas d'usage
│   ├── activeContext.md          # Focus actuel et historique
│   ├── progress.md               # Avancement réel du projet
│   ├── systemPatterns.md         # Patterns cryptographiques et machine à états egui
│   ├── decisionLog.md            # Registre ADR (ADR-005 : Architecture 100% Rust + egui)
│   ├── backlog.json              # Source de vérité du backlog synchronisé
│   ├── task.md                   # Rendu Markdown du backlog
│   ├── specs/                    # Spécifications détaillées
│   │   ├── SPEC-01-fonctionnelle.md
│   │   ├── SPEC-01-auth-2fa-totp.md
│   │   ├── SPEC-02-technique-architecture.md
│   │   └── SPEC-02-vault-encryption.md
│   └── import/                   # Scripts et données de migration (usage local)
├── src/
│   ├── main.rs                   # Point d'entrée natif eframe::run_native
│   ├── crypto/                   # Argon2id, AES-GCM, TOTP RFC 6238, QR code matrix, générateur
│   ├── storage/                  # Modèle Vault, format P2TV01, persistance atomique, import CSV
│   │   ├── model.rs              #   Structure sérialisable du coffre (Vault, CredentialEntry)
│   │   ├── file_store.rs         #   Chargement/sauvegarde chiffré (MAGIC_HEADER "P2TV01")
│   │   └── csv_import.rs         #   Importeur CSV standard (nom, login, mot de passe)
│   └── ui/                       # Vues egui, thème cyber sombre, QR widget, clipboard
├── tests/                        # Tests d'intégration et unitaires Rust
│   ├── crypto_tests.rs           #   Argon2id, AES-GCM, TOTP, entropie
│   └── vault_storage_tests.rs    #   Sauvegarde atomique et intégrité
└── scripts/                      # Scripts de build du bundle macOS
    ├── build_app.sh              #   Création de P2TSecret.app
    └── build_dmg.sh              #   Création de P2TSecret.dmg
```

---

## 🚀 Lancement & compilation

### 1. Lancer les tests automatisés
```bash
cargo test
```

### 2. Démarrer l'application en mode développement
```bash
cargo run
```

### 3. Compiler pour la production (binaire optimisé et allégé)
```bash
cargo build --release
```
L'exécutable optimisé se trouve dans `target/release/p2tsecret`.

### 4. Générer l'application macOS (.app)
```bash
./scripts/build_app.sh
```
L'application autonome `target/release/P2TSecret.app` est prête à être ouverte ou glissée dans `/Applications`.

### 5. Générer l'image disque d'installation macOS (.dmg)
```bash
./scripts/build_dmg.sh
```
L'image disque `target/release/P2TSecret.dmg` permet d'installer l'application d'un simple glisser-déposer vers `Applications`.

---

## 🧪 Développement

Le projet suit une approche **TDD** (Test-Driven Development) et respecte le cycle de vie de l'Agentic Factory (`memory_bank/`).

```bash
# Lancer l'ensemble des tests (crypto, stockage, import CSV)
cargo test

# Vérifier la compilation sans exécuter
cargo build
```

Directives clés : zéro régression sur les tests de sécurité, aucun secret en clair dans le code ou les logs, et toute nouvelle fonctionnalité est couverte par au moins un test unitaire.

---

## 📄 Licence

Sous **Apache License 2.0** — voir [`LICENSE`](LICENSE).
