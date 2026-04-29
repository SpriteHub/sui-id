-- 0003: TOTP MFA per user.
--
-- ## user_totp
--
-- One row per user that has TOTP either configured or activated.
-- `enabled = 0` means a secret has been allocated but the user has not
-- yet confirmed it with a verification code; `enabled = 1` means TOTP
-- is required at login.
--
-- `secret_enc` holds the raw TOTP secret (20 bytes, the RFC 6238
-- recommended size for HMAC-SHA1) sealed with the master key.
--
-- `recovery_codes_enc` holds a JSON array of Argon2id hashes (one per
-- single-use recovery code), itself sealed with the master key. We
-- seal the array because the encrypted column lets us atomically
-- rewrite "all codes" when the user regenerates them.
--
-- `last_used_step` stores the most recently accepted RFC 6238 time
-- step. A second submission with the same step is rejected — a basic
-- but effective replay defence in the 30-second granularity TOTP
-- already commits to.
--
-- ## login_pending_mfa
--
-- Holds short-lived "password verified, MFA pending" tokens. The user
-- gets a temporary cookie pointing here after a successful password
-- check; on submission of a TOTP code we promote the row into a real
-- `sessions` row and delete this one. Rows expire after 5 minutes
-- regardless.
CREATE TABLE IF NOT EXISTS user_totp (
    user_id            TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    secret_enc         BLOB NOT NULL,
    enabled            INTEGER NOT NULL DEFAULT 0,
    recovery_codes_enc BLOB,
    last_used_step     INTEGER NOT NULL DEFAULT 0,
    created_at         TEXT NOT NULL,
    confirmed_at       TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS login_pending_mfa (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_login_pending_mfa_expires_at
    ON login_pending_mfa(expires_at);
