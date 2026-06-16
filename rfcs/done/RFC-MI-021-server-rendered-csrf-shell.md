# RFC-MI-021: Server-Rendered CSRF for Shell-Level Forms

```toml
id = "RFC-MI-021"
title = "Server-Rendered CSRF for Shell-Level Forms"
status = "Implemented (v0.51.0)"
phase = "Phase 2"
created = "2026-05-18"
implemented = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## Implementation note (added on transition to `done/`)

Implemented in **v0.51.0** alongside RFC-MI-020.

### What changed

**`layout.rs::Shell`** — Added `csrf_token: String` as a required
prop. Shell passes it through to `<Nav>`, which renders it directly
into the sign-out form's hidden `<input name="_csrf">` field.
Sign-out now works with JavaScript disabled.

**`layout.rs::Nav`** — Removed the fallback logic
(`if csrf_token.is_empty() { "".into() } else { csrf_token }`)
and the `<script src="/static/logout-csrf.js">` reference. The
token is always real; the JS fallback is no longer needed.

**`crates/sui-id/static/logout-csrf.js` deleted.** The file that
read the `sui_id_csrf` cookie and populated the hidden input at
submit time is removed. It no longer ships with the product.

**27 Shell call sites updated** across 19 pages/render files.
Every call passes a real CSRF token:
- Most files: `csrf_token=csrf_token.clone()` (token already in
  scope as a render function parameter).
- `confirm.rs` (5 calls) + `me_security/overview.rs` + `users.rs`
  (user detail): `csrf_token=data.csrf_token.clone()` (token is a
  field on the data struct).

**5 render function signatures updated** to accept `csrf_token`:
`render_dashboard`, `render_audit`, `render_settings_authentication`,
`render_settings_logs`, `render_settings_other`. Their handlers
already called `csrf::ensure_token(&jar)` but discarded the result
after the cookie-set; the token is now also forwarded to the
render layer.

**`AuthShell`** — not changed. `AuthShell` does not render a nav or
a sign-out form; its forms (login, setup, password-reset) already
received CSRF tokens through their per-page `csrf_token` parameters.

### Acceptance criteria

- [x] No `Shell` call site passes an empty CSRF token (all 27 call sites pass a real token; empty-string fallback removed from Nav).
- [x] Sign-out works with JavaScript disabled (hidden `_csrf` input value is server-rendered into the HTML).
- [x] Existing POST CSRF validation still passes (all tests pass; no change to `enforce_csrf` / `ensure_token`).
- [x] `logout-csrf.js` is removed (file deleted from `crates/sui-id/static/`; script tag removed from Nav).
- [x] No new CSRF bypass introduced (sign-out always requires a valid session-bound token; the server-side path is identical).

---

## 1. Summary

Replace shell-level JavaScript CSRF population with server-rendered CSRF fields.

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

- Make shell-level POST forms work without JavaScript.
- Thread real CSRF tokens into `Shell` rendering.
- Remove or reduce reliance on `logout-csrf.js`.
- Preserve existing server-side CSRF validation.
- Make future mockup shell actions safe by default.

## 4. Non-Goals

- Do not change CSRF token generation semantics.
- Do not make CSRF cookies HttpOnly if current form architecture requires client-readable cookies elsewhere.
- Do not add new unsafe POST routes.

## 5. Dependencies

- `RFC-MI-020`

## 6. External Design

This RFC is a blocker before interactive shell adoption.

The sign-out form must be rendered as:

```html
<form method="post" action="/admin/logout">
  <input type="hidden" name="_csrf" value="...">
  <button type="submit">Sign out</button>
</form>
```

The value must be supplied by the handler/render boundary, not by JavaScript.


## 7. Detailed Design

### Render Signature Strategy

Preferred direction: introduce a small shared shell context to reduce repeated
parameter churn.

```rust
pub struct ShellContext {
    pub current: ShellCurrent,
    pub csrf_token: String,
    pub dev_mode: bool,
}
```

Alternatively, add `csrf_token: String` to every render function that uses
`Shell`.

### Handler Strategy

Each authenticated GET handler that renders `Shell` must obtain or renew the
CSRF token and pass it into the renderer.

### JavaScript Strategy

After server-rendered CSRF is complete:

- keep `logout-csrf.js` only if still needed by legacy pages
- otherwise remove it from `Shell`
- document the removal in CHANGELOG


## 8. Data / State / API Model

ABDD and safety requirements:

- sign-out remains a normal button in a normal form
- no JS requirement for sign-out
- no hidden interaction surprises
- keyboard users can reach and submit the form
- failure returns a safe error or login redirect, not internal details


## 9. UI/UX and ABDD Requirements

No persistence changes.

State boundary:

- CSRF token remains request/session state.
- Rendered HTML receives the token as a string.
- POST handlers continue to call existing `enforce_csrf`.

Potential affected render functions include all admin and self-service pages.


## 10. Migration Plan

1. Add `csrf_token` to `Shell` or `ShellContext`.
2. Update render functions using `Shell`.
3. Update handlers to pass the token.
4. Render shell sign-out hidden input server-side.
5. Remove or de-scope `logout-csrf.js`.
6. Add no-JS test coverage.


## 11. Acceptance Criteria

- [ ] No `Shell` call site passes an empty CSRF token.
- [ ] Sign-out works with JavaScript disabled.
- [ ] Existing POST CSRF validation still passes.
- [ ] `logout-csrf.js` is removed or explicitly retained only for documented legacy reasons.
- [ ] No new CSRF bypass is introduced.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- Integration test: GET admin page, extract `_csrf`, POST `/admin/logout` succeeds.
- Integration test: POST `/admin/logout` without CSRF fails.
- Manual no-JS sign-out test.

## 13. Risks and Mitigations

- **Risk:** Wide render signature churn causes mistakes.  
  **Mitigation:** Prefer a shared `ShellContext` and update call sites mechanically.

- **Risk:** Token rendered into unexpected contexts.  
  **Mitigation:** Only render into hidden form fields for POST forms.


## 15. Rollback Plan

Revert render signature changes and restore `logout-csrf.js`. Security reviewer approval is required before rollback because this RFC improves no-JS CSRF integrity.
