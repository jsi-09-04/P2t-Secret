"""
backend/core/totp_service.py - Service TOTP conforme RFC 6238 pour Microsoft Authenticator.
"""
import io
import os
import secrets
import base64
import hashlib
from typing import List, Tuple
import pyotp
import qrcode
import qrcode.image.svg


ISSUER_NAME = "P2TSecret"


class TotpService:
    """Service de gestion du 2FA TOTP (Microsoft Authenticator / Google Authenticator)."""

    @staticmethod
    def generate_secret() -> str:
        """
        Génère une clé secrète aléatoire de 160 bits (20 octets) encodée en Base32.
        Parfaitement compatible avec Microsoft Authenticator.
        """
        return pyotp.random_base32(length=32)

    @staticmethod
    def build_provisioning_uri(username: str, secret_base32: str) -> str:
        """
        Génère l'URI standard otpauth:// reconnue par Microsoft Authenticator.
        Exemple : otpauth://totp/P2TSecret:admin?secret=...&issuer=P2TSecret
        """
        totp = pyotp.TOTP(secret_base32, interval=30, digits=6)
        return totp.provisioning_uri(name=username, issuer_name=ISSUER_NAME)

    @classmethod
    def generate_qr_code_svg(cls, username: str, secret_base32: str) -> Tuple[str, str]:
        """
        Génère le QR Code vectoriel SVG et l'URI provisioning pour Microsoft Authenticator.
        Retourne (uri_provisioning, svg_content_str).
        """
        uri = cls.build_provisioning_uri(username, secret_base32)
        
        # Génération du QR Code SVG en mémoire
        factory = qrcode.image.svg.SvgPathImage
        img = qrcode.make(uri, image_factory=factory, box_size=10, border=2)
        stream = io.BytesIO()
        img.save(stream)
        svg_str = stream.getvalue().decode("utf-8")

        return uri, svg_str

    @staticmethod
    def verify_totp_code(secret_base32: str, code: str, valid_window: int = 1) -> bool:
        """
        Vérifie le code TOTP à 6 chiffres.
        valid_window = 1 permet d'accepter le code du pas précédent, actuel et suivant (fenêtre de 90s),
        pour gérer les légères dérives d'horloge de Microsoft Authenticator.
        """
        if not secret_base32 or not code:
            return False
        clean_code = code.strip().replace(" ", "")
        if len(clean_code) != 6 or not clean_code.isdigit():
            return False

        totp = pyotp.TOTP(secret_base32, interval=30, digits=6)
        return totp.verify(clean_code, valid_window=valid_window)

    @staticmethod
    def generate_backup_codes(count: int = 8) -> Tuple[List[str], List[str]]:
        """
        Génère une liste de codes de secours au format XXXX-XXXX et leurs empreintes hachées.
        Retourne (codes_en_clair_pour_utilisateur, hashes_pour_stockage).
        """
        plain_codes = []
        hashed_codes = []

        for _ in range(count):
            raw = secrets.token_hex(4).upper()
            formatted = f"{raw[:4]}-{raw[4:]}"
            plain_codes.append(formatted)

            # Hachage SHA-256 pour stockage en BDD
            h = hashlib.sha256(formatted.encode("utf-8")).hexdigest()
            hashed_codes.append(h)

        return plain_codes, hashed_codes

    @staticmethod
    def verify_backup_code(plain_code: str, hashed_code_in_db: str) -> bool:
        """Vérifie un code de secours en temps constant."""
        formatted = plain_code.strip().upper()
        h = hashlib.sha256(formatted.encode("utf-8")).hexdigest()
        return secrets.compare_digest(h, hashed_code_in_db)


totp_service = TotpService()
