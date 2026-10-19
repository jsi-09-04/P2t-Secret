"""
backend/main.py - Point d'entrée principal de l'API FastAPI P2TSecret.
"""
import os
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse

from backend.models.database import init_database
from backend.routers import auth, vault


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Démarrage : initialisation de la base SQLite
    init_database()
    yield


app = FastAPI(
    title="P2TSecret API",
    description="Application sécurisée de gestion des identifiants avec 2FA Microsoft Authenticator & chiffrement AES-256-GCM",
    version="1.0.0",
    lifespan=lifespan,
)

# CORS Middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Montage des routeurs API
app.include_router(auth.router, prefix="/api/v1")
app.include_router(vault.router, prefix="/api/v1")


@app.get("/api/health")
async def health_check():
    return {
        "status": "healthy",
        "service": "P2TSecret",
        "security": "AES-256-GCM + TOTP RFC 6238",
        "two_factor": "Microsoft Authenticator Compatible",
    }


# Service des fichiers statiques du Frontend
frontend_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../frontend"))
if os.path.isdir(frontend_dir):
    app.mount("/static", StaticFiles(directory=frontend_dir), name="static")

    @app.get("/")
    async def serve_index():
        index_file = os.path.join(frontend_dir, "index.html")
        if os.path.exists(index_file):
            return FileResponse(index_file)
        return {"message": "P2TSecret API en ligne. Consultez /docs"}
