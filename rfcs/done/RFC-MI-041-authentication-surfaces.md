# RFC-MI-041: Authentication Surface Integration

```toml
id = "RFC-MI-041"
title = "Authentication Surface Integration"
status = "Implemented (v0.53.0)"
phase = "Phase 4"
created = "2026-05-18"
implemented = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## Implementation note (added on transition to `done/`)

Implemented in **v0.53.0** — the first Phase-4 release. Shipped
**ahead of RFC-MI-040** because the auth surfaces are tighter in
scope and security-sensitive; the setup wizard work (RFC-MI-040)
follows in v0.53.1.

### Security guarantee

**Zero copy changed. Zero i18n keys changed.** A line-level diff
of `pages/auth.rs` and the entire `sui-id-i18n` crate against
v0.52.0 (excluding `class=`/`style=` attributes) is empty.
Anti-enumeration wording, MFA failure copy, step-up purpose copy,
and reset-token failure copy are byte-identical to v0.52.0. No
backend auth logic is touched.

### Changes made

**Three inline styles eliminated in `pages/auth.rs`:**

| Site | Before | After |
|---|---|---|
| Login "Forgot password?" link | `<p class="muted" style="margin-top:…;text-align:center;font-size:…">` | `<p class="muted auth-meta-link">` |
| MFA setup TOTP QR code | `<div inner_html=qr_svg style="max-width:240px;margin-bottom:…">` | `<div inner_html=qr_svg class="qr-display">` |
| Password change card | `<div class="card" style="max-width:var(--content-narrow-width)">` | `<div class="card card--narrow">` |

**Two new CSS classes in `components/setup.rs`:**

- `.auth-meta-link` — muted, caption-size, centered, top-margined.
  For "Forgot password?", "Back to sign-in", and similar meta links
  below auth forms.
- `.qr-display` — bounded TOTP QR-code container
  (`max-width: 240px; margin-bottom: --space-3`).

**One new CSS variant in `components/cards.rs`:**

- `.card--narrow` — constrains card to `--content-narrow-width`.
  Used by the password-change form and any other isolated
  single-action card.

**ABDD: flash banner role per kind (`pages/common.rs`).**
`FlashKind::aria_role()` returns `"alert"` for `Error` and
`"status"` for `Info`/`Warn`. Error banners now interrupt
assistive tech immediately (login failure, MFA failure, step-up
failure, reset-token failure) while informational banners stay
polite. The helper change is transparent to every caller.

### Acceptance criteria

- [x] Login failure remains generic — wording unchanged from v0.52.0.
- [x] Forgot-password request does not disclose account existence —
  the existing neutral confirmation page is preserved.
- [x] MFA failure wording remains safe — unchanged.
- [x] Step-up purpose is clearly explained — unchanged.
- [x] All text localised — no new visible strings introduced.
- [x] No-JS form submission still works (no script change; forms
  remain plain `method="post"` with hidden `_csrf` server-rendered
  per RFC-MI-021).
- [x] `inline-style-bound` decreases (10 → 7 in this release).
- [x] Errors are announced with `role="alert"` (FlashKind::Error
  now maps to `role="alert"`; non-error flashes keep `role="status"`).

---

## 1. Summary

Adopt mockup UX improvements for login, MFA, password reset, and step-up screens while preserving security wording and timing protections.

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

- Improve clarity of authentication screens.
- Preserve anti-enumeration wording.
- Preserve server-driven MFA and reset flows.
- Preserve no-JS operation.
- Provide consistent error/flash placement.

## 4. Non-Goals

- Do not change password hashing, lockout, MFA, or reset-token logic.
- Do not reveal whether an account exists.
- Do not add client-side-only validation as a security dependency.

## 5. Dependencies

- `RFC-MI-011`
- `RFC-MI-020`
- `RFC-MI-021`

## 6. External Design

Affected surfaces:

- login
- MFA challenge
- password change outside tab if present
- forgot-password request
- forgot-password sent
- reset-password
- reset-password invalid/expired
- step-up authentication

External layout should use `AuthShell` and narrow card composition.

Error copy must remain generic where enumeration risk exists.


## 7. Detailed Design

### Error Presentation

Use a consistent banner/helper pattern:

- global generic failure banner for login failure
- inline validation for malformed local input
- neutral confirmation page for password reset email request
- generic invalid/expired reset link page

### Step-Up

Step-up is security-critical and may be visually improved, but its purpose must
remain clear: confirm the current user before sensitive operations.


## 8. Data / State / API Model

ABDD requirements:

- forms have labels, not placeholders only
- errors are announced with `role='alert'` or appropriate live region
- recovery paths are visible but not noisy
- submit buttons have disabled/loading style if server supports pending state
- sensitive errors avoid account enumeration


## 9. UI/UX and ABDD Requirements

No database migration.

Potential render data:

```rust
pub struct AuthFormState {
    pub flash: Option<Flash>,
    pub field_errors: Vec<FieldError>,
    pub return_to_label: Option<String>,
}
```

Do not store field errors persistently. They are render-only.


## 10. Migration Plan

1. Define shared auth form primitives.
2. Update login render.
3. Update MFA challenge render.
4. Update forgot/reset password renders.
5. Update step-up render.
6. Security-review all changed copy.


## 11. Acceptance Criteria

- [ ] Login failure remains generic.
- [ ] Forgot-password request does not disclose account existence.
- [ ] MFA failure wording remains safe.
- [ ] Step-up purpose is clearly explained.
- [ ] All text is localized.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- Integration test for login failure wording.
- Integration test for forgot-password neutral response.
- Manual screen-reader check for error announcement.
- No-JS form submission check for login and reset.

## 13. Risks and Mitigations

- **Risk:** UX clarity accidentally reveals account or token validity.  
  **Mitigation:** Security review all failure copy and keep neutral states.


## 15. Rollback Plan

Restore previous auth render functions. Do not change core auth logic in this RFC.
