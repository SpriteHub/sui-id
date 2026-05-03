-- 0013: WebAuthn step-up support.
--
-- v0.21.1 adds WebAuthn as a step-up factor — until now step-up only
-- accepted TOTP / recovery-code input via `step_up::verify_totp_code`.
-- A user with passkeys but no TOTP enrolled should be able to prove
-- a fresh strong factor for sensitive admin actions just as cleanly.
--
-- The webauthn_pending table already has a `kind` discriminator with
-- a CHECK constraint listing the two values it knew about ('register'
-- and 'authenticate'). Step-up is a *third* context: same low-level
-- assertion as login-time authenticate, but the success path is
-- "touch this session's last_step_up_at" rather than "promote a
-- pending login row to a session". We tag the pending row with a
-- distinct kind so a step-up ceremony can never be misused as a
-- login-MFA verification (and vice versa) even by accident.
--
-- ## Why a new kind value rather than reusing 'authenticate'
--
-- The pending rows are short-lived (5 minutes) and per-session-bound
-- by the `session_id` cookie that ferries the pending_id back, so
-- abuse would already require session theft. But:
-- - reusing the same kind would force any reader of the table to
--   know "is this pending row for a login-MFA promotion or a
--   step-up touch?" by *looking elsewhere*, which is the kind of
--   ambient knowledge that drifts;
-- - the parser layer (WebauthnPendingKind::parse) is the only place
--   that needs touching here, and the audit log distinguishing
--   `auth.mfa.success` from `auth.step_up.success` already wants
--   the two flows separated upstream.
--
-- ## CHECK constraint update
--
-- SQLite doesn't support ALTER TABLE ... DROP / ADD CHECK directly,
-- so the rebuild dance: copy data into a new table that has the
-- looser constraint, drop the old, rename the new. Indices have to
-- be recreated on the new table — they don't survive the rename.
-- The pending rows themselves are short-lived (5-minute TTL) and
-- are routinely empty, so we don't lose anything if a few rows
-- slip through during the migration: the worst case is one
-- in-flight WebAuthn ceremony has to be retried.

CREATE TABLE webauthn_pending_new (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL CHECK (kind IN ('register', 'authenticate', 'step_up')),
    user_id     TEXT REFERENCES users(id) ON DELETE CASCADE,
    state_json  TEXT NOT NULL,
    expires_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL
) STRICT;

INSERT INTO webauthn_pending_new (id, kind, user_id, state_json, expires_at, created_at)
SELECT id, kind, user_id, state_json, expires_at, created_at
  FROM webauthn_pending;

DROP TABLE webauthn_pending;
ALTER TABLE webauthn_pending_new RENAME TO webauthn_pending;

CREATE INDEX IF NOT EXISTS idx_webauthn_pending_expires_at
    ON webauthn_pending(expires_at);
