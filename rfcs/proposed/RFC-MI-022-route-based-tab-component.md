# RFC-MI-022: Route-Based Tab Component

```toml
id = "RFC-MI-022"
title = "Route-Based Tab Component"
status = "Proposed"
phase = "Phase 2"
created = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## 1. Summary

Create a reusable SSR route-tab helper that preserves deep-linkable product routes and rejects query-param tab state.

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

- Preserve `/me/security/*` distinct routes.
- Preserve `/admin/settings/*` distinct routes.
- Provide a shared tab visual and semantic component.
- Use anchor links with `aria-current='page'`.
- Avoid client-side tab state or query-param dependency.

## 4. Non-Goals

- Do not introduce SPA routing.
- Do not collapse tabs into one handler.
- Do not use `?tab=` as the product state model.

## 5. Dependencies

- `RFC-MI-010`
- `RFC-MI-020`
- `RFC-MI-021`

## 6. External Design

The mockup's tab intent is useful, but its query-parameter model must be adapted.

Tabs must be rendered as normal links:

```html
<nav class="route-tabs" aria-label="Security sections">
  <a href="/me/security/overview" aria-current="page">Overview</a>
  <a href="/me/security/mfa">MFA</a>
</nav>
```

The active tab is computed server-side from an explicit tab enum or current key.


## 7. Detailed Design

### Proposed API

```rust
pub struct RouteTab {
    pub key: &'static str,
    pub href: &'static str,
    pub label: &'static str,
    pub description: Option<&'static str>,
}

pub fn route_tabs(
    label: &'static str,
    current: &'static str,
    tabs: &'static [RouteTab],
) -> impl IntoView
```

If localization requires runtime labels, use owned/string label fields instead.

### Target Consumers

- `MeShellData` / `/me/security/*`
- settings tab shell / `/admin/settings/*`

### CSS

`tabs.rs` owns `.route-tabs`, `.route-tabs__link`, and active/focus states.
Existing `.me-tabs` may remain as alias during migration but should not grow.


## 8. Data / State / API Model

ABDD requirements:

- tab set uses `nav` or `role='navigation'`, not ARIA tabs unless client-side
  panel switching is implemented
- active link uses `aria-current='page'`
- focus order follows visual order
- tab labels are short and localized
- current state is text/structure-visible, not color-only


## 9. UI/UX and ABDD Requirements

No persistence changes.

Possible enum additions:

```rust
pub enum MeSecurityTab {
    Overview,
    Mfa,
    Sessions,
    Passkeys,
    Language,
    Password,
}

pub enum SettingsTab {
    Basic,
    Authentication,
    Email,
    Security,
    Logs,
    Other,
}
```

Enums must map to routes, not query values.


## 10. Migration Plan

1. Add `RouteTab` helper.
2. Migrate `/me/security/*` shell to it.
3. Migrate `/admin/settings/*` shell to it.
4. Remove duplicated tab CSS where safe.
5. Update tests to assert route links and active state.


## 11. Acceptance Criteria

- [ ] No product tab uses `?tab=` as its state model.
- [ ] Every tab is directly reachable by URL.
- [ ] `aria-current='page'` is present on active tab.
- [ ] Tabs work without JavaScript.
- [ ] Both self-service and settings tabs use the shared helper or a documented equivalent.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- HTML assertion: `/me/security/mfa` contains links to all self-service tab routes.
- HTML assertion: active tab has `aria-current='page'`.
- Manual keyboard navigation across tabs.

## 13. Risks and Mitigations

- **Risk:** Developers copy mockup query-param URLs.  
  **Mitigation:** Document explicit rejection and add tests for path links.

- **Risk:** ARIA tabs are misused for page navigation.  
  **Mitigation:** Use navigation semantics instead of widget-tab semantics.


## 15. Rollback Plan

Restore existing hand-written tab markup. Do not introduce query-param tabs during rollback.
