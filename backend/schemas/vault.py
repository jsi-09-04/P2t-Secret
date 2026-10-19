"""
backend/schemas/vault.py - Schémas Pydantic pour le coffre-fort et les mots de passe.
"""
from typing import Optional, List, Dict, Any
from pydantic import BaseModel, Field


class CredentialCreate(BaseModel):
    title: str = Field(..., min_length=1, max_length=100)
    url: Optional[str] = Field(None, max_length=255)
    username: str = Field(..., min_length=1, max_length=100)
    password: str = Field(..., min_length=1, max_length=500)
    notes: Optional[str] = Field(None, max_length=2000)
    category: Optional[str] = Field("Général", max_length=50)
    tags: Optional[str] = Field("", max_length=200)


class CredentialUpdate(BaseModel):
    title: str = Field(..., min_length=1, max_length=100)
    url: Optional[str] = Field(None, max_length=255)
    username: str = Field(..., min_length=1, max_length=100)
    password: Optional[str] = Field(None, max_length=500)
    notes: Optional[str] = Field(None, max_length=2000)
    category: Optional[str] = Field("Général", max_length=50)
    tags: Optional[str] = Field("", max_length=200)


class CredentialListItem(BaseModel):
    id: str
    user_id: str
    title: str
    url: Optional[str]
    username: str
    category: str
    tags: str
    created_at: str
    updated_at: str


class CredentialDetail(BaseModel):
    id: str
    user_id: str
    title: str
    url: Optional[str]
    username: str
    password: str
    notes: Optional[str]
    category: str
    tags: str
    created_at: str
    updated_at: str


class GeneratePasswordRequest(BaseModel):
    length: int = Field(20, ge=8, le=128)
    use_upper: bool = True
    use_lower: bool = True
    use_digits: bool = True
    use_symbols: bool = True
    exclude_ambiguous: bool = True


class GeneratePasswordResponse(BaseModel):
    password: str
    info: Dict[str, Any]


class AuditLogItem(BaseModel):
    id: str
    action: str
    target_title: Optional[str]
    status: str
    details: Optional[str]
    timestamp: str
