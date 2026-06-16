-- Migration 0027 — RFC 071: Auditor role (v0.59.0)
--
-- Adds an explicit role column replacing the boolean is_admin flag.
-- Values: 'admin' | 'auditor' | 'user'. The is_admin column is kept
-- for two further migrations as a backward-compat safety net, then
-- dropped. Until dropped, writes mirror both columns.
--
-- role='admin'   → full administrative capability (was is_admin=1)
-- role='auditor' → read-only access to all admin surfaces
-- role='user'    → end-user self-service only (was is_admin=0)

ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'user'
    CHECK (role IN ('admin', 'auditor', 'user'));

UPDATE users SET role = CASE WHEN is_admin = 1 THEN 'admin' ELSE 'user' END;

CREATE INDEX idx_users_role ON users(role) WHERE is_deleted = 0;
