"""
backend/schemas/auth.py - Schémas Pydantic pour l'authentification et le 2FA.
"""
from typing import Optional, List
from pydantic import BaseModel, Field


class RegisterRequest(BaseModel):
    username: str = Field(..., min_length=3, max_length=50)
    password: str = Field(..., min_length=8, max_length=128)


class LoginRequest(BaseModel):
    username: str
    password: str


class Setup2FAResponse(BaseModel):
    secret: str
    otpauth_url: str
    qr_svg: str


class Confirm2FARequest(BaseModel):
    code: str = Field(..., min_length=6, max_length=6)


class Confirm2FAResponse(BaseModel):
    message: str
    backup_codes: List[str]
    access_token: str
    token_type: str = "bearer"


class Verify2FARequest(BaseModel):
    temp_token: str
    code: str = Field(..., min_length=6, max_length=6)


class Recovery2FARequest(BaseModel):
    temp_token: str
    recovery_code: str


class AuthTokenResponse(BaseModel):
    access_token: str
    token_type: str = "bearer"
    user_id: str
    username: str


class LoginResponse(BaseModel):
    requires_2fa: bool
    temp_token: Optional[str] = None
    access_token: Optional[str] = None
    user_id: Optional[str] = None
    username: Optional[str] = None
