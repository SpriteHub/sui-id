-- Migration 0021 (revised v0.29.8): Safe schema improvements — RFC 021
--
-- REVISION HISTORY
-- ─────────────────
-- v0.29.7 (original): Attempted to add boolean CHECK constraints by
--   rebuilding parent tables (users, credentials, clients, signing_keys,
--   user_totp) in-place with DROP TABLE + CREATE. This was unsafe:
--
--   SQLite documents that PRAGMA foreign_keys = OFF "is a no-op within
--   a transaction" (https://www.sqlite.org/pragma.html#pragma_foreign_keys).
--   Since each migration runs inside its own transaction (see migrations.rs),
--   the PRAGMA had no effect. DROP TABLE users therefore fired ON DELETE
--   CASCADE on all child tables, wiping credentials, sessions,
--   refresh_tokens, user_totp, and other rows for existing users.
--
-- v0.29.8 (this revision): Parent table rebuilds are deferred to a future
--   migration that will use a safe evacuation approach (rename children
--   out first, rebuild parent, restore children). The three safe changes
--   that do not require parent table drops are retained:
--
--   § 3 (index only): signing_keys single-active partial unique index.
--      No table rebuild; CREATE UNIQUE INDEX is additive and safe.
--   § 4: consents table redesign. consents is a child table with no
--      tables referencing it; DROP + CREATE is safe regardless of FK mode.
--   § 7: sessions active-alive query index. Additive, no rebuild.
--
-- Deferred to a future migration (boolean-check-safe.sql):
--   § 1: boolean CHECK on users, credentials, user_totp
--   § 2: clients.confidential ↔ secret_hash consistency CHECK
--   § 3 (CHECK only): signing_keys.is_active IN (0,1) CHECK

-- § 3 (index only): At most one active signing key ────────────────────────
-- The application rotation path (signing_keys::rotate_atomic) already
-- enforces the retire-then-insert order; this index makes the constraint
-- visible and enforceable at the DB layer independently.

CREATE UNIQUE INDEX IF NOT EXISTS idx_signing_keys_single_active
    ON signing_keys(is_active) WHERE is_active = 1;

-- § 4: Consents table redesign ────────────────────────────────────────────
-- consents has no tables with ON DELETE CASCADE pointing at it, so dropping
-- it cannot trigger cascades regardless of PRAGMA foreign_keys state.
-- The new schema adds proper FKs, a composite primary key, updated_at,
-- and a CHECK on granted_scopes.

DROP TABLE IF EXISTS consents;

CREATE TABLE consents (
    user_id        TEXT NOT NULL REFERENCES users(id)   ON DELETE CASCADE,
    client_id      TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    granted_scopes TEXT NOT NULL,
    granted_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    PRIMARY KEY (user_id, client_id),
    CHECK (length(granted_scopes) > 0)
);

-- § 7: Sessions active-alive query index ──────────────────────────────────
-- Supports the hot query shape: active sessions for a user that have not
-- yet expired. The existing idx_sessions_user_active is kept for FIFO
-- eviction queries that order by created_at without an expires_at filter.

CREATE INDEX IF NOT EXISTS idx_sessions_user_active_alive
    ON sessions(user_id, expires_at, created_at)
    WHERE revoked_at IS NULL;
