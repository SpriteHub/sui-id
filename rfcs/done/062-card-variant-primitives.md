# RFC 062 — Card variant primitives

**Status.** Implemented (v0.46.0)
**Priority.** P0 — Phase E (v0.46.0)
**Tracks.** PDF "visual hierarchy" — make warnings draw the eye more
than ordinary cards without resorting to inline tricks.
**Touches.** `crates/sui-id-web/src/components.rs` (CSS rules),
`crates/sui-id-web/src/pages.rs` (call sites that currently use
inline `border-left:4px solid var(--warning-default)` etc.).
Depends on RFC 061's `--{semantic}-subtle` tokens.

## Background

Today `.card` is one rule:

```css
.card {
  background: var(--surface-default);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  padding: var(--space-4);
}
```

Every card in the admin UI looks the same. Where a card needs to
read as "this is a warning, look here first," the call site reaches
for inline style:

```rust
<section class="card"
         style="border-left:4px solid var(--warning-default);
                margin-bottom:var(--space-4)"
         aria-label=t.dashboard_aria_action_required>
```

This pattern:
1. Couples visual semantics to inline style. A future palette change
   forces a sweep through `pages.rs`.
2. Doesn't extend cleanly. There's no equivalent for "info" or
   "success" callouts; each new one re-invents the trick.
3. Doesn't show in dark mode without inline overrides — which the
   current code skips, leaving the warning underemphasised at night.

## Goal

A small set of card variant classes that compose with `.card`:

```html
<section class="card card--warn">
<section class="card card--info">
<section class="card card--success">
<section class="card card--callout">
```

Each variant changes:
- A `border-left` accent (4px, semantic colour)
- A subtle background tint (`--{semantic}-subtle`)
- Optionally a leading icon character before the title

Light/dark parity is built in because tokens carry per-mode values
(RFC 061).

## Design

### CSS

Added to `components.rs`:

```css
.card--warn {
  background: var(--warning-subtle);
  border-color: var(--warning-default);
  border-left-width: 4px;
}
.card--info {
  background: var(--info-subtle);
  border-color: var(--info-default);
  border-left-width: 4px;
}
.card--success {
  background: var(--success-subtle);
  border-color: var(--success-default);
  border-left-width: 4px;
}
.card--callout {
  /* Accent (lavender) callout, e.g. "next steps" cards on setup */
  background: var(--accent-subtle);
  border-color: var(--accent-default);
  border-left-width: 4px;
}
```

The `border-left-width: 4px` interacts with the existing `border:
1px solid` rule so the box still has a 1px top/right/bottom border
and a 4px left accent. That gives the asymmetric look the inline
hack was trying to achieve, while keeping the rest of the box
visible.

### Why not full-bleed colored backgrounds?

The PDF doesn't ask for vivid warning backgrounds. Operators looking
at the dashboard need warnings to **read** as warnings without being
visually offensive. The subtle tint pattern (RFC 061) is sufficient
for "this card is different" while leaving the body text readable
in standard typography.

The exception: `.flash` (which is for *transient* feedback after
an action) is intentionally more vivid because it must catch the
eye for 2–3 seconds. `.card--{variant}` is for *persistent* state.

### Migration

Three sites in `pages.rs` use the inline trick:

1. `render_dashboard` — the "action required" warning card.
2. `render_setup_done` — the "next steps" callout card (currently
   uses plain `.card` even though it's clearly a callout).
3. `render_settings_security` — the "security recommendations" card
   (currently uses plain `.card`).

Migration:

```rust
// before
<section class="card" style="border-left:4px solid var(--warning-default);
                              margin-bottom:var(--space-4)">

// after
<section class="card card--warn" style="margin-bottom:var(--space-4)">
```

The `margin-bottom` inline stays — it's spacing, not visual semantics.

### Icon convention

Variants may carry a leading character before the title:

| Variant | Symbol | Meaning |
|---------|:-:|---------|
| warn | ⚠ | "Look here first; something needs attention." |
| info | ℹ | "Background info; non-urgent." |
| success | ✓ | "This worked." |
| callout | — | No symbol; structural emphasis only. |

These are **convention not enforcement**. The CSS doesn't inject
characters via pseudo-elements (which would break ja/en/zh screen
reader reading order). Callers spell them out in the title text,
which already happens for the existing warning card
(`"⚠ " {t.dashboard_action_required_title}`).

### Composition with `.card__title`

`.card__title` rules in components.rs are untouched. Variants
inherit the title styling; they don't override it.

## Test plan

1. Render dashboard with `warn_smtp_not_configured = true` —
   warning card has amber tint + amber left accent + amber border
   in light mode; deep-amber tint + amber-bright accent in dark.
2. Render dashboard with all warnings off — no warning card visible.
3. Render setup done page — callout card has lavender tint + accent
   instead of plain card.
4. Open `/admin/settings/security` — security recommendations card
   has info tint.
5. Manual: side-by-side compare with v0.45.0 screenshot. The
   warning card should clearly draw the eye more than v0.45.0,
   because v0.45.0 was rendering with no tint (broken).

## Rollout

Single release. Visual change is intentional. The same pages
existed before; they now look more like the PDF intended.

## Future work

A `.card--neutral` (no accent, just slight emphasis) for cards that
need to read as "important but not coloured." Deferred until a
concrete need emerges.
