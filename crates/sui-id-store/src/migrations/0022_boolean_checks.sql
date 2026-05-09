-- MIGRATION:FK_DISABLE_REQUIRED
-- Migration 0022: Boolean CHECK constraints via safe table evacuation
--
-- This migration adds the boolean CHECK constraints that were deferred in
-- migration 0021 (v0.29.8) because the naive DROP + RENAME approach would
-- have triggered ON DELETE CASCADE on child tables.
--
-- SAFE EVACUATION TECHNIQUE
-- ─────────────────────────
-- The migration runner (migrations.rs) detects the FK_DISABLE_REQUIRED
-- marker above and sets `PRAGMA foreign_keys = OFF` BEFORE starting the
-- transaction that wraps this SQL. Unlike PRAGMAs inside a transaction
-- (which are no-ops per the SQLite docs), a PRAGMA set before the
-- transaction takes effect for the entire transaction.
--
-- The result: DROP TABLE <parent> does not fire ON DELETE CASCADE on child
-- tables. Child tables keep their FK constraint declarations, and once the
-- renamed parent is in place they automatically re-attach.
--
-- After COMMIT, the runner re-enables FK enforcement and runs
-- `PRAGMA foreign_key_check`. An FK violation aborts startup with an error.
--
-- PRE-FLIGHT REQUIRED
-- ───────────────────
-- Run docs/operators/preflight-0022.sql before upgrading. If any row has
-- a boolean column value outside {0, 1}, the INSERT ... SELECT below will
-- fail on the CHECK constraint and the migration will abort (which is the
-- correct behaviour — better than a silent data change).
--
-- TABLES REBUILT
-- ──────────────
-- § 1  users         — is_admin, is_disabled, is_deleted CHECK + user_uuid length CHECK
-- § 2  credentials   — must_change CHECK
-- § 3  clients       — confidential, is_disabled, is_deleted CHECK
--                      + confidential ↔ secret_hash consistency CHECK
-- § 4  signing_keys  — is_active CHECK (partial-unique index re-created)
-- § 5  user_totp     — enabled CHECK (STRICT mode preserved from migration 0003)
--
-- NOTE: PRAGMA statements are intentionally absent from this SQL.
-- The runner sets PRAGMA foreign_keys = OFF before BEGIN and
-- PRAGMA foreign_keys = ON after COMMIT.

-- ─── Pre-step: backfill any empty user_uuid ───────────────────────────────
-- Migration 0004 already backfilled these, but defensive handling ensures
-- the CHECK (length = 36) below does not cause an unexpected failure.

UPDATE users
   SET user_uuid = lower(hex(randomblob(4))) || '-' ||
                   lower(hex(randomblob(2))) || '-4' ||
                   substr(lower(hex(randomblob(2))), 2) || '-' ||
                   substr('89ab', 1 + (abs(random()) % 4), 1) ||
                   substr(lower(hex(randomblob(2))), 2) || '-' ||
                   lower(hex(randomblob(6)))
 WHERE user_uuid = '';

-- ─── § 1: users ───────────────────────────────────────────────────────────

CREATE TABLE _users_new (
    id                  TEXT    PRIMARY KEY,
    username            TEXT    NOT NULL UNIQUE,
    display_name        TEXT,
    is_admin            INTEGER NOT NULL DEFAULT 0
                                    CHECK (is_admin    IN (0, 1)),
    is_disabled         INTEGER NOT NULL DEFAULT 0
                                    CHECK (is_disabled IN (0, 1)),
    is_deleted          INTEGER NOT NULL DEFAULT 0
                                    CHECK (is_deleted  IN (0, 1)),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    -- user_uuid: 36-char UUID string (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx).
    -- The DEFAULT '' is removed; the CHECK enforces a proper UUID-length value.
    user_uuid           TEXT    NOT NULL
                                    CHECK (length(user_uuid) = 36),
    failed_login_count  INTEGER NOT NULL DEFAULT 0,
    locked_until        TEXT,
    email               TEXT,
    preferred_lang      TEXT,
    email_normalized    TEXT,
    email_verified_at   TEXT
);

-- Fail-fast: if any row violates a CHECK, the migration aborts here.
INSERT INTO _users_new
SELECT id, username, display_name,
       is_admin, is_disabled, is_deleted,
       created_at, updated_at, user_uuid,
       failed_login_count, locked_until,
       email, preferred_lang, email_normalized, email_verified_at
FROM users;

DROP TABLE users;
ALTER TABLE _users_new RENAME TO users;

-- Restore indices from migrations 0020.
CREATE UNIQUE INDEX idx_users_user_uuid
    ON users(user_uuid);
CREATE UNIQUE INDEX idx_users_email_normalized
    ON users(email_normalized)
    WHERE email_normalized IS NOT NULL;

-- ─── § 2: credentials ─────────────────────────────────────────────────────

CREATE TABLE _credentials_new (
    user_id       TEXT    PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    password_hash TEXT    NOT NULL,
    must_change   INTEGER NOT NULL DEFAULT 0
                              CHECK (must_change IN (0, 1)),
    updated_at    TEXT    NOT NULL
);

INSERT INTO _credentials_new
SELECT user_id, password_hash, must_change, updated_at
FROM credentials;

DROP TABLE credentials;
ALTER TABLE _credentials_new RENAME TO credentials;

-- ─── § 3: clients ─────────────────────────────────────────────────────────

CREATE TABLE _clients_new (
    id                          TEXT    PRIMARY KEY,
    name                        TEXT    NOT NULL,
    confidential                INTEGER NOT NULL
                                            CHECK (confidential IN (0, 1)),
    secret_hash                 TEXT,
    redirect_uris               TEXT    NOT NULL,
    is_disabled                 INTEGER NOT NULL DEFAULT 0
                                            CHECK (is_disabled  IN (0, 1)),
    is_deleted                  INTEGER NOT NULL DEFAULT 0
                                            CHECK (is_deleted   IN (0, 1)),
    allowed_scopes              TEXT    NOT NULL DEFAULT '',
    post_logout_redirect_uris   TEXT    NOT NULL DEFAULT '[]',
    created_at                  TEXT    NOT NULL,
    updated_at                  TEXT    NOT NULL,
    -- Confidential clients must have a secret hash; public clients must not.
    CHECK (
        (confidential = 1 AND secret_hash IS NOT NULL) OR
        (confidential = 0 AND secret_hash IS NULL)
    )
);

INSERT INTO _clients_new
SELECT id, name, confidential, secret_hash, redirect_uris,
       is_disabled, is_deleted, allowed_scopes, post_logout_redirect_uris,
       created_at, updated_at
FROM clients;

DROP TABLE clients;
ALTER TABLE _clients_new RENAME TO clients;

-- ─── § 4: signing_keys ────────────────────────────────────────────────────

CREATE TABLE _signing_keys_new (
    id               TEXT    PRIMARY KEY,
    algorithm        TEXT    NOT NULL,
    private_key_enc  BLOB    NOT NULL,
    public_key       BLOB    NOT NULL,
    is_active        INTEGER NOT NULL DEFAULT 0
                                 CHECK (is_active IN (0, 1)),
    created_at       TEXT    NOT NULL,
    rotated_at       TEXT
);

INSERT INTO _signing_keys_new
SELECT id, algorithm, private_key_enc, public_key,
       is_active, created_at, rotated_at
FROM signing_keys;

DROP TABLE signing_keys;
ALTER TABLE _signing_keys_new RENAME TO signing_keys;

-- Restore the partial-unique index added in migration 0021.
CREATE UNIQUE INDEX idx_signing_keys_single_active
    ON signing_keys(is_active) WHERE is_active = 1;

-- ─── § 5: user_totp ───────────────────────────────────────────────────────
-- STRICT mode is maintained (migration 0003 created the table as STRICT).

CREATE TABLE _user_totp_new (
    user_id            TEXT    PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    secret_enc         BLOB    NOT NULL,
    enabled            INTEGER NOT NULL DEFAULT 0
                                   CHECK (enabled IN (0, 1)),
    recovery_codes_enc BLOB,
    last_used_step     INTEGER NOT NULL DEFAULT 0,
    created_at         TEXT    NOT NULL,
    confirmed_at       TEXT
) STRICT;

INSERT INTO _user_totp_new
SELECT user_id, secret_enc, enabled, recovery_codes_enc,
       last_used_step, created_at, confirmed_at
FROM user_totp;

DROP TABLE user_totp;
ALTER TABLE _user_totp_new RENAME TO user_totp;
