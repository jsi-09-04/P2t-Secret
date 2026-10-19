"""
backend/core/crypto_vault.py - Moteur de chiffrement AES-256-GCM et générateur de mots de passe.
"""
import os
import base64
import secrets
import string
import math
from typing import Tuple, Dict, Any
from cryptography.hazmat.primitives.ciphers.aead import AESGCM


class CryptoVault:
    """Moteur cryptographique AES-256-GCM pour le coffre-fort d'identifiants."""

    @staticmethod
    def encrypt(plaintext: str, key_32bytes: bytes, associated_data: bytes = b"") -> str:
        """
        Chiffre une chaîne en AES-256-GCM.
        Génère un IV unique de 96 bits (12 octets).
        Le mode GCM produit un ciphertext + tag d'authentification de 16 octets.
        Format résultant : Base64(IV [12 bytes] + CiphertextWithTag).
        """
        if not plaintext:
            return ""

        aesgcm = AESGCM(key_32bytes)
        nonce = os.urandom(12)  # 96 bits recommandé pour GCM
        data = plaintext.encode("utf-8")
        ciphertext = aesgcm.encrypt(nonce, data, associated_data)

        # Concaténation nonce + ciphertext (inclut le tag d'authentification de 16 octets à la fin)
        payload = nonce + ciphertext
        return base64.urlsafe_b64encode(payload).decode("ascii")

    @staticmethod
    def decrypt(encrypted_token: str, key_32bytes: bytes, associated_data: bytes = b"") -> str:
        """
        Déchiffre un token AES-256-GCM et valide le tag d'intégrité.
        Lève une exception si le token est corrompu ou la clé invalide.
        """
        if not encrypted_token:
            return ""

        payload = base64.urlsafe_b64decode(encrypted_token.encode("ascii"))
        if len(payload) < 28:  # 12 octets nonce + 16 octets tag minimum
            raise ValueError("Token chiffré trop court ou corrompu.")

        nonce = payload[:12]
        ciphertext = payload[12:]

        aesgcm = AESGCM(key_32bytes)
        decrypted_bytes = aesgcm.decrypt(nonce, ciphertext, associated_data)
        return decrypted_bytes.decode("utf-8")

    @staticmethod
    def generate_strong_password(
        length: int = 20,
        use_upper: bool = True,
        use_lower: bool = True,
        use_digits: bool = True,
        use_symbols: bool = True,
        exclude_ambiguous: bool = True,
    ) -> Tuple[str, Dict[str, Any]]:
        """
        Génère un mot de passe cryptographiquement sécurisé avec calcul d'entropie.
        """
        length = max(12, min(length, 128))

        upper = string.ascii_uppercase
        lower = string.ascii_lowercase
        digits = string.digits
        symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?"

        if exclude_ambiguous:
            # Retirer caractères ambigus : 0, O, o, l, 1, I
            ambiguous = "0Ool1I|`'\";:"
            upper = "".join(c for c in upper if c not in ambiguous)
            lower = "".join(c for c in lower if c not in ambiguous)
            digits = "".join(c for c in digits if c not in ambiguous)
            symbols = "".join(c for c in symbols if c not in ambiguous)

        pool = ""
        mandatory_chars = []

        if use_upper and upper:
            pool += upper
            mandatory_chars.append(secrets.choice(upper))
        if use_lower and lower:
            pool += lower
            mandatory_chars.append(secrets.choice(lower))
        if use_digits and digits:
            pool += digits
            mandatory_chars.append(secrets.choice(digits))
        if use_symbols and symbols:
            pool += symbols
            mandatory_chars.append(secrets.choice(symbols))

        if not pool:
            pool = string.ascii_letters + string.digits

        # Compléter avec des caractères aléatoires
        remaining_length = length - len(mandatory_chars)
        random_chars = [secrets.choice(pool) for _ in range(remaining_length)]
        all_chars = mandatory_chars + random_chars

        # Mélange cryptographique
        secrets.SystemRandom().shuffle(all_chars)
        password = "".join(all_chars)

        # Calcul d'entropie : E = L * log2(pool_size)
        pool_size = len(pool)
        entropy_bits = round(length * math.log2(pool_size), 1)

        strength = "Faible"
        if entropy_bits >= 80:
            strength = "Très Robuste (Militaire)"
        elif entropy_bits >= 60:
            strength = "Robuste"
        elif entropy_bits >= 45:
            strength = "Moyen"

        info = {
            "length": length,
            "entropy_bits": entropy_bits,
            "strength": strength,
            "pool_size": pool_size,
        }

        return password, info


crypto_vault = CryptoVault()
