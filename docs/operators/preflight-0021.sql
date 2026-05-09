-- preflight-0021.sql
-- Pre-flight checks for migration 0021 (v0.29.7).
--
-- ┌─────────────────────────────────────────────────────────────────────┐
-- │ NOTE: v0.29.7 has been RETRACTED due to a data-loss bug in          │
-- │ migration 0021. Do NOT upgrade to v0.29.7.                          │
-- │                                                                     │
-- │ If you are upgrading from v0.29.6, skip directly to v0.29.10 or    │
-- │ later. The pre-flight checks for migration 0022 (which replaces the  │
-- │ boolean-CHECK work that was intended for migration 0021) are in      │
-- │ docs/operators/preflight-0022.sql.                                  │
-- │                                                                     │
-- │ This file is retained for reference only. The queries below are     │
-- │ a subset of preflight-0022.sql.                                     │
-- └─────────────────────────────────────────────────────────────────────┘
--
-- Original description (historical):
-- Pre-flight checks for migration 0021 (Schema invariant CHECKs).
-- This file referred to boolean-CHECK constraints that were deferred from
-- 0021 (v0.29.7) to 0022 (v0.29.10) after the data-loss bug was discovered.


-- ── Boolean columns out of {0, 1} ─────────────────────────────────────────
-- Expected: all counts are 0. If any are non-zero, the migration will fail
-- on that table's CHECK constraint. Repair: set the offending value to 0
-- or 1 via: UPDATE <table> SET <col> = 0 WHERE <col> NOT IN (0, 1);

SELECT 'users.is_admin'           AS col, count(*) AS bad
  FROM users WHERE is_admin NOT IN (0, 1)
UNION ALL
SELECT 'users.is_disabled',        count(*)
  FROM users WHERE is_disabled NOT IN (0, 1)
UNION ALL
SELECT 'users.is_deleted',         count(*)
  FROM users WHERE is_deleted NOT IN (0, 1)
UNION ALL
SELECT 'credentials.must_change',  count(*)
  FROM credentials WHERE must_change NOT IN (0, 1)
UNION ALL
SELECT 'clients.confidential',     count(*)
  FROM clients WHERE confidential NOT IN (0, 1)
UNION ALL
SELECT 'clients.is_disabled',      count(*)
  FROM clients WHERE is_disabled NOT IN (0, 1)
UNION ALL
SELECT 'clients.is_deleted',       count(*)
  FROM clients WHERE is_deleted NOT IN (0, 1)
UNION ALL
SELECT 'signing_keys.is_active',   count(*)
  FROM signing_keys WHERE is_active NOT IN (0, 1)
UNION ALL
SELECT 'user_totp.enabled',        count(*)
  FROM user_totp WHERE enabled NOT IN (0, 1);


-- ── clients: confidential/secret_hash consistency ─────────────────────────
-- Expected: empty result set. Any returned rows violate the constraint that
-- confidential=1 implies secret_hash IS NOT NULL and vice versa.
--
-- Repair options:
--   • A confidential client with no secret_hash: regenerate the secret
--     through the admin UI (Settings → Clients → Regenerate Secret) on
--     the current version BEFORE upgrading.
--   • A public client with a secret_hash: clear it with:
--       UPDATE clients SET secret_hash = NULL WHERE id = '<id>';

SELECT id, name,
       confidential,
       CASE WHEN secret_hash IS NULL THEN 'NULL' ELSE 'present' END AS secret_hash
FROM   clients
WHERE  (confidential = 1 AND secret_hash IS NULL)
    OR (confidential = 0 AND secret_hash IS NOT NULL);


-- ── signing_keys: multiple active keys ────────────────────────────────────
-- Expected: 0 or 1. More than 1 active key means the old rotation path ran
-- in a crash window; the extras must be retired manually before migration.
--
-- Repair: UPDATE signing_keys SET is_active = 0, rotated_at = datetime('now')
--         WHERE id NOT IN (SELECT id FROM signing_keys
--                          WHERE is_active = 1
--                          ORDER BY created_at DESC LIMIT 1)
--           AND is_active = 1;

SELECT count(*) AS active_count,
       CASE WHEN count(*) <= 1 THEN 'OK' ELSE 'REPAIR NEEDED' END AS status
FROM signing_keys WHERE is_active = 1;
