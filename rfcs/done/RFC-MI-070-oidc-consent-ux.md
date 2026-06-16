# RFC-MI-070: OIDC Consent UX Integration

```toml
id = "RFC-MI-070"
title = "OIDC Consent UX Integration"
status = "Implemented (v0.56.0)"
phase = "Phase 7"
created = "2026-05-18"
implemented = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## Implementation note (added on transition to `done/`)

Implemented in **v0.56.0**.

### Changes made

**Four new CSS classes** added to `components/setup.rs` (which owns
the auth-card centred layout — the logical home for consent-screen
styles):

- `.consent-card` — `max-width: 32rem` modifier applied on top of
  `.auth-card`. The consent screen needs 512px vs the login screen's
  448px (`--content-narrow-width`) to accommodate the scope list
  comfortably.
- `.consent-intro` — `margin: var(--space-3) 0` for the "App X
  wants access to:" paragraph. Replaces `style="margin:var(--space-3) 0"`.
- `.consent-scope-list` — `list-style: none; padding: 0; margin: 0
  0 var(--space-4); display: flex; flex-direction: column; gap:
  var(--space-1)`. Replaces
  `style="list-style:none;padding:0;margin-bottom:var(--space-4)"`.
- `.consent-scope-item` — `display: flex; flex-direction: column; gap: 2px`
  for each scope row (vertical stack: title over description).
- `.consent-scope-item__title` — bold scope label, fg-default.
- `.consent-scope-item__desc` — caption-size muted description sentence.

**`render_consent` in `pages/oidc.rs` rewritten:**
- `<div class="auth-card" style="max-width:32rem">` → `<div class="auth-card consent-card">`
- `<p style="margin:var(--space-3) 0">` → `<p class="consent-intro">`
- `<ul style="list-style:none;…">` → `<ul class="consent-scope-list">`
- `<li style="margin:var(--space-1) 0">` → `<li class="consent-scope-item">`

**Four scope description i18n keys added** (×3 locales — en/ja/zh):
`consent_scope_openid_desc`, `consent_scope_profile_desc`,
`consent_scope_email_desc`, `consent_scope_offline_access_desc`.
Each scope item now renders as:
```
<span class="consent-scope-item__title">Verify your identity</span>
<span class="consent-scope-item__desc">Confirms your sign-in and provides a unique identifier.</span>
<code class="text-caption muted">openid</code>
```
Unmapped scopes show `"—"` as title with no description and the raw slug as
the code element.

**`inline-style-bound` reaches 0.** This RFC eliminates the last 4
inline styles in the codebase — every `style=` attribute in
`pages/oidc.rs` is now a CSS class. The MI arc's inline-style
discipline target is met.

### Protocol behaviour unchanged

- Authorization Code + PKCE flow is unchanged.
- Exact redirect URI validation is unchanged.
- Approve and Deny are both POST forms with CSRF hidden fields.
- Deny is rendered as a `secondary` button with equal keyboard
  access to Approve — not hidden as a text link.

### Acceptance criteria

- [x] Authorization Code + PKCE behaviour is unchanged.
- [x] Consent screen explains scopes in user-readable language — scope label + description sentence; raw slug shown as `<code>` for developer context.
- [x] Approve and Deny are visually clear and accessible (both are `<button>` in `<form method="post">` — keyboard reachable, no JS required).
- [x] All consent text is localised — four new `consent_scope_*_desc` keys added in all three locale files.
- [x] `inline-style-bound` = 0 (all four `oidc.rs` inline styles eliminated).

---

## 1. Summary

Integrate mockup consent-screen clarity while preserving OIDC protocol correctness and exact security behavior.

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

- Make relying-party consent understandable to end users.
- Preserve Authorization Code + PKCE behavior.
- Preserve exact redirect URI validation.
- Make approve and deny both first-class actions.
- Localize scope explanations.

## 4. Non-Goals

- Do not change token issuance.
- Do not add unsupported OIDC flows.
- Do not show raw scopes as the only explanation.
- Do not change redirect validation.

## 5. Dependencies

- `RFC-MI-041`

## 6. External Design

The consent screen should answer:

- Which application is asking?
- What will it access?
- Which account is being used?
- What happens if the user allows?
- What happens if the user denies?

External layout:

```text
AuthShell
└── Consent Card
    ├── Client identity
    ├── Account context
    ├── Scope explanation list
    ├── Redirect/domain safety note if appropriate
    └── Approve / Deny actions
```


## 7. Detailed Design

### Scope Explanation Map

Introduce or extend render-side mapping:

```rust
pub struct ConsentScopeView {
    pub scope: String,
    pub title: String,
    pub description: String,
    pub tone: SurfaceTone,
}
```

Raw scope may be present as secondary developer detail only if useful and safe.

### Actions

Approve and deny are both POST actions with CSRF.

Deny must not be hidden as a small text link. It is a valid privacy-preserving
choice.


## 8. Data / State / API Model

ABDD requirements:

- scope list is a real list
- approve and deny are keyboard reachable
- deny is visible and understandable
- client identity is text, not just icon/logo
- screen does not rely on color to distinguish allow/deny


## 9. UI/UX and ABDD Requirements

No database migration.

Render data may add:

```rust
pub struct ConsentClientView {
    pub display_name: String,
    pub client_id: String,
    pub redirect_origin: String,
    pub is_confidential: bool,
}
```

Use existing authorization validation results. Do not re-validate protocol
rules in the renderer.


## 10. Migration Plan

1. Add scope explanation i18n keys.
2. Update `render_consent`.
3. Preserve handler/core protocol behavior.
4. Add consent HTML tests.
5. Run OIDC flow tests.


## 11. Acceptance Criteria

- [ ] Authorization Code + PKCE behavior is unchanged.
- [ ] Consent screen explains scopes in user-readable language.
- [ ] Approve and deny are visually clear and accessible.
- [ ] All consent text is localized.
- [ ] Protocol tests still pass.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- OIDC authorization flow integration tests.
- HTML assertion for scope explanation list.
- Keyboard check for approve/deny actions.

## 13. Risks and Mitigations

- **Risk:** Scope descriptions overpromise or misrepresent access.  
  **Mitigation:** Review scope copy with protocol owner.


## 15. Rollback Plan

Restore previous `render_consent`. No protocol data migration is involved.
