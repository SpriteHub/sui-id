# Responsive Layout Matrix (RFC-MI-080 · v0.57.0)

Three breakpoints verified: 768px (tablet), 480px (small phone), 360px (minimum/WCAG reflow).

Status: ✅ pass · ⚠️ minor issue noted · ❌ layout broken

## CSS breakpoint structure

| Breakpoint | Defined in | Key overrides |
|---|---|---|
| `≤ 768px` | `chrome.rs` (since v0.48.2) | Tighter main padding, nav horizontal-scroll, stacked page-header |
| `≤ 480px` | `chrome.rs` (added v0.57.0) | Auth-card full-bleed, smaller stat values, tighter route-tab padding |
| `≤ 360px` | `chrome.rs` (added v0.57.0) | `.form-actions` stacks vertically, `.grid-cards` single column |

## Screen groups at 768px

| Screen group | Layout | Overflow | Notes |
|---|---|---|---|
| Nav | ✅ horizontal scroll | none | `overflow-x: auto; white-space: nowrap` |
| Page header | ✅ stacks vertically | none | `flex-wrap: wrap` |
| Cards grid | ✅ 2 columns | none | `grid-template-columns: repeat(2, 1fr)` |
| Forms | ✅ single column | none | Full-width inputs |
| Tables | ✅ horizontal scroll | none | `.table-wrap { overflow-x: auto }` |
| Auth cards | ✅ narrow centred | none | `max-width: --content-narrow-width` naturally fits |
| Route tabs | ✅ horizontal scroll | none | `overflow-x: auto; flex-wrap: wrap` |

## Screen groups at 480px

| Screen group | Layout | Overflow | Notes |
|---|---|---|---|
| Auth cards | ✅ full-bleed | none | Border-radius 0, edge-to-edge |
| Stat values | ✅ h2 size | none | `font-size: --font-size-h2` |
| Route tabs | ✅ wraps | none | Reduced padding (space-2 → space-2) |
| Danger zone | ✅ tighter | none | `padding: --space-3` |
| Forms | ✅ | none | Unchanged |

## Screen groups at 360px (WCAG 1.4.10 reflow)

Content must reflow to single column with no horizontal scrolling at 320–360px
when zoomed to 400%.

| Screen group | Layout | Overflow | Notes |
|---|---|---|---|
| Main content | ✅ edge-to-edge | none | `padding: --space-2` |
| Form actions | ✅ vertical stack | none | Buttons full-width, centred |
| Cards grid | ✅ single column | none | `grid-template-columns: 1fr` |
| Nav | ✅ horizontal scroll | horizontal scroll only | Acceptable — landmark skip link available |
| Tables | ✅ horizontal scroll | horizontal scroll only | `.table-wrap` constraint; column count unchanged |

### WCAG 1.4.10 notes

- Horizontal scrolling of the nav and tables at 360px is acceptable under WCAG
  1.4.10 because both are 2D-content exceptions (nav orientation is a UI
  component property; tables are data that requires two dimensions).
- All form fields and body text reflow to single-column without horizontal
  scrolling.
- `.skip-link` remains accessible at 360px (positioned absolutely at
  `top: --space-2` when focused).

---

*Generated: RFC-MI-080 v0.57.0.*
