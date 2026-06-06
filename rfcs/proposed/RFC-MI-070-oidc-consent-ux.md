# RFC-MI-070: OIDC Consent UX Integration

```toml
id = "RFC-MI-070"
title = "OIDC Consent UX Integration"
status = "Proposed"
phase = "Phase 7"
created = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

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
