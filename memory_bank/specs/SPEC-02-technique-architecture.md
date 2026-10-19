# Spécification Technique & Architecture : P2TSecret (Rust + egui)

## 1. Principes Directeurs
1. **Zéro Runtime Externe** : Compilé en binaire statique natif unique, sans dépendance WebKit ni interpréteur.
2. **Sécurité Mémoire Stricte (Zeroize-on-Drop)** : Les structures contenant les clés maîtres et mots de passe implémentent le trait `zeroize::Zeroize` et `zeroize::ZeroizeOnDrop`, garantissant la réécriture physique par des zéros en RAM dès leur libération.
3. **Format de Stockage Sécurisé & Atomique** : Le fichier du coffre-fort (`vault.p2t` ou `p2tsecret.vault`) est chiffré dans son intégralité avec AES-256-GCM. Chaque enregistrement d'écriture passe par un fichier temporaire avant renommage atomique pour prévenir toute corruption en cas de coupure de courant.
4. **Standard 2FA TOTP RFC 6238** : Clé secrète 160 bits (Base32), intervalle 30 secondes, fonction HMAC-SHA1 pour une compatibilité native et universelle avec **Microsoft Authenticator**.

---

## 2. Pile Technologique & Écosystème Rust

| Composant | Crate Rust | Version / Rôle |
|---|---|---|
| **Moteur Graphique UI** | `eframe` / `egui` | 0.27+ : Framework graphique en mode immédiat (Immediate Mode), 100% Rust, multi-plateforme. |
| **Chiffrement Symétrique** | `aes-gcm` | AES-256 en mode Galois/Counter Mode (AEAD : Confidentialité + Intégrité). |
| **Dérivation de Clé (KDF)** | `argon2` | Argon2id avec sel de 16 octets (recommandations OWASP contre les attaques GPU/ASIC). |
| **2FA Microsoft Authenticator** | `totp-rs` | Implémentation certifiée RFC 6238, génération d'URIs `otpauth://` et vérification de fenêtre temporelle (+/- 1 pas de 30s). |
| **Rendu QR Code** | `qrcodegen` | Génération de la matrice QR code convertie en pixels natifs pour texture `egui::ColorImage`. |
| **Sécurité Mémoire** | `zeroize` | Effacement cryptographique déterministe de la mémoire RAM (`ZeroizeOnDrop`). |
| **Sérialisation** | `serde`, `serde_json` | Encodage des structures internes du coffre avant chiffrement. |
| **Génération Aléatoire** | `rand`, `rand_core` | Générateur cryptographiquement sûr (`OsRng`). |
| **Presse-Papier Multi-OS** | `arboard` | Gestionnaire natif du clipboard pour Linux (X11/Wayland), Windows et macOS avec thread de temporisation d'effacement. |

---

## 3. Architecture Logicielle & Modules

```
p2tsecret/
├── Cargo.toml                    # Dépendances et métadonnées du package Rust
├── memory_bank/                  # Référentiel de gouvernance Agentic Factory
├── src/
│   ├── main.rs                   # Initialisation eframe, configuration des fenêtres et boucle d'événements
│   ├── crypto/                   # Noyau Cryptographique & Sécurité
│   │   ├── mod.rs
│   │   ├── kdf.rs                # Dérivation Argon2id (Master Key depuis Master Password + Salt)
│   │   ├── cipher.rs             # Chiffrement / Déchiffrement AES-256-GCM avec Zeroize
│   │   ├── totp.rs               # Gestion 2FA Microsoft Authenticator, QR Code & Backup Codes
│   │   └── generator.rs          # Générateur cryptographique de mots de passe et calcul d'entropie
│   ├── storage/                  # Persistance & Gestion du Coffre
│   │   ├── mod.rs
│   │   ├── model.rs              # Modèles de données (Vault, CredentialEntry, Category, AuditRecord)
│   │   └── file_store.rs         # Chargement, sauvegarde atomique et intégrité du fichier vault
│   └── ui/                       # Interface Utilisateur Native egui
│       ├── mod.rs
│       ├── app.rs                # Machine à états globale (AppState, transition d'écrans)
│       ├── theme.rs              # Palette cyber sombre (accents cyan néon, surfaces contrastées)
│       ├── views_init.rs         # Premier lancement : mot de passe maître & QR code 2FA
│       ├── views_unlock.rs       # Écran de déverrouillage : mot de passe + challenge TOTP 2FA
│       ├── views_vault.rs        # Tableau de bord principal, liste des identifiants et filtres
│       └── views_modal.rs        # Modales : nouvel identifiant, éditeur, générateur et codes de secours
└── tests/                        # Tests unitaires et d'intégration Rust
    ├── crypto_tests.rs
    ├── totp_tests.rs
    └── vault_storage_tests.rs
```

---

## 4. Modèle Cryptographique & En-Tête du Fichier Coffre

Le fichier de stockage (ex: `~/.p2tsecret/vault.dat`) est structuré comme suit :

```
+-----------------------------------------------------------------------------------+
| Magic Header "P2TV01" (6 octets)                                                  |
+-----------------------------------------------------------------------------------+
| Sel Argon2id (16 octets aléatoires générés à l'initialisation)                     |
+-----------------------------------------------------------------------------------+
| Nonce / IV AES-256-GCM (12 octets aléatoires à chaque sauvegarde)                  |
+-----------------------------------------------------------------------------------+
| Tag d'Authentification AES-GCM (16 octets pour validation d'intégrité)            |
+-----------------------------------------------------------------------------------+
| Données Chiffrées (JSON sérialisé des entrées + métadonnées TOTP)                 |
+-----------------------------------------------------------------------------------+
```

### Processus de Chiffrement :
1. Dérivation de la clé maîtresse AES-256 :
   $$\text{MasterKey} = \text{Argon2id}(\text{MasterPassword}, \text{Salt}, \text{m\_cost}=65536, \text{t\_cost}=3, \text{p\_cost}=4)$$
2. Chiffrement de la charge utile (JSON du coffre) avec AES-256-GCM et le Nonce unique.
3. Écriture atomique sur le disque.

### Processus de Déchiffrement & Validation 2FA :
1. L'utilisateur saisit son mot de passe maître.
2. Dérivation de la clé avec le sel extrait du fichier.
3. Déchiffrement AES-256-GCM. Si le mot de passe est faux ou le fichier altéré, la vérification du tag GCM échoue instantanément (`TagMismatch`).
4. Si le déchiffrement réussit, le secret TOTP stocké dans les métadonnées chiffrées est utilisé pour valider le code à 6 chiffres de **Microsoft Authenticator**.
5. Une fois le code validé, l'accès au tableau de bord des identifiants est déverrouillé.

---

## 5. Gestion Sécurisée du Presse-Papier
- Lorsqu'un utilisateur clique sur "Copier", la chaîne est copiée dans le presse-papier via `arboard`.
- Un thread asynchrone en arrière-plan lance un compte à rebours de 30 secondes.
- À l'expiration des 30 secondes, si le presse-papier contient toujours le mot de passe copié, il est écrasé par une chaîne vide.
