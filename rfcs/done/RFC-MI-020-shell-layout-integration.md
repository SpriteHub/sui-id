# RFC-MI-020: Shell Layout Integration

```toml
id = "RFC-MI-020"
title = "Shell Layout Integration"
status = "Implemented (v0.51.0)"
phase = "Phase 2"
created = "2026-05-18"
implemented = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## Implementation note (added on transition to `done/`)

Implemented in **v0.51.0** alongside RFC-MI-021.

**Decision: Preserve top-nav model.** No structural shell code
changes were required in this release; the decision record below
is the primary deliverable.

### Shell layout decision record

```
nav-model         = "top-nav"
shell-split       = "Shell (authenticated) + AuthShell (public)"
sidebar-adopted   = false
justification     = "The current top-nav satisfies all IA requirements
                     surfaced in the Phase-0 screen-map inventory.
                     The mockup itself uses a top-nav model for the same
                     seven nav items. A sidebar was not proven necessary
                     by the screen-map analysis."
mobile-model      = "horizontal-scroll nav at 768px breakpoint (v0.48.2 Bug 8)"
active-state      = "current=Option<String> key matched in Nav; aria-current='page'"
skip-link         = "deferred — add if shell density increases in Phase 3+"
rfcs-mi-021       = "CSRF threading (v0.51.0) is the structural partner of this
                     decision: it adds csrf_token to Shell, which is the primary
                     Shell API change in Phase 2."
```

### Navigation streams

Per §7 — **not mixed**:

| Stream | Shell | Nav section |
|---|---|---|
| Setup | `AuthShell` | — |
| Login / auth | `AuthShell` | — |
| OIDC consent | `AuthShell` | — |
| Admin operations | `Shell` | Full 7-item nav |
| Self-service security | `Shell` | "Security" item (`current="me"`) |

Self-service shares the Shell chrome but its `current="me"` key
causes only the "Security" nav link to be highlighted — admin-only
links (Users, Clients, Signing Keys, etc.) remain visible but
unemphasised. This is acceptable for an operator-facing product
where all authenticated users are known operators.

### ShellCurrent enum

The RFC §7 proposed a `ShellCurrent` enum to replace the
`current: Option<String>` stringly-typed parameter. This is
**deferred to RFC-MI-022** (Route-Based Tab Component) where the
same pages are already touched for the tab-helper migration.
Adding the enum there avoids a second pass over the same call
sites.

### Acceptance criteria

- [x] Admin navigation active state is accurate (current nav-key system unchanged; `aria-current="page"` set on active link).
- [x] Self-service navigation does not expose admin-only actions to normal users (correct — user role checked by handler; nav key is "me" not an admin key).
- [x] Shell works without JavaScript (sign-out form server-renders CSRF token via RFC-MI-021 — the JS fallback `logout-csrf.js` is removed).
- [x] Mobile navigation is usable at 768px and narrower (`@media (max-width: 768px)` rules in `chrome.rs::CHROME_RESPONSIVE_CSS`; `white-space: nowrap` keeps nav labels on one line; horizontal scroll is the current fallback).
- [x] No new frontend routing or hydration introduced.

---

## 1. Summary

Integrate the mockup's global shell and navigation intent while preserving SSR, route clarity, and no-JS baseline behavior.

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

- Define the final admin shell navigation model.
- Preserve `Shell` and `AuthShell` separation unless a new shell is justified.
- Preserve clear active navigation state.
- Define mobile behavior at the current 768px breakpoint.
- Avoid new JavaScript dependencies.

## 4. Non-Goals

- Do not change authentication extractors.
- Do not change route semantics.
- Do not merge setup/login/OIDC/admin/self-service streams.
- Do not add a frontend router.

## 5. Dependencies

- `RFC-MI-010`
- `RFC-MI-011`
- `RFC-MI-012`

## 6. External Design

The product currently has two top-level shells:

- `Shell` for authenticated admin and self-service pages
- `AuthShell` for setup, login, MFA, password reset, and error pages

This RFC decides whether the mockup's sidebar model should be adopted or whether
the current top-nav model should evolve conservatively.

### Recommended Default

Adopt the mockup's orientation improvements without forcing a sidebar unless
the screen-map proves that top navigation cannot support the required IA.

If sidebar is chosen:

- it must be SSR-rendered
- it must collapse without JS
- it must remain keyboard accessible
- it must not introduce focus traps
- it must preserve route-based active state


## 7. Detailed Design

### Shell API Direction

`Shell` should evolve only as much as required:

```rust
pub struct ShellNavItem {
    pub key: &'static str,
    pub href: &'static str,
    pub label: &'static str,
}

pub enum ShellCurrent {
    Dashboard,
    Users,
    Clients,
    Security,
    Settings,
    Audit,
    MeSecurity,
}
```

Use enums or stable keys, not stringly-typed page state where avoidable.

### Navigation Streams

Do not mix:

- uninitialized setup
- login/authentication
- OIDC consent
- admin operations
- self-service security

Self-service may share `Shell` chrome, but its navigation must remain
distinguishable from admin-only actions.


## 8. Data / State / API Model

ABDD requirements:

- `nav` landmark for navigation
- `main` landmark for page content
- `aria-current="page"` on the active navigation target
- skip link if shell density increases
- visible focus ring on nav links
- no color-only active indication
- mobile nav must be reachable by keyboard without JS


## 9. UI/UX and ABDD Requirements

No database changes.

Possible new view data:

```rust
pub struct ShellData {
    pub current: ShellCurrent,
    pub csrf_token: String,
    pub dev_mode: bool,
    pub version: String,
}
```

If introduced, it must not force excessive render signature churn before
RFC-MI-021 settles CSRF threading.


## 10. Migration Plan

1. Decide top-nav vs sidebar.
2. Update shell CSS in `chrome.rs`.
3. Preserve current route links.
4. Add active state semantics.
5. Verify mobile layout at 768px and narrower.
6. Keep login/setup/AuthShell untouched unless explicitly required.


## 11. Acceptance Criteria

- [ ] Admin navigation active state is accurate.
- [ ] Self-service navigation does not expose admin-only actions to normal users.
- [ ] Shell works without JavaScript.
- [ ] Mobile navigation is usable at 768px and narrower.
- [ ] No new frontend routing or hydration is introduced.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- Keyboard-only navigation through shell links.
- Manual screen-reader landmark check.
- Mobile viewport check at 768px, 480px, and 360px.

## 13. Risks and Mitigations

- **Risk:** Sidebar introduces mobile complexity.  
  **Mitigation:** Keep top-nav unless sidebar is strongly justified.

- **Risk:** Shell refactor creates wide render signature churn.  
  **Mitigation:** Coordinate with RFC-MI-021 and use a small `ShellData` if helpful.


## 15. Rollback Plan

Restore previous `Shell` layout and `chrome.rs` classes. Do not roll back RFC-MI-021 if server-rendered CSRF has already improved security.
