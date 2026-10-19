"""
tests/test_totp.py - Tests du protocole 2FA TOTP pour Microsoft Authenticator.
"""
import pyotp
from backend.core.totp_service import totp_service


def test_totp_secret_and_uri_format():
    secret = totp_service.generate_secret()
    assert len(secret) == 32  # Base32 160 bits

    uri, svg = totp_service.generate_qr_code_svg("james@projet2team.com", secret)
    assert uri.startswith("otpauth://totp/P2TSecret:")
    assert "james" in uri
    assert f"secret={secret}" in uri
    assert "issuer=P2TSecret" in uri
    assert "<svg" in svg
    assert "</svg>" in svg


def test_totp_verification_valid():
    secret = totp_service.generate_secret()
    totp = pyotp.TOTP(secret, interval=30, digits=6)
    valid_code = totp.now()

    assert totp_service.verify_totp_code(secret, valid_code) is True


def test_totp_verification_invalid():
    secret = totp_service.generate_secret()
    assert totp_service.verify_totp_code(secret, "000000") is False
    assert totp_service.verify_totp_code(secret, "abcdef") is False
    assert totp_service.verify_totp_code(secret, "123") is False


def test_backup_codes_generation_and_verification():
    plain_codes, hashed_codes = totp_service.generate_backup_codes(count=8)

    assert len(plain_codes) == 8
    assert len(hashed_codes) == 8

    # Vérifier que le format est XXXX-XXXX
    for code in plain_codes:
        assert len(code) == 9
        assert code[4] == "-"

    # Vérifier la validation en temps constant
    assert totp_service.verify_backup_code(plain_codes[0], hashed_codes[0]) is True
    assert totp_service.verify_backup_code("XXXX-YYYY", hashed_codes[0]) is False
