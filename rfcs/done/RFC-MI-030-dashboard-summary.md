# RFC-MI-030: Dashboard and Summary Surface Integration

```toml
id = "RFC-MI-030"
title = "Dashboard and Summary Surface Integration"
status = "Implemented (v0.52.0)"
phase = "Phase 3"
created = "2026-05-18"
implemented = "2026-05-18"
project = "sui-id"
scope = "Mockup integration into sui-id v0.48.4"
language = "English"
```

## Implementation note (added on transition to `done/`)

Implemented in **v0.52.0** alongside RFC-MI-031.

### Changes made

**Warning callout migration** — The operator warning section
(SMTP not configured, HIBP off, insecure cookie) was rendered as
`<section class="card card--warn">` with an inline-styled `<h2>`.
It now uses `<section class="callout callout--warning">` (the
neutral callout primitive introduced in v0.50.1) with `<h2
class="callout__title">`. The `.callout__title` rule was added to
`cards.rs` (`font-size: body; font-weight: medium; margin: 0 0 space-2`).

**Sparkline layout classes** — Four inline styles in the sparkline
section replaced with CSS classes added to `utilities.rs`:
`.sparkline-container` (SVG dimensions), `.sparkline-header`
(flex row), `.sparkline-title` (h3 reset), `.sparkline-legend`
(legend row).

**No DashboardData struct changes.** The existing fields
(`user_count`, `client_count`, `active_session_count`, warning
flags, sparkline) are sufficient for Phase 3. `DashboardMetric`
and `RecentActivityItem` structs from §7 are deferred.

**Metric card links** — The existing metric cards (users, clients,
sessions) already render as `.card > .stat` with nearby `<a>`
links. No structural change needed; the existing links suffice.

### Acceptance criteria

- [x] Dashboard warning section uses `.callout--warning` primitive.
- [x] No inline `style=` attributes on the dashboard page's warning
  section or sparkline section.
- [x] No destructive action on the dashboard (unchanged).
- [x] All text localised (unchanged — no new i18n keys needed).
- [x] Mobile layout unchanged (`.grid-cards` CSS unchanged).

---

## 1. Summary

Adopt the mockup's dashboard information hierarchy and summary card surfaces without changing backend semantics.

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

- Improve dashboard scanability.
- Adopt reusable metric and summary card primitives.
- Preserve existing `render_dashboard` boundary.
- Keep dashboard read-only except existing safe links.
- Represent health and status semantically.

## 4. Non-Goals

- Do not add analytics features.
- Do not change audit log generation.
- Do not change session, user, client, or key data models.

## 5. Dependencies

- `RFC-MI-011`
- `RFC-MI-020`
- `RFC-MI-021`

## 6. External Design

The dashboard should be the admin entry point, not a dense control center.

External layout:

```text
Admin Shell
└── Main
    ├── Page header: title + short state explanation
    ├── Status callout / dev mode notice if applicable
    ├── Metric card grid
    │   ├── Users
    │   ├── Clients
    │   ├── Sessions / recent sign-ins
    │   └── Audit / system status
    ├── Recent important activity
    └── Next operational actions
```

Primary information appears above the fold. Dangerous operations do not appear
on the dashboard.


## 7. Detailed Design

### Render Boundary

Keep:

```rust
pub fn render_dashboard(
    data: DashboardData,
    flash: Option<Flash>,
    dev_mode: bool,
    lang: Locale,
) -> String
```

or migrate to `ShellContext` if RFC-MI-021 introduces it.

### DashboardData Extensions

Only add fields already available from existing handlers/repositories unless a
separate backend RFC is approved.

Candidate additions:

```rust
pub struct DashboardMetric {
    pub label: String,
    pub value: String,
    pub tone: SurfaceTone,
    pub href: Option<String>,
    pub help: Option<String>,
}

pub struct RecentActivityItem {
    pub label: String,
    pub timestamp: String,
    pub actor: Option<String>,
    pub tone: SurfaceTone,
}
```


## 8. Data / State / API Model

ABDD requirements:

- metric cards must have text labels, not only icons
- status must use semantic text and badge/callout tone
- cards must be reachable in meaningful reading order
- sparklines are decorative unless accompanied by textual summary
- empty state must explain what to do next


## 9. UI/UX and ABDD Requirements

No database migration.

Possible render data changes only:

- `DashboardMetric`
- `RecentActivityItem`
- `SystemHealthSummary`

If a value cannot be obtained cheaply from existing handler-side data, show a
stable empty/unknown state rather than adding expensive queries in this RFC.


## 10. Migration Plan

1. Add or reuse metric card primitive.
2. Adapt `DashboardData` minimally.
3. Update `render_dashboard`.
4. Ensure no destructive action appears on the dashboard.
5. Add i18n keys for new labels.


## 11. Acceptance Criteria

- [ ] Dashboard visually follows mockup summary hierarchy.
- [ ] No destructive action is exposed on dashboard.
- [ ] All new text is localized.
- [ ] No backend protocol or security behavior changes.
- [ ] Mobile card layout remains readable.

## 12. Test Plan

- `cargo fmt --check`.
- `cargo clippy --workspace --all-targets -D warnings`.
- `cargo test --workspace`.
- `text-leaks` invariant: no literal `>t.some_key<` leaks.
- `css-tokens` invariant: every `var(--*)` reference resolves.
- `semantic-palette-parity` invariant remains green.
- `inline-style-bound` remains within the project limit.
- HTML smoke test for metric cards.
- Manual mobile check for card grid collapse.
- Screen-reader reading order check.

## 13. Risks and Mitigations

- **Risk:** Dashboard becomes an operations cockpit with too many actions.  
  **Mitigation:** Keep it as orientation + links, not direct mutation.


## 15. Rollback Plan

Restore previous `render_dashboard` while keeping shared primitives if other pages use them.
