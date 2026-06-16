-- Migration 0028 — RFC 071: record actor role in audit log (v0.59.0)
--
-- Adds actor_role to audit_log so reviewers can see whether an action
-- was taken by an admin or (unexpectedly) by an auditor. NULL means
-- the row predates this migration.

ALTER TABLE audit_log ADD COLUMN actor_role TEXT
    CHECK (actor_role IN ('admin', 'auditor', 'user') OR actor_role IS NULL);
