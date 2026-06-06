# Open Issues — sui-id Mockup Handoff

Each issue is formatted for direct import into the issue tracker.
Owner and decision-maker are explicit; no item is "TBD by the engineer".

---

## Q1 — Admin-side MFA reset

**Classification**: Security
**Decision owner**: Security Reviewer
**Priority**: High (blocks any user who loses their MFA device)

### Why it's open

The mockup gives users a self-service MFA disable flow on
`/me/security?tab=mfa`, gated by step-up authentication. But what
happens when a user **loses their authenticator** and **runs out of
recovery codes**? The mockup does not surface an admin-side recovery
flow.

### Current mockup assumption

Admin contacts user out-of-band, verifies identity, and uses a
recovery-code regeneration flow that doesn't exist in the mockup.

### Current implementation reality

Unknown — backend may or may not have admin-side MFA reset.

### Recommended decisions

- Define the **identity verification** the admin must perform before
  resetting another user's MFA.
- Decide whether this should be:
  - A force-disable button on `/admin/users/{id}` (gated by step-up).
  - A separate ticket-based flow that requires another admin's
    approval (two-person rule).
- Define audit row name (`admin.mfa.reset_other`?) and impact-summary
  wording.

---

## Q2 — Settings step-up granularity

**Classification**: UX (with security implications)
**Decision owner**: Product Manager + Security Reviewer
**Priority**: Medium

### Why it's open

Every settings tab currently routes through
`settings.update.<tab>` step-up. Some settings are cosmetic
(`service_name`, `default_lang`); requiring re-auth for them is
over-protective and trains operators to enter their MFA code reflexively
— which weakens the protection on changes that really matter.

### Current mockup assumption

All settings changes go through step-up uniformly.

### Current implementation reality

Unknown — backend may already enforce this differently.

### Recommended decisions

- Classify each setting as **security-sensitive** vs **cosmetic**.
- Cosmetic settings (service name, default language, theme defaults)
  could save inline without step-up.
- Security-sensitive settings (HIBP mode, session timeout, MFA
  policy, signing key retention window) keep step-up.

### Mockup areas affected

- `/admin/settings?tab=basic` form save behaviour.
- `/admin/settings?tab=email` form save behaviour.
- The remaining four tabs likely stay step-up-gated.

---

## Q3 — Audit log retention and export

**Classification**: Technical feasibility (with product implications)
**Decision owner**: Architect
**Priority**: Medium

### Why it's open

The mockup shows "Export NDJSON" on `/admin/audit` but doesn't
define:

- retention policy (how long are rows kept?)
- export filtering (date range? event types? cap on size?)
- whether export itself is a step-up-gated action

### Current mockup assumption

Full log is exportable, no retention defined.

### Current implementation reality

Depends on backend storage. For SQLite-backed
self-hosted installs, "indefinite" is plausible. For larger
deployments, retention is needed.

### Recommended decisions

- Define a default retention (e.g. 365 days) configurable in
  `/admin/settings?tab=logs`.
- Decide if export is gated (probably yes — it produces PII).
- Decide if export filtering is required for the first release.

---

## Q4 — Bulk operations

**Classification**: Product scope
**Decision owner**: Product Manager
**Priority**: Low (do not block release on this)

### Why it's open

Every list view supports single-row actions only. Operators with
larger user bases will want bulk suspend, bulk delete, bulk session
revoke.

### Current mockup assumption

Out of scope. One user, one action.

### Current implementation reality

Not designed.

### Recommended decisions

- Decide whether this is in v1 or post-v1.
- If post-v1, document the omission explicitly in the user manual
  to prevent operator surprise.
- If in scope, design the UI carefully — bulk destructive actions
  need a different impact-summary pattern than single-row.

---

## Q5 — Real-time updates

**Classification**: Technical feasibility
**Decision owner**: Architect
**Priority**: Low (deferrable)

### Why it's open

The mockup is pure SSR with no live updates. Audit log doesn't
auto-refresh; session list doesn't auto-refresh.

### Current mockup assumption

Manual refresh.

### Current implementation reality

Not designed. SSE / WebSocket would be a significant scope
expansion.

### Recommended decisions

- For v1: accept manual refresh. Add an explicit "Refresh" button
  to `/admin/audit` if operator feedback requests it.
- For v2: consider SSE for audit-tail and session-list pages.
- Do **not** add JS-driven polling to a page that doesn't have a
  visible refresh control — the operator must know when data is
  stale.

---

## Q6 — Localisation scope

**Classification**: Product scope
**Decision owner**: Product Manager
**Priority**: Low

### Why it's open

The mockup ships Japanese + English. Adding a third language is
mechanical (the data flow is settled in RFC 011 for email and in
`i18n.rs` for UI strings) but adds maintenance cost.

### Current mockup assumption

ja / en only.

### Current implementation reality

Adding is non-blocking, but each new language requires translator
sign-off on security-sensitive wording (especially `auth.*` and
`stepup.*` strings).

### Recommended decisions

- Decide which languages are in v1.
- Decide who owns translation review for security wording (the
  anti-enumeration rules in §11.1 of HANDOFF.md must be preserved
  per locale).

---

## Q7 — Self-service password change vs admin reset

**Classification**: Security
**Decision owner**: Security Reviewer
**Priority**: Medium

### Why it's open

`/me/security?tab=password` lets the user change their password
(requires current password + new password). Admin-side password
reset is **not** in the mockup.

### Current mockup assumption

Admin uses the standard forgot-password flow on behalf of the user
— i.e. triggers an email with a reset link.

### Current implementation reality

TBD. Some backends expose an admin-side direct password set
function; that should be discouraged unless audited.

### Recommended decisions

- Confirm that admin password reset goes through the
  forgot-password email flow (no direct admin-set).
- If a direct-set is required (e.g. for legal compliance), it
  needs:
  - step-up authentication
  - mandatory new-password-on-first-login enforcement
  - audit row clearly distinguishing "admin set password" from
    "user changed password"

---

## Q8 — Session listing accuracy under concurrent eviction

**Classification**: Technical feasibility
**Decision owner**: Architect
**Priority**: Low

### Why it's open

The mockup's session list shows the current state at GET time.
Under FIFO eviction (RFC 014), a user with many devices could see
a session in the list that gets evicted before they can revoke it,
causing a "session not found" 404 on revoke.

### Current mockup assumption

Eviction is rare; the race is acceptable.

### Current implementation reality

Depends on session-store implementation.

### Recommended decisions

- Decide whether revoke-by-id should be idempotent (silently
  succeed if the session is already gone).
- The mockup's step-up flow already handles "consume returns None"
  gracefully — the same approach probably applies here.

---

## Q9 — OIDC client display name and homepage

**Classification**: Product scope / UX
**Decision owner**: Product Manager
**Priority**: Medium (consent screen depends on it)

### Why it's open

The consent screen needs a **user-readable** client name and
ideally a homepage URL to give context. The mockup assumes both
exist on the `Client` domain type.

### Current mockup assumption

`Client.name` is a human-readable display name; a homepage URL
field will be added.

### Current implementation reality

Backend may currently only have `client_id` (machine identifier).

### Recommended decisions

- Add `display_name` (required) and `homepage_uri` (optional) to
  the client model.
- Surface both in `/admin/clients` creation form.

---

## Q10 — Setup token transport

**Classification**: Security
**Decision owner**: Security Reviewer
**Priority**: Medium

### Why it's open

The mockup assumes the setup token is printed to stderr on first
boot and the operator copies it into a `?token=...` URL or sets it
manually in a cookie. The exact transport (env var vs stderr vs
file) is operations-dependent.

### Current mockup assumption

stderr print + URL parameter that lands in the cookie.

### Current implementation reality

Backend's bootstrap may use a different mechanism.

### Recommended decisions

- Align the mockup's gate logic with the real bootstrap mechanism.
- Document the transport in the operator manual.
- If the token can leak into shell history via URL, consider an
  alternative transport (e.g. local-file path that the operator
  reads and pastes into a textarea).

---

## Q11 — Internationalised email subjects and dates

**Classification**: UX / locale
**Decision owner**: Product Manager
**Priority**: Low

### Why it's open

RFC 011 defines typed mail contexts and per-locale templates. The
mockup's mock impl stores the `context_json` string but never
renders. Date formatting and subject lines in real emails need
locale-specific handling.

### Current mockup assumption

Templates are picked by `(template_key, locale)`. Each pair has a
hand-translated subject and body.

### Current implementation reality

When `lettre`-based SMTP transport lands, locale resolution must
happen at template-render time, not at template-store time.

### Recommended decisions

- Confirm that locale resolution happens **per recipient**, not
  per process.
- Time formatting in emails follows the recipient's locale's
  conventions (ISO-8601 for ja, RFC-2822 for en?). Decide.

---

## Q12 — Theme cookie scope

**Classification**: UX
**Decision owner**: Product Manager (low-stakes)
**Priority**: Low

### Why it's open

The theme cookie is scoped to path `/`. If a self-hosted install
is at `/sui-id/` rather than the root, the cookie path needs to
match.

### Current mockup assumption

Service is at the root.

### Current implementation reality

Backend may run under a path prefix.

### Recommended decisions

- Make the cookie path configurable (read from the same setting
  that drives external URLs).
- Or: switch to a session-bound theme preference (set once after
  login, stored server-side). Discuss before deciding.

---

## Issue triage

| Priority | Issues |
| --- | --- |
| **High** | Q1 (admin MFA reset) |
| **Medium** | Q2, Q3, Q7, Q9, Q10 |
| **Low** | Q4, Q5, Q6, Q8, Q11, Q12 |

The implementation team should resolve High-priority items before
v1 ships. Medium items can be tracked alongside v1 work. Low items
are explicitly deferrable.
