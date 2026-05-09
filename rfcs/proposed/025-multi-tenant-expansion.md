# RFC 025 — Multi-tenant expansion path: detailed design

**Status.** Proposed (longer-term, no scheduled delivery)
**Priority.** Low. Detailed-design RFC for an expansion that
is realistically in scope but is not in a release queue. The
purpose of this RFC is not to schedule the work; it is to
ensure that *if* the work happens, the design is settled
enough that an implementer is not building it from scratch,
and that current decisions (RFC 022's single-realm scope
statement, current schema choices) can be made with full
knowledge of the path forward.
**Tracks.** v0.29.5 data-model review-2 §3, §4, §6 (case B).
Supersedes [RFC 007 (Multi-tenancy)](../archive/007-multi-
tenancy.md), which had the topic but not the detail.
**Touches.** Substantial. Schema changes propagate to nearly
every table; routing changes affect the issuer URL space;
admin authorisation gains a new dimension (global admin vs
tenant admin); audit logging gains tenant scope; setup flow
gains a "create first tenant" step. Detailed below.

## Summary

sui-id is single-realm by design and by RFC 022. There is a
real, articulated demand profile for which a multi-tenant
shape would fit — small and medium SaaS deployments where the
SaaS operator wants to expose isolated IdP per customer
without running N processes, but where Keycloak is too much
machinery and Auth0 is too much SaaS dependency.

This RFC describes what that expansion looks like inside
sui-id's design philosophy. It is a *detailed design*, not a
schedule: at the time of writing, the maintainer has stated
that case B is "現実的な視野に入れている" but has no commitment
to deliver. The RFC's job is to make sure the design is on
paper so that:

- present-day decisions (e.g. "should column X be globally
  unique?") can account for the eventual multi-tenant shape,
- the sister RFC 022 (single-realm scope statement) can
  reference a real expansion path rather than a vague
  intention,
- when the maintainer decides to schedule the work, the
  implementer reads one RFC instead of designing from
  scratch.

If the maintainer subsequently decides not to pursue
multi-tenant, this RFC moves to `archive/` with status
`Withdrawn`. Until then it is `Proposed` and informs other
RFCs.

## Scope of this design

In scope:

- A `tenant` model and its propagation through every
  primary table.
- Routing: how `/oauth2/authorize` etc. become tenant-scoped.
- Issuer / discovery / JWKS per tenant.
- Admin authorisation: global admin vs tenant admin.
- Migration path: how a single-realm 0.x deployment becomes
  a multi-tenant 1.x deployment.
- Setup flow changes.

Explicitly **out of scope** (deferred to subsequent RFCs even
if multi-tenant ships):

- **Organisation hierarchies.** Tenants are flat. An
  organisation tree (departments, business units) inside a
  tenant is a separate model that the data review's review-2
  §3 lists alongside multi-tenancy. This RFC does not
  attempt to settle organisation modelling.
- **Group / role / permission systems.** The current
  `is_admin` boolean and the new `tenant_admin` flag (see
  § 4) are the only roles. Per-resource permissions (read
  this client, write that user) are out of scope.
- **Custom claims / arbitrary user attributes.** The userinfo
  shape stays minimal (RFC 020 plus future RFCs). Per-tenant
  claim mapping is a follow-up.
- **Tenant marketplace / self-service tenant signup.** The
  tenant-creation flow is admin-only.
- **Cross-tenant federation.** A user in tenant A cannot log
  into a client in tenant B. Federation between sui-id
  tenants would be its own design.

## Why this RFC, not a smaller skeleton

A real risk in publishing a half-detailed multi-tenancy RFC
is that it invites premature compromise: a future PR adds
`tenant_id TEXT NULL DEFAULT 'default'` to `users` and calls
it tenancy. That shape is worse than no tenancy because it
breaks invariants without delivering isolation.

This RFC carries enough detail (schema, routing, admin
authorisation, migration) that a reader can see the full
shape and can therefore tell when a partial change is moving
toward it or away from it.

## Background: what's there today

Quick recap of the single-realm shape (RFC 022):

```
users(id, username, email, ...)              -- flat
clients(id, name, redirect_uris, ...)        -- flat
sessions(user_id, ...)                       -- one per user
refresh_tokens(user_id, client_id, ...)
audit_log(actor, target, ...)                -- flat
server_settings(id = 'singleton', ...)       -- one config
smtp_config(id = 'singleton', ...)           -- one config
```

One issuer URL: the operator's deployment URL. One JWKS. One
admin role.

## Design

### § 1. The `tenants` table

```sql
CREATE TABLE tenants (
    id TEXT PRIMARY KEY,             -- ulid or uuid
    slug TEXT NOT NULL UNIQUE,       -- url-safe, e.g. "acme"
    name TEXT NOT NULL,              -- human label, e.g. "Acme Corp."
    status TEXT NOT NULL             -- 'active' | 'suspended' | 'deleted'
        CHECK (status IN ('active', 'suspended', 'deleted')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

`slug` is the URL-visible identifier. It must match
`^[a-z0-9](-?[a-z0-9])*$`, length 2..64. The slug is
immutable after creation (changing it would break OIDC
issuer URLs already issued in tokens).

A reserved slug `_global` is used internally to represent
"not bound to a tenant" — see § 4 admin scoping.

### § 2. `tenant_id` propagation

Every table that today represents per-realm data gains a
`tenant_id` column. The list:

```
users.tenant_id
credentials      (joined to users; no own tenant_id, lives via FK)
clients.tenant_id
auth_codes.tenant_id (or via clients FK; see below)
sessions          (lives via users FK)
refresh_tokens    (lives via users FK + clients FK)
user_totp         (lives via users FK)
user_webauthn_credentials (via users FK)
password_reset_tokens (via users FK)
revoked_access_tokens (via users + clients FK)
audit_log.tenant_id   (explicit; the actor/target may be
                      cross-tenant for global-admin events)
consents          (via users FK + clients FK)
```

**Single-source-of-truth principle.** When a row already
joins to a `tenant`-bearing table via FK, it does not also
carry its own `tenant_id`. `sessions` belongs to a `user`,
which belongs to a `tenant`; the tenancy of a session is
recoverable through the join. This avoids the
"tenant_id drift" problem (a session row whose tenant_id
disagrees with its user's tenant_id).

**Tenant-bearing tables** (carry their own `tenant_id`):
`users`, `clients`, `audit_log`. Everything else inherits
through FK.

`audit_log` carries its own `tenant_id` because audit events
have actors/targets that may not exist (deleted users,
external clients) and the row needs tenant scope at write
time, not at read time.

**Singleton config tables become per-tenant tables.**
`server_settings` and `smtp_config` lose their `id =
'singleton'` shape and gain `(tenant_id) PRIMARY KEY`:

```sql
CREATE TABLE tenant_settings (
    tenant_id TEXT PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,
    default_lang TEXT,
    hibp_mode TEXT NOT NULL,
    idle_session_timeout_secs INTEGER NOT NULL,
    max_concurrent_sessions INTEGER,
    -- ...
);
```

A new `global_settings` singleton table holds settings that
are deployment-wide and not per-tenant: cookie name prefix,
listen address (set at startup, not from DB), feature flags,
the global admin's bootstrap state.

### § 3. Login identifier scoping

The data review-2 §6 lists this as a per-deployment
decision. This RFC makes it.

**`username` is unique within a tenant.** Two tenants can both
have a user "alice". The pair `(tenant_id, username)` is the
unique key.

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    username TEXT NOT NULL,
    email TEXT,
    email_normalized TEXT,
    -- ... (other RFC-020 fields)
    UNIQUE (tenant_id, username),
    UNIQUE (tenant_id, email_normalized)  -- partial, where email_normalized IS NOT NULL
);
```

**Email is unique within a tenant.** Same model. A user with
`alice@acme.com` can exist in tenant `acme` and also in tenant
`globex` (likely a different person, possibly the same person
working at two companies).

**`sub` claim is globally unique by being the user's `id`.**
The OIDC `sub` is the user's row id (ulid/uuid). RPs receive
opaque per-row ids. They do not see `username`. This means
`sub` is automatically tenant-aware via the routing layer:
tenant `acme`'s issuer issues subs that resolve to `acme`
users only.

**Login form.** The login form is *always* tenant-scoped:
`/{tenant-slug}/login`. The user types only their username
or email; the tenant is implicit in the URL. There is no
"tenant selector" dropdown. This avoids the enumeration
problem (a tenant selector lets an attacker enumerate
tenants).

### § 4. Admin scoping

Two roles, neither carried in the same column:

- **Global admin.** Operates on `_global` scope. Can create
  / suspend / delete tenants, manage signing keys,
  manage global-only settings. Cannot directly manage users
  or clients in any specific tenant — that requires acting
  *as* a tenant admin (a deliberate auth-step that emits an
  audit row).
- **Tenant admin.** Bound to a specific tenant. Manages users,
  clients, settings within their tenant. Cannot see other
  tenants.

Schema:

```sql
ALTER TABLE users ADD COLUMN is_global_admin INTEGER NOT NULL DEFAULT 0
    CHECK (is_global_admin IN (0, 1));
ALTER TABLE users ADD COLUMN is_tenant_admin INTEGER NOT NULL DEFAULT 0
    CHECK (is_tenant_admin IN (0, 1));
-- the existing is_admin column is dropped after migration
```

A user is one of:

- a regular user (both flags 0),
- a tenant admin (`is_tenant_admin = 1`, `is_global_admin
  = 0`, `tenant_id = some real tenant`),
- a global admin (`is_global_admin = 1`, `is_tenant_admin =
  0`, `tenant_id = _global`).

A user *cannot* be both a tenant admin and a global admin
(CHECK constraint). The two roles have different mental
models and combining them invites confusion.

A global admin can explicitly **assume** a tenant: a session
with `assumed_tenant_id` set acts within that tenant for the
duration. The assumption is logged in audit. Without
assumption, a global admin has no tenant-data access.

The login flow distinguishes:

- `/{slug}/login` — tenant login. Authenticates a user of
  that tenant.
- `/_global/login` — global-admin login. Authenticates a
  global admin. Only reachable via direct URL, not linked
  from any tenant page. Bootstraps in setup.

### § 5. Issuer / discovery / JWKS per tenant

Today: one issuer URL, one discovery doc, one JWKS.

After multi-tenant:

- **Issuer URL.** `https://{deployment-host}/{tenant-slug}/`.
  The issuer URL embeds the tenant slug. Tokens issued by
  tenant `acme` carry `iss: "https://idp.example.com/acme/"`.
- **Discovery.** `/{slug}/.well-known/openid-configuration`.
  Same shape as today, scoped to the tenant.
- **JWKS.** `/{slug}/.well-known/jwks.json`. The signing keys
  in the response are the tenant's keys.

**Per-tenant signing keys vs shared signing keys.** Two
options:

- **(A) Per-tenant signing keys.** Each tenant has its own
  `signing_keys` rows. Tokens for tenant `acme` are signed
  with `acme`'s key. RPs in tenant `acme` fetch
  `/acme/.well-known/jwks.json`. Strong isolation; any
  signing-key compromise affects only that tenant.
- **(B) Shared signing keys.** One set of keys signs all
  tokens; the JWKS is the same per tenant; the issuer URL
  differs. RPs do not learn other tenants exist. Simpler
  key rotation.

Recommended **(A)**, per-tenant signing keys. Reasons:

- Isolation is the entire point of tenancy. Sharing the
  signing key undermines that.
- RFC 021 § 3's single-active invariant becomes per-tenant
  (`UNIQUE (tenant_id, is_active) WHERE is_active = 1`).
- Operationally not much harder: rotation runs per-tenant
  on a schedule.

```sql
ALTER TABLE signing_keys ADD COLUMN tenant_id TEXT
    REFERENCES tenants(id) ON DELETE CASCADE;
-- once backfilled and verified:
-- ALTER TABLE signing_keys ALTER COLUMN tenant_id SET NOT NULL;
-- (SQLite via rebuild)
```

### § 6. Routing layer

Every URL gains a tenant prefix except the truly global
ones (global admin login, deployment-level health, etc.).

```
Today:                            Multi-tenant:
/.well-known/openid-configuration /{slug}/.well-known/openid-configuration
/.well-known/jwks.json            /{slug}/.well-known/jwks.json
/oauth2/authorize                 /{slug}/oauth2/authorize
/oauth2/token                     /{slug}/oauth2/token
/oauth2/userinfo                  /{slug}/oauth2/userinfo
/oauth2/revoke                    /{slug}/oauth2/revoke
/oauth2/introspect                /{slug}/oauth2/introspect
/admin/login                      /{slug}/admin/login        (tenant admin)
                                  /_global/admin/login        (global admin)
/admin/users                      /{slug}/admin/users
/admin/clients                    /{slug}/admin/clients
/admin/settings                   /{slug}/admin/settings
/admin/audit                      /{slug}/admin/audit         (tenant audit)
                                  /_global/admin/audit         (global audit)
/admin/tenants                    /_global/admin/tenants
/me/security                      /{slug}/me/security
/setup                            /setup                       (one-shot global)
```

**Slug parsing.** A small middleware extracts the tenant slug
from the path and validates it against the `tenants` table.
A nonexistent slug returns 404 (not 401) — the tenant is
effectively unknown to the world.

A suspended tenant returns 503 with a generic "service
unavailable" page. The 503 path is privacy-preserving (does
not differ from a real outage) so suspended-tenant existence
isn't leakable.

### § 7. Setup flow changes

Today: setup creates the first admin user and a default config.

After multi-tenant, two stages:

1. **Bootstrap stage.** Setup wizard runs once per
   deployment. It creates:
   - the `_global` reserved tenant row (slug literally
     `_global`),
   - the first global admin user,
   - the deployment-wide `global_settings` row,
   - the master key (existing behaviour),
   - global signing keys (used to sign global-admin auth
     events; *not* used for tenant tokens).
2. **Tenant creation.** A global admin, after bootstrap,
   creates the first real tenant via `/_global/admin/tenants`.
   Creating a tenant generates that tenant's signing keys
   and creates the tenant's first admin user.

The setup wizard does *not* create a real tenant. The wizard
is for the *deployment*; tenants are created *afterwards* by
the global admin. This split keeps the wizard small and
matches RFC 012's "setup is one-shot bootstrap" contract.

### § 8. Migration from single-realm to multi-tenant

The hardest part of this design. The trade-offs are real and
shape what version this lands in.

**Strategy: a one-shot upgrade that creates one
default tenant containing everything.**

```sql
-- pseudo: this is the migration script for the single-realm
-- → multi-tenant transition

-- 1. add tenants table
CREATE TABLE tenants (...);

-- 2. create the default tenant containing all current data
INSERT INTO tenants (id, slug, name, status, created_at, updated_at)
VALUES ('default-tenant-uuid', 'default', 'Default tenant',
        'active', now(), now());

-- 3. add tenant_id columns and backfill to 'default-tenant-uuid'
ALTER TABLE users ADD COLUMN tenant_id TEXT NOT NULL
  DEFAULT 'default-tenant-uuid' REFERENCES tenants(id) ON DELETE CASCADE;
ALTER TABLE clients ADD COLUMN tenant_id TEXT NOT NULL
  DEFAULT 'default-tenant-uuid' REFERENCES tenants(id);
-- ... (each tenant-bearing table)

-- 4. drop singleton-config singleton constraint, copy to per-tenant
INSERT INTO tenant_settings (tenant_id, ...)
SELECT 'default-tenant-uuid', ... FROM server_settings WHERE id = 'singleton';
DROP TABLE server_settings;

-- 5. promote existing admin to is_tenant_admin
UPDATE users SET is_tenant_admin = is_admin;
ALTER TABLE users DROP COLUMN is_admin;

-- 6. routing change: existing URLs continue to work via
-- a redirect /admin/X → /default/admin/X

-- 7. issuer URL change: existing tokens carry the old issuer.
-- The discovery doc at the old path returns the new issuer URL.
-- Existing access tokens and refresh tokens remain valid via
-- a one-release transition window where both issuers are
-- accepted on the verify path.
```

**The issuer-URL transition is the hard part.** Existing
RPs cache `iss` in their stored ID tokens. Those RPs, when
they next call `/userinfo`, fetch discovery, see a new issuer,
and their conformance check rejects the old `iss`. Three
options:

- **Hard-cut.** New deployment major version requires RPs
  to re-register. Brutal.
- **Both-issuer transition.** The IdP signs new tokens with
  the new issuer URL but accepts old issuer URLs on token-
  verify endpoints (introspect, revoke) for one release.
  RPs naturally re-issue tokens via refresh and end up with
  new-issuer tokens.
- **No URL change for the default tenant.** The default
  tenant uses the existing issuer URL (no slug prefix).
  Other tenants get slug-prefixed URLs. The migration is
  backwards-compatible at the cost of a special-case in
  routing.

Recommend **(B)** with a one-version transition. Cleaner
end state than (C), less brutal than (A). Document the
transition window explicitly in the release notes for the
multi-tenant version.

### § 9. Audit log scoping

`audit_log` gets `tenant_id`. Read paths:

- A tenant admin sees rows where `tenant_id = their tenant`.
- A global admin sees all rows. The audit screen offers a
  tenant filter.
- Cross-tenant events (a global admin acts as tenant admin
  in `acme`) carry the tenant they acted *in*; the row also
  notes the global admin's identity in `actor`.

The hash chain is **per-tenant**. A separate hash chain per
tenant gives strong tenant isolation: a tenant admin cannot
see another tenant's audit volume even by side-channel. The
trade-off is that hash-chain verification runs per-tenant.

A separate `_global` audit chain covers global-admin events
and tenant-creation / suspension events.

### § 10. Rate limiting and lockout

Per-tenant. The rate-limit key derivation includes
`tenant_id`. A lockout in one tenant doesn't lock out the
same username in another tenant.

The data review notes this; mostly a small implementation
detail.

## Migration cohort summary

This is a major version (1.0.0 or similar). It is not a
0.x.y patch. The migration is one-shot at a major version
boundary, and operators are expected to read upgrade docs.

The migration is destructive in the soft sense (changes
issuer URLs, RPs may need to refresh tokens) but not in the
hard sense (no data loss). A backup-before-upgrade discipline
remains adequate.

## Tests

Substantial, but described abstractly here because details
follow the implementation:

- Per-tenant isolation: e2e tests for "tenant A admin cannot
  read tenant B users".
- Routing: 404 on bogus slug, 503 on suspended tenant, 200
  on active tenant.
- Per-tenant signing-key isolation: a tenant A token does
  not verify against tenant B JWKS.
- Login identifier scoping: same username in two tenants is
  legal; cross-tenant login attempt fails with `invalid_grant`.
- Migration regression tests: an upgraded single-realm DB
  produces correct multi-tenant invariants.
- Global-admin assumption: assumption is logged, scope
  limited.

## Security considerations

The fundamental concern with multi-tenancy is **cross-tenant
data access**. Threats:

1. **Path traversal in slug.** The middleware (§ 6) must
   reject slugs that are not strictly `[a-z0-9-]`. A slug
   `../other` must 404 before reaching tenant logic.
2. **Token confusion.** A token issued for tenant A sent to
   a tenant B endpoint must be rejected. Achieved by
   per-tenant JWKS (§ 5 option A) plus per-endpoint
   `iss`-claim verification.
3. **Cookie scope.** Session cookies are tenant-scoped via
   path: `Path=/{slug}/`. The cookie name does not encode
   the tenant; the path does.
4. **Cross-tenant CSRF.** The CSRF token is session-bound,
   which is tenant-bound; a token issued in tenant A does
   not authorise a request to tenant B.
5. **Global-admin impersonation.** A global admin assuming a
   tenant has full power within that tenant. The assumption
   audit row makes this discoverable, and the global-admin
   account is high-value (recommend mandatory MFA via the
   global settings).
6. **Suspended-tenant data.** A suspended tenant returns 503
   uniformly. The data is preserved (status='suspended', not
   deleted). A reactivated tenant resumes; a hard-deleted
   tenant cascades-deletes all its data.
7. **Tenant-creation race.** Slug uniqueness is enforced by
   the unique index on `tenants.slug`. Two simultaneous
   tenant-creation attempts on the same slug: one wins, one
   errors. No reliance on serialisation at the application
   layer.

## Why not now

This RFC is "Proposed (longer-term)" because the work is
large and the demand is not urgent enough to displace the
near-term backlog (RFC 019 / 020 / 021 / 022 / 023 / 024,
plus medium-priority RFCs 013 / 014 / 017 / 002). A schedule
would form when one of:

- A real deployment surfaces a need that "one process per
  tenant" cannot accommodate.
- The maintainer judges the design has been stable in
  `proposed/` for long enough that the implementation can
  start.

Until then, this document's job is to prevent piecemeal
multi-tenancy from sneaking into the codebase and to
provide a written reference for design choices that
implicitly bound multi-tenant feasibility.

## Open questions

1. **Slug case-sensitivity.** § 1 says lowercase. URL slugs
   are conventionally lowercase. Confirm.
2. **`_global` as a slug literal.** Using `_global` as a
   reserved literal is one option; another is to route
   global-admin via a path segment that cannot collide with
   any valid slug (e.g. `/$global` or `/admin/_global`).
   Decide at implementation time. The reserved-literal
   approach is simpler and matches the design here.
3. **Per-tenant SMTP.** § 2's `tenant_settings` should
   probably include SMTP, so each tenant sends emails from
   its own domain. The data review-2 alludes to this.
   Confirmed in scope; called out here for clarity.
4. **Deployment with one tenant only.** A deployment that
   uses the multi-tenant binary with a single tenant slug
   should feel as light as the single-realm version did.
   The migration § 8(C) option (no slug prefix for the
   default tenant) handles this if explicitly supported.
   Otherwise the operator can hide the slug via reverse-
   proxy rewrite.
5. **Future-proofing for organisation-within-tenant.** Out
   of scope here, but the schema choices (FK propagation,
   `tenant_settings` shape) should not preclude adding an
   `organization_id` column later. They don't, by
   construction — organisation modelling is orthogonal.
