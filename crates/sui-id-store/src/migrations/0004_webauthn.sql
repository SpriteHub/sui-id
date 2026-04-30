-- 0004: WebAuthn / passkey support.
--
-- ## users.user_uuid
--
-- WebAuthn requires a stable per-user UUID handle. We can't reuse the
-- `users.id` because that's a sui-id `UserId` (also a UUID, but tied to
-- the typed-id scheme); the WebAuthn spec wants a `user.id` as raw
-- bytes that the relying party assigns. We add a separate column so the
-- two stay decoupled — if a user's WebAuthn handle ever needs to be
-- rotated (it shouldn't, but) the typed user id stays as is.
--
-- Backfill: existing rows get a fresh UUID v4 via a trigger-free
-- one-shot UPDATE in the migration below.
ALTER TABLE users ADD COLUMN user_uuid TEXT NOT NULL DEFAULT '';
UPDATE users SET user_uuid = lower(hex(randomblob(4))) || '-' ||
                              lower(hex(randomblob(2))) || '-4' ||
                              substr(lower(hex(randomblob(2))), 2) || '-' ||
                              substr('89ab', 1 + (abs(random()) % 4), 1) ||
                              substr(lower(hex(randomblob(2))), 2) || '-' ||
                              lower(hex(randomblob(6)))
              WHERE user_uuid = '';

-- ## user_webauthn_credentials
--
-- One row per registered passkey. A user may have multiple — security
-- key + platform authenticator + recovery yubikey. Each row stores the
-- whole `webauthn_rs::prelude::Passkey` value sealed under the master
-- key; that struct contains the public key, signature counter, and
-- attestation metadata, and webauthn-rs is the canonical interpreter.
--
-- `credential_id` is the raw passkey credential id (the byte string the
-- authenticator hands back). We index it because authentication needs
-- to look up by it; the rest of the row is opaque to sui-id.
--
-- `nickname` lets the user label devices ("YubiKey 5C", "MacBook Touch
-- ID") so the credentials list page is meaningful.
CREATE TABLE IF NOT EXISTS user_webauthn_credentials (
    id             TEXT PRIMARY KEY,
    user_id        TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id  BLOB NOT NULL UNIQUE,
    passkey_enc    BLOB NOT NULL,
    nickname       TEXT NOT NULL,
    created_at     TEXT NOT NULL,
    last_used_at   TEXT
) STRICT;

CREATE INDEX IF NOT EXISTS idx_user_webauthn_user_id
    ON user_webauthn_credentials(user_id);

-- ## webauthn_pending
--
-- Short-lived (5-minute) holding table for in-flight WebAuthn
-- ceremonies. webauthn-rs returns `PasskeyRegistration` /
-- `PasskeyAuthentication` values that the relying party must remember
-- between the two halves of a registration or login challenge. We
-- serialise them to JSON and store here, keyed by an opaque id we hand
-- back to the browser as a cookie.
--
-- The `kind` column is `'register'` for an enrolment ceremony and
-- `'authenticate'` for a login ceremony. `user_id` is `NULL` for
-- authentication ceremonies that don't yet know which user (we don't
-- expose this code path today, but the column shape allows for it).
CREATE TABLE IF NOT EXISTS webauthn_pending (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL CHECK (kind IN ('register', 'authenticate')),
    user_id     TEXT REFERENCES users(id) ON DELETE CASCADE,
    state_json  TEXT NOT NULL,
    expires_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_webauthn_pending_expires_at
    ON webauthn_pending(expires_at);
