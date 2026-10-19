"""
backend/routers/vault.py - Endpoints de gestion du coffre-fort d'identifiants et de chiffrement.
"""
import os
import hashlib
from typing import List
from fastapi import APIRouter, HTTPException, Depends, Request, status

from backend.schemas.vault import (
    CredentialCreate,
    CredentialUpdate,
    CredentialListItem,
    CredentialDetail,
    GeneratePasswordRequest,
    GeneratePasswordResponse,
    AuditLogItem,
)
from backend.routers.auth import get_current_user
from backend.core.crypto_vault import crypto_vault
from backend.models.database import db_service

router = APIRouter(prefix="/vault", tags=["Coffre-fort & Mots de passe"])


def get_user_encryption_key(user: dict) -> bytes:
    """
    Dérive la clé AES-256 (32 octets) dédiée à cet utilisateur,
    en combinant la clé système maîtresse, l'ID utilisateur et son sel cryptographique unique.
    """
    master_seed = os.getenv("P2T_SECRET_MASTER_VAULT_KEY", "p2t-vault-system-master-seed-2026")
    salt = bytes.fromhex(user["salt_hex"])
    key_material = f"{master_seed}:{user['id']}".encode("utf-8")
    return hashlib.pbkdf2_hmac("sha256", key_material, salt, 100_000, dklen=32)


@router.get("", response_model=List[CredentialListItem])
async def list_credentials(request: Request, user: dict = Depends(get_current_user)):
    """Liste tous les identifiants enregistrés (les mots de passe restent masqués)."""
    items = db_service.list_credentials(user["id"])
    return items


@router.post("", response_model=CredentialListItem, status_code=status.HTTP_201_CREATED)
async def create_credential(req: CredentialCreate, request: Request, user: dict = Depends(get_current_user)):
    """Chiffre et stocke un nouvel identifiant dans le coffre-fort."""
    key = get_user_encryption_key(user)
    
    # Chiffrement AES-256-GCM du mot de passe et des notes
    enc_password = crypto_vault.encrypt(req.password, key)
    enc_notes = crypto_vault.encrypt(req.notes or "", key) if req.notes else ""

    item = db_service.create_credential(
        user_id=user["id"],
        title=req.title,
        url=req.url or "",
        username=req.username,
        enc_password=enc_password,
        enc_notes=enc_notes,
        category=req.category or "Général",
        tags=req.tags or "",
    )

    ip = request.client.host if request.client else "unknown"
    db_service.log_audit(user["id"], "credential_created", target_title=req.title, ip_address=ip, status="success")

    return item


@router.post("/generate-password", response_model=GeneratePasswordResponse)
async def generate_password(req: GeneratePasswordRequest):
    """Générateur cryptographique de mot de passe hautement sécurisé avec entropie."""
    pwd, info = crypto_vault.generate_strong_password(
        length=req.length,
        use_upper=req.use_upper,
        use_lower=req.use_lower,
        use_digits=req.use_digits,
        use_symbols=req.use_symbols,
        exclude_ambiguous=req.exclude_ambiguous,
    )
    return GeneratePasswordResponse(password=pwd, info=info)


@router.get("/audit", response_model=List[AuditLogItem])
async def get_audit_trail(user: dict = Depends(get_current_user)):
    """Consulte l'historique d'audit des actions sur le coffre-fort."""
    logs = db_service.get_recent_audit_logs(user["id"])
    return logs


@router.get("/{cred_id}", response_model=CredentialDetail)
async def get_credential_detail(cred_id: str, request: Request, user: dict = Depends(get_current_user)):
    """Récupère et déchiffre à la volée le mot de passe et les notes de l'identifiant."""
    raw = db_service.get_credential_by_id(cred_id, user["id"])
    if not raw:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Identifiant introuvable dans le coffre.")

    key = get_user_encryption_key(user)
    try:
        dec_password = crypto_vault.decrypt(raw["encrypted_password"], key)
        dec_notes = crypto_vault.decrypt(raw["encrypted_notes"], key) if raw["encrypted_notes"] else ""
    except Exception:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="Erreur lors du déchiffrement du secret.")

    ip = request.client.host if request.client else "unknown"
    db_service.log_audit(user["id"], "credential_accessed", target_title=raw["title"], ip_address=ip, status="success", details="Mot de passe déchiffré pour consultation")

    return CredentialDetail(
        id=raw["id"],
        user_id=raw["user_id"],
        title=raw["title"],
        url=raw["url"],
        username=raw["username"],
        password=dec_password,
        notes=dec_notes,
        category=raw["category"],
        tags=raw["tags"],
        created_at=raw["created_at"],
        updated_at=raw["updated_at"],
    )


@router.put("/{cred_id}", response_model=CredentialListItem)
async def update_credential(cred_id: str, req: CredentialUpdate, request: Request, user: dict = Depends(get_current_user)):
    """Met à jour un identifiant avec rechiffrement AES-256-GCM."""
    raw = db_service.get_credential_by_id(cred_id, user["id"])
    if not raw:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Identifiant introuvable.")

    key = get_user_encryption_key(user)

    if req.password:
        enc_password = crypto_vault.encrypt(req.password, key)
    else:
        enc_password = raw["encrypted_password"]

    if req.notes is not None:
        enc_notes = crypto_vault.encrypt(req.notes, key) if req.notes else ""
    else:
        enc_notes = raw["encrypted_notes"]

    success = db_service.update_credential(
        cred_id=cred_id,
        user_id=user["id"],
        title=req.title,
        url=req.url or "",
        username=req.username,
        enc_password=enc_password,
        enc_notes=enc_notes,
        category=req.category or "Général",
        tags=req.tags or "",
    )

    if not success:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Échec de la mise à jour.")

    ip = request.client.host if request.client else "unknown"
    db_service.log_audit(user["id"], "credential_updated", target_title=req.title, ip_address=ip, status="success")

    updated = db_service.get_credential_by_id(cred_id, user["id"])
    return updated


@router.delete("/{cred_id}", status_code=status.HTTP_200_OK)
async def delete_credential(cred_id: str, request: Request, user: dict = Depends(get_current_user)):
    """Supprime définitivement un identifiant du coffre."""
    raw = db_service.get_credential_by_id(cred_id, user["id"])
    if not raw:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Identifiant introuvable.")

    db_service.delete_credential(cred_id, user["id"])

    ip = request.client.host if request.client else "unknown"
    db_service.log_audit(user["id"], "credential_deleted", target_title=raw["title"], ip_address=ip, status="success")

    return {"message": "Identifiant supprimé du coffre avec succès."}

