"""
tests/test_crypto.py - Tests unitaires du moteur cryptographique AES-256-GCM et du générateur de mot de passe.
"""
import os
import pytest
from backend.core.crypto_vault import crypto_vault
from backend.core.security import hash_password, verify_password


def test_aes_gcm_encrypt_decrypt():
    key = os.urandom(32)
    secret_text = "MonSuperMotDePasseSecret123!#"

    # Chiffrement
    token = crypto_vault.encrypt(secret_text, key)
    assert token != secret_text
    assert len(token) > 28

    # Déchiffrement
    decrypted = crypto_vault.decrypt(token, key)
    assert decrypted == secret_text


def test_aes_gcm_unique_iv():
    key = os.urandom(32)
    secret_text = "MemeTexteSecret"

    # Deux chiffrements du même texte doivent produire des tokens différents (IV aléatoire)
    token1 = crypto_vault.encrypt(secret_text, key)
    token2 = crypto_vault.encrypt(secret_text, key)
    assert token1 != token2

    assert crypto_vault.decrypt(token1, key) == secret_text
    assert crypto_vault.decrypt(token2, key) == secret_text


def test_aes_gcm_tampered_token_fails():
    key = os.urandom(32)
    secret_text = "DonnéeUltraConfidentielle"
    token = crypto_vault.encrypt(secret_text, key)

    # Altération d'un caractère du token chiffré
    tampered_bytes = bytearray(token.encode("ascii"))
    tampered_bytes[10] = ord("A") if tampered_bytes[10] != ord("A") else ord("B")
    tampered_token = tampered_bytes.decode("ascii")

    with pytest.raises(Exception):
        crypto_vault.decrypt(tampered_token, key)


def test_aes_gcm_wrong_key_fails():
    key1 = os.urandom(32)
    key2 = os.urandom(32)
    secret_text = "SecretImportant"

    token = crypto_vault.encrypt(secret_text, key1)
    with pytest.raises(Exception):
        crypto_vault.decrypt(token, key2)


def test_password_hashing():
    pwd = "AdminMasterPassword2026!"
    hashed = hash_password(pwd)

    assert hashed.startswith("pbkdf2_sha256$")
    assert verify_password(pwd, hashed) is True
    assert verify_password("MauvaisMotDePasse", hashed) is False


def test_strong_password_generator():
    pwd, info = crypto_vault.generate_strong_password(length=24, use_symbols=True)

    assert len(pwd) == 24
    assert info["length"] == 24
    assert info["entropy_bits"] >= 80  # Entropie militaire
    assert info["strength"] == "Très Robuste (Militaire)"
