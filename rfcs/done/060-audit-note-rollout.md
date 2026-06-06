# RFC 060 — Audit-note rollout

**Status.** Implemented (v0.45.0)
**Priority.** P1 — Phase D (v0.45.0)
**Tracks.** PDF slide "Dangerous operations make themselves visible" —
the audit-log half. Extends the RFC 045 pattern (operator-supplied
reason on `user.disable`) to every dangerous action.
**Touches.** `crates/sui-id-core/src/admin.rs` (audit calls in 6 use
cases switch from `audit_ok` to `audit_with_note`), `crates/sui-id/src/handlers/admin.rs`
(form structs gain optional `reason`), `crates/sui-id/src/handlers/me_security.rs`
(self-service dangerous routes — mfa_disable, passkey_delete), `crates/sui-id-web/src/pages.rs`
(confirm screens get a reason textarea via the RFC 059 component).

## Background

The audit log table has a `note` column (added at v0.40.0 per RFC 045
to support `user.disable` reason). One action populates it:
`user.disable` / `user.enable`. The other 8 dangerous actions write
`note = None`:

| Action | Use case fn | Current audit |
|--------|-------------|---------------|
| user.disable / user.enable | `set_user_disabled` | `audit_with_note` ✅ |
| user.delete | `delete_user` | `audit_ok` ❌ |
| user.mfa_reset | (in admin_uc) | `audit_ok` ❌ |
| user.reset_password | `reset_password` | `audit_ok` ❌ |
| client.create | `create_client` | `audit_ok` (not dangerous) |
| client.disable / enable | `set_client_disabled` | `audit_ok` ❌ |
| client.delete | `delete_client` | `audit_ok` ❌ |
| client.rotate_secret | `rotate_client_secret` | `audit_ok` ❌ |
| signing_key.rotate | `rotate_signing_keys` | `audit_ok` ❌ |
| signing_key.delete | `delete_signing_key` | `audit_ok` ❌ |
| mfa.enable | (handler) | `note: None` ❌ |
| mfa.disable | (handler) | `note: None` ❌ |
| mfa.recovery_codes_regenerate | (handler) | `note: None` ❌ |
| webauthn.credential.register | (handler) | `note: None` (low risk) |
| webauthn.credential.delete | (handler) | `note: None` ❌ |

The operator-provided reason is the audit forensics signal: when a
user is locked out of production at 03:00 UTC, the log entry should
say *why* — "suspected account takeover" vs "off-boarding" vs
"compromised credential". Without `note`, the audit log is a list of
*what happened* with no *why*.

## Goal

Every dangerous action accepts an optional `reason` field on its
form, and the use case threads that reason through to `audit_with_note`.
The confirm screen (RFC 059) renders a small textarea for the reason
on each dangerous action.

## Design

### Scope: dangerous actions get note

The 7 dangerous actions getting note treatment:

| Action | Form param | Carries reason |
|--------|-----------|---------------|
| user.disable / user.enable | `reason` (existing) | ✅ already done (RFC 045) |
| user.delete | `reason` (new) | new |
| user.mfa_reset | `reason` (new) | new |
| client.disable / enable | `reason` (new) | new |
| client.delete | `reason` (new) | new |
| client.rotate_secret | `reason` (new) | new |
| signing_key.rotate | `reason` (new) | new |
| signing_key.delete | `reason` (new) | new |

The 3 self-service dangerous actions get note treatment too:

| Action | Form param | Carries reason |
|--------|-----------|---------------|
| mfa.disable (self) | (none) | canonical short code: `note: "self"` |
| webauthn.credential.delete (self) | (none) | canonical short code: `note: "self"` |
| sessions.revoke_all_others | (none) | canonical short code: `note: "self"` |

The self-service routes don't prompt for a reason — that would be
annoying friction on the user's own account. Instead, the audit log
shows `note = "self"` so operators triaging suspicious activity can
distinguish "user reduced their own MFA from /me/security" (often
benign) from "admin reset MFA on user X" (always suspicious if
unexpected).

### Use case signature changes

Six functions in `sui-id-core/src/admin.rs` gain an optional `reason`
parameter:

```rust
pub async fn delete_user(
    db: &Database, actor: UserId, target: UserId,
    reason: Option<String>,    // new
) -> CoreResult<()> { ... }

pub async fn reset_mfa(
    db: &Database, clock: &SharedClock, actor: UserId, target: UserId,
    reason: Option<String>,    // new
) -> CoreResult<MfaResetReport> { ... }

pub async fn set_client_disabled(
    db: &Database, clock: &SharedClock, actor: UserId, target: ClientId,
    disabled: bool,
    reason: Option<String>,    // new
    caches: &Caches,
) -> CoreResult<()> { ... }

pub async fn delete_client(
    db: &Database, actor: UserId, target: ClientId,
    reason: Option<String>,    // new
    caches: &Caches,
) -> CoreResult<()> { ... }

pub async fn rotate_client_secret(
    db: &Database, clock: &SharedClock, actor: UserId, target: ClientId,
    reason: Option<String>,    // new
    caches: &Caches,
) -> CoreResult<String> { ... }

pub async fn rotate_signing_keys(
    db: &Database, clock: &SharedClock, actor: UserId, algorithm: &str,
    reason: Option<String>,    // new
    caches: &Caches,
) -> CoreResult<SigningKeyId> { ... }

pub async fn delete_signing_key(
    db: &Database, clock: &SharedClock, actor: UserId, target: SigningKeyId,
    reason: Option<String>,    // new
    caches: &Caches,
) -> CoreResult<()> { ... }
```

All audit calls inside switch from `audit_ok(...)` to
`audit_with_note(..., reason)`.

### Form-data changes

The relevant `*Form` structs gain an optional `reason` field:

```rust
#[derive(Debug, Deserialize, Default)]
pub struct ConfirmedReasonForm {
    #[serde(rename = "_csrf", default)]
    pub csrf: String,
    #[serde(rename = "_confirmed", default)]
    pub confirmed: String,
    #[serde(default)]
    pub reason: String,    // trimmed; "" → None
}
```

Several handlers currently use `ConfirmedForm` (no reason); they
migrate to `ConfirmedReasonForm` or, where the reason is canonical,
keep `ConfirmedForm` and pass a fixed string.

### `_confirmed=1` enforcement on disable routes (bug fix)

A latent bypass: `users_set_disabled` and `clients_set_disabled` use
`DisableForm` (without `_confirmed`). The confirm screen exists and
emits `_confirmed=1`, but the handler doesn't check it. A direct POST
with just `_csrf` succeeds, skipping the confirm screen.

RFC 060 fixes this by:
- Defining `DisableForm` with `_confirmed` field (already has
  `disabled` and `reason`).
- Calling `require_confirmed(&form.confirmed)?` in the handlers.

The forthcoming `users_disable_confirm_get` and the new
`clients_disable_confirm_get` already emit `_confirmed=1` per RFC 030;
the bypass closes silently.

### Self-service note canonicalisation

For `mfa_disable`, `passkey_delete`, `revoke_all_others`, the handler
writes `note: Some("self".into())` directly when calling
`audit::append`. This is unambiguously distinct from operator-set
free-text reasons (which won't equal the literal token "self" by
convention).

## Test plan

1. Unit: each affected use case test confirms `note` is written
   correctly:
   - Operator-supplied reason → stored verbatim (trimmed).
   - Empty reason → `note: None`.
   - Self-service paths → `note: "self"`.
2. E2E: POST each dangerous action with a `reason=…` form field;
   GET the audit log; assert the note appears in the rendered row.
3. Manual: trigger each dangerous action via the confirm screen
   with a typed reason; check `/admin/audit` shows the note.

## Rollout

Single release. The audit table's `note` column already exists
(RFC 045, v0.40.0); only the **write side** changes. Historical rows
with `note = None` continue to render correctly (the audit view
already handles `None`).

The form-data change is additive: handlers that don't yet send
`reason=…` continue to work because `#[serde(default)]` accepts
absence.

## Future work

A future RFC could add a typed "reason category" dropdown (off-boarding,
suspected compromise, scheduled rotation, etc.) for faster triage.
RFC 060 deliberately keeps the field free-text to match the existing
RFC 045 pattern; the dropdown is a separate UX decision.
