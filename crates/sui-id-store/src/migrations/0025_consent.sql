-- Migration 0025 — OIDC per-client consent policy (RFC 038)
--
-- consent_policy controls whether the authorize endpoint shows a consent
-- screen. Values: 'none' (skip), 'first_time' (show once), 'always'.
ALTER TABLE clients ADD COLUMN consent_policy TEXT NOT NULL DEFAULT 'none';

-- Stored per-user consent grants. Keyed on (user_id, client_id).
-- granted_scopes is a space-separated list of scope tokens.
CREATE TABLE user_consent (
    user_id        TEXT NOT NULL,
    client_id      TEXT NOT NULL,
    granted_scopes TEXT NOT NULL,
    granted_at     TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id, client_id),
    FOREIGN KEY (user_id)   REFERENCES users   (id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES clients (id) ON DELETE CASCADE
);
