-- preflight-0022.sql
-- Pre-flight checks for migration 0022 (Boolean CHECK constraints).
--
-- Run against your database BEFORE upgrading to v0.29.10.
-- Unlike migration 0021 (which used safe evacuation via FK_DISABLE),
-- migration 0022 will ABORT with a CHECK constraint error if any row
-- contains a boolean value outside {0, 1}. This is the intended
-- fail-fast behaviour — it prevents silent data corruption.
--
-- An empty result for every query means you are safe to upgrade.
--
-- Usage: sqlite3 /path/to/sui-id.db < docs/operators/preflight-0022.sql


-- ── 1. Boolean columns out of {0, 1} ─────────────────────────────────────────
-- Expected: all counts are 0.
-- Repair: UPDATE <table> SET <col> = CASE WHEN <col> != 0 THEN 1 ELSE 0 END
--         WHERE <col> NOT IN (0, 1);
-- WARNING: only do this after confirming the correct intended value.
-- A non-zero is_disabled that should be "disabled" must become 1, not 0.

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


-- ── 2. clients: confidential/secret_hash consistency ─────────────────────────
-- Expected: empty result set.
-- Repair:
--   • confidential=1, no secret: regenerate via admin UI before upgrading.
--   • confidential=0, has secret: UPDATE clients SET secret_hash = NULL WHERE id = '<id>';

SELECT id, name,
       confidential,
       CASE WHEN secret_hash IS NULL THEN 'NULL' ELSE 'present' END AS secret_hash_status
FROM   clients
WHERE  (confidential = 1 AND secret_hash IS NULL)
    OR (confidential = 0 AND secret_hash IS NOT NULL);


-- ── 3. signing_keys: multiple active keys ─────────────────────────────────────
-- Expected: 0 or 1. More than 1 → retire extras before upgrading.
-- (Migration 0021 added the unique index; this checks the data invariant.)

SELECT count(*) AS active_count,
       CASE WHEN count(*) <= 1 THEN 'OK' ELSE 'REPAIR NEEDED' END AS status
FROM   signing_keys WHERE is_active = 1;


-- ── 4. user_uuid: any empty-string rows ───────────────────────────────────────
-- Expected: count = 0. Migration 0022 backfills these automatically
-- (runs UPDATE before the table rebuild), but this check confirms the
-- pre-existing data so you can verify the backfill ran correctly.

SELECT count(*) AS empty_user_uuid_count
FROM   users
WHERE  user_uuid = '';
