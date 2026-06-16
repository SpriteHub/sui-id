# RFC 071 — Auditor role

**Status.** Proposed
**Priority.** P1 — addresses the largest operational gap. No deployment with
more than one human operator can run this product safely without role
separation; shared admin credentials are the only alternative today.
**Tracks.** UX rethink — role model (see audit notes, v0.57.1 session).
**Touches.** `crates/sui-id-store` (migrations, users repo, models),
`crates/sui-id-core` (admin operations, authorization checks),
`crates/sui-id` (handlers, middleware, AppState), `crates/sui-id-web`
(admin nav, danger zones, mutation buttons), `crates/sui-id-i18n`.

---

## Background

The product has one administrative role: full-control "admin." In any
deployment with two or more operators (SRE rotation, compliance reviewers,
incident-response staff), the choices today are:

1. Share one admin account across multiple humans (insecure, untraceable).
2. Give each operator their own admin account (insecure — anyone can
   delete users, rotate keys, change settings; the audit log records *who*
   but does not prevent *what*).

Both are unacceptable for production. The product needs a read-only role
that lets a person look at users, apps, sessions, the audit log, signing
keys, and settings without holding any mutation capability. This is the
**Auditor**.

## Non-goals

- **No granular permissions matrix.** Three roles (admin, auditor, user) is
  the entire model. Per-resource ACLs are post-1.0 work (RFC 025).
- **No delegation.** Auditors cannot "act as" anyone else.
- **No role escalation flow.** An admin promoting an auditor to admin is
  a normal user-edit operation; there is no "request escalation" pathway.
- **No federation of role mapping.** When RFC 004 (federation) lands, role
  is local; the upstream IdP does not set it.

## Goal

Add a third human role `auditor` that:

- Sees every admin-readable screen (users, user detail, apps, app detail,
  audit log, signing-key list, settings) with the same data an admin sees.
- Cannot mutate any state. Every `POST` / `PUT` / `DELETE` returns
  HTTP 403 with the standard sui-id error page.
- Cannot see secret material that admins also cannot see (private signing
  keys, password hashes — both already inaccessible from UI).
- Cannot see secret material that admins **can** see at issuance time:
  newly-rotated client secrets in particular. Auditors see "secret last
  rotated on $date" but never the value.
- Has the same `/me/*` self-service as anyone else (their own password,
  MFA, passkeys, sessions).

## Design

### Schema migration `0027_users_role.sql`

```sql
-- Add role column with check constraint. Default ‘user’ matches the
-- existing semantics: rows where is_admin = 0 are end users.
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'user'
    CHECK (role IN ('admin', 'auditor', 'user'));

-- Backfill from is_admin. After this migration, is_admin is no longer
-- consulted; it is kept for two further migrations as a safety net,
-- then dropped in 0029_users_drop_is_admin.sql once observed safe.
UPDATE users SET role = 'admin' WHERE is_admin = 1;

CREATE INDEX idx_users_role ON users(role) WHERE is_deleted = 0;
```

`is_admin` is **not dropped** in this migration. Read paths continue to
work off `role`; write paths set both. A subsequent migration removes
`is_admin` after the new column has soaked.

### Rust types

```rust
// In sui-id-store::models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Auditor,
    User,
}

impl Role {
    /// True if the role can mutate any administrative state.
    pub fn is_admin(self) -> bool { matches!(self, Self::Admin) }
    /// True if the role can read administrative state (admin or auditor).
    pub fn can_read_admin(self) -> bool { matches!(self, Self::Admin | Self::Auditor) }
}
```

`User { is_admin: bool }` is replaced by `User { role: Role }`. The
`is_admin` accessor remains as `fn is_admin(&self) -> bool { self.role.is_admin() }`
to avoid churning every call site at once.

### Middleware: `require_admin_read` and `require_admin_write`

Today's `require_admin_session` extractor returns 401/302 if the session
is unauthenticated and 403 if the user is not admin. Split it:

- **`require_admin_read`** — passes if `role ∈ {admin, auditor}`.
- **`require_admin_write`** — passes if `role = admin` only.

Apply `require_admin_read` to GET routes, `require_admin_write` to POST /
PUT / DELETE routes. The mapping is mechanical: a `routes_admin()` helper
in `routes.rs` already separates them.

### UI changes

Every page that currently shows mutation controls (Edit, Add, Delete,
Disable, Reset MFA, Save) takes a `role: Role` parameter (already in
`Shell` via `csrf_token` propagation pattern from RFC-MI-021). Render
mutation controls iff `role == Admin`.

Specifically:

- **User list**: hide "Add user" button; rows have no "Edit" link.
- **User detail**: hide entire danger-zone section; hide "Edit" link.
- **App list**: hide "Add app"; rows have no "Edit" link.
- **App edit**: page becomes a read-only view ("App detail"); no form,
  no save button, no danger-zone.
- **Audit log**: read-identical. No mutation controls existed here anyway.
- **Signing keys**: hide "Rotate," "Retire," "Delete."
- **Settings**: every form's submit button is hidden; values render as
  read-only text (`<input>` becomes `<output>` or `<dd>`).
- **Dashboard**: action items can still be shown; the click-through
  destinations are read-only for auditors. The "Getting Started" checklist
  (RFC 073) is hidden — it's an admin task.

### Admin operations for managing roles

**Add auditor**: New admin-only form on the user list page (or as a
separate "Invite" button). Same fields as "Add user" plus a role selector
defaulting to `user`.

**Change role**: New admin-only control on the user detail page, in a new
"Access" section between "Identity" and the danger zone. Roles available:
admin, auditor, user. Changing to `admin` requires extra confirmation
(it is a privilege escalation).

**Safeguards**:
- An admin cannot demote themselves while they are the only admin.
- An admin cannot delete their own account (already enforced).
- Promoting a user to admin is logged with `audit_action = 'role_change'`,
  capturing old role, new role, actor.

### Audit log

Every mutating action already records the actor's user ID. Extend the
audit-log row schema to include `actor_role` (the role at the time of
action). Schema migration `0028_audit_actor_role.sql`:

```sql
ALTER TABLE audit_log ADD COLUMN actor_role TEXT
    CHECK (actor_role IN ('admin', 'auditor', 'user') OR actor_role IS NULL);
```

NULL means "pre-migration row." All new rows are required to populate it.

The audit log UI gains a column "By" showing `{actor_username} (admin)`,
`{actor_username} (auditor)`, etc., so a reader can quickly see whether
a destructive action was an admin or a (read-only) auditor — the latter
should never happen, but if it does it indicates a bug.

### Failure modes

- **Bug allowing auditor mutation**: middleware test for every POST route
  asserting auditor receives 403. Integration tests can iterate over the
  route table.
- **Stale session after demotion**: if an admin demotes user A to auditor
  while A has an active session, A's next request hits the middleware
  with a session linked to a user record whose role is now auditor.
  This works correctly with no changes — the role is read from the user
  record on every request (via session→user join), not cached in the
  session itself.
- **Privilege escalation via role-change attempt by auditor**: the role
  change endpoint is admin-only; auditor attempts return 403.

## Migration order and rollback

1. Ship migration 0027 (add `role` column, backfill from `is_admin`).
2. Ship code that reads `role` and writes both `role` and `is_admin`.
3. Soak for one minor release.
4. Ship migration 0028 (`audit_log.actor_role`).
5. Soak for one minor release.
6. Ship migration 0029 (drop `is_admin`).

Rollback path between (1) and (3): previous code still reads `is_admin`,
ignores `role`. Auditors created in this window appear as non-admin to
old code (i.e., they cannot reach admin pages — fail-closed, safe).

## Acceptance criteria

- [ ] Migrations 0027 and 0028 land; existing admin rows have `role = 'admin'`.
- [ ] `Role` enum and `users.role` reads in place; `is_admin` writes go through
  a thin compatibility layer.
- [ ] Every GET admin route accepts auditor; every POST admin route returns
  403 for auditor (validated by integration test that iterates the route
  table).
- [ ] UI hides mutation controls for auditors on every admin page.
- [ ] Admin can create, demote, promote auditors via the user detail page.
- [ ] Last-admin safeguard prevents the system from being locked out.
- [ ] Audit log includes `actor_role` on all new entries.
- [ ] CI invariants unchanged: `text-leaks` = 0, `inline-style-bound` = 0,
  `css-tokens` = 148, `semantic-parity` = 36.
- [ ] All existing tests pass; new tests added for role middleware and
  the UI conditional rendering.

## Risks

| Risk | Mitigation |
|---|---|
| Auditor sees something they should not (e.g., a freshly-rotated client secret) | UI surface designed to render secrets only once at rotation time, in an admin-only handler return path |
| Admin demotes themselves and locks out the system | Last-admin check on every role-change; refuses demotion if `count(admin where is_deleted=0) == 1` |
| Existing integrations rely on `is_admin` | Compatibility layer keeps the column for two releases; `is_admin` writes mirror `role` writes |
| Auditor accumulates over time in deployments without delete | Same as any other user; admin can demote auditors back to `user` or delete them |
