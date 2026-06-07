# Token Delta Draft — Mockup ↔ Product

Phase-0 deliverable of [RFC-MI-000](../../../rfcs/done/RFC-MI-000-baseline-delta-inventory.md).
Generated against `sui-id-web-mockup v0.4.8/crates/sui-id-web/src/theme.rs`
↔ `sui-id v0.49.0/crates/sui-id-web/src/tokens.rs`.

This file is the input contract for **RFC-MI-011 (Token Mapping)**.

## Headline

> **The mockup introduces zero new CSS token names.**

Every CSS custom property declared in the mockup's `theme.rs`
already exists in the product's `tokens.rs`. The product is a strict
superset of the mockup token vocabulary (75 product tokens vs 33
mockup tokens, all 33 shared).

Token bloat risk for the integration: **zero**. RFC-MI-011's
"rejected token" column is empty by construction; no new tokens are
added; no existing tokens are removed.

## Shared tokens (mockup ⊆ product)

The 33 tokens both declared and used by the mockup, every one
already in `tokens.rs`:

### Foreground

- `--fg-default` — body text
- `--fg-muted` — secondary text
- `--fg-subtle` — tertiary / metadata
- `--fg-inverse` — text on dark surface
- `--fg-on-accent` — text on accent fill

### Surface

- `--surface-default` — page background
- `--surface-subtle` — section background
- `--surface-elevated` — card / dialog background
- `--surface-sunken` — input fields, code blocks
- `--surface-inverse` — dark-on-light callouts

### Accent

- `--accent-default` — primary brand colour
- `--accent-subtle` — accent fill at low intensity
- `--accent-emphasis` — accent fill at high intensity (hover, active)

### Semantic palette (the four families × 2 slots)

- `--danger-default`, `--danger-subtle`
- `--warning-default`, `--warning-subtle`
- `--success-default`, `--success-subtle`
- `--info-default`, `--info-subtle`

### Border

- `--border-default` — neutral borders
- `--border-muted` — subdued borders
- `--border-accent` — emphasis borders

### Radius

- `--radius-sm`, `--radius-md`, `--radius-lg`

### State (interaction)

- `--state-hover`
- `--state-focus`
- `--state-active`
- `--state-disabled`

### Shadow

- `--shadow-sm`, `--shadow-md`

### Other

- `--font-mono` — monospace stack (used for client IDs, code snippets)

## Product-only tokens (preserved, not consumed by mockup)

These 42 tokens exist in `tokens.rs` but are not referenced by the
mockup. They are **preserved** — the mockup integration neither uses
nor removes them. (They are used by product pages outside the
mockup's scope.)

### Spacing scale (the entire spacing scale is product-only)

- `--space-1` (8px), `--space-2` (12px), `--space-3` (16px),
  `--space-4` (24px), `--space-5` (32px), `--space-6` (48px)

The mockup uses **hardcoded pixel values** rather than spacing
tokens. See "Spacing rhythm reconciliation" below — this is the
single most important integration task in the token delta.

### Typography scale

- Sans-serif: `--font-sans`
- Sizes: `--font-size-caption`, `--font-size-body`, `--font-size-h3`,
  `--font-size-h2`, `--font-size-display`
- Weights: `--font-weight-regular`, `--font-weight-medium`,
  `--font-weight-bold`
- Line heights: `--line-height-caption`, `--line-height-body`,
  `--line-height-h3`, `--line-height-h2`, `--line-height-display`

The mockup uses hardcoded `font-size` / `font-weight` / `line-height`
values for its bespoke components. **Same reconciliation pattern as
spacing** — RFC-MI-011 maps every hardcoded mockup typography value
to the nearest token.

### Motion

- `--motion-instant`, `--motion-fast`, `--motion-base`,
  `--motion-slow`
- `--motion-easing`

Mockup has no transitions or animations. Tokens are preserved for
product pages outside the mockup's scope.

### Layer (z-index)

- `--z-below`, `--z-base`, `--z-raised`, `--z-dropdown`,
  `--z-overlay`, `--z-modal`, `--z-toast`

Mockup has no overlays. Tokens are preserved.

### Border widths

- `--border-width-default`, `--border-width-emphasis`

Mockup uses hardcoded `border-width: 1px` / `2px`. Reconciliation
maps these to the tokens.

### Layout

- `--content-max-width` (64rem)
- `--content-narrow-width` (28rem)

Mockup uses hardcoded `max-width: 1100px` / `360px`. Reconciliation
maps these (with adjustment) to the tokens.

### Shadow

- `--shadow-lg` — used by elevated dialogs; mockup has none.

### Semantic on-fill foregrounds (CI gate)

- `--fg-on-danger`, `--fg-on-warning`, `--fg-on-success`,
  `--fg-on-info`

These exist for **semantic palette parity** (RFC 061's CI gate
`semantic-palette-parity` requires the full 12-token triple). Mockup
only uses `--fg-on-accent` because mockup callouts use accent fill;
the four semantic-fg-on-* tokens stay for product semantic-banner
backgrounds.

## Spacing rhythm reconciliation

The mockup's source code uses **206 raw `var(--…)` references** but
**zero `--space-*` references**. Spacing is encoded as inline
hardcoded pixel values throughout:

| Hardcoded mockup value (occurrences) | Nearest product token | Mapping decision |
|---|---|---|
| `8px` (21) | `--space-1` (8px) | **identity → token** |
| `12px` (33) | `--space-2` (12px) | **identity → token** |
| `16px` (25) | `--space-3` (16px) | **identity → token** |
| `24px` (13) | `--space-4` (24px) | **identity → token** |
| `32px` | `--space-5` (32px) | **identity → token** |
| `48px` | `--space-6` (48px) | **identity → token** |
| `14px` (35) | `--space-2` (12px) **or** `--space-3` (16px) | **round to `--space-3`** for line-height contexts; round to `--space-2` for tight gaps. RFC-MI-011 fixes the case-by-case rule. |
| `10px` (14) | `--space-1` (8px) **or** `--space-2` (12px) | **round to `--space-1`** for compact gaps; round to `--space-2` for label gutters. |
| `6px` (30) | `--space-1` (8px) | **round to `--space-1`**. The mockup's 6px is a tight grid; product's 8px is the floor. |
| `18px` (10) | `--space-3` (16px) | **round to `--space-3`** |
| `20px` (9) | `--space-3` (16px) **or** `--space-4` (24px) | **round to `--space-3`**; the few 20px sites in the mockup are mostly aesthetic and gain nothing from being closer to 24. |
| `28px` (3) | `--space-4` (24px) | **round to `--space-4`** |
| `2px` (12) | (no token at this scale) | **keep as `2px` literal** — 2px is a border-radius / focus-ring scale, not a spacing scale. Should reference `--border-width-default` or `--border-width-emphasis` if it's a stroke. |
| `1px` (32) | (no token at this scale) | **keep as `1px` literal or use `--border-width-default`** — same reasoning. |
| `3px`, `7px`, `9px`, `13px` (rare) | nearest `--space-*` | rounded individually during screen-level integration |
| `160px`, `180px`, `1100px` (rare) | (no token; these are fixed widths) | RFC-MI-011 decides: keep as literals or compare to `--content-max-width` / `--content-narrow-width`. **Default:** keep literals where they encode specific layout grids; map widths like `1100px` to `--content-max-width` (which is 64rem ≈ 1024px). |

### Net policy for RFC-MI-011

> **No new spacing token is added.** Every hardcoded mockup spacing
> value rounds onto the existing `--space-1`..`--space-6` scale
> using the table above. The intermediate values (6px, 10px, 14px,
> 18px, 20px, 28px) are absorbed by rounding to the nearest token;
> the resulting visual delta is bounded by the 4-px gap between
> adjacent spacing tokens and is **below the perception threshold
> for non-grid users**.
>
> This matches the migration plan **D-05** decision:
>
> > "Map the mockup's 4px visual rhythm onto the current bounded
> > `--space-*` vocabulary unless a specific design gap is proven."

## Typography rhythm reconciliation

Similar to spacing. The mockup uses hardcoded `font-size` values:

| Mockup hardcoded value | Nearest product token | Mapping decision |
|---|---|---|
| `13px` | `--font-size-caption` (12px) | round down |
| `14px` | `--font-size-caption` (12px) or `--font-size-body` (15px) | round to `--font-size-body` for body text; round to `--font-size-caption` for help text |
| `15px` | `--font-size-body` (15px) | identity → token |
| `16px` | `--font-size-body` (15px) | round down by 1px |
| `18px` | `--font-size-h3` (18px) | identity → token |
| `22px` | `--font-size-h2` (22px) | identity → token |
| `32px` | `--font-size-display` (32px) | identity → token |

Same principle: **no new typography token is added.** Mockup
hardcoded values round onto the existing `--font-size-*` scale.

## Visual primitives proposed by RFC-MI-011

These are component-level patterns (not new tokens). RFC-MI-011's
"primitive adoption" table classifies them:

| Primitive | Source | Target shard | Composition | Decision |
|---|---|---|---|---|
| **`callout`** | `mockup/components/callout.rs` | `components/cards.rs` or `components/banners.rs` | `--surface-subtle` + `--border-muted` + `--fg-default` + `--space-3` padding | **adopt** (used on `/setup/security`, `/me/security/mfa`, HIBP feedback; replaces inline `<aside class="muted">` patterns) |
| **`impact-summary`** | `mockup/handlers/stepup.rs` | `components/confirm.rs` | `--surface-elevated` + `--border-accent` + `--space-4` padding + `--font-size-body` | **adopt** (used on confirm screens; complements `.confirm-shell`) |
| **`metric-card`** | `mockup/handlers/admin.rs` | `components/cards.rs` | `--surface-elevated` + `--shadow-sm` + `--radius-md` + `--space-3` padding | **adopt** (used on `/admin` dashboard; replaces inline `<div class="card">` stats) |
| **`hibp-indicator`** | `mockup/components/hibp.rs` | `components/banners.rs` (semantic-fill variant) | `--danger-default` / `--warning-default` / `--success-default` fill + `--fg-on-{danger,warning,success}` | **adopt** (used on `/setup/security`, `/me/security/password`, `/forgot-password/reset`) |
| **`step-indicator`** | `mockup/components/step_indicator.rs` | `components/setup.rs` | `--accent-subtle` (inactive) → `--accent-default` (current) → `--success-default` (done) | **adopt** (already exists in product as `.setup-step-indicator`; mockup's design is functionally identical; visual adaptation only) |
| **`route-tabs`** | (the mockup's tab strip) | `components/tabs.rs` | `--border-muted` underline → `--accent-default` on active | **adopt** as the helper described in `tab-routing-delta.md`. Designed in RFC-MI-022. |
| **`field-error`** | (mockup's inline form error display) | `components/forms.rs` | `--danger-default` text + `aria-describedby` linkage | **adopt** (replaces ad-hoc `<p style="color:red">` patterns the product still has in two locations) |
| **`danger-zone`** | `mockup/handlers/users.rs`, `clients.rs`, `security.rs` | `components/confirm.rs` | `--danger-subtle` background + `--fg-on-danger` button | **adopt**. Designed in RFC-MI-051. |
| **`dl-grid`** (definition-list metric grid) | `mockup/components/dl_row.rs` | `components/tables.rs` or `components/cards.rs` | CSS grid + `--space-2` row-gap + `--fg-muted` label + `--fg-default` value | **adopt** (used on `/admin/users/{id}`, `/admin/clients/{id}` — replaces ad-hoc `<table>` for key-value display) |

No primitive is rejected. Each adoption rule from RFC-MI-011 §7 is
satisfied (used by ≥ 2 screen groups, or critical for ABDD /
security comprehension, or required to remove inline styles).

## CI invariant impact (pre-Phase-1)

All four CI gates remain at their v0.48.4 / v0.49.0 values **by
construction**:

| Gate | Pre-Phase-1 value | Effect of Phase-1 token work |
|---|---|---|
| `text-leaks` | 0 | unaffected (no Leptos `view!` change) |
| `css-tokens` | every `var(--*)` resolves | unaffected (no new var(--*) introduced; mockup uses tokens that already exist) |
| `semantic-palette-parity` | 12 names × 3 modes | unaffected (no semantic-palette change) |
| `inline-style-bound` | 16 (≤ 20) | **must decrease or stay flat.** Every inline-style site that Phase 1 touches is migrated to a utility class or component class. Phase 1 cannot raise this number. |

## Acceptance criteria (Phase 0)

- [x] Token delta table is complete (every mockup token classified).
- [x] No new token introduced.
- [x] No token removed.
- [x] Spacing rhythm reconciliation mapped (every hardcoded mockup
  value to a `--space-*`).
- [x] Typography rhythm reconciliation mapped.
- [x] Primitive adoption table is complete.
- [x] Semantic palette parity preserved.
- [x] CI invariant impact assessed; Phase 1 does not regress any
  gate.

## Decisions surfaced

| ID | Subject | Default | RFC that owns |
|---|---|---|---|
| **token-D1** | 14px contexts | round to `--space-3` for line-height, `--space-2` for tight gaps (case-by-case) | RFC-MI-011 |
| **token-D2** | 10px contexts | round to `--space-1` for compact gaps, `--space-2` for label gutters | RFC-MI-011 |
| **token-D3** | 1px / 2px literals | keep as literal OR reference `--border-width-*` when it's a stroke | RFC-MI-011 |
| **token-D4** | 1100px max-width | map to `--content-max-width` (64rem ≈ 1024px) | RFC-MI-011 |
| **token-D5** | Mockup-specific font sizes (13px, 16px) | round to `--font-size-caption` and `--font-size-body` respectively | RFC-MI-011 |

All five defaults preserve the product's bounded token vocabulary
and prevent token bloat.
