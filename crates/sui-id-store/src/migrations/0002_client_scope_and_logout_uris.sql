-- 0002: per-client scope policy and post-logout redirect URIs.
--
-- These two columns are added together because they share a migration
-- shape (both are "JSON array of strings stored as TEXT, defaulting to
-- empty/permissive") and both require the same backfill sweep across
-- existing rows.
--
-- ## allowed_scopes
--
-- A space-separated list of scopes the client is permitted to request at
-- /oauth2/authorize. Stored as TEXT for parity with the OAuth wire format.
-- An empty string means "no policy configured" — we interpret that as
-- "permit any scope" for backwards compatibility with rows created before
-- this migration. New rows from v0.6.0 onwards default to "openid profile"
-- via the application layer.
--
-- ## post_logout_redirect_uris
--
-- A JSON array of strings. When the RP-initiated logout endpoint receives
-- a `post_logout_redirect_uri`, sui-id checks it against this list. If the
-- list is empty (the on-disk default for rows created before this
-- migration), sui-id falls back to checking against `redirect_uris` so
-- that existing deployments do not silently break — but the application
-- layer logs a deprecation warning when the fallback is taken.

ALTER TABLE clients ADD COLUMN allowed_scopes TEXT NOT NULL DEFAULT '';
ALTER TABLE clients ADD COLUMN post_logout_redirect_uris TEXT NOT NULL DEFAULT '[]';
