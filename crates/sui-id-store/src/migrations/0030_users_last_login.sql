-- Migration 0030 — RFC 074: track each user's most recent successful login (v0.61.0)
--
-- Used by /me/security/overview to render an anti-phishing
-- "You last signed in on {date}" line.
ALTER TABLE users ADD COLUMN last_login_at TIMESTAMP;
