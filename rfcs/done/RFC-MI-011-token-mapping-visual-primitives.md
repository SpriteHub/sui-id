# RFC-MI-011: Mockup Token Mapping and Visual Primitive Adoption

```toml
id = "RFC-MI-011"
title = "Mockup Token Mapping and Visual Primitive Adoption"
status = "Implemented (v0.50.1)"
phase = "Phase 1"
created = "2026-05-18"
implemented = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## Implementation note (added on transition to `done/`)

Implemented in **v0.50.1** alongside RFC-MI-012. The Phase 0
inventory (`docs/mockup-integration/inventory/token-delta-draft.md`)
already contained the token delta table; the key findings:

### Token delta table

| Mockup need | Existing mapping | New token? | Reason |
|---|---|---|---|
| Colour semantics (fg, surface, accent, semantic palette) | All 33 mockup tokens ⊂ existing 75 tokens | **none** | Mockup vocabulary is a strict subset of the product's |
| Spacing rhythm (8px, 12px, 16px, 24px, 32px, 48px) | `--space-1..--space-6` (identity mapping) | **none** | Perfect match |
| Intermediate spacing (14px, 10px, 6px, 18px, 20px, 28px) | Rounded to nearest `--space-*` (see token-delta-draft §4) | **none** | Within 4px of an existing token; below perception threshold |
| Typography scale | `--font-size-caption / body / h3 / h2 / display` | **none** | All mockup font-size values map to existing scale |
| Border widths, radius, shadow | Existing `--border-width-*`, `--radius-*`, `--shadow-*` | **none** | All map directly |

**Zero new tokens added.** `tokens.rs` is unchanged.

### Primitives adopted

Three of the six candidate primitives are adopted in this release;
three are explicitly deferred to their owning RFCs:

| Primitive | Shard | Status | Notes |
|---|---|---|---|
| `.callout` + tone variants | `cards.rs` | ✅ adopted | Neutral explanatory block (surface-subtle + border-muted + space-3). Four semantic variants (info, success, warning, danger). Supplements but does not replace `.card--callout` (which uses accent fill for CTA blocks). |
| `.field__error` + `.field--invalid` | `forms.rs` | ✅ adopted | Inline validation error (danger-default text, caption size). `.field--invalid` triggers red border on contained inputs. Both require `aria-describedby` linkage from the input — not enforced in CSS, must be enforced in page templates. |
| `.dl-grid` | `utilities.rs` | ✅ adopted | Definition-list key-value grid for admin detail screens. Semantic `<dl>/<dt>/<dd>` wrapper; replaces ad-hoc `<table>` for non-tabular key-value data. |
| `metric-card` | — | not needed | Already covered by existing `.card` + `.stat` composition. Documented only. |
| `impact-summary` | `confirm.rs` | ⏳ deferred | Designed fully in RFC-MI-051 (Phase 5). |
| `route-tabs` | `tabs.rs` | ⏳ deferred | Designed fully in RFC-MI-022 (Phase 2). |
| `danger-zone` | `confirm.rs` | ⏳ deferred | Designed fully in RFC-MI-051 (Phase 5). |

### Acceptance criteria

- [x] Token delta table committed (above — zero new tokens).
- [x] Every new token has a documented reason (n/a — none added).
- [x] No mockup spacing token added without mapping analysis (spacing reconciliation in token-delta-draft.md §4).
- [x] Semantic palette parity remains green (36 declarations, unchanged).
- [x] New primitives are reusable and accessible (`.callout`, `.field__error`, `.dl-grid` use token vars throughout; focus states inherited from global `:focus-visible`; `aria-describedby` linkage documented in `.field__error` comment).

---

## 1. Summary

Map the mockup visual language onto the current bounded CSS token vocabulary and adopt only justified visual primitives.

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

- Create a complete token delta table.
- Map the mockup's spacing rhythm to existing `--space-*` tokens wherever possible.
- Preserve semantic palette discipline.
- Adopt mockup primitives only as reusable, accessible components/classes.
- Avoid token bloat.

## 4. Non-Goals

- Do not replace screen layouts.
- Do not change route or handler behavior.
- Do not add a design system package or build step.
- Do not introduce arbitrary one-off CSS variables.

## 5. Dependencies

- `RFC-MI-000`
- `RFC-MI-010`

## 6. External Design

The visual system must remain small. Existing token categories are the default:

- spacing
- foreground
- surface
- accent
- semantic palette
- border / radius / state
- typography
- layout widths

The mockup's 4px-based thinking should be expressed through existing tokens
where practical. If the product token is 8px-based at the exposed scale, do not
add intermediate values unless the mockup cannot meet accessibility or layout
requirements without them.

Token delta table format:

| Mockup need | Existing mapping | New token? | Reason | Affected screens |
|---|---|---|---|---|


## 7. Detailed Design

### Primitive Adoption Rules

A primitive may be adopted if it is:

- used by at least two screen groups, or
- critical for ABDD / security comprehension, or
- required to remove inline styles, or
- required to replace a mockup-only visual pattern with a product-safe one.

Candidate primitives:

| Primitive | Target shard | Notes |
|---|---|---|
| `callout` | `cards.rs` / `banners.rs` | For explanatory setup/security blocks |
| `impact-summary` | `confirm.rs` | For destructive operation review |
| `metric-card` | `cards.rs` | For dashboard summary cards |
| `route-tabs` | `tabs.rs` | Designed fully in RFC-MI-022 |
| `field-error` | `forms.rs` | For accessible validation states |
| `danger-zone` | `confirm.rs` | Designed fully in RFC-MI-051 |

### Rejection Rule

If a mockup primitive is only decorative and not necessary for clarity, safety,
or consistency, reject it or map it to an existing class.


## 8. Data / State / API Model

Visual primitives must support:

- visible focus states
- dark and light modes
- reduced motion
- semantic labels
- non-color state indication
- screen-reader friendly content order

A callout must not become the only place where critical instructions exist; it
must supplement headings, labels, and form hints.


## 9. UI/UX and ABDD Requirements

No database changes.

No new persistent state.

Possible additions:

```rust
pub enum SurfaceTone {
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}
```

This enum is optional and must not duplicate `FlashKind` or `StatusKind` unless
the RFC justifies separate semantics.


## 10. Migration Plan

1. Build token delta table from Phase 0.
2. Reject unnecessary tokens.
3. Add only justified tokens to `tokens.rs`.
4. Add or adapt primitives in the appropriate shard.
5. Update representative existing pages only where needed to prove the primitive.
6. Defer full page migrations to later RFCs.


## 11. Acceptance Criteria

- [ ] Token delta table is committed with the RFC.
- [ ] Every new token has a documented reason.
- [ ] No mockup spacing token is added without mapping analysis.
- [ ] Semantic palette parity remains green.
- [ ] New primitives are reusable and accessible.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- Manual dark/light theme visual check.
- Manual focus-visible check for new interactive primitives.
- Reduced-motion check for any transitions.

## 13. Risks and Mitigations

- **Risk:** Token growth makes the design system harder to maintain.  
  **Mitigation:** Require explicit rejection/mapping table.

- **Risk:** Primitive names encode appearance instead of purpose.  
  **Mitigation:** Name classes by responsibility or semantic role.


## 15. Rollback Plan

Remove newly added tokens and primitives. Because this RFC should not migrate pages broadly, rollback should be low-risk.
