# Spécification Technique : SPEC-01 — Authentification 2FA TOTP Microsoft Authenticator

## 1. Objectif
Définir l'architecture et les exigences pour l'enrôlement, la génération et la vérification des jetons d'authentification à double facteur (2FA) conformes au standard **TOTP (RFC 6238)** et pleinement compatibles avec l'application mobile **Microsoft Authenticator**.

## 2. Standard Cryptographique TOTP
- **Algorithme de hachage** : SHA-1 (conformité universelle avec Microsoft Authenticator et RFC 6238 section 4).
- **Nombre de chiffres (digits)** : 6 chiffres décimaux.
- **Intervalle de temps (time step $T_0$)** : 30 secondes.
- **Clé secrète partagée** : 20 octets (160 bits) pseudo-aléatoires générés de manière cryptographiquement sûre (`os.urandom(20)`), encodés en Base32 (32 caractères sans padding `=`).
- **Tolérance temporelle** : +/- 1 intervalle (fenêtre de 90 secondes) pour absorber les dérives d'horloge du terminal mobile.
- **Protection anti-rejeu** : Mémorisation du dernier timestamp validé pour refuser l'utilisation deux fois du même code OTP dans le même intervalle.

## 3. Format de l'URI d'enrôlement
L'URI standard transmise au client pour la génération du QR code est :
```
otpauth://totp/P2TSecret:{username}?secret={secret_base32}&issuer=P2TSecret&algorithm=SHA1&digits=6&period=30
```
Ce format est instantanément reconnu par :
- Microsoft Authenticator (iOS / Android)
- Google Authenticator
- Aegis Authenticator
- Bitwarden / 1Password

## 4. Codes de Secours (Backup Recovery Codes)
Lors de l'activation du 2FA, 8 codes de secours aléatoires (format `XXXX-XXXX`, 8 caractères hexadécimaux ou alphanumériques) sont générés.
- Chaque code est stocké sous forme hachée (PBKDF2/SHA-256).
- L'utilisation d'un code de secours invalide immédiatement celui-ci (usage unique).

## 5. Endpoints de l'API
| Méthode | Route | Description | Entrée | Sortie |
|---|---|---|---|---|
| `POST` | `/api/v1/auth/register` | Inscription initiale | `username, password` | `user_id, status` |
| `POST` | `/api/v1/auth/login` | Étape 1 : Vérification mot de passe | `username, password` | `temp_token, is_2fa_enabled` |
| `POST` | `/api/v1/auth/2fa/setup` | Demande d'enrôlement 2FA | En-tête `Authorization` | `secret_base32, qr_svg, otpauth_url` |
| `POST` | `/api/v1/auth/2fa/confirm` | Confirmation de l'enrôlement | `code` (6 chiffres) | `backup_codes, access_token` |
| `POST` | `/api/v1/auth/2fa/verify` | Étape 2 de connexion normale | `temp_token, code` | `access_token, expires_in` |
| `POST` | `/api/v1/auth/2fa/recovery` | Connexion via code de secours | `temp_token, recovery_code` | `access_token, remaining_codes` |
