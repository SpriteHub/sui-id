# RFC 058 — Dangerous-action step-up enforcement

**Status.** Implemented (v0.45.0)
**Priority.** P0 — Phase D (v0.45.0)
**Tracks.** PDF slide "Dangerous operations". v0.41.0 audit identified
4 routes that meet the "dangerous" definition but lack the step-up
guard. Phase D closes the gap.
**Touches.** `crates/sui-id/src/handlers/admin.rs` (users_set_disabled,
clients_set_disabled), `crates/sui-id/src/handlers/me_security.rs`
(mfa_disable, passkey_delete).

## Background

The PDF defines "dangerous" operations as actions that meaningfully
reduce the security or availability of a principal: disable, delete,
MFA reset, force logout, rotate client secret, rotate signing key,
disable own MFA, delete own passkey. Every such action must:

1. Show a confirm screen explaining what happens and what is
   reversible (RFC 030 / RFC 040).
2. Re-authenticate via step-up immediately before the action
   (RFC 020 / RFC 021).
3. Commit the result to the audit log with `note` populated
   (RFC 045 pattern; RFC 060 rolls it out fully).

The v0.41.0 review found 6 routes correctly guarded by
`require_fresh_step_up`: `users.delete`, `users.mfa_reset`,
`clients.delete`, `clients.rotate_secret`, `signing_keys.rotate`,
`signing_keys.delete`. Four dangerous routes lacked the guard:

| Route | Handler | Risk |
|-------|---------|------|
| `POST /admin/users/{id}/disabled` | `users_set_disabled` | An attacker with a stale cookie can lock out arbitrary users (incl. admins). |
| `POST /admin/clients/{id}/disabled` | `clients_set_disabled` | An attacker can disable production OIDC clients, breaking dependent services. |
| `POST /me/security/mfa/disable` | `mfa_disable` (post-v0.44.0) | An attacker with a stale cookie can downgrade their target's account security. |
| `POST /me/security/passkeys/{id}/delete` | `passkey_delete` (post-v0.44.0) | Similar: attacker removes a legitimate factor before phishing the survivor. |

## Goal

The four routes above gain `require_fresh_step_up` immediately after
CSRF enforcement, matching the pattern already used by their stronger
cousins (`users_delete`, `clients_delete`, etc.).

## Design

Each handler is updated in the same shape:

```rust
pub async fn users_set_disabled(
    state_ext: AppStateExt,
    CurrentAdmin(admin_id): CurrentAdmin,
    ctx: crate::handlers::SessionContext,    // (new) needed for step-up
    jar: CookieJar,
    Path(id): Path<String>,
    Form(form): Form<DisableForm>,
) -> Result<Response, HttpError> {
    let State(app) = state_ext;
    crate::handlers::enforce_csrf(&jar, Some(&form.csrf))?;
    // (new) Step-up immediately after CSRF.
    if let Err(redirect) =
        crate::handlers::require_fresh_step_up(&app, &ctx, "/admin/users").await
    {
        return Ok(redirect);
    }
    // ... rest unchanged
}
```

The `SessionContext` extractor is already in use by every step-up
guarded handler; adding it here is mechanical.

### Return-to URLs

| Handler | return_to |
|---------|-----------|
| `users_set_disabled` | `/admin/users` |
| `clients_set_disabled` | `/admin/clients` |
| `mfa_disable` (self) | `/me/security/mfa` |
| `passkey_delete` (self) | `/me/security/passkeys` |

The self-service routes return to the relevant tab so that, after
step-up completes, the user lands back on the page they were
clicking from.

### Confirm-screen flow (out of scope but related)

`users_set_disabled` already has a paired `users_disable_confirm_get`
handler at admin.rs L714 that renders a confirm screen, but the POST
endpoint never enforces `_confirmed=1` (no `ConfirmedForm`). Direct
POSTs that skip the confirm screen succeed — a bypass that RFC 060
closes by also requiring `_confirmed=1` on these four routes. This
RFC 058 only adds step-up; the `_confirmed` enforcement comes via
RFC 060.

### Self-service framing

The two self-service routes (`mfa_disable`, `passkey_delete`) are
acting on the **user's own** account, not someone else's. The
"dangerous" framing still applies — reducing your own MFA is exactly
the kind of action an attacker would take after compromising a
session — but the step-up redirect target lives at
`/me/security/step-up` (same as the existing `revoke_all_others`
step-up). The guard's `return_to` parameter ensures the user lands
back where they were, not on `/admin/users`.

## Test plan

1. **Unit-style integration**: for each of the 4 routes, an e2e test
   submits the POST without a fresh step-up cookie; expect 302
   redirect to `/admin/login/step-up?return=...` or
   `/me/security/step-up?return=...`. Existing tests for the route
   that DO have a fresh step-up cookie continue to pass.
2. **Pattern parity**: a grep verifying that every POST handler
   under `users_*`, `clients_*`, `signing_keys_*` that mutates state
   contains `require_fresh_step_up` returns no false negatives.
3. **No regression** on the 11 routes already guarded.

## Rollout

Single release. Operators with admin sessions that have completed
step-up within the freshness window (5 minutes by default) see no
difference. Operators with stale sessions see one extra
re-authentication prompt on disable/delete actions, matching the
existing prompt on `delete`/`rotate`.

The CHANGELOG flags this as a security-hardening change with no
schema migration.
