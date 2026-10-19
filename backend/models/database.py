"""
backend/models/database.py - Gestionnaire de base de données SQLite pour P2TSecret.
"""
import os
import sqlite3
import uuid
from datetime import datetime, timezone
from typing import Optional, Dict, Any, List

DB_PATH = os.getenv("P2T_SECRET_DB_PATH", os.path.abspath(os.path.join(os.path.dirname(__file__), "../../data/p2tsecret.db")))


def get_db_connection():
    """Crée une connexion SQLite thread-safe avec row_factory."""
    os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)
    conn = sqlite3.connect(DB_PATH, timeout=10.0, check_same_thread=False)
    conn.row_factory = sqlite3.Row
    # Activer WAL pour de meilleures performances et concurrence
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute("PRAGMA foreign_keys=ON;")
    return conn


def init_database():
    """Initialise le schéma de base de données s'il n'existe pas."""
    conn = get_db_connection()
    try:
        with conn:
            # 1. Table Utilisateurs
            conn.execute("""
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                hashed_password TEXT NOT NULL,
                salt_hex TEXT NOT NULL,
                totp_secret_base32 TEXT,
                is_2fa_enabled INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );
            """)

            # 2. Table Codes de Secours (Backup recovery codes)
            conn.execute("""
            CREATE TABLE IF NOT EXISTS recovery_codes (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                hashed_code TEXT NOT NULL,
                used INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                used_at TEXT,
                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
            );
            """)

            # 3. Table des Identifiants du Coffre-Fort (Credentials)
            conn.execute("""
            CREATE TABLE IF NOT EXISTS credentials (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT,
                username TEXT NOT NULL,
                encrypted_password TEXT NOT NULL,
                encrypted_notes TEXT,
                category TEXT DEFAULT 'Général',
                tags TEXT DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
            );
            """)

            # 4. Table des Journaux d'Audit (Audit Logs)
            conn.execute("""
            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                action TEXT NOT NULL,
                target_title TEXT,
                ip_address TEXT,
                status TEXT NOT NULL,
                details TEXT,
                timestamp TEXT NOT NULL
            );
            """)
    finally:
        conn.close()


class DatabaseService:
    """Opérations CRUD de base de données pour P2TSecret."""

    @staticmethod
    def get_user_by_username(username: str) -> Optional[Dict[str, Any]]:
        conn = get_db_connection()
        try:
            cur = conn.execute("SELECT * FROM users WHERE username = ? COLLATE NOCASE", (username,))
            row = cur.fetchone()
            return dict(row) if row else None
        finally:
            conn.close()

    @staticmethod
    def get_user_by_id(user_id: str) -> Optional[Dict[str, Any]]:
        conn = get_db_connection()
        try:
            cur = conn.execute("SELECT * FROM users WHERE id = ?", (user_id,))
            row = cur.fetchone()
            return dict(row) if row else None
        finally:
            conn.close()

    @staticmethod
    def create_user(username: str, hashed_password: str, salt_hex: str) -> Dict[str, Any]:
        user_id = str(uuid.uuid4())
        created_at = datetime.now(timezone.utc).isoformat()
        conn = get_db_connection()
        try:
            with conn:
                conn.execute(
                    "INSERT INTO users (id, username, hashed_password, salt_hex, is_2fa_enabled, created_at) VALUES (?, ?, ?, ?, 0, ?)",
                    (user_id, username, hashed_password, salt_hex, created_at),
                )
            return {"id": user_id, "username": username, "is_2fa_enabled": False, "created_at": created_at}
        finally:
            conn.close()

    @staticmethod
    def set_user_totp_secret(user_id: str, secret_base32: str):
        conn = get_db_connection()
        try:
            with conn:
                conn.execute("UPDATE users SET totp_secret_base32 = ? WHERE id = ?", (secret_base32, user_id))
        finally:
            conn.close()

    @staticmethod
    def enable_user_2fa(user_id: str, secret_base32: str, hashed_backup_codes: List[str]):
        now = datetime.now(timezone.utc).isoformat()
        conn = get_db_connection()
        try:
            with conn:
                conn.execute("UPDATE users SET is_2fa_enabled = 1, totp_secret_base32 = ? WHERE id = ?", (secret_base32, user_id))
                # Supprimer les anciens codes de secours
                conn.execute("DELETE FROM recovery_codes WHERE user_id = ?", (user_id,))
                for h in hashed_backup_codes:
                    code_id = str(uuid.uuid4())
                    conn.execute(
                        "INSERT INTO recovery_codes (id, user_id, hashed_code, used, created_at) VALUES (?, ?, ?, 0, ?)",
                        (code_id, user_id, h, now),
                    )
        finally:
            conn.close()

    @staticmethod
    def consume_backup_code(user_id: str, raw_code: str) -> bool:
        """Tente de consommer un code de secours."""
        import hashlib
        import secrets
        h = hashlib.sha256(raw_code.strip().upper().encode("utf-8")).hexdigest()
        now = datetime.now(timezone.utc).isoformat()
        conn = get_db_connection()
        try:
            with conn:
                cur = conn.execute("SELECT id, hashed_code FROM recovery_codes WHERE user_id = ? AND used = 0", (user_id,))
                rows = cur.fetchall()
                for r in rows:
                    if secrets.compare_digest(r["hashed_code"], h):
                        conn.execute("UPDATE recovery_codes SET used = 1, used_at = ? WHERE id = ?", (now, r["id"]))
                        return True
            return False
        finally:
            conn.close()

    @staticmethod
    def list_credentials(user_id: str) -> List[Dict[str, Any]]:
        conn = get_db_connection()
        try:
            cur = conn.execute(
                "SELECT id, user_id, title, url, username, category, tags, created_at, updated_at FROM credentials WHERE user_id = ? ORDER BY title COLLATE NOCASE ASC",
                (user_id,),
            )
            return [dict(r) for r in cur.fetchall()]
        finally:
            conn.close()

    @staticmethod
    def get_credential_by_id(cred_id: str, user_id: str) -> Optional[Dict[str, Any]]:
        conn = get_db_connection()
        try:
            cur = conn.execute("SELECT * FROM credentials WHERE id = ? AND user_id = ?", (cred_id, user_id))
            row = cur.fetchone()
            return dict(row) if row else None
        finally:
            conn.close()

    @staticmethod
    def create_credential(user_id: str, title: str, url: str, username: str, enc_password: str, enc_notes: str, category: str, tags: str) -> Dict[str, Any]:
        cred_id = str(uuid.uuid4())
        now = datetime.now(timezone.utc).isoformat()
        conn = get_db_connection()
        try:
            with conn:
                conn.execute(
                    """
                    INSERT INTO credentials (id, user_id, title, url, username, encrypted_password, encrypted_notes, category, tags, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (cred_id, user_id, title, url, username, enc_password, enc_notes, category, tags, now, now),
                )
            return {
                "id": cred_id,
                "user_id": user_id,
                "title": title,
                "url": url,
                "username": username,
                "category": category,
                "tags": tags,
                "created_at": now,
                "updated_at": now,
            }
        finally:
            conn.close()

    @staticmethod
    def update_credential(cred_id: str, user_id: str, title: str, url: str, username: str, enc_password: str, enc_notes: str, category: str, tags: str) -> bool:
        now = datetime.now(timezone.utc).isoformat()
        conn = get_db_connection()
        try:
            with conn:
                cur = conn.execute(
                    """
                    UPDATE credentials
                    SET title = ?, url = ?, username = ?, encrypted_password = ?, encrypted_notes = ?, category = ?, tags = ?, updated_at = ?
                    WHERE id = ? AND user_id = ?
                    """,
                    (title, url, username, enc_password, enc_notes, category, tags, now, cred_id, user_id),
                )
                return cur.rowcount > 0
        finally:
            conn.close()

    @staticmethod
    def delete_credential(cred_id: str, user_id: str) -> bool:
        conn = get_db_connection()
        try:
            with conn:
                cur = conn.execute("DELETE FROM credentials WHERE id = ? AND user_id = ?", (cred_id, user_id))
                return cur.rowcount > 0
        finally:
            conn.close()

    @staticmethod
    def log_audit(user_id: Optional[str], action: str, target_title: Optional[str] = None, ip_address: Optional[str] = None, status: str = "success", details: Optional[str] = None):
        log_id = str(uuid.uuid4())
        now = datetime.now(timezone.utc).isoformat()
        conn = get_db_connection()
        try:
            with conn:
                conn.execute(
                    "INSERT INTO audit_logs (id, user_id, action, target_title, ip_address, status, details, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    (log_id, user_id, action, target_title, ip_address, status, details, now),
                )
        finally:
            conn.close()

    @staticmethod
    def get_recent_audit_logs(user_id: str, limit: int = 50) -> List[Dict[str, Any]]:
        conn = get_db_connection()
        try:
            cur = conn.execute(
                "SELECT * FROM audit_logs WHERE user_id = ? ORDER BY timestamp DESC LIMIT ?",
                (user_id, limit),
            )
            return [dict(r) for r in cur.fetchall()]
        finally:
            conn.close()


db_service = DatabaseService()
