# AGENTS.md — Directives de Développement (P2TSecret)

## À propos de ce document
Ce fichier documente la structure, l'architecture et les directives pour les agents de développement (LLM CLI / IDE / Agentic Factory) travaillant sur le projet **P2TSecret**.

**P2TSecret** est une application sécurisée de gestion des identifiants (logins et mots de passe), dotée d'une authentification renforcée en double facteur (2FA TOTP compatible Microsoft Authenticator), d'un chiffrement militaire (AES-256-GCM / Argon2id) et d'un coffre-fort hautement confidentiel.

---

## 📂 Structure du projet & Memory Bank

```
.
├── memory_bank/                  # Référentiel de vérité et contexte du projet
│   ├── productContext.md         # Raison d'être, vision produit et cas d'usage
│   ├── activeContext.md          # Focus actuel, décisions récentes, prochaine étape
│   ├── progress.md               # État d'avancement réel et reste à faire
│   ├── systemPatterns.md         # Architecture logicielle et patterns cryptographiques
│   ├── decisionLog.md            # Registre des décisions d'architecture (ADR)
│   ├── backlog.json              # SOURCE DE VÉRITÉ du backlog (task.md n'en est qu'un rendu)
│   ├── task.md                   # Rendu lecture seule de backlog.json
│   ├── import/                   # Scripts et données de migration (CSV)
│   ├── plans/                    # Plans d'implémentation techniques
│   └── specs/                    # Spécifications fonctionnelles et d'architecture
├── src/                          # Code source natif Rust
│   ├── crypto/                   # Argon2id, AES-256-GCM, TOTP RFC 6238, Générateur
│   ├── storage/                  # Modèle Vault, format P2TV01 et import CSV universel
│   ├── ui/                       # Interface native egui / eframe (thème clair moderne, modales)
│   ├── lib.rs                    # Export des modules de la librairie
│   └── main.rs                   # Point d'entrée de l'application desktop
├── tests/                        # Tests d'intégration et de sécurité Rust (TDD)
│   ├── crypto_tests.rs           # Tests Argon2id, AES-GCM, TOTP, entropie
│   └── vault_storage_tests.rs    # Tests sauvegarde atomique et intégrité
├── scripts/                      # Scripts d'automatisation
│   └── build_app.sh              # Création du bundle macOS P2TSecret.app
├── data/                         # Dossier local de stockage du coffre chiffré (p2t_vault.enc)
├── Cargo.toml                    # Définition du projet et dépendances Rust
├── AGENTS.md                     # Ce document de référence
└── README.md
```

---

## 🤖 Directives des Agents de Développement

### Principes Fondamentaux
1. **Spec-Driven Development** : Toute implémentation s'appuie sur une spécification claire et validée dans `memory_bank/specs/`.
2. **Cycle TDD (Test-Driven Development)** : Les tests de sécurité et fonctionnels sont écrits et validés à chaque étape. Zéro régression tolérée.
3. **Maintien de la Memory Bank** : Les agents mettent à jour `task.md` (cases `[x]`), `backlog.json` et `activeContext.md` au fil de leurs avancées.
4. **Sécurité par Conception (Security by Design)** :
   - Aucun mot de passe en clair en base de données ni dans les logs.
   - Chiffrement symétrique AES-256-GCM avec sel (salt) et vecteur d'initialisation (IV) uniques par entrée.
   - TOTP conforme RFC 6238 (fenêtre de validité stricte, protection contre le rejeu).
   - Codes de secours (Recovery Codes) hachés.
   - Protection contre les attaques par force brute (rate limiting, temporisation).

### Rôle des Agents
- **Orchestrator** : Analyse le besoin, découpe les Epics et élabore les plans techniques.
- **Coder / Implementer** : Écrit le code conformément aux plans validés.
- **Test Writer** : Rédige la couverture de tests unitaires et cryptographiques.
- **Validator** : Valide la conformité du code, l'absence de vulnérabilités et le respect des standards de sécurité.

---

## 🚀 Synchronisation avec l'Agentic Factory
Ce projet est synchronisé avec le **Cockpit Studio Orchestrateur** de l'Agentic Factory. Chaque modification du backlog ou des spécifications est répercutée en temps réel.
