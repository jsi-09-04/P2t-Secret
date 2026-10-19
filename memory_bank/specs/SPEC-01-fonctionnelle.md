# Spécification Fonctionnelle : P2TSecret (Rust + egui)

## 1. Présentation Générale
**P2TSecret** est une application de bureau locale autonome, multi-plateforme (Linux, Windows, macOS), conçue pour sécuriser et gérer l'ensemble des identifiants (identifiants de connexion, mots de passe maîtres, clés d'accès, notes secrètes) avec une exigence de sécurité maximale :
- **Authentification forte à double facteur (2FA)** obligatoire, compatible avec **Microsoft Authenticator** (et Google Authenticator).
- **Zéro dépendance Web** : application native 100% Rust compilée avec `egui`, garantissant une empreinte mémoire minime, l'absence de failles XSS/DOM et une sécurité totale en mémoire (RAM).
- **Chiffrement militaire au repos** : chiffrement AES-256-GCM avec dérivation de clé Argon2id et destruction garantie des secrets en mémoire (`zeroize`).

---

## 2. Personas & Environnements Cibles
- **Systèmes d'exploitation** :
  - **Linux** : Ubuntu, Debian, Fedora, Arch (compatible X11 & Wayland).
  - **Windows** : Windows 10 & 11 (64-bit).
  - **macOS** : Apple Silicon (M1/M2/M3/M4) et Intel x86_64.
- **Utilisateurs cibles** : Ingénieurs, développeurs, administrateurs et collaborateurs de l'écosystème Projet2Team nécessitant un accès sécurisé, hors-ligne et souverain à leurs identifiants.

---

## 3. Parcours Utilisateur & Cas d'Usage Détaillés

### SF-01 : Premier Lancement & Enrôlement 2FA Microsoft Authenticator
1. **Création du Coffre-Fort** :
   - L'utilisateur lance l'exécutable pour la première fois.
   - L'écran d'initialisation invite à définir un **Mot de passe maître** robuste (minimum 10 caractères, vérification de confirmation).
2. **Génération de l'enrôlement 2FA** :
   - L'application génère une clé secrète TOTP aléatoire (160 bits Base32) conforme à la RFC 6238.
   - Affichage immédiat d'un **QR Code vectoriel natif** sur l'interface graphique `egui`.
   - Affichage de la clé textuelle Base32 pour saisie manuelle si besoin.
3. **Scan et Validation** :
   - L'utilisateur ouvre son application **Microsoft Authenticator** sur smartphone, appuie sur `+` (Autre compte) et scanne le QR code.
   - L'utilisateur saisit le code à 6 chiffres affiché par Microsoft Authenticator pour valider la synchronisation d'horloge.
4. **Délivrance des Codes de Secours (Backup Codes)** :
   - 8 codes de secours à usage unique (format `XXXX-XXXX`) sont générés et présentés à l'utilisateur.
   - Bouton de copie / sauvegarde obligatoire avant validation.
   - Le coffre chiffré est initialisé sur le disque.

### SF-02 : Déverrouillage Quotidien du Coffre
1. **Étape 1 : Mot de passe maître** :
   - Saisie du mot de passe maître de l'utilisateur.
2. **Étape 2 : Challenge 2FA Microsoft Authenticator** :
   - Saisie du code dynamique à 6 chiffres fourni par Microsoft Authenticator.
   - Option alternative : "Utiliser un code de secours" en cas de perte ou indisponibilité du smartphone.
3. **Déverrouillage** :
   - Déchiffrement du coffre en mémoire vive.

### SF-03 : Consultation et Gestion des Identifiants (Vault)
1. **Vue Liste / Cartes ergonomiques** :
   - Affichage des entrées avec : Titre du service, URL/Portail, Identifiant/Email, Catégorie, Tags.
   - Le mot de passe reste **masqué par défaut** (`••••••••••••`).
2. **Démasquage temporaire** :
   - Bouton "Afficher/Masquer" (œil) pour voir le mot de passe en clair.
   - Re-masquage automatique après 15 secondes.
3. **Copie Sécurisée dans le Presse-Papier (Clipboard)** :
   - Bouton de copie en 1 clic.
   - Notification visuelle dans l'interface.
   - **Purge automatique du presse-papier** : Le presse-papier du système d'exploitation est automatiquement effacé après 30 secondes pour éviter toute fuite.
4. **Recherche et Filtrage** :
   - Barre de recherche instantanée (filtrage sur titre, login, URL, tags et catégorie).
   - Sélecteur de catégorie (Serveurs, Cloud, Portails, Banques, Outils...).

### SF-04 : Ajout et Modification d'un Identifiant
1. **Formulaire d'entrée** :
   - Titre du compte / service (obligatoire).
   - Identifiant / login / adresse email (obligatoire).
   - Mot de passe secret (obligatoire).
   - URL d'accès (optionnel, avec bouton d'ouverture dans le navigateur par défaut).
   - Catégorie & Tags séparés par des virgules.
   - Notes secrètes (multi-lignes, chiffrées).
2. **Générateur de mot de passe intégré** :
   - Bouton accessible directement à côté du champ mot de passe.
   - Curseur de longueur (8 à 64 caractères).
   - Options : Majuscules, Minuscules, Chiffres, Symboles, Exclusion des caractères ambigus (`0`, `O`, `l`, `1`, `I`).
   - Jauge d'entropie (bits) et appréciation de la robustesse en temps réel.
   - Injection directe dans le formulaire en 1 clic.

### SF-05 : Suppression d'Identifiant
- Demande de confirmation explicite pour éviter toute perte accidentelle.

### SF-06 : Verrouillage Automatique & Sécurité de Session
- Bouton "Verrouiller le coffre" instantané dans la barre d'outils.
- Verrouillage automatique après 10 minutes d'inactivité.
- Lors du verrouillage, toutes les structures mémoire contenant les secrets déchiffrés sont effacées avec `Zeroize`.
