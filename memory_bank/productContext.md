# Contexte Produit — P2TSecret

## 1. Raison d'être & Vision
Dans les organisations et équipes techniques modernes, la gestion des identifiants (comptes administrateurs, consoles cloud, bases de données, clés API, portails partenaires) représente un enjeu de sécurité critique. Les pratiques comme les fichiers texte non protégés, les partages d'identifiants par messagerie non chiffrée ou les mots de passe faibles constituent les vecteurs d'attaque les plus fréquents.

**P2TSecret** a pour vocation de fournir un gestionnaire d'identifiants ultra-sécurisé, souverain et ergonomique, garantissant :
- Une authentification forte obligatoire à double facteur (2FA) basée sur le standard TOTP (compatible Microsoft Authenticator, Google Authenticator).
- Un chiffrement de niveau bancaire/militaire (AES-256-GCM avec dérivation de clé Argon2id / PBKDF2) pour le stockage des mots de passe.
- Une expérience utilisateur fluide avec génération de mots de passe forts, copie sécurisée avec auto-effacement du presse-papier et audit trail des consultations.

## 2. Cas d'Usage Principaux
1. **Enrôlement et Connexion 2FA Sécurisée** :
   - L'utilisateur s'enregistre ou se connecte avec son compte principal.
   - Scan du QR Code via **Microsoft Authenticator** (ou saisie de la clé secrète Base32).
   - Validation immédiate du jeton TOTP à 6 chiffres.
   - Génération et téléchargement de codes de secours à usage unique.
2. **Gestion du Coffre-fort d'Identifiants (Vault)** :
   - Ajout, modification, consultation et archivage de secrets (login, mot de passe, URL, notes secrètes, tags/catégories).
   - Mots de passe masqués par défaut, démasquables à la demande ou copiables en 1 clic.
   - Nettoyage automatique du presse-papier après 30 secondes pour prévenir les fuites de mémoire.
3. **Générateur de Mots de Passe Forts** :
   - Génération paramétrable (longueur, caractères spéciaux, chiffres, minuscules/majuscules).
   - Évaluation en temps réel de l'entropie et de la robustesse.
4. **Audit & Journalisation de Sécurité** :
   - Traçabilité de chaque consultation, tentative de connexion ou modification d'identifiant.
   - Détection des tentatives suspectes (brute-force).

## 3. Personas Cibles
- **Administrateur Système / Développeur** : a besoin d'accéder rapidement à des identifiants de production de façon isolée et sécurisée.
- **Chef de Projet / Utilisateur Interne P2T** : centralise les accès aux différents portails de l'entreprise avec une sécurité sans faille.
- **Responsable Sécurité (RSSI)** : exige la conformité 2FA, l'audit trail et l'absence de mots de passe en clair.

## 4. Métriques de Succès
- 100% des comptes protégés par 2FA TOTP (Microsoft Authenticator).
- 0 mot de passe stocké en clair ou dans les logs.
- Temps de déverrouillage et consultation < 1 seconde.
- Couverture de tests de sécurité et unitaires > 90%.
