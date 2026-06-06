# RFC 063 — Dashboard signal vs. noise pass

**Status.** Implemented (v0.46.0)
**Priority.** P1 — Phase E (v0.46.0)
**Tracks.** PDF "honest visual hierarchy" — open the dashboard with
two warnings, and the warnings should draw the eye before the stats.
**Touches.** `crates/sui-id-web/src/pages.rs::render_dashboard`,
small CSS rule additions in `components.rs` (stat-card grouping).
Depends on RFC 061 (tokens) + RFC 062 (`.card--warn`, `.card--info`).

## Background

The dashboard currently renders, top-to-bottom:

1. Page header (greeting + lede)
2. Action-required card (only if warnings exist) — pinned at top
3. Stat cards (users, clients, sessions, issuer) — row of 4
4. Sparkline (login activity, with range tabs)
5. Recent important events (audit excerpt)

This is mostly OK. The signal-vs-noise problems are subtle:

- The four stat cards (users, clients, sessions, issuer) all use
  identical `.card` styling, so the eye can't distinguish "active
  sessions" (operator-relevant) from "issuer" (static config).
- The sparkline section is dense and visually heavy, but its
  in-page priority is "background trend", not "act on this."
- The "Recent important events" section uses plain `.card`. The PDF
  describes it as a primary operator-action surface, but its
  current weight matches the static stat cards.

The goal: re-prioritise the layout so the **action-required band +
recent-important events** read first; sparkline reads as
background; stat cards read as reference numbers.

## Goal

A dashboard that:

1. Pins the action-required card at top (unchanged).
2. Brings "Recent important events" higher in the visual hierarchy
   — either second from top, or visually emphasised via
   `.card--info` accent — so an operator sees recent dangerous
   actions immediately.
3. Tightens the stat-card row so it reads as a single "current
   counts" block rather than four equal-weight peers competing
   with the sparkline.
4. Lowers the sparkline's visual weight without removing detail —
   it's reference, not action.

## Design

### New order

Top-to-bottom:

1. **Header** (unchanged)
2. **Action-required card** (`.card--warn`, conditional)
3. **Recent important events card** (`.card--info`, two states: rows or empty)
4. **Quick stats row** (a single `.card` with a 4-column inline grid
   for users / clients / sessions / issuer; not four separate cards)
5. **Login activity** (`.card`, dim header) — sparkline + range tabs

### Recent important events promotion

Two changes:

- Move the section above the stat row.
- Apply `.card--info` (RFC 062) so it reads as an info callout.

In ja/en/zh, the title remains `t.dashboard_recent_important_title`;
the variant class adds the leading "ℹ " icon via the existing
convention in RFC 062 (icons are spelt out in the title key).

If the list is empty, the card still renders with
"No important events in the last 24 hours" (via the RFC 044
`<EmptyState>` from RFC 064 once that lands; this RFC just uses
plain prose for now and RFC 064 retrofits the primitive).

### Quick stats row

Today:

```html
<div class="grid grid-cols-4">
  <div class="card"><h3>Users</h3><p class="big">{n}</p></div>
  <div class="card"><h3>Clients</h3>...</div>
  <div class="card"><h3>Sessions</h3>...</div>
  <div class="card"><h3>Issuer</h3>...</div>
</div>
```

After:

```html
<section class="card">
  <dl class="kv-grid kv-grid--4col">
    <div><dt>Users</dt><dd class="stat-value">{n}</dd></div>
    <div><dt>Clients</dt><dd class="stat-value">{n}</dd></div>
    <div><dt>Sessions</dt><dd class="stat-value">{n}</dd></div>
    <div><dt>Issuer</dt><dd class="stat-value-mono">{issuer}</dd></div>
  </dl>
</section>
```

`.kv-grid--4col` is a new helper: 4-column grid for stat groups.
`.stat-value` styles the large number (existing pattern). The
issuer string gets `.stat-value-mono` because it's a URL.

Net: one card not four. Reads as "reference numbers, glance and
move on."

### Sparkline weight

Two changes:

- Reduce the sparkline's section title from `h2` to `h3`.
- Add `style="opacity: 0.92"` to the sparkline `<svg>` (subtle
  desaturation that nudges it into the "background" register
  without compromising readability).

These are deliberate small tweaks. The PDF's signal: operators
should glance at the sparkline once to confirm "things look
normal," not stare at it as the main view.

### Range tabs

Unchanged.

## Mock-up: which cards exist when

| State | Cards top-to-bottom |
|-------|---------------------|
| No warnings, no recent events | Header → Stats → Login activity |
| No warnings, some events | Header → Recent events (info) → Stats → Login activity |
| Some warnings, no events | Header → Action required (warn) → Stats → Login activity |
| Some warnings, some events | Header → Action required (warn) → Recent events (info) → Stats → Login activity |

The acceptance test ("open with 2 warnings, vs 0 warnings; first
draws attention to warning rows; second sits calmly") is satisfied
because in the no-warnings case, no card has a coloured accent.

## Implementation note

This RFC is mostly a render-order shuffle in
`render_dashboard`. No new data, no new handlers.

The `DashboardData` struct stays the same; the rendering reorders
the existing fields.

## Test plan

1. Render `/admin` with `warn_smtp_not_configured = true` and three
   recent events. Verify order: header → warn → info → stats →
   sparkline.
2. Render with no warnings, no recent events. Verify order: header
   → stats → sparkline (no coloured cards).
3. Render with no warnings but recent events. Verify order: header
   → info → stats → sparkline (info card up top).
4. CSS contrast: verify `.kv-grid--4col` works at 1024px wide;
   collapses to 2-column at narrow widths (existing responsive
   patterns).
5. Manual side-by-side with v0.45.0 dashboard. The warning case
   should clearly read warning-first.

## Rollout

Single release. Visual reorder; no API change. Operators who
bookmarked anchors within the dashboard (none exist) would be
affected — they don't, so this is non-breaking.

## Future work

A per-operator "pin this section to top" preference. Out of scope
for v1.0; if it materialises, it's a `me_security` preference
field, not a dashboard change.
