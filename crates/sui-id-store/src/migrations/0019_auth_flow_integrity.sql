-- Migration 0019: Auth flow data integrity hardening (RFC 019)
--
-- Three changes bundled in one migration because they share one
-- logical concern — closing gaps in the token-issuance path:
--
-- § 1. Rebuild auth_codes with ON DELETE CASCADE foreign keys to
--      users(id) and clients(id). SQLite cannot ADD FK constraints
--      to an existing table, so the table is dropped and recreated.
--      Auth codes have a 60-second TTL; any outstanding rows at
--      migration time are either expired or about to be exchanged,
--      so the drop is safe (equivalent to a restart lasting > 60s).
--
-- § 2. Add refresh_tokens.token_hash for indexed O(log n) lookup.
--      The previous design required decrypting every active row
--      until the correct plaintext was found — O(n) in active
--      refresh tokens. The index is partial (WHERE token_hash IS
--      NOT NULL) because existing rows are backfilled by a
--      background task after startup; their token_hash starts NULL.
--
-- The GC fix (§ 5 in the RFC) is code-only — no SQL change needed:
-- repos/refresh_tokens.rs::purge_expired already issues
-- `DELETE FROM refresh_tokens WHERE expires_at < ?1`
-- (no `OR revoked_at IS NOT NULL` clause). Verified before landing.

-- § 1: Rebuild auth_codes with foreign keys ---------------------------

PRAGMA foreign_keys = OFF;

DROP TABLE IF EXISTS auth_codes;

CREATE TABLE auth_codes (
    code_hash             TEXT PRIMARY KEY,
    client_id             TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    user_id               TEXT NOT NULL REFERENCES users(id)   ON DELETE CASCADE,
    redirect_uri          TEXT NOT NULL,
    scope                 TEXT NOT NULL,
    nonce                 TEXT,
    code_challenge        TEXT NOT NULL,
    code_challenge_method TEXT NOT NULL,
    expires_at            TEXT NOT NULL,
    consumed              INTEGER NOT NULL DEFAULT 0
                              CHECK (consumed IN (0, 1)),
    created_at            TEXT NOT NULL,
    auth_methods          TEXT NOT NULL DEFAULT '[]'
);

CREATE INDEX idx_auth_codes_expires ON auth_codes(expires_at);

PRAGMA foreign_keys = ON;

-- § 2: Add token_hash to refresh_tokens --------------------------------

ALTER TABLE refresh_tokens
    ADD COLUMN token_hash BLOB;

-- Partial unique index: NULL rows (not yet backfilled) are excluded.
-- Once the background backfill task has populated every row the
-- application invariant is effectively "NOT NULL"; the partial
-- predicate is retained to avoid a second rebuild migration.
CREATE UNIQUE INDEX idx_refresh_tokens_token_hash
    ON refresh_tokens(token_hash)
    WHERE token_hash IS NOT NULL;
