# Git & GitHub — Guide de commit et d'authentification

Ce document explique comment committer et pousser ses changements sur le dépôt
GitHub du projet, et comment reproduire la même configuration sur un autre
projet (par exemple **p2tsecret**).

---

## 1. Configuration actuelle du dépôt

| Élément | Valeur |
|---|---|
| Dépôt distant (`origin`) | `https://github.com/projet2team/FondationLedoux.git` |
| Branche par défaut | `main` |
| Protocole | HTTPS |
| Gestionnaire d'identifiants (credential helper) | `osxkeychain` (Trousseau macOS) |
| Clé SSH | Aucune configurée pour l'instant |

L'authentification se fait donc via un **jeton personnel GitHub (PAT)** stocké
dans le trousseau macOS, et non via une clé SSH.

---

## 2. Authentification « login » GitHub

Le projet n'utilisant pas de clé SSH, l'accès se fait par jeton personnel
(Personal Access Token). Voici la marche à suivre, **identique pour `p2tsecret`**.

### Étape 1 — Créer un jeton sur GitHub

1. Se connecter sur https://github.com → **Settings** (profil, en haut à droite).
2. Bas-vers-le-bas → **Developer settings** → **Personal access tokens** →
   **Tokens (classic)** ou **Fine-grained tokens**.
3. Générer un **token (classic)**.
4. Scopes nécessaires pour pousser du code :
   - `repo` (accès complet aux dépôts privés)
   - `workflow` (CI/CD si utilisé)
5. Copier le jeton généré (`ghp_...`). Il n'apparaîtra **qu'une seule fois**.

### Étape 2 — Configurer son identité Git (une seule fois, au niveau global)

```bash
git config --global user.name "James SIMON"
git config --global user.email "james.simon@projet2team.fr"
```

> Remplace par ton nom et ton email réels. Ces infos apparaîtront sur chaque commit.

### Étape 3 — Se connecter au trousseau macOS

La première fois que tu pousses (`git push`), Git te demandera un identifiant :

- **Username** → ton nom d'utilisateur GitHub (ex: `projet2team` ou ton pseudo).
- **Password** → **COLLE LE JETON** (`ghp_...`), **PAS** ton mot de passe GitHub.

Le jeton est alors mémorisé automatiquement dans le trousseau macOS grâce au
`credential helper = osxkeychain`. Les prochains `git push` se feront sans
demande d'identifiant.

### Vérifier que l'authentification fonctionne

```bash
git ls-remote origin
```

Si la liste des branches s'affiche sans demander de mot de passe, c'est bon.

---

## 3. Reproduire la configuration sur `p2tsecret`

Sur le nouveau projet (dépôt cloné ou à cloner), **seules les étapes 1 et 4 sont
nécessaires** : l'identité globale (étape 2) et le jeton dans le trousseau
(étape 3) sont partagés sur toute la machine.

```bash
# 1. Cloner le dépôt p2tsecret
git clone https://github.com/jsi-09-04/P2t-Secret.git
.git
cd p2tsecret

# 2. Vérifier la distant et se connecter à la première poussée
git remote -v
git push origin main        # <- demandera username + le jeton (mot de passe)
```

Le jeton saisi sera stocké dans le trousseau et réutilisé pour tous les dépôts.

---

## 4. Faire un commit (workflow du projet)

Le projet suit la **boucle de développement sécurisée** décrite dans `AGENTS.md`.
Avant tout commit, s'assurer que :

- [ ] Le code respecte le typage strict (`declare(strict_types=1);`).
- [ ] Les SQL sont préparées (PDO).
- [ ] Les sorties HTML passent par la fonction `h()`.
- [ ] Les tests passent : `./vendor/bin/phpunit`.
- [ ] Le plan dans `memory_bank/plans/` est à jour (cases cochées, statut OK).

### Commandes

```bash
# 1. Vérifier l'état des modifications
git status

# 2. Ajouter les fichiers modifiés (jamais de secrets/clés)
git add app/ tests/ memory_bank/plans/ CHANGELOG.md

# 3. Committer avec une message « Conventional Commits » (en français)
git commit -m "feat: description courte et précise du changement"

# 4. Pousser sur la branche distante
git push origin main
```

### Convention de messages (utilisée dans le projet)

| Préfixe | Usage | Exemple |
|---|---|---|
| `feat:` | Nouvelle fonctionnalité | `feat(2fa): fiabilisation du workflow 2FA` |
| `fix:` | Correction de bug | `fix(finances): conversion du statut en varchar` |
| `docs:` | Documentation | `docs: guide de commit GitHub` |
| `refactor:` | Refactor sans changer le comportement | `refactor(router): nettoyage redirect()` |
| `test:` | Ajout/modification de tests | `test: couverture auth rate limiting` |
| `chore:` | Tâches techniques / deps | `chore: mise à jour composer` |

Garder les messages **en français**, au présent, concis et descriptifs.

---

## 5. Bonnes pratiques & sécurité

- **Ne jamais committer** de secrets, clés API, mots de passe ou jetons.
- Vérifier `git status` / `git diff` avant `git add` pour ne pas ajouter de
  fichiers sensibles (`.env`, bases de données, logs).
- Un commit ne doit contenir qu'une modification cohérente et testée.
- Pousser régulièrement sur `main` avec des messages explicites.
