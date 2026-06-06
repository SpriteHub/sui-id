# RFC-MI-031: Audit Log and Read-Only Table Integration

```toml
id = "RFC-MI-031"
title = "Audit Log and Read-Only Table Integration"
status = "Proposed"
phase = "Phase 3"
created = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## 1. Summary

Adopt mockup table readability improvements for audit and other read-only surfaces while preserving copy and audit contracts.

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

- Improve audit/table scanability.
- Preserve audit-row copy contract.
- Define wrapping behavior for IDs, timestamps, actors, and free text.
- Improve empty states and filter presentation where already supported.

## 4. Non-Goals

- Do not add new audit event types.
- Do not change audit hash-chain verification.
- Do not add export formats unless already implemented.
- Do not add client-side filtering.

## 5. Dependencies

- `RFC-MI-010`
- `RFC-MI-011`
- `RFC-MI-030`

## 6. External Design

External layout for audit:

```text
Admin Shell
└── Audit Log
    ├── Header: title + chain status
    ├── Verification/action row
    ├── Filter form if supported
    ├── Audit table
    └── Empty state
```

Read-only tables should have stable column behavior:

| Column type | Behavior |
|---|---|
| ID / hash | single-line, copy button where applicable |
| timestamp | single-line where possible |
| actor | wrap only if necessary |
| event name | controlled vocabulary |
| free-text detail | may wrap using `.cell-wrap` |


## 7. Detailed Design

### Table Classes

`tables.rs` owns:

- `.table-wrap`
- `.cell-wrap`
- `.cell-nowrap`
- `.cell-id`
- `.cell-actions`
- `.table-empty-row`

### Copy Contract

Rows with opaque IDs must retain:

```html
<button data-copy="..." data-copy-done="...">Copy id</button>
```

Do not replace this with a new JS mechanism.


## 8. Data / State / API Model

ABDD requirements:

- tables must have headers
- empty states must be explicit
- copy buttons must have accessible labels
- filters must be real forms if present
- responsive behavior must not destroy reading order


## 9. UI/UX and ABDD Requirements

No database migration.

Render data may add presentation-only fields:

```rust
pub struct CopyableId {
    pub value: String,
    pub label: String,
}

pub enum TableCellKind {
    Text,
    Id,
    Timestamp,
    Status,
    Actions,
}
```

Only add these if they reduce duplicated markup and do not obscure page-specific
semantics.


## 10. Migration Plan

1. Extract common table classes into `tables.rs`.
2. Update audit page first.
3. Update other read-only tables only if covered by this RFC's test plan.
4. Preserve existing copy.js behavior.
5. Add mobile wrapping checks.


## 11. Acceptance Criteria

- [ ] Audit table preserves copy behavior.
- [ ] Free-text columns wrap while ID/timestamp columns remain stable.
- [ ] Empty state is readable and localized.
- [ ] No audit hash-chain logic changes.
- [ ] No client-side filtering dependency is introduced.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- HTML assertion for `data-copy` buttons.
- Manual narrow viewport table check.
- Keyboard access check for copy buttons.

## 13. Risks and Mitigations

- **Risk:** Responsive table CSS hides important audit details.  
  **Mitigation:** Prefer horizontal scroll or controlled wrapping over column hiding.


## 15. Rollback Plan

Restore previous audit/table markup and keep any harmless table classes if already used elsewhere.
