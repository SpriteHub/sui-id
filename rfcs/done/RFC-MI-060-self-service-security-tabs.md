# RFC-MI-060: Self-Service Security Tab Integration

```toml
id = "RFC-MI-060"
title = "Self-Service Security Tab Integration"
status = "Implemented (v0.55.0)"
phase = "Phase 6"
created = "2026-05-18"
implemented = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## Implementation note (added on transition to `done/`)

Implemented in **v0.55.0**.

### Primary change: password tab now has the tab strip

`render_password_change` in `pages/auth.rs` was the only one of
the six `/me/security/*` routes that did not show the route-based
tab strip. This was explicitly deferred from RFC-MI-022.

Changes made:
- `show_nav=false` → `show_nav=true` (admin nav now visible)
- `current=None` → `current=Some("me")` (Security nav item highlighted)
- `{me_security_tabs(MeTab::Password, lang)}` inserted above the
  page header
- Import `use super::me_security::{me_security_tabs, MeTab}` added
  to `pages/auth.rs`
- Cancel link updated from `/me/security` to `/me/security/overview`
  (the new canonical overview route from v0.51.1)
- Form submit/cancel buttons migrated to `.form-actions` class
  (using the RFC-MI-050 primitive)

All six `/me/security/*` routes now consistently show:
1. The route-based tab strip (`.route-tabs`)
2. `aria-current="page"` on the active tab
3. `show_nav=true current="me"` so the top-nav "Security" link
   is highlighted

### MFA enable/disable decision record

**Option 2 chosen:** self-service MFA enable (TOTP setup via
`/me/security/mfa`) + admin-initiated reset (via
`/admin/users/{id}/mfa-reset-confirm`).

- Users can enrol TOTP at any time from the MFA tab.
- Users can delete their own TOTP via the confirmation flow
  on the MFA tab.
- Admins can forcibly reset a user's MFA via the user detail
  page's danger zone (RFC-MI-051 v0.54.0).
- Step-up (`/me/step-up`) is required before any TOTP change
  (enforced since v0.45.0, RFC 058).
- No additional code changes were needed; the existing product
  already implements this model.

### Recovery code warnings

Existing recovery-code countdown on the MFA tab already uses
a text count ("N remaining") alongside a `.badge--danger` pill
when the count is low. Non-colour indicator (text count) satisfies
ABDD §8 "recovery code warnings are not color-only".

### Session revocation labels

Existing session labels already distinguish "This device" from
other sessions, and the revoke-all-others button is labelled
`me_security_sessions_revoke_all_others` (i18n key). Meets
RFC §8 "session revocation labels identify scope".

### Acceptance criteria

- [x] All self-service tabs remain path-based (`aria-current="page"`).
- [x] MFA state is clear and safe (MFA decision documented above).
- [x] Session revocation scope is unambiguous (existing labels).
- [x] Passkey actions preserve existing constraints (unchanged).
- [x] All new text is localised — no new i18n keys needed;
  existing `me_tab_password` key from v0.51.1 is used.

---

## 1. Summary

Integrate mockup IA and visual improvements into `/me/security/*` while preserving route-based tabs and security boundaries.

## 2. Background

The mockup integration must be treated as a controlled architectural migration,
not as a direct visual replacement. The current product is already a working
Rust / Axum / Leptos SSR service with security-sensitive identity flows.
The mockup provides UI/UX intent: information hierarchy, screen relationships,
ABDD behavior, visual language, and operational clarity.

This RFC preserves the following project-level constraints:

- Leptos SSR only.
- No hydration dependency.
- No third-party CSS framework.
- Preserve public `render_*` entry points unless this RFC explicitly changes them.
- Preserve handler-side owned `*Data` structs.
- Preserve i18n table discipline.
- Preserve CSRF, step-up, confirmation, audit, and anti-enumeration contracts.
- Preserve CI gates for text leaks, CSS tokens, semantic palette parity, and inline-style bounds.

## 3. Goals

- Improve self-service security overview clarity.
- Preserve path-based tab routes.
- Clarify MFA, passkey, session, language, and password areas.
- Keep user-facing security actions safe and understandable.
- Surface open decision for MFA enable/disable ownership.

## 4. Non-Goals

- Do not convert self-service tabs to query parameters.
- Do not add unsupported account-management features.
- Do not weaken admin/user permission boundaries.

## 5. Dependencies

- `RFC-MI-022`
- `RFC-MI-050`
- `RFC-MI-051`

## 6. External Design

Affected routes:

```text
/me/security/overview
/me/security/mfa
/me/security/sessions
/me/security/passkeys
/me/security/language
/me/security/password
```

The area should read as "your security controls", not as an admin console.

External layout:

```text
Shell
└── Self-service Security
    ├── Route tabs
    ├── Current tab page title
    ├── Security status / explanatory callout
    └── Tab-specific content
```


## 7. Detailed Design

### MFA Enable/Disable Decision

This RFC must decide or explicitly defer where MFA enable/disable controls live.

Options:

1. self-service enable/disable
2. self-service enable + admin reset
3. admin reset only
4. require step-up before self-service MFA changes

Recommended direction: self-service management is acceptable if step-up and
recovery paths are clear; admin reset remains for recovery/support.

### Session Controls

Session revocation must distinguish:

- revoke one session
- revoke all other sessions
- current session behavior

### Passkeys

Passkey add/remove actions must preserve current WebAuthn constraints and
no-JS fallback limits.


## 8. Data / State / API Model

ABDD requirements:

- each tab has a clear heading
- sensitive state is explained in plain language
- recovery code warnings are not color-only
- session revocation labels identify scope
- route tabs use `aria-current='page'`
- all actions work without client-side routing


## 9. UI/UX and ABDD Requirements

No database migration unless existing backend lacks fields required by UI.

Render data may include:

```rust
pub struct SecurityOverviewData {
    pub mfa_state: MfaStateSummary,
    pub passkey_count: usize,
    pub active_session_count: usize,
    pub recent_events: Vec<SecurityEventSummary>,
}

pub enum MfaStateSummary {
    NotEnabled,
    TotpEnabled { recovery_codes_remaining: usize },
    PasskeyAvailable { count: usize },
}
```

Use existing backend data where possible.


## 10. Migration Plan

1. Apply route tab helper.
2. Update overview tab.
3. Update MFA tab after MFA control decision.
4. Update sessions and passkeys tabs.
5. Update language and password tabs.
6. Add i18n keys and security-copy review.


## 11. Acceptance Criteria

- [ ] All self-service tabs remain path-based.
- [ ] MFA state is clear and safe.
- [ ] Session revocation scope is unambiguous.
- [ ] Passkey actions preserve existing constraints.
- [ ] All new text is localized.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- HTML assertions for route tab links.
- Integration test for self-service access requiring `CurrentUser`.
- Manual no-JS tab navigation check.
- Security review for MFA and session action copy.

## 13. Risks and Mitigations

- **Risk:** Self-service controls become too powerful without step-up.  
  **Mitigation:** Decide MFA/session action protections explicitly.


## 15. Rollback Plan

Restore previous `/me/security/*` renderers while preserving route-tab helper if already used by settings.
