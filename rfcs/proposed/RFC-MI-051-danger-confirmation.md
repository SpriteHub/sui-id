# RFC-MI-051: Danger Zone and Confirmation Screen Integration

```toml
id = "RFC-MI-051"
title = "Danger Zone and Confirmation Screen Integration"
status = "Proposed"
phase = "Phase 5"
created = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## 1. Summary

Adopt the mockup's danger-zone visual model while preserving product-specific confirmation routes, CSRF, step-up, and audit behavior.

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

- Physically and semantically isolate destructive operations.
- Preserve existing `render_confirm_*` routes.
- Preserve step-up requirements.
- Preserve CSRF enforcement.
- Introduce impact summaries for destructive operations.

## 4. Non-Goals

- Do not introduce generic `/confirm/{token}` routing.
- Do not replace confirmation GET pages with inline-only prompts.
- Do not weaken audit logging.

## 5. Dependencies

- `RFC-MI-050`
- `RFC-MI-021`

## 6. External Design

External detail page structure:

```text
Detail Page
├── Read surface
├── Safe settings/actions
└── Danger Zone
    ├── explanation
    ├── operation-specific impact summary
    └── link/button to confirmation route
```

Confirmation pages remain product-specific, for example:

- disable user
- delete user
- reset MFA
- delete client
- delete signing key


## 7. Detailed Design

### ConfirmScreenData

Extend only if needed:

```rust
pub struct ConfirmImpactItem {
    pub label: String,
    pub value: String,
    pub tone: SurfaceTone,
}

pub struct ConfirmScreenData {
    // existing fields
    pub impact: Vec<ConfirmImpactItem>,
    pub irreversible: bool,
}
```

If existing `ConfirmScreenData` already supports this through generic fields,
do not add new fields.

### Danger Zone CSS

`confirm.rs` owns:

- `.danger-zone`
- `.danger-zone__title`
- `.danger-zone__body`
- `.impact-summary`
- `.impact-summary__item`


## 8. Data / State / API Model

ABDD requirements:

- danger meaning must be text and structure, not only red color
- confirmation page must identify the target object
- irreversible consequences must be clear
- cancel path must be visible and keyboard reachable
- focus order must reach safe cancel and final danger action predictably


## 9. UI/UX and ABDD Requirements

No database migration.

No new confirmation persistence.

All confirmation POSTs must continue to include:

- explicit action route
- CSRF field
- operation-specific target identifier
- existing audit event behavior


## 10. Migration Plan

1. Add danger-zone and impact-summary primitives.
2. Update user/client/signing-key detail pages to use danger zone.
3. Update existing confirmation renderers with impact summaries.
4. Confirm CSRF and step-up behavior is unchanged.
5. Security-review all destructive action copy.


## 11. Acceptance Criteria

- [ ] No destructive action is inline-only.
- [ ] Existing product confirmation routes remain.
- [ ] CSRF is present on destructive POSTs.
- [ ] Step-up requirements are preserved.
- [ ] Audit event expectations are preserved.
- [ ] Danger meaning is accessible without color.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- Integration test for destructive route still requiring CSRF.
- Integration test for confirmation GET before destructive POST where applicable.
- Manual keyboard check for cancel and danger submit order.

## 13. Risks and Mitigations

- **Risk:** Mockup generic confirm route leaks into product.  
  **Mitigation:** Explicitly reject `/confirm/{token}` in this RFC.


## 15. Rollback Plan

Restore previous confirmation page markup. Do not roll back security route behavior unless separately approved.
