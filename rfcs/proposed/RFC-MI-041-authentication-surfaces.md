# RFC-MI-041: Authentication Surface Integration

```toml
id = "RFC-MI-041"
title = "Authentication Surface Integration"
status = "Proposed"
phase = "Phase 4"
created = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

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
