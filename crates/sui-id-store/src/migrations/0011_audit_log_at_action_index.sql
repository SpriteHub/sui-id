-- 0011_audit_log_at_action_index.sql
--
-- Composite index on the audit log to keep dashboard sparkline
-- queries fast as the table grows.
--
-- The dashboard's login-activity sparkline runs queries shaped like
--
--   SELECT bucket, action, COUNT(*) FROM audit_log
--   WHERE at >= ? AND at < ?
--     AND action IN ('auth.login.success', 'auth.login.failure')
--   GROUP BY bucket, action;
--
-- without an index this is a full table scan over the entire audit
-- history every time an operator opens /admin. With (at, action) the
-- planner can range-scan a date window directly. We deliberately put
-- `at` first because every dashboard query is bounded by time first
-- and then refined by action — the typical SQLite advice that the
-- most-selective column should lead doesn't apply here, since `at`
-- *is* the most-selective column for these queries.
--
-- The non-composite `at` index also remains useful for
-- audit::recent(...) and audit::recent_for_user(...), which only
-- order/filter on time. SQLite will use the leading column of a
-- composite index for that purpose, so we don't need to keep both.

CREATE INDEX IF NOT EXISTS idx_audit_log_at_action
  ON audit_log (at, action);
