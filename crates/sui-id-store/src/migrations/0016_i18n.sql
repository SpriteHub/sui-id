-- 0016: Multilingual support v1.
--
-- v0.23.0 introduces a typed i18n layer (`sui-id-i18n` crate) and
-- a four-step locale resolution chain:
--
--   1. user.preferred_lang  (authenticated, set on /me/profile)
--   2. Cookie sui_id_lang    (per-browser override)
--   3. Accept-Language       (browser default, no UI)
--   4. server_settings.default_lang (admin-configured fallback)
--   5. hard-coded fallback   (Locale::Ja, in code)
--
-- This migration adds the storage for tiers 1 and 4.
--
-- ## users.preferred_lang
--
-- Nullable TEXT. NULL means "no preference set; use the browser /
-- server default". Constrained at the application layer to one of
-- the BCP-47 tags `sui_id_i18n::Locale` knows about (currently
-- 'ja', 'en'). We deliberately do NOT pin the CHECK constraint to
-- a fixed set of tags — adding a locale should not require a
-- schema migration. If a stale tag is read back from the DB after
-- a downgrade, the application falls through to the next tier in
-- the resolution chain.
--
-- ## server_settings — singleton row
--
-- Modeled like `smtp_config`: a single row keyed on the literal
-- string 'singleton', containing process-wide configuration
-- previously hard-coded or sourced from `sui-id.toml`. Today this
-- holds only the default language; future settings (UI theme
-- defaults, etc) can extend the row without a fresh migration.
--
-- A row is auto-inserted with conservative defaults so the
-- application never sees `None` for the singleton — the upsert
-- pattern in `repos::server_settings` writes-or-updates on save
-- and a default-row INSERT runs as part of this migration.

ALTER TABLE users ADD COLUMN preferred_lang TEXT;

CREATE TABLE server_settings (
    id            TEXT PRIMARY KEY CHECK (id = 'singleton'),
    default_lang  TEXT NOT NULL DEFAULT 'ja',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

INSERT INTO server_settings (id, default_lang, created_at, updated_at)
VALUES ('singleton', 'ja',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
