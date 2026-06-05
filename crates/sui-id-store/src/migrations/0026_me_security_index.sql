-- Migration 0026 — index on users.preferred_lang for /me/security/language (RFC 040)
CREATE INDEX IF NOT EXISTS idx_users_preferred_lang
  ON users(preferred_lang)
  WHERE preferred_lang IS NOT NULL;
