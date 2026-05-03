-- 0015: Password-reset tokens.
--
-- Backs the forgot-password flow added in v0.22.0:
--
--   1. User POSTs `/forgot-password` with their email.
--   2. If a user with that email exists, sui-id generates a fresh
--      32-byte random token, stores its SHA-256 hash here, and
--      mails the user a link `/reset-password?token=<base64>`.
--      The plaintext token never touches the database.
--   3. User clicks the link. sui-id hashes the parameter and looks
--      up this table. If the row exists, hasn't been consumed, and
--      hasn't expired, the user is shown the new-password form.
--   4. User POSTs the new password. sui-id verifies the token row
--      one more time, updates the password, marks the row consumed.
--
-- ## Why hash the token
--
-- A leak of the database (e.g. via backup) must not allow an
-- attacker to redeem live reset tokens. SHA-256 is sufficient
-- here: the underlying token is 32 bytes of CSPRNG output, so
-- there is no realistic dictionary attack — we only need
-- preimage resistance, which SHA-256 provides at the same cost
-- as Argon2 for inputs that are themselves uniformly random.
--
-- ## Single-use, time-limited
--
-- - `consumed_at` is NULL until the token is redeemed. After
--   redemption the row is left in place (not deleted) so a
--   replay attempt sees a consumed row and is rejected with a
--   neutral error rather than a "token not found" that leaks
--   timing.
-- - `expires_at` is `issued_at + 30 minutes` by default
--   (configurable; see `forgot_password::DEFAULT_TOKEN_TTL`).
--   30 minutes is the typical reset-link lifetime for
--   user-friendly delivery delays without leaving a wide window
--   of attack.
--
-- ## Audit trail
--
-- - `requester_ip` is the source IP of the `/forgot-password`
--   POST, copied for after-the-fact incident review (e.g.
--   "did all these tokens come from the same address?"). Not
--   indexed; it's a forensic field, not a query field.
-- - `user_id` is FK with `ON DELETE CASCADE`: deleting a user
--   should drop their pending reset tokens.
--
-- ## Indices
--
-- - `idx_password_reset_tokens_token_hash`: the hot path is
--   `WHERE token_hash = ? AND consumed_at IS NULL`. The unique
--   constraint also enforces that a token (after hashing) can
--   only ever appear once.
-- - `idx_password_reset_tokens_expires_at`: lets a periodic GC
--   prune expired rows efficiently.

CREATE TABLE password_reset_tokens (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash   BLOB NOT NULL,
    issued_at    TEXT NOT NULL,
    expires_at   TEXT NOT NULL,
    consumed_at  TEXT,
    requester_ip TEXT
) STRICT;

CREATE UNIQUE INDEX idx_password_reset_tokens_token_hash
    ON password_reset_tokens (token_hash);

CREATE INDEX idx_password_reset_tokens_expires_at
    ON password_reset_tokens (expires_at);
