# Dangerous-Action Map — Mockup ↔ Product

Phase-0 deliverable of [RFC-MI-000](../../../rfcs/done/RFC-MI-000-baseline-delta-inventory.md).
Generated against `sui-id-web-mockup v0.4.8` ↔ `sui-id v0.49.0`.

## The non-negotiable

Every destructive operation in the product traverses two distinct
HTTP exchanges:

1. **GET** a dedicated confirmation page (`/admin/.../delete-confirm`,
   `/admin/.../disable-confirm`, etc.) that renders the impact
   summary and a form whose action is the destructive POST.
2. **POST** the destructive action, gated by an unexpired step-up
   ticket and a valid CSRF token.

This is the **RFC 030 + RFC 058 + RFC 059** dangerous-operation
pattern. The mockup uses a **generic** `/stepup?action=X` →
`/confirm/{token}` pattern instead, with one render path for all
operations. Migration plan §D-02 and RFC-MI-051 require the **named
confirm route** model to be preserved. The generic `/confirm/{token}`
route is rejected.

The integration work for every row below is therefore:

- The mockup's `/stepup?action=<value>&return_to=Y` link is rewritten
  to point at the named GET confirm route (e.g.
  `/admin/users/{id}/delete-confirm`).
- The named confirm page already issues the step-up requirement.
- The destructive POST already enforces step-up + CSRF + audit
  emission.
- No protocol or audit-vocabulary change is needed.

## Mockup step-up actions ↔ product confirm routes

The mockup defines **18 step-up action values**. Each is mapped below.

### User management (`/admin/users/*`)

| Mockup `?action=` | Product confirm GET | Product destructive POST | `render_confirm_*` | Audit event | Step-up | CSRF | Decision |
|---|---|---|---|---|---|---|---|
| `user.suspend` | `/admin/users/{id}/disable-confirm` | `POST /admin/users/{id}/disabled` (`is_disabled=1`) | `render_confirm_disable_user` | `admin.user.disabled` | ✓ (RFC 058) | ✓ | mockup link rewrite; behaviour preserved |
| `user.delete` | `/admin/users/{id}/delete-confirm` | `POST /admin/users/{id}/delete` | `render_confirm_delete_user` | `admin.user.deleted` | ✓ (RFC 058) | ✓ | mockup link rewrite; behaviour preserved |
| `user.mfa_reset` | `/admin/users/{id}/mfa-reset-confirm` | `POST /admin/users/{id}/mfa-reset` | `render_confirm_reset_mfa` | `admin.user.mfa_reset` | ✓ (RFC 058) | ✓ | mockup link rewrite; behaviour preserved |
| `user.force_logout` | (no product route as of v0.49.0) | n/a | n/a | n/a | n/a | n/a | **D-OPEN.** Mockup surfaces a per-user "force logout" action; product has admin force-logout via `users_set_disabled` (which revokes sessions as a side-effect) but no dedicated route. **Default:** rely on the disabled-side-effect; do not add a route. RFC-MI-031 may revisit. |

User "resume" (un-disable) is not gated by step-up in the product
(RFC 058 §4 excluded "restorative" operations). Mockup does not flag
this as dangerous either.

### OIDC client management (`/admin/clients/*`)

| Mockup `?action=` | Product confirm GET | Product destructive POST | `render_confirm_*` | Audit event | Step-up | CSRF | Decision |
|---|---|---|---|---|---|---|---|
| `client.delete` | `/admin/clients/{id}/delete-confirm` | `POST /admin/clients/{id}/delete` | `render_confirm_delete_client` | `admin.client.deleted` | ✓ (RFC 058) | ✓ | mockup link rewrite; behaviour preserved |
| `client.secret.rotate` | (no dedicated confirm page; product POSTs directly with step-up) | `POST /admin/clients/{id}/rotate-secret` | (inline confirmation in client detail) | `admin.client.secret_rotated` (per RFC 047) | ✓ (step-up middleware) | ✓ | **needs-handler-review.** The product rotates without a separate `-confirm` GET because the operation is **reversible** (the new secret is shown once; the old is invalidated, but no data is destroyed). RFC-MI-051 must decide whether to add a confirm GET for visual consistency. **Default:** leave inline; the action is restoration-safe. |

The mockup does not surface a "client.disable" action distinct from
"delete"; the product has one (`POST /admin/clients/{id}/disabled`)
that does not require step-up (consistent with `user.suspend`
being step-up-only when the inverse is reversible).

### Signing keys (`/admin/signing-keys/*`)

| Mockup `?action=` | Product confirm GET | Product destructive POST | `render_confirm_*` | Audit event | Step-up | CSRF | Decision |
|---|---|---|---|---|---|---|---|
| `signing_key.publish` | (no confirm page; product POSTs directly) | `POST /admin/signing-keys/rotate` | (inline action) | `admin.signing_key.rotated` | (no step-up — rotate is additive) | ✓ | **needs-handler-review.** Mockup gates "publish new key" behind step-up. Product treats key rotation as additive (the new key joins the JWKS alongside the existing active key). The destructive event is *retirement* and *deletion*, not publishing. **Default:** keep product behaviour — rotation does not need step-up. |
| `signing_key.activate` | (no product route as of v0.49.0) | (no route) | n/a | n/a | n/a | n/a | **D-OPEN.** Product has no "activate pending key" route — it auto-activates the most recent key. RFC-MI-031 may surface this if Phase 3 absorbs the mockup's lifecycle table. **Default:** do-not-implement-yet. |
| `signing_key.retire` | (no product route as of v0.49.0) | (no route) | n/a | n/a | n/a | n/a | **D-OPEN.** Same status — product retires implicitly on next rotation. **Default:** do-not-implement-yet. |
| `signing_key.delete` | `/admin/signing-keys/{id}/delete-confirm` | `POST /admin/signing-keys/{id}/delete` | `render_confirm_delete_signing_key` | `admin.signing_key.deleted` | ✓ (RFC 058) | ✓ | mockup link rewrite; behaviour preserved |

### Settings (admin) — `/admin/settings/*`

| Mockup `?action=` | Product behaviour | Audit event | Step-up | CSRF | Decision |
|---|---|---|---|---|---|
| `settings.update` (umbrella for the "Review changes" → step-up → execute flow) | The product does not gate settings updates behind step-up. Each settings POST writes inline with CSRF; audit emits `admin.settings.<field>.updated`. | `admin.settings.*` | (none) | ✓ | **D-OPEN.** Mockup gates every settings update behind step-up; product does not. The migration plan does not flag settings updates as destructive enough to require step-up (they are reversible). **Default:** keep product behaviour. RFC-MI-051 may revisit if the mockup's intent is specifically about non-reversible settings (e.g. master-key rotation, which already requires the offline CLI). |

### Session management

| Mockup `?action=` | Surface | Product POST | Audit event | Step-up | CSRF | Decision |
|---|---|---|---|---|---|---|
| `me.session.revoke` | self-service single-session revoke | `POST /me/security/sessions/{id}/revoke` | `auth.session.revoked` | (none — user revoking own session is not destructive) | ✓ | mockup link rewrite. No step-up needed (RFC 058 §4 exempts self-targeted reversibility). |
| `me.sessions.revoke_all` | self-service "revoke all others" | `POST /me/security/sessions/revoke-all-others` | `auth.session.revoked_all_others` | (none) | ✓ | mockup link rewrite. Same rationale — user cannot lock themselves out (current session excluded). |
| `sessions.revoke_all` | admin force-logout (mockup surfaces this on `/admin/security`) | (none as of v0.49.0; admin force-logout is achieved by toggling user disabled) | n/a | n/a | n/a | **do-not-implement-yet.** Mockup's global "revoke all sessions" lever does not exist in product; admin must disable/re-enable target users. RFC-MI-031 may revisit. |

### Self-service MFA / passkeys

| Mockup `?action=` | Surface | Product POST | Audit event | Step-up | CSRF | Decision |
|---|---|---|---|---|---|---|
| `me.mfa.disable` | `/me/security/mfa` | `POST /me/security/mfa/disable` | `auth.mfa.disabled` | ✓ (RFC 058 — TOTP secret destruction) | ✓ | mockup link rewrite; behaviour preserved |
| `me.mfa.regen_recovery` | `/me/security/mfa` | `POST /me/security/mfa/recovery-codes/regenerate` | `auth.mfa.recovery_codes_regenerated` | (none — old codes invalidated, new shown once; mockup gates it but product treats it as reversible-by-regeneration) | ✓ | **D-OPEN.** Mockup gates this behind step-up; product does not. The argument for step-up is that an attacker with session access could regenerate recovery codes and exfiltrate them. **Recommendation:** add step-up to recovery-code regen in a future RFC (post-MI) — outside scope of the mockup integration arc. **Default:** keep product behaviour for v0.49.x. |
| `me.passkey.delete` | `/me/security/passkeys` | `POST /me/security/passkeys/{id}/delete` | `auth.passkey.deleted` | ✓ (RFC 058) | ✓ | mockup link rewrite; behaviour preserved |

## Confirm-screen template invariants

Every confirm GET emits a `confirm-shell` body whose form's `action`
is the destructive POST URL, includes the CSRF token, includes a
`_confirmed=1` hidden input (RFC 058), and includes the audit-reason
textarea (RFC 060). The integration must preserve all four
components:

- `_csrf` hidden input (CSRF) — `RFC-MI-021` (server-rendered CSRF
  for the Shell-level forms) ensures the token reaches the page;
  confirm screens already receive it.
- `_confirmed=1` hidden input — RFC 058 enforces server-side that
  this is present on any step-up-gated POST.
- `_reason` textarea — RFC 060 wrote this into every confirm
  template. Mockup callouts must absorb the reason input as a
  first-class field.
- Form action targets the named POST route. No generic
  `/confirm/{token}` POST exists.

## Step-up ticket flow (RFC-MI-051 detail)

When the mockup's `?action=X` link redirects to the product's named
confirm GET, the user has not yet performed step-up. The product's
named-confirm GET checks for the step-up ticket and, if absent,
redirects to `/me/security/step-up` with a `?next=` parameter
carrying the confirm GET URL. After successful re-auth, the user is
redirected back to the confirm GET, the ticket is now valid, and the
confirm page renders. The destructive POST then succeeds.

This means **the mockup's two-step `/stepup → /confirm/{token}` flow
maps cleanly onto the product's two-step `/me/security/step-up →
/admin/<resource>/{id}/<action>-confirm` flow** — only the URLs
differ.

## Aggregate

Out of **18 mockup `?action=` values**:

- **link-rewrite only (behaviour preserved)**: 9
  - `user.suspend`, `user.delete`, `user.mfa_reset`, `client.delete`,
    `signing_key.delete`, `me.session.revoke`,
    `me.sessions.revoke_all`, `me.mfa.disable`, `me.passkey.delete`.
- **link-rewrite, step-up retained but inline (no confirm GET)**: 1
  - `client.secret.rotate`. (RFC-MI-051 may add a confirm GET if
    visual consistency wins.)
- **do-not-implement-yet**: 5
  - `user.force_logout` (covered by disable side-effect),
    `signing_key.publish` (additive in product), `signing_key.activate`,
    `signing_key.retire`, `sessions.revoke_all` (global admin
    force-logout).
- **step-up policy delta (default: keep product)**: 3
  - `settings.update` (mockup gates; product does not),
    `me.mfa.regen_recovery` (mockup gates; product does not),
    `signing_key.publish` (mockup gates; product does not).

The 3 step-up policy deltas are surfaced for RFC-MI-051 review. They
do not block Phase 1. Each is a one-line behaviour switch that can be
adopted incrementally if security review agrees.

## Decisions surfaced

| ID | Action | Decision needed |
|---|---|---|
| **danger-D1** | `user.force_logout` | Add a dedicated route or rely on disable-side-effect? **Default:** rely on side-effect. |
| **danger-D2** | `client.secret.rotate` | Add confirm GET for visual consistency? **Default:** keep inline. |
| **danger-D3** | `signing_key.{publish,activate,retire}` | Surface as explicit actions? **Default:** keep additive rotation model. |
| **danger-D4** | `sessions.revoke_all` (admin global) | Add a global "kick everyone" lever? **Default:** no (rely on per-user disable). |
| **danger-D5** | `settings.update` step-up | Adopt mockup's "every settings change is gated" model? **Default:** no (reversible). |
| **danger-D6** | `me.mfa.regen_recovery` step-up | Adopt mockup's gating? **Default:** no for v0.49.x; revisit in a follow-up RFC. |
