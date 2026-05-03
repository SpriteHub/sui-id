-- 0014: SMTP configuration table.
--
-- v0.22.0 introduces email features (forgot-password reset and
-- password-change notification). The SMTP submission credentials
-- and connection parameters live in the database rather than in
-- `sui-id.toml` for several reasons (see the v0.22.0 CHANGELOG
-- entry for the full rationale):
--
-- - Operators can change settings without restarting the server,
--   which matters when troubleshooting delivery issues.
-- - The admin settings page can offer a "Test Connection" button
--   that runs a real EHLO/STARTTLS/AUTH dance and surfaces the
--   result inline.
-- - Credentials sit alongside the rest of our encrypted columns
--   (XChaCha20-Poly1305 sealing via the master key), so the
--   storage hardening is uniform.
-- - Setting changes feed the audit chain naturally
--   (`auth.smtp_config.changed`).
--
-- ## Singleton-row design
--
-- There is only ever one effective SMTP configuration. Rather than
-- model that with a separate `is_current` flag or version table,
-- the row's primary key is hard-coded to `'singleton'`. Inserts
-- conflict by construction; updates are addressed by id.
--
-- ## Encryption
--
-- - `password_enc` is a sealed XChaCha20-Poly1305 ciphertext over
--   the SMTP password (or OAuth bearer token), AAD =
--   `b"smtp.password"`. Plaintext never touches the database.
-- - `username` is plaintext: it's the SMTP account name
--   (typically the same as `from_address`), is shown in the admin
--   UI, and is not secret in the threat model where the master
--   key is.
-- - `host`, `port`, `tls_mode`, `from_address`, `from_name`,
--   `base_url` are plaintext.
--
-- ## TLS mode
--
-- `tls_mode` is one of:
--
-- - `'implicit'`  — port-465-style TLS-from-the-start (the
--   default; what `wasm-smtp-tokio::TokioTlsTransport::connect_implicit_tls`
--   produces).
-- - `'starttls'`  — port-587-style plaintext-then-upgrade. We
--   never ship a `'plain'` mode: TLS is required by `wasm-smtp` at
--   the API surface, and a plaintext SMTP relay should not be
--   used to deliver password-reset links over the open internet.
--
-- ## `enabled`
--
-- A row may exist with `enabled = 0` so an operator can stage a
-- new configuration without flipping email features on (the
-- forgot-password endpoint is gated on this flag). When `enabled
-- = 0` no mail-feature endpoint sends mail.

CREATE TABLE smtp_config (
    id              TEXT PRIMARY KEY CHECK (id = 'singleton'),
    enabled         INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0, 1)),
    host            TEXT NOT NULL,
    port            INTEGER NOT NULL CHECK (port BETWEEN 1 AND 65535),
    tls_mode        TEXT NOT NULL CHECK (tls_mode IN ('implicit', 'starttls')),
    username        TEXT,
    -- Sealed via XChaCha20-Poly1305 with AAD = b"smtp.password".
    -- NULL when the SMTP relay does not require authentication
    -- (rare on the open internet but valid for trusted internal
    -- relays such as a co-located Postfix submission service).
    password_enc    BLOB,
    from_address    TEXT NOT NULL,
    from_name       TEXT,
    -- The public origin sui-id is reachable at, used to construct
    -- absolute URLs in outgoing mail (the password-reset link). We
    -- can't reuse `Config.server.issuer` for this because the
    -- issuer URL is sometimes a back-channel and the user-facing
    -- origin is different. Always an https:// URL in production.
    base_url        TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
) STRICT;
