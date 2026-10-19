"""
tests/test_api_flow.py - Tests fonctionnels d'intégration de bout en bout (Auth, 2FA, Vault).
"""
import pytest
from fastapi.testclient import TestClient
import pyotp

from backend.main import app
from backend.models.database import init_database


@pytest.fixture(autouse=True)
def setup_db():
    init_database()


def test_full_auth_2fa_and_vault_flow():
    client = TestClient(app)
    import uuid

    # 1. Inscription d'un utilisateur
    username = f"user_{uuid.uuid4().hex[:8]}"
    password = "MasterPassword2026!"
    res_reg = client.post("/api/v1/auth/register", json={"username": username, "password": password})
    assert res_reg.status_code == 201

    # 2. Login initial (2FA pas encore configurée)
    res_login1 = client.post("/api/v1/auth/login", json={"username": username, "password": password})
    assert res_login1.status_code == 200
    data_login1 = res_login1.json()
    assert data_login1["requires_2fa"] is False
    setup_token = data_login1["access_token"]

    # 3. Initiation configuration 2FA Microsoft Authenticator
    headers = {"Authorization": f"Bearer {setup_token}"}
    res_setup = client.post("/api/v1/auth/2fa/setup", headers=headers)
    assert res_setup.status_code == 200
    data_setup = res_setup.json()
    secret = data_setup["secret"]
    assert "<svg" in data_setup["qr_svg"]
    assert "otpauth://" in data_setup["otpauth_url"]

    # 4. Confirmation 2FA avec code valide généré
    totp = pyotp.TOTP(secret, interval=30, digits=6)
    code = totp.now()
    res_conf = client.post("/api/v1/auth/2fa/confirm", headers=headers, json={"code": code})
    assert res_conf.status_code == 200
    data_conf = res_conf.json()
    assert len(data_conf["backup_codes"]) == 8
    access_token = data_conf["access_token"]

    # 5. Deuxième Login : 2FA doit maintenant être OBLIGATOIRE
    res_login2 = client.post("/api/v1/auth/login", json={"username": username, "password": password})
    assert res_login2.status_code == 200
    data_login2 = res_login2.json()
    assert data_login2["requires_2fa"] is True
    temp_token = data_login2["temp_token"]

    # 6. Étape 2FA : Vérification avec mauvais code -> Rejeté
    res_verify_bad = client.post("/api/v1/auth/2fa/verify", json={"temp_token": temp_token, "code": "000000"})
    assert res_verify_bad.status_code == 400

    # 7. Étape 2FA : Vérification avec bon code -> Accès accordé
    valid_code = totp.now()
    res_verify_ok = client.post("/api/v1/auth/2fa/verify", json={"temp_token": temp_token, "code": valid_code})
    assert res_verify_ok.status_code == 200
    vault_token = res_verify_ok.json()["access_token"]
    vault_headers = {"Authorization": f"Bearer {vault_token}"}

    # 8. Ajout d'un identifiant dans le coffre chiffré
    cred_payload = {
        "title": "Serveur Cloud Production",
        "url": "https://console.p2t.cloud",
        "username": "root_admin",
        "password": "UltraSecretPassword987!#",
        "notes": "Clé SSH et port 2222",
        "category": "Serveurs",
        "tags": "production,cloud",
    }
    res_cred_create = client.post("/api/v1/vault", headers=vault_headers, json=cred_payload)
    assert res_cred_create.status_code == 201
    created_cred = res_cred_create.json()
    cred_id = created_cred["id"]

    # 9. Liste des identifiants (mot de passe n'apparaît pas)
    res_list = client.post = client.get("/api/v1/vault", headers=vault_headers)
    assert res_list.status_code == 200
    items = res_list.json()
    assert len(items) >= 1
    assert "password" not in items[0]

    # 10. Consultation détaillée avec déchiffrement à la volée
    res_detail = client.get(f"/api/v1/vault/{cred_id}", headers=vault_headers)
    assert res_detail.status_code == 200
    detail = res_detail.json()
    assert detail["password"] == "UltraSecretPassword987!#"
    assert detail["notes"] == "Clé SSH et port 2222"

    # 11. Journal d'audit accessible
    res_audit = client.get("/api/v1/vault/audit", headers=vault_headers)
    assert res_audit.status_code == 200
    logs = res_audit.json()
    assert len(logs) > 0
