"""
backend/routers/auth.py - Endpoints d'authentification et 2FA Microsoft Authenticator.
"""
import os
from fastapi import APIRouter, HTTPException, Depends, Request, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials

from backend.schemas.auth import (
    RegisterRequest,
    LoginRequest,
    LoginResponse,
    Setup2FAResponse,
    Confirm2FARequest,
    Confirm2FAResponse,
    Verify2FARequest,
    Recovery2FARequest,
    AuthTokenResponse,
)
from backend.core.security import (
    hash_password,
    verify_password,
    create_access_token,
    create_temp_2fa_token,
    decode_token,
)
from backend.core.totp_service import totp_service
from backend.models.database import db_service

router = APIRouter(prefix="/auth", tags=["Authentification & 2FA"])
security_bearer = HTTPBearer(auto_error=False)


def get_current_user(credentials: HTTPAuthorizationCredentials = Depends(security_bearer)) -> dict:
    """Valide le jeton JWT d'accès complet."""
    if not credentials:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Jeton d'authentification manquant.")
    payload = decode_token(credentials.credentials)
    if not payload or payload.get("type") != "access":
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Jeton invalide ou expiré.")
    user = db_service.get_user_by_id(payload.get("sub"))
    if not user:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Utilisateur introuvable.")
    return user


@router.post("/register", status_code=status.HTTP_201_CREATED)
async def register(req: RegisterRequest, request: Request):
    """Inscription initiale d'un compte."""
    existing = db_service.get_user_by_username(req.username)
    if existing:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Ce nom d'utilisateur est déjà utilisé.")

    salt_hex = os.urandom(16).hex()
    hashed = hash_password(req.password)
    user = db_service.create_user(req.username, hashed, salt_hex)

    ip = request.client.host if request.client else "unknown"
    db_service.log_audit(user["id"], "user_registered", target_title=req.username, ip_address=ip, status="success")

    return {
        "message": "Compte créé avec succès. Veuillez activer la 2FA pour finaliser la sécurisation.",
        "user_id": user["id"],
        "username": user["username"],
    }


@router.post("/login", response_model=LoginResponse)
async def login(req: LoginRequest, request: Request):
    """
    Première étape d'authentification : vérification mot de passe.
    Si 2FA activée : retourne un temp_token pour le challenge 2FA Microsoft Authenticator.
    Si 2FA non encore activée : invite à l'activer.
    """
    ip = request.client.host if request.client else "unknown"
    user = db_service.get_user_by_username(req.username)

    if not user or not verify_password(req.password, user["hashed_password"]):
        db_service.log_audit(user["id"] if user else None, "login_failed", target_title=req.username, ip_address=ip, status="failure", details="Identifiants incorrects")
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Nom d'utilisateur ou mot de passe incorrect.")

    # Si 2FA est activée : challenge obligatoire
    if user["is_2fa_enabled"]:
        temp_token = create_temp_2fa_token(user["id"], user["username"])
        db_service.log_audit(user["id"], "2fa_challenge_issued", target_title=user["username"], ip_address=ip, status="pending")
        return LoginResponse(requires_2fa=True, temp_token=temp_token)

    # Si 2FA non activée : délivre un token d'accès temporaire pour configurer le 2FA
    access_token = create_access_token({"sub": user["id"], "username": user["username"], "is_2fa_setup": False})
    db_service.log_audit(user["id"], "login_no_2fa", target_title=user["username"], ip_address=ip, status="success", details="2FA non configurée")
    return LoginResponse(
        requires_2fa=False,
        access_token=access_token,
        user_id=user["id"],
        username=user["username"],
    )


@router.post("/2fa/setup", response_model=Setup2FAResponse)
async def setup_2fa(user: dict = Depends(get_current_user)):
    """
    Génère la clé secrète Base32 et le QR Code SVG pour Microsoft Authenticator.
    """
    secret = totp_service.generate_secret()
    uri, qr_svg = totp_service.generate_qr_code_svg(user["username"], secret)
    db_service.set_user_totp_secret(user["id"], secret)

    return Setup2FAResponse(
        secret=secret,
        otpauth_url=uri,
        qr_svg=qr_svg,
    )


@router.post("/2fa/confirm", response_model=Confirm2FAResponse)
async def confirm_2fa(req: Confirm2FARequest, request: Request, user: dict = Depends(get_current_user)):
    """
    Valide le premier code saisi depuis Microsoft Authenticator, active la 2FA
    et retourne les codes de secours à usage unique.
    """
    ip = request.client.host if request.client else "unknown"
    secret = user.get("totp_secret_base32")
    if not secret:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Configuration 2FA non initiée. Appelez /2fa/setup d'abord.")

    if not totp_service.verify_totp_code(secret, req.code):
        db_service.log_audit(user["id"], "2fa_setup_rejected", target_title="Microsoft Authenticator", ip_address=ip, status="failure", details="Code TOTP invalide")
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Code à 6 chiffres invalide ou expiré.")

    # Générer codes de secours
    plain_backup_codes, hashed_backup_codes = totp_service.generate_backup_codes(count=8)
    db_service.enable_user_2fa(user["id"], secret, hashed_backup_codes)

    access_token = create_access_token({"sub": user["id"], "username": user["username"], "is_2fa_setup": True})
    db_service.log_audit(user["id"], "2fa_enabled", target_title="Microsoft Authenticator", ip_address=ip, status="success")

    return Confirm2FAResponse(
        message="Authentification 2FA Microsoft Authenticator activée avec succès !",
        backup_codes=plain_backup_codes,
        access_token=access_token,
    )


@router.post("/2fa/verify", response_model=AuthTokenResponse)
async def verify_2fa(req: Verify2FARequest, request: Request):
    """
    Seconde étape de connexion : vérification du code à 6 chiffres issu de Microsoft Authenticator.
    """
    ip = request.client.host if request.client else "unknown"
    payload = decode_token(req.temp_token)
    if not payload or payload.get("type") != "2fa_challenge":
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Jeton de challenge 2FA invalide ou expiré.")

    user_id = payload.get("sub")
    user = db_service.get_user_by_id(user_id)
    if not user or not user["totp_secret_base32"]:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Utilisateur ou configuration 2FA introuvable.")

    if not totp_service.verify_totp_code(user["totp_secret_base32"], req.code):
        db_service.log_audit(user_id, "2fa_verify_failed", target_title=user["username"], ip_address=ip, status="failure", details="Code TOTP incorrect")
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Code Microsoft Authenticator invalide ou expiré.")

    access_token = create_access_token({"sub": user["id"], "username": user["username"], "is_2fa_setup": True})
    db_service.log_audit(user_id, "2fa_verify_success", target_title=user["username"], ip_address=ip, status="success")

    return AuthTokenResponse(
        access_token=access_token,
        user_id=user["id"],
        username=user["username"],
    )


@router.post("/2fa/recovery", response_model=AuthTokenResponse)
async def verify_recovery_code(req: Recovery2FARequest, request: Request):
    """
    Connexion de secours en cas de perte du smartphone avec Microsoft Authenticator.
    """
    ip = request.client.host if request.client else "unknown"
    payload = decode_token(req.temp_token)
    if not payload or payload.get("type") != "2fa_challenge":
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Jeton de challenge 2FA invalide ou expiré.")

    user_id = payload.get("sub")
    user = db_service.get_user_by_id(user_id)
    if not user:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Utilisateur introuvable.")

    consumed = db_service.consume_backup_code(user_id, req.recovery_code)
    if not consumed:
        db_service.log_audit(user_id, "recovery_code_failed", target_title=user["username"], ip_address=ip, status="failure", details="Code de secours invalide ou déjà consommé")
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Code de secours invalide ou déjà utilisé.")

    access_token = create_access_token({"sub": user["id"], "username": user["username"], "is_2fa_setup": True})
    db_service.log_audit(user_id, "recovery_code_used", target_title=user["username"], ip_address=ip, status="success", details="Connexion via code de secours")

    return AuthTokenResponse(
        access_token=access_token,
        user_id=user["id"],
        username=user["username"],
    )


@router.get("/me")
async def get_current_user_profile(user: dict = Depends(get_current_user)):
    """Retourne le profil de l'utilisateur connecté."""
    return {
        "id": user["id"],
        "username": user["username"],
        "is_2fa_enabled": bool(user["is_2fa_enabled"]),
        "created_at": user["created_at"],
    }
