# Spécification Technique : SPEC-02 — Chiffrement et Gestion du Coffre (Vault)

## 1. Objectif
Définir les spécifications de sécurité pour le stockage, le chiffrement symétrique et la manipulation des identifiants (login, mot de passe, URL, notes) au sein de **P2TSecret**.

## 2. Algorithme de Chiffrement : AES-256-GCM
- **Chiffreur** : `AES` avec taille de clé 256 bits (32 octets).
- **Mode d'opération** : `GCM` (Galois/Counter Mode), assurant à la fois la **confidentialité** et l'**authenticité** (AEAD).
- **Vecteur d'initialisation (IV / Nonce)** : 96 bits (12 octets) généré aléatoirement pour chaque enregistrement via `os.urandom(12)`. L'IV ne doit JAMAIS être réutilisé avec la même clé.
- **Tag d'authentification** : 128 bits (16 octets) calculé automatiquement par GCM pour détecter toute altération du texte chiffré.
- **Payload stocké** : Concaténation sérialisée `IV (12 octets) + Tag (16 octets) + Ciphertext`, encodée en Base64 URL-safe pour le stockage.

## 3. Schéma de Données (Credential)
Chaque entrée dans le coffre-fort possède :
- `id` : UUID v4 unique.
- `user_id` : Identifiant de l'utilisateur propriétaire.
- `title` : Nom du service ou portail (ex: "Console AWS Production", "Dolibarr Admin").
- `url` : URL d'accès optionnelle (ex: `https://erp.projet2team.com`).
- `username` : Identifiant de connexion ou email.
- `encrypted_password` : Mot de passe chiffré en AES-256-GCM.
- `encrypted_notes` : Notes confidentielles chiffrées en AES-256-GCM (optionnel).
- `category` : Catégorie (Serveurs, Cloud, Outils Métier, Banques, etc.).
- `tags` : Mots-clés pour le filtrage rapide.
- `created_at` / `updated_at` : Horodatages UTC.

## 4. Endpoints de l'API Coffre (Vault)
Toutes ces routes nécessitent l'en-tête `Authorization: Bearer <JWT_ACCESS_TOKEN>`.

| Méthode | Route | Description |
|---|---|---|
| `GET` | `/api/v1/vault` | Liste tous les identifiants de l'utilisateur (avec mots de passe masqués) |
| `POST` | `/api/v1/vault` | Crée une nouvelle entrée sécurisée (chiffrement automatique) |
| `GET` | `/api/v1/vault/{id}` | Récupère le détail d'une entrée avec mot de passe déchiffré à la volée |
| `PUT` | `/api/v1/vault/{id}` | Met à jour une entrée (rechiffrement avec nouvel IV) |
| `DELETE` | `/api/v1/vault/{id}` | Supprime une entrée du coffre-fort |
| `POST` | `/api/v1/vault/generate-password` | Générateur cryptographique de mot de passe fort avec score d'entropie |
