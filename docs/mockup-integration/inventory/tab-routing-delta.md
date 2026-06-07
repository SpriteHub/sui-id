# Tab Routing Delta — Mockup ↔ Product

Phase-0 deliverable of [RFC-MI-000](../../../rfcs/done/RFC-MI-000-baseline-delta-inventory.md).
Generated against `sui-id-web-mockup v0.4.8` ↔ `sui-id v0.49.0`.

## The non-negotiable

Migration plan **§D-02** (and RFC-MI-022) declares that path-based
deep-linkable tabs are mandatory:

> A reusable tab helper may be introduced, but it must emit
> route-based anchors:
>
> ```html
> <a href="/me/security/mfa" aria-current="page">MFA</a>
> ```
>
> The active state must be derived from the server-rendered
> current-route key or explicit page data. It must not require
> client-side routing, hydration, or query state.

The mockup encodes tab state as `?tab=…` query parameters. The
product encodes tab state as **distinct paths**. The integration
**rewrites every mockup `?tab=` anchor to a path-based anchor**.

Acceptance criteria from the migration plan §D-02 §4:

- Each tab remains directly addressable by URL ✓ (product already
  satisfies)
- Browser back/forward behaviour remains native ✓
- Tabs work with JavaScript disabled ✓
- `aria-current="page"` applied to the active tab ✓ (RFC-MI-022 §6)
- Tab helper supports both `/me/security/*` and `/admin/settings/*` ✓
- No new frontend routing library or hydration dependency ✓

## Tab group 1: `/me/security/*`

Mockup base: `/me/security?tab=…`
Product base: `/me/security/{slug}`

| Mockup `?tab=` value | Product path slug | `render_*` | Notes |
|---|---|---|---|
| `overview` | `/me/security/overview` | `render_me_overview` | identity mapping |
| `password` | `/me/security/password` | `render_me_security` | identity mapping; render name `render_me_security` is historical (predates RFC 040 tab split — the password tab was the first to ship) |
| `mfa` | `/me/security/mfa` | `render_me_mfa` | identity mapping |
| `passkey` (singular) | `/me/security/passkeys` (plural) | `render_me_passkey` | **rename:** `passkey` → `passkeys`. Product plural matches the route (list of multiple passkeys); render function keeps singular `render_me_passkey` for historical reasons (RFC 040 §3). |
| `sessions` | `/me/security/sessions` | `render_me_sessions` | identity mapping |
| `language` | `/me/security/language` | `render_me_language` | identity mapping |
| `recovery` (mockup-only) | (none — folded into `/me/security/mfa`) | (render_me_mfa branch) | The mockup separates the recovery-codes view; the product keeps it inside the MFA tab. **Default:** keep product fold. The mockup link target rewrites to `/me/security/mfa#recovery` (anchor) or just `/me/security/mfa`. |
| `totp` (mockup-only, during enrolment) | (none — POST flow internal to `/me/security/mfa`) | — | Mockup uses `?tab=mfa&enroll=totp&step=N` to walk the enrolment wizard; product progresses via POST handlers (`mfa_enroll_start`, `mfa_enroll_confirm`). No URL change needed. |

## Tab group 2: `/admin/settings/*`

Mockup base: `/admin/settings?tab=…`
Product base: `/admin/settings/{slug}`

| Mockup `?tab=` value | Product path slug | `render_*` | Notes |
|---|---|---|---|
| `basic` | `/admin/settings/basic` | `render_settings_basic` | identity mapping |
| `auth` | `/admin/settings/authentication` | `render_settings_authentication` | **rename:** `auth` → `authentication`. Product slug is the full word — consistent with the `/oauth2/authorize` and `/admin/login` long-form vocabulary. No change. |
| `security` | `/admin/settings/security` | `render_settings_security` | identity mapping |
| `email` | `/admin/settings/email` | `render_settings_email` | identity mapping |
| `logs` | `/admin/settings/logs` | `render_settings_logs` | identity mapping |
| `other` | `/admin/settings/other` | `render_settings_other` | identity mapping |

## Tab helper API (RFC-MI-022 forward declaration)

The Phase-0 inventory does not implement the tab helper, but it
records the API shape that the screen-map and i18n-delta presume.
RFC-MI-022 owns the implementation. The expected signature:

```rust
/// Render a route-based tab strip.
///
/// `current` is the active tab slug (e.g. "mfa", "basic"). It is
/// matched against the `slug` field of each `TabEntry`; the entry
/// whose slug equals `current` receives `aria-current="page"`.
///
/// Anchors are emitted as `<a href="{base}/{slug}">…</a>`; never as
/// `?tab=…`.
pub fn tabs(
    base: &str,                      // e.g. "/me/security"
    entries: &[TabEntry<'_>],        // ordered list
    current: &str,                   // active slug
    strings: &sui_id_i18n::Strings,  // for tab labels
) -> impl IntoView { … }

pub struct TabEntry<'a> {
    pub slug: &'a str,                                    // path segment
    pub label_fn: fn(&sui_id_i18n::Strings) -> &str,      // i18n accessor
}
```

The helper lives in `crates/sui-id-web/src/components/tabs.rs` per
the RFC-MI-010 sharding plan. Active-state CSS:

```css
.tab-strip__link[aria-current="page"] {
  color: var(--accent-emphasis);
  border-bottom-color: var(--accent-default);
}
```

No JavaScript. Both tab groups (`/me/security/*` and
`/admin/settings/*`) call the same helper with their respective
`base` and `entries` arguments.

## Active-state computation

The handler passes `current` to the render function as a `&str`
matching the tab slug. The render function passes it to the tab
helper. No client-side computation, no query-string parsing in JS.

Example call in `handlers::me_security::mfa_get`:

```rust
let body = render_me_mfa(MfaData {
    // …
    tab_current: "mfa".to_string(),  // matches the route slug
});
```

Inside `render_me_mfa`:

```rust
view! {
    {tabs("/me/security", ME_SECURITY_TABS, &data.tab_current, &strings)}
    // …per-tab content…
}
```

This pattern matches the existing v0.49.0 code (each tab handler
already passes its own slug). RFC-MI-022's contribution is the
**helper** that DRY-removes the per-tab tab-strip duplication.

## Mockup query-parameter sub-states that survive

The mockup uses **two** query-parameter sub-states that the product
preserves because they are **page-internal step indicators**, not
tab selectors:

1. `/me/security?tab=mfa&enroll=totp&step=1|2|3` — enrolment wizard.
   Product equivalent: progressive POST handlers (`mfa_enroll_start`
   → `mfa_enroll_confirm`). No `?step=` parameter is needed; each
   POST writes server state and returns the next view. The mockup's
   `?step=N` is a presentation hint only.
2. `/forgot-password/reset?token=…` — reset token. Product
   equivalent: `/reset-password?token=…` — identical pattern,
   preserved as-is.

These are **not** tabs and do not interact with the route-based-tab
contract.

## Anchor rewrite map (for visual-language integration)

When implementing RFC-MI-030 / RFC-MI-031 / RFC-MI-040 / RFC-MI-060
(the screen-level RFCs), every mockup anchor of the form
`?tab=<value>` must be rewritten according to this table. The
implementer applies the rewrites screen-by-screen; the helper
introduced by RFC-MI-022 then DRY-removes the per-screen tab-strip
HTML.

| Mockup anchor (literal) | Product anchor (literal) |
|---|---|
| `<a href="/me/security?tab=overview">…</a>` | `<a href="/me/security/overview">…</a>` |
| `<a href="/me/security?tab=password">…</a>` | `<a href="/me/security/password">…</a>` |
| `<a href="/me/security?tab=mfa">…</a>` | `<a href="/me/security/mfa">…</a>` |
| `<a href="/me/security?tab=passkey">…</a>` | `<a href="/me/security/passkeys">…</a>` (**plural**) |
| `<a href="/me/security?tab=sessions">…</a>` | `<a href="/me/security/sessions">…</a>` |
| `<a href="/me/security?tab=language">…</a>` | `<a href="/me/security/language">…</a>` |
| `<a href="/me/security?tab=recovery">…</a>` | `<a href="/me/security/mfa">…</a>` (folded; section anchor optional) |
| `<a href="/admin/settings?tab=basic">…</a>` | `<a href="/admin/settings/basic">…</a>` |
| `<a href="/admin/settings?tab=auth">…</a>` | `<a href="/admin/settings/authentication">…</a>` (**rename**) |
| `<a href="/admin/settings?tab=security">…</a>` | `<a href="/admin/settings/security">…</a>` |
| `<a href="/admin/settings?tab=email">…</a>` | `<a href="/admin/settings/email">…</a>` |
| `<a href="/admin/settings?tab=logs">…</a>` | `<a href="/admin/settings/logs">…</a>` |
| `<a href="/admin/settings?tab=other">…</a>` | `<a href="/admin/settings/other">…</a>` |

## Form-action rewrite map

Some mockup forms POST to `/me/security?tab=…` because the mockup
collapses every tab's POST into a single handler that dispatches on
the `tab` parameter. The product uses **per-action POST routes**.
The integration rewrites these as well.

| Mockup form action | Product form action |
|---|---|
| `<form method="post" action="/me/security?tab=password">…</form>` | `<form method="post" action="/me/security/password">…</form>` |
| `<form method="post" action="/me/security?tab=passkey">…</form>` | (per-action: `/me/security/passkeys/register/start`, `/me/security/passkeys/{id}/delete`, `/me/security/passkeys/{id}/rename`) |
| `<form method="post" action="/me/security?tab=language">…</form>` | `<form method="post" action="/me/security/language">…</form>` |
| `<form method="post" action="/me/security">…</form>` (generic dispatch) | (per-action routes as above) |
| `<form method="post" action="/admin/settings">…</form>` (generic dispatch) | (per-field: `/admin/settings/basic/lang`, `/admin/settings/security/idle-timeout`, …) |

The product's per-action route pattern is **not negotiable**:

- Each POST has a narrow, auditable surface.
- CSRF and step-up middleware can be applied per-route.
- Audit-event emission is per-action (`admin.settings.idle_timeout.updated`
  vs the mockup's generic `settings.update`).
- A failed POST returns a 4xx on its own route, not a tab-restoring
  redirect.

RFC-MI-021 (server-rendered CSRF) ensures every per-action form
receives a real CSRF token from the Shell render path.

## Acceptance criteria (Phase 0)

- [x] Every mockup `?tab=` value has a product path mapping.
- [x] Tabs that exist in mockup but not product are explicitly
  classified (`recovery` and `totp` are folded into the MFA tab).
- [x] Tabs that exist in product but not mockup are noted (none —
  the product is a strict superset of the mockup's tab vocabulary).
- [x] The anchor-rewrite map is complete and ready for screen-level
  RFC implementers to apply mechanically.
- [x] The form-action rewrite map is complete.
- [x] No query-parameter sub-state survives that violates the
  route-based-tab contract.

## Decisions surfaced

None. Every tab-related decision required by D-02 is resolved by
"adopt the product's path-based model unchanged." No tab introduces a
delta that needs new design work.
