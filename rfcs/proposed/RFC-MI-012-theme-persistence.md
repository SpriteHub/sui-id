# RFC-MI-012: Theme Persistence Decision

```toml
id = "RFC-MI-012"
title = "Theme Persistence Decision"
status = "Proposed"
phase = "Phase 1"
created = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## 1. Summary

Decide and document how theme preference is stored and applied before visual mockup adoption proceeds.

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

- Resolve the current product model vs mockup expectation for theme persistence.
- Prevent avoidable theme flash or inconsistent server/client state.
- Keep the solution lightweight and compatible with SSR-only rendering.
- Define precedence between system preference, localStorage, and cookies if multiple mechanisms are used.

## 4. Non-Goals

- Do not redesign the color palette.
- Do not introduce hydration.
- Do not add a full preferences subsystem unless separately RFC'd.

## 5. Dependencies

- `RFC-MI-000`

## 6. External Design

Three options are allowed:

### Option A — Preserve Current Model

- `theme-init.js` reads `localStorage`.
- It sets `data-theme` on `<html>`.
- Absence means system preference.

This is the recommended default because it is lightweight and already aligned
with the product.

### Option B — Cookie-Backed Theme

- Server reads a theme cookie.
- SSR emits `data-theme`.
- Client updates cookie and possibly `localStorage`.

This reduces mismatch but introduces cookie policy and precedence complexity.

### Option C — Hybrid Compatibility

- Server respects cookie if present.
- Client keeps `localStorage` for existing installs.
- A migration rule resolves conflicts.

This is only justified if the mockup requires server-visible theme state.


## 7. Detailed Design

### Required Decision Record

The RFC implementation must add a short decision record under docs or RFC
appendix:

```text
theme-persistence = "localStorage" | "cookie" | "hybrid"
precedence = [...]
fouc-mitigation = [...]
rollback = [...]
```

### Theme Toggle Contract

Regardless of storage model:

- theme choices remain `system`, `light`, `dark`
- theme toggle remains keyboard accessible
- labels are localized
- no inline event handlers are introduced
- `prefers-reduced-motion` is respected


## 8. Data / State / API Model

The theme choice must not be the only way to perceive state. All status,
warning, success, and danger meanings must remain textually and structurally
clear.

Theme switching must not trap focus or reset the user's page context.


## 9. UI/UX and ABDD Requirements

No database changes.

Possible cookie model if chosen:

| Name | Value | Scope | SameSite | HttpOnly |
|---|---|---|---|---|
| `sui_id_theme` | `system` / `light` / `dark` | `/` | `Lax` | `false` |

If a cookie is introduced, the decision must explain why it is not sensitive and
why it is acceptable for client-side modification.


## 10. Migration Plan

1. Compare product theme model and mockup expectations.
2. Choose Option A, B, or C.
3. Update `ThemeToggle` and `theme-init.js` only if required.
4. Update docs describing theme persistence.
5. Run dark/light/system checks on representative pages.


## 11. Acceptance Criteria

- [ ] Theme persistence model is explicitly documented.
- [ ] No visible theme flash regression is introduced.
- [ ] Theme toggle remains no-hydration and no-framework.
- [ ] All theme labels are localized.
- [ ] Rollback path is documented.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- Manual test: system → light → dark → system.
- Manual test: reload after theme change.
- Manual test: first paint in light and dark OS modes.

## 13. Risks and Mitigations

- **Risk:** Cookie and localStorage diverge.  
  **Mitigation:** Avoid hybrid unless necessary; define precedence if used.

- **Risk:** Server-side theme state creates unnecessary complexity.  
  **Mitigation:** Prefer current `theme-init.js` unless there is a proven need.


## 15. Rollback Plan

Return to the existing `theme-init.js` + `localStorage` behavior and remove any newly introduced theme cookie handling.
