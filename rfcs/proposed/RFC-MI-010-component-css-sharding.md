# RFC-MI-010: Component CSS Sharding and Export Discipline

```toml
id = "RFC-MI-010"
title = "Component CSS Sharding and Export Discipline"
status = "Proposed"
phase = "Phase 1"
created = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## 1. Summary

Split the current monolithic component CSS surface into bounded shards before adopting additional mockup UI primitives.

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

- Replace the single large `components.rs` CSS holder with bounded component/CSS shards.
- Keep CSS injection compatible with the existing SSR full-document render path.
- Preserve the existing `status_badge` public component and controlled status vocabulary.
- Prepare a maintainable home for mockup primitives such as callouts, route tabs, form states, and danger-zone surfaces.
- Keep token and inline-style CI invariants intact.

## 4. Non-Goals

- Do not redesign the visual language yet.
- Do not migrate page layouts.
- Do not change routes or handler behavior.
- Do not add third-party CSS tooling.

## 5. Dependencies

- `RFC-MI-000`

## 6. External Design

This RFC creates the CSS/component file structure that later RFCs will use.

Target structure:

```text
crates/sui-id-web/src/
├── components.rs
└── components/
    ├── mod.rs
    ├── chrome.rs
    ├── cards.rs
    ├── forms.rs
    ├── tables.rs
    ├── buttons.rs
    ├── banners.rs
    ├── badges.rs
    ├── tabs.rs
    ├── confirm.rs
    ├── setup.rs
    └── utilities.rs
```

`components.rs` should become a small compatibility/export surface. It should
concatenate shard CSS constants into the same final CSS string consumed by
`layout.rs`.

Example public shape:

```rust
pub const COMPONENTS_CSS: &str = concat!(
    chrome::CHROME_CSS,
    cards::CARDS_CSS,
    forms::FORMS_CSS,
    tables::TABLES_CSS,
    buttons::BUTTONS_CSS,
    banners::BANNERS_CSS,
    badges::BADGES_CSS,
    tabs::TABS_CSS,
    confirm::CONFIRM_CSS,
    setup::SETUP_CSS,
    utilities::UTILITIES_CSS,
);
```


## 7. Detailed Design

### Shard Responsibilities

| Shard | Responsibility |
|---|---|
| `chrome.rs` | app header, admin nav, sidebar/top-nav if retained, footer, theme toggle |
| `cards.rs` | card, panel, summary, metric, callout surfaces |
| `forms.rs` | field, label, hint, validation, required marker, form grouping |
| `tables.rs` | table, wrapping, responsive behavior, copy cell affordances |
| `buttons.rs` | button variants, action groups, disabled/loading states |
| `banners.rs` | flash, alert, notice, status message surfaces |
| `badges.rs` | `status_badge`, status variants, semantic labels |
| `tabs.rs` | route-based tabs only |
| `confirm.rs` | confirmation and danger-zone surfaces |
| `setup.rs` | setup wizard progress and setup-specific layout |
| `utilities.rs` | bounded utility classes used to avoid inline styles |

### Export Discipline

- `components/mod.rs` owns shard declarations.
- `components.rs` remains the external import point for existing call sites.
- New Rust-rendered components belong near their CSS shard.
- Utility classes must remain bounded and named by role, not arbitrary styling whim.


## 8. Data / State / API Model

The split must improve maintainability without making the UI harder to reason
about. Each shard should represent one user-facing responsibility.

ABDD requirements:

- focus styles remain global and consistent
- semantic colors remain semantic
- disabled and danger states are not color-only
- utility classes do not obscure semantic HTML structure


## 9. UI/UX and ABDD Requirements

No persistence changes.

Possible Rust API changes:

```rust
pub use components::badges::{status_badge, StatusKind};
pub use components::banners::flash_banner;
```

`flash_banner` may be introduced here only if it is a pure rendering helper with
no behavior change. If flash unification is deferred, leave a TODO and keep
existing per-page helpers.


## 10. Migration Plan

1. Add `components/` module tree.
2. Move CSS families one at a time without changing class names.
3. Keep `COMPONENTS_CSS` output stable.
4. Move `StatusKind` and `status_badge` to `components/badges.rs`.
5. Run CI after each move if implementing incrementally.
6. Commit no page layout changes in this RFC.


## 11. Acceptance Criteria

- [ ] `components.rs` is no longer the monolithic CSS holder.
- [ ] Existing pages render with the same class names.
- [ ] `status_badge` remains available from the existing public import path.
- [ ] No new visible UI behavior is introduced.
- [ ] All CSS token references still resolve.
- [ ] No new inline styles are introduced.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- Snapshot or HTML smoke test on representative pages before/after class split.
- Manual visual check for dashboard, login, settings, audit, and confirm pages.

## 13. Risks and Mitigations

- **Risk:** Moving CSS changes cascade order.  
  **Mitigation:** Preserve original family ordering in `concat!`.

- **Risk:** New shards become dumping grounds.  
  **Mitigation:** Each shard must include a responsibility header and reject unrelated classes.


## 15. Rollback Plan

Restore the old `components.rs` from the previous release. Since this RFC is class-preserving, rollback should not require route or handler changes.
