-- Migration 0020: User identity invariants and OIDC claim consistency
--                 (RFC 020)
--
-- Three coupled changes on the users table:
--
-- § 1. email_normalized — case-folded (lower + trim) form of email.
--      The unique index moves from the raw email column to this one,
--      so that "Alice@Example.com" and "alice@example.com" are
--      treated as the same address throughout the system (forgot-
--      password lookup, duplicate prevention, etc.).
--      The original `email` column is preserved so that the UI
--      can display the case the user chose at registration.
--
-- § 2. email_verified_at — timestamp at which the address was
--      confirmed. NULL for all existing users and for every user
--      until an email-verification flow ships (a future RFC). The
--      column exists now so that userinfo can honestly report
--      `email_verified: false` instead of omitting the claim.
--
-- § 3. user_uuid UNIQUE index — the column has existed since
--      migration 0004 (DEFAULT ''), but no uniqueness constraint was
--      applied. WebAuthn uses user_uuid as the stable user handle;
--      a duplicate would conflate two users at the credential layer.
--      A partial index (WHERE user_uuid <> '') excludes any legacy
--      empty-string sentinel rows from the uniqueness check.

-- § 1: email_normalized ------------------------------------------------

ALTER TABLE users ADD COLUMN email_normalized TEXT;

-- Backfill from existing rows. Rows with email IS NULL remain NULL.
UPDATE users
   SET email_normalized = lower(trim(email))
 WHERE email IS NOT NULL;

-- Move the uniqueness guarantee from the raw column to the normalised
-- one. The original idx_users_email (if present from migration 0012)
-- is dropped first; the new index carries the same semantics but
-- tolerates case differences.
DROP INDEX IF EXISTS idx_users_email;

CREATE UNIQUE INDEX idx_users_email_normalized
    ON users(email_normalized)
    WHERE email_normalized IS NOT NULL;

-- § 2: email_verified_at -----------------------------------------------

ALTER TABLE users ADD COLUMN email_verified_at TEXT;

-- § 3: user_uuid UNIQUE index ------------------------------------------
-- The partial predicate excludes the legacy DEFAULT '' sentinel so
-- that existing empty-string rows do not collide with each other or
-- with new UUIDs. New user creation always writes a real UUIDv4, so
-- the exception domain shrinks over time as rows are updated.

CREATE UNIQUE INDEX idx_users_user_uuid
    ON users(user_uuid)
    WHERE user_uuid <> '';
