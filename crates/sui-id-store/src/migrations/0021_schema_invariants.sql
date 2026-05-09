-- Migration 0021: Schema invariant CHECKs and improvements (RFC 021)
--
-- Bundles five schema improvements:
--
-- § 1. Boolean CHECKs on INTEGER columns that encode booleans. SQLite
--      does not support adding CHECK constraints to existing tables;
--      affected tables are rebuilt (CREATE new + INSERT SELECT + DROP +
--      RENAME). All existing indices on the rebuilt tables are
--      re-created on the new tables.
-- § 2. clients.confidential ↔ secret_hash consistency CHECK. Bundled
--      inside the clients rebuild from § 1.
-- § 3. signing_keys single-active partial unique index. After the index
--      lands, rotation must retire the current key before inserting the
--      new one (the application code in admin.rs::rotate_signing_key
--      was updated in the same changeset).
-- § 4. consents rebuild: proper FKs, composite PK, space-separated
--      granted_scopes column, updated_at. The table has no production
--      consumers yet (RFC 008 third-party-posture is the consumer), so
--      a DROP-then-CREATE is safe.
-- § 7. Better sessions index for the active-and-not-expired query shape.
--      The existing idx_sessions_user_active is retained for FIFO
--      eviction; the new index covers the expires_at filter too.
--
-- Pre-flight queries (run before upgrading, see docs/operators.md):
--
--   SELECT 'users.is_admin' col, count(*) bad
--     FROM users WHERE is_admin NOT IN (0,1)
--   UNION ALL
--   SELECT 'users.is_disabled', count(*) FROM users WHERE is_disabled NOT IN (0,1)
--   UNION ALL
--   SELECT 'users.is_deleted',  count(*) FROM users WHERE is_deleted  NOT IN (0,1)
--   UNION ALL
--   SELECT 'credentials.must_change', count(*)
--     FROM credentials WHERE must_change NOT IN (0,1)
--   UNION ALL
--   SELECT 'clients.confidential', count(*)
--     FROM clients WHERE confidential NOT IN (0,1)
--   UNION ALL
--   SELECT 'clients.is_disabled', count(*)
--     FROM clients WHERE is_disabled NOT IN (0,1)
--   UNION ALL
--   SELECT 'clients.is_deleted', count(*)
--     FROM clients WHERE is_deleted NOT IN (0,1)
--   UNION ALL
--   SELECT 'signing_keys.is_active', count(*)
--     FROM signing_keys WHERE is_active NOT IN (0,1);
--
--   SELECT id FROM clients
--    WHERE (confidential = 1 AND secret_hash IS NULL)
--       OR (confidential = 0 AND secret_hash IS NOT NULL);
--
--   SELECT count(*) FROM signing_keys WHERE is_active = 1;
--   -- expected: 0 or 1


PRAGMA foreign_keys = OFF;

-- ─── § 1 + § 2: users rebuild ──────────────────────────────────────────────

CREATE TABLE _users_new (
    id           TEXT PRIMARY KEY,
    username     TEXT NOT NULL UNIQUE,
    display_name TEXT,
    is_admin     INTEGER NOT NULL DEFAULT 0
                     CHECK (is_admin     IN (0, 1)),
    is_disabled  INTEGER NOT NULL DEFAULT 0
                     CHECK (is_disabled  IN (0, 1)),
    is_deleted   INTEGER NOT NULL DEFAULT 0
                     CHECK (is_deleted   IN (0, 1)),
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    user_uuid    TEXT NOT NULL DEFAULT '',
    failed_login_count  INTEGER NOT NULL DEFAULT 0,
    locked_until        TEXT,
    email               TEXT,
    preferred_lang      TEXT,
    email_normalized    TEXT,
    email_verified_at   TEXT
);

INSERT INTO _users_new
SELECT id, username, display_name,
       CASE WHEN is_admin    NOT IN (0,1) THEN 0 ELSE is_admin    END,
       CASE WHEN is_disabled NOT IN (0,1) THEN 0 ELSE is_disabled END,
       CASE WHEN is_deleted  NOT IN (0,1) THEN 0 ELSE is_deleted  END,
       created_at, updated_at, user_uuid,
       failed_login_count, locked_until, email, preferred_lang,
       email_normalized, email_verified_at
FROM users;

DROP TABLE users;
ALTER TABLE _users_new RENAME TO users;

-- Re-create indices from migrations 0004, 0012, 0020.
CREATE UNIQUE INDEX idx_users_user_uuid
    ON users(user_uuid) WHERE user_uuid <> '';
CREATE UNIQUE INDEX idx_users_email_normalized
    ON users(email_normalized) WHERE email_normalized IS NOT NULL;

-- ─── credentials rebuild ───────────────────────────────────────────────────

CREATE TABLE _credentials_new (
    user_id      TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    must_change  INTEGER NOT NULL DEFAULT 0
                     CHECK (must_change IN (0, 1)),
    updated_at   TEXT NOT NULL
);

INSERT INTO _credentials_new
SELECT user_id, password_hash,
       CASE WHEN must_change NOT IN (0,1) THEN 0 ELSE must_change END,
       updated_at
FROM credentials;

DROP TABLE credentials;
ALTER TABLE _credentials_new RENAME TO credentials;

-- ─── clients rebuild (§ 1 booleans + § 2 confidential/secret_hash) ─────────

CREATE TABLE _clients_new (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    confidential INTEGER NOT NULL
                     CHECK (confidential IN (0, 1)),
    secret_hash  TEXT,
    redirect_uris               TEXT NOT NULL,
    is_disabled  INTEGER NOT NULL DEFAULT 0
                     CHECK (is_disabled  IN (0, 1)),
    is_deleted   INTEGER NOT NULL DEFAULT 0
                     CHECK (is_deleted   IN (0, 1)),
    allowed_scopes              TEXT NOT NULL DEFAULT '',
    post_logout_redirect_uris   TEXT NOT NULL DEFAULT '[]',
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    -- Confidential clients must have a secret; public clients must not.
    CHECK (
        (confidential = 1 AND secret_hash IS NOT NULL) OR
        (confidential = 0 AND secret_hash IS NULL)
    )
);

INSERT INTO _clients_new
SELECT id, name,
       CASE WHEN confidential NOT IN (0,1) THEN 0 ELSE confidential END,
       secret_hash, redirect_uris,
       CASE WHEN is_disabled  NOT IN (0,1) THEN 0 ELSE is_disabled  END,
       CASE WHEN is_deleted   NOT IN (0,1) THEN 0 ELSE is_deleted   END,
       allowed_scopes, post_logout_redirect_uris, created_at, updated_at
FROM clients;

DROP TABLE clients;
ALTER TABLE _clients_new RENAME TO clients;

-- ─── signing_keys rebuild ──────────────────────────────────────────────────

CREATE TABLE _signing_keys_new (
    id               TEXT PRIMARY KEY,
    algorithm        TEXT NOT NULL,
    private_key_enc  BLOB NOT NULL,
    public_key       BLOB NOT NULL,
    is_active        INTEGER NOT NULL DEFAULT 0
                         CHECK (is_active IN (0, 1)),
    created_at       TEXT NOT NULL,
    rotated_at       TEXT
);

INSERT INTO _signing_keys_new
SELECT id, algorithm, private_key_enc, public_key,
       CASE WHEN is_active NOT IN (0,1) THEN 0 ELSE is_active END,
       created_at, rotated_at
FROM signing_keys;

DROP TABLE signing_keys;
ALTER TABLE _signing_keys_new RENAME TO signing_keys;

-- § 3: Enforce "at most one active key" with a partial unique index.
-- The rotation code (admin.rs::rotate_signing_key) retires the current key
-- before inserting the new one to avoid briefly violating this constraint.
CREATE UNIQUE INDEX idx_signing_keys_single_active
    ON signing_keys(is_active) WHERE is_active = 1;

-- ─── user_totp rebuild ─────────────────────────────────────────────────────
-- Adds CHECK (enabled IN (0, 1)). Exact column set mirrors migration 0003,
-- which created the table in STRICT mode (type enforcement already present
-- there; we drop STRICT here since CHECK is sufficient and non-STRICT
-- tables are more compatible with the ALTER TABLE steps elsewhere in this
-- migration).

CREATE TABLE _user_totp_new (
    user_id            TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    secret_enc         BLOB NOT NULL,
    enabled            INTEGER NOT NULL DEFAULT 0
                           CHECK (enabled IN (0, 1)),
    recovery_codes_enc BLOB,
    last_used_step     INTEGER NOT NULL DEFAULT 0,
    created_at         TEXT NOT NULL,
    confirmed_at       TEXT
);

INSERT INTO _user_totp_new
SELECT user_id, secret_enc,
       CASE WHEN enabled NOT IN (0,1) THEN 0 ELSE enabled END,
       recovery_codes_enc, last_used_step, created_at, confirmed_at
FROM user_totp;

DROP TABLE user_totp;
ALTER TABLE _user_totp_new RENAME TO user_totp;

-- ─── § 4: consents redesign ────────────────────────────────────────────────
-- Drop the old table (no production consumers; RFC 008 will build on this
-- shape). Re-create with FKs, composite PK, and space-separated scopes.

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

-- ─── § 7: sessions index improvement ──────────────────────────────────────
-- Keep the existing idx_sessions_user_active for FIFO eviction queries
-- (ORDER BY created_at without an expires_at filter). Add a second index
-- that covers the hot "active and alive" query path, which filters on
-- user_id, revoked_at IS NULL, and expires_at.
CREATE INDEX idx_sessions_user_active_alive
    ON sessions(user_id, expires_at, created_at)
    WHERE revoked_at IS NULL;

PRAGMA foreign_keys = ON;
