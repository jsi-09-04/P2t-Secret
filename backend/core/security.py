"""
backend/core/security.py - Fonctions de sécurité cryptographique, hachage et tokens JWT.
"""
import os
import hmac
import hashlib
import base64
from datetime import datetime, timedelta, timezone
from typing import Optional, Dict, Any
import jwt

# Configuration par défaut
SECRET_KEY = os.getenv("P2T_SECRET_JWT_KEY", base64.b64encode(os.urandom(32)).decode("ascii"))
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 60
TEMP_TOKEN_EXPIRE_MINUTES = 5
PBKDF2_ITERATIONS = 600_000


def hash_password(password: str) -> str:
    """
    Hache un mot de passe avec PBKDF2-HMAC-SHA256 et un sel aléatoire de 16 octets.
    Format retourné : pbkdf2_sha256$<iterations>$<salt_b64>$<hash_b64>
    """
    salt = os.urandom(16)
    key = hashlib.pbkdf2_hmac(
        "sha256",
        password.encode("utf-8"),
        salt,
        PBKDF2_ITERATIONS,
        dklen=32,
    )
    salt_b64 = base64.b64encode(salt).decode("ascii")
    hash_b64 = base64.b64encode(key).decode("ascii")
    return f"pbkdf2_sha256${PBKDF2_ITERATIONS}${salt_b64}${hash_b64}"


def verify_password(plain_password: str, hashed_password: str) -> bool:
    """Vérifie un mot de passe en temps constant."""
    try:
        parts = hashed_password.split("$")
        if len(parts) != 4 or parts[0] != "pbkdf2_sha256":
            return False
        iterations = int(parts[1])
        salt = base64.b64decode(parts[2])
        expected_hash = base64.b64decode(parts[3])

        key = hashlib.pbkdf2_hmac(
            "sha256",
            plain_password.encode("utf-8"),
            salt,
            iterations,
            dklen=32,
        )
        return hmac.compare_digest(key, expected_hash)
    except Exception:
        return False


def create_access_token(data: Dict[str, Any], expires_delta: Optional[timedelta] = None) -> str:
    """Crée un jeton d'accès JWT signé."""
    to_encode = data.copy()
    now = datetime.now(timezone.utc)
    expire = now + (expires_delta if expires_delta else timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES))
    to_encode.update({"exp": expire, "iat": now, "type": "access"})
    return jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)


def create_temp_2fa_token(user_id: str, username: str) -> str:
    """Crée un jeton temporaire (5 min) valide uniquement pour résoudre le challenge 2FA."""
    now = datetime.now(timezone.utc)
    expire = now + timedelta(minutes=TEMP_TOKEN_EXPIRE_MINUTES)
    payload = {
        "sub": user_id,
        "username": username,
        "type": "2fa_challenge",
        "iat": now,
        "exp": expire,
    }
    return jwt.encode(payload, SECRET_KEY, algorithm=ALGORITHM)


def decode_token(token: str) -> Optional[Dict[str, Any]]:
    """Décode et valide un jeton JWT."""
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        return payload
    except Exception:
        return None


def derive_master_vault_key(master_password: str, user_salt_hex: str) -> bytes:
    """
    Dérive une clé AES-256 (32 octets) à partir du mot de passe maître de l'utilisateur
    et de son sel cryptographique unique.
    """
    salt = bytes.fromhex(user_salt_hex)
    return hashlib.pbkdf2_hmac(
        "sha256",
        master_password.encode("utf-8"),
        salt,
        100_000,
        dklen=32,
    )
