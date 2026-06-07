# Mockup Integration Inventory

Phase-0 deliverable produced by implementing
[RFC-MI-000](../../../rfcs/done/RFC-MI-000-baseline-delta-inventory.md).
Shipped in **v0.49.1**.

These six documents are the **frozen baseline** the screen-level
Mockup Integration RFCs (`RFC-MI-010` onward) reference when planning
each phase. They quantify every difference between the mockup
(`sui-id-web-mockup v0.4.8`) and the product (`sui-id v0.49.0`)
without proposing any code change.

## Files

| File | What it answers |
|---|---|
| [`screen-map.md`](./screen-map.md) | "For every mockup screen, what is the product route, render function, handler, shell, and integration status?" — 35 mockup routes mapped to their product equivalents with one of five status values. |
| [`dangerous-action-map.md`](./dangerous-action-map.md) | "How does the mockup's generic `/stepup?action=…` → `/confirm/{token}` flow map onto the product's per-operation `/admin/.../*-confirm` flow?" — 18 mockup action values mapped, with audit-event, step-up, and CSRF requirements. |
| [`tab-routing-delta.md`](./tab-routing-delta.md) | "How does the mockup's `?tab=` query-parameter model become the product's path-based `/me/security/{slug}` and `/admin/settings/{slug}` model without breaking deep-linking, back/forward, or no-JS?" — anchor and form-action rewrite tables. |
| [`token-delta-draft.md`](./token-delta-draft.md) | "Which mockup CSS tokens are new, which map onto the existing vocabulary, and how do the mockup's hardcoded pixel values fold onto the product's bounded `--space-*` scale?" — **headline finding: zero new tokens.** Hardcoded values round onto the existing token scale. |
| [`i18n-copy-delta-draft.md`](./i18n-copy-delta-draft.md) | "Of the 382 mockup-only i18n key names, which are renames of existing product keys, which are rewords of existing concepts, and which are genuinely new copy that needs translation work?" — **headline finding: ~58 net new keys × 3 locales = ~174 translation entries**, far smaller than the apparent 382-key delta. |
| [`route-render-handler-map.md`](./route-render-handler-map.md) | "For every one of the product's ~82 routes, what is the handler, the render function, the auth requirement, the CSRF requirement, and the audit event(s) emitted?" — the product-side reference. |

## Cross-cutting findings

These observations are surfaced by the inventory work, and inform
the phase-by-phase RFC implementations that follow:

1. **No new CSS tokens.** The mockup's token vocabulary is a strict
   subset of the product's (33 ⊂ 75 tokens). RFC-MI-011's
   token-bloat-risk column is empty by construction.

2. **Spacing rhythm is the only real reconciliation work.** The
   mockup uses 206 hardcoded pixel values (`14px`, `12px`, `8px`,
   etc.) where the product uses `--space-*` tokens (8/12/16/24/32/48
   px). Every hardcoded value rounds onto the existing scale; no
   new spacing token is added (per migration plan §D-05).

3. **Tabs need an anchor rewrite only.** Mockup `?tab=…` → product
   `/{base}/{slug}`. RFC-MI-022's tab helper DRY-removes the
   per-tab strip duplication; the active state computes from the
   route slug the handler already passes.

4. **Dangerous actions need a link rewrite only.** Mockup
   `?action=X` → product named confirm GET. Out of 18 mockup
   action values, **9 are pure link-rewrites** with behaviour
   preserved, **5 are "do-not-implement-yet"** (gated by product
   design constraints — `signing_key.publish` is additive, etc.),
   **3 surface step-up-policy deltas** for RFC-MI-051 review, and
   **1 is inline-only** (`client.secret.rotate`).

5. **i18n delta is mostly renames.** Only ~58 of the 382
   mockup-only keys are net-new copy (mostly the `impact_*`
   cluster for the new impact-summary surface). The rest are
   renames (`action_save` → `button_save`) or rewords (settings
   labels). Translation effort is bounded.

6. **No protocol-layer change is required.** Every mockup route
   that interacts with OIDC (`/authorize`, `/consent`,
   `/.well-known/*`) maps onto the product's existing endpoints
   with no contract change. RFC-MI-070 is a presentation RFC, not
   a protocol RFC.

7. **CI invariants remain at their v0.48.4 values by construction.**
   No phase increases inline-style count, token references, text
   leaks, or semantic-palette parity counts. Phases 1–8 each must
   decrease or stay flat on those gates.

## Decisions surfaced (consolidated)

The six files raise the following decisions for the architect /
project manager / security reviewer. None blocks Phase 1.

| ID | Subject | Default | Owning RFC |
|---|---|---|---|
| screen-D1 | Setup wizard: combine lang+HIBP? | Keep product split | RFC-MI-040 |
| screen-D2 | Forgot-password `/sent` separate route? | Keep product flash | RFC-MI-041 |
| screen-D3 | Client detail path | Keep `/edit` suffix | RFC-MI-031 |
| screen-D4 | Audit export format | Keep CSV | RFC-MI-031 |
| screen-D5 | Recovery codes view | Keep folded in MFA tab | RFC-MI-060 |
| danger-D1 | `user.force_logout` route | Rely on disable side-effect | RFC-MI-031 |
| danger-D2 | `client.secret.rotate` confirm GET | Keep inline | RFC-MI-051 |
| danger-D3 | Signing-key activate/retire actions | Additive rotation only | RFC-MI-031 |
| danger-D4 | Global "revoke all sessions" lever | Do not add | RFC-MI-031 |
| danger-D5 | Settings updates → step-up? | Do not adopt | RFC-MI-051 |
| danger-D6 | `me.mfa.regen_recovery` step-up? | Defer to post-MI follow-up | RFC-MI-051 |
| token-D1 | 14px → `--space-2` or `--space-3`? | Case-by-case in RFC-MI-011 | RFC-MI-011 |
| token-D2 | 10px → `--space-1` or `--space-2`? | Case-by-case in RFC-MI-011 | RFC-MI-011 |
| token-D3 | 1px/2px literals → `--border-width-*`? | When it's a stroke | RFC-MI-011 |
| token-D4 | 1100px → `--content-max-width`? | Yes | RFC-MI-011 |
| token-D5 | Mockup font sizes 13px/16px | Round to nearest | RFC-MI-011 |
| i18n-D1 | Mockup `state_*` vocabulary | Reject — keep RFC 044 vocabulary | RFC-MI-031 |
| i18n-D2 | "suspend" vs "disable" | Keep "disable" | RFC-MI-031 |
| i18n-D3 | "Review changes" button copy | Do not introduce | RFC-MI-051 |
| i18n-D4 | zh visibility | Translate; hidden per D-11 | — |
| i18n-D5 | Audit filter copy | Defer 12 keys | RFC-MI-031 |

Defaults are recommendations from the inventory; the owning RFC
makes the binding decision.

## What happens next

`RFC-MI-000` moves to `rfcs/done/` in this release (v0.49.1). The
RFC's `Status` field updates to `Implemented (v0.49.1)`.

Phase 1 (`RFC-MI-010` component CSS sharding, `RFC-MI-011` token
mapping, `RFC-MI-012` theme persistence) becomes eligible to start.
The three RFCs read this inventory as required input.
