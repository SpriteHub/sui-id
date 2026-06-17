# RFC 074 — Navigation restructuring and UX polish

**Status.** Implemented (v0.61.0)
**Priority.** P2 — pre-1.0 polish; four nav/UX inconsistencies deferred
from the UX-rethink arc.
**Tracks.** Post-UX-rethink cleanup.
**Touches.** `crates/sui-id-store` (migration 0030, users repo),
`crates/sui-id-core` (session login path), `crates/sui-id-web`
(layout.rs Nav + Shell, chrome.rs, settings.rs, me_security/overview.rs),
`crates/sui-id-i18n`. No handler changes.

## Implementation note (v0.61.0)

### Item 1 — User-menu dropdown (replaces flat "Security" nav link)

`Shell` gained an optional `admin_username: Option<String>` prop. The `Nav`
component was rewritten:

- Nav item count: 7 → 6 (removed "Security" / `me` entry).
- The `<form method="post" action="/admin/logout">` button is now inside a
  `<details>/<summary>` user-menu that appears when `admin_username` is
  `Some`. When `None` (e.g. `/me/*` pages that don't supply a username),
  the old flat sign-out form renders as a fallback.
- User-menu panel contains: "My account" → `/me/security/overview` + Sign out.
- No JavaScript — `<details>/<summary>` is native HTML.

New CSS classes in `components/chrome.rs`: `.user-menu`, `.user-menu__toggle`,
`.user-menu__panel`, `.user-menu__item`, `.user-menu__form`.

### Item 2 — "Clients" → "Apps" in nav label

Nav item for `/admin/clients` now uses `t.nav_apps` instead of
`t.nav_clients`. The route and all handler code are unchanged. New i18n
key `nav_apps` (en: "Apps"; ja: "アプリ"; zh: "应用").

### Item 3 — Settings tab labels

`settings_tabs()` in `pages/settings.rs` now maps:
- `SettingsTab::Basic` → `t.settings_tab_general` ("General")
- `SettingsTab::Other` → `t.settings_tab_advanced` ("Advanced")

URLs and underlying pages unchanged. Full 6→4 group consolidation
(merging security+authentication, merging email+logs) is deferred to
a future RFC — that work requires handler merging.

### Item 4 — "Last signed in" anti-phishing line on `/me/security/overview`

**Migration 0030** — `ALTER TABLE users ADD COLUMN last_login_at TIMESTAMP`.

**`set_last_login(db, user_id, now)`** — new best-effort helper in the
users repo. Called in `session.rs` after `sessions::insert`, before
`LoginOutcome::SessionEstablished` is returned.

**`MeOverviewData.last_login_at: Option<DateTime<Utc>>`** — added. The
handler reads `user.last_login_at` (always present from the session context
query). The overview render shows:
- `Some(ts)` → "{date}" rendered via `fmt_time`; text from `me_overview_last_login`
  i18n key (e.g. "You last signed in on 2026-05-23 01:02 UTC.").
- `None` → first-login welcome message from `me_overview_first_login`.

Both appear as a `<p class="muted text-caption">` in the `<header>` below
the `<h1>`.

### New i18n keys (6 × 3 locales)

`nav_apps`, `nav_my_account`, `settings_tab_general`, `settings_tab_advanced`,
`me_overview_last_login`, `me_overview_first_login`.

### Test fixes

7 `UserRow` constructors in `sui-id-core` gained `last_login_at: None`.

### Acceptance criteria (verified)

- [x] Admin nav has no "Security" link. User-menu dropdown appears at top-right
  with "My account" and "Sign out."
- [x] Admin nav label reads "Apps" (not "Clients").
- [x] Settings "General" and "Advanced" labels render correctly.
- [x] `/me/security/overview` shows the last-login line; first-time shows
  welcome message.
- [x] All changes are HTML/CSS/i18n; no new JavaScript.
- [x] `cargo check --workspace` clean; 175/175 library tests pass.
- [x] CI invariants: `text-leaks`=0, `inline-style-bound`=0,
  `css-tokens`=148, `semantic-parity`=36.

---
**Priority.** P2 — pre-1.0 polish; no correctness or security gap, but
several small inconsistencies noted in the post-MI-arc audit that survived
the UX-rethink arc (RFCs 071–073).
**Tracks.** Post-UX-rethink cleanup.
**Touches.** `crates/sui-id-web` (nav, shell, admin layout), `crates/sui-id`
(settings handler grouping), `crates/sui-id-i18n`. No schema changes.

---

## Background

The UX-rethink arc (v0.58.0–v0.60.0) closed three structural gaps: the
missing Auditor role, the missing end-user app-access surface, and the
non-actionable dashboard. The post-MI-arc audit identified four additional
small inconsistencies that were intentionally deferred. This RFC addresses
them in a single focused pass.

## Items

### 1. Admin top-nav: "Security" link moves to user menu dropdown

The current admin top-nav contains a "Security" link that goes to
`/me/security/overview` — the *current admin's own self-service page*.
This mixes two scopes in one row:

- **Administrative items** (Users, Apps, Audit, Settings, Dashboard):
  act on *other people or the system*.
- **Self-service item** (Security): acts on *the signed-in admin's own
  account*.

Fix: remove "Security" from the main nav. Create a user-menu dropdown in
the top-right corner (matching the pattern used by virtually every web
product) containing:

```
[username] ▼
  My account → /me/security/overview
  Sign out   → POST /admin/logout
```

The dropdown is a `<details>/<summary>` element — no JavaScript needed.
The admin's display name (or username if no display name) appears as the
dropdown label.

### 2. Rename "Clients" → "Apps" in admin nav

"Clients" is an OAuth 2.0 protocol term. Operators reading the admin
panel — who are not necessarily OAuth experts — benefit from the plain
English "Apps." The RFC 072 audit used "Apps" in `/me/apps` for the same
reason. The nav label should be consistent with the end-user surface.

Change: nav label `Clients` → `Apps`. The route (`/admin/clients`) and
underlying code stay unchanged to avoid breaking any bookmarks or operator
scripts.

### 3. Settings tabs: 6 → 4 logical groups

The current Settings section has six tabs (basic, security,
authentication, logs, email, advanced) that split closely-related
decisions across screens. The proposed consolidation:

| New group | Absorbs |
|---|---|
| **General** | basic |
| **Authentication** | security + authentication |
| **Email & Notifications** | email + logs (alert threshold) |
| **Advanced** | logs (retention) + advanced |

Cosmetic change only — no setting is moved out of the product, only
the grouping. The URLs (`/admin/settings/basic`, etc.) can remain for
backward-compat; the new tab bar links them under the new labels.

### 4. "Last signed in from" anti-phishing line on `/me/security/overview`

Every modern IdP shows a "You last signed in from X on Y" line on the
account overview page. Its function is anti-phishing: if the displayed
location surprises the user, they know to investigate. Implementation:

- On each successful login, record `last_login_at TIMESTAMP` and
  `last_login_ip TEXT` on the session row or the user row.
- The `/me/security/overview` page reads these fields and renders a single
  sentence: "You last signed in on {date}." (IP/location display is opt-in
  config, off by default, to respect deployment privacy policies.)
- If no previous login exists (first sign-in), render nothing.

This is the only schema-touching item in this RFC. The addition is
minimal: one `ALTER TABLE users ADD COLUMN last_login_at TIMESTAMP`
(possibly already set by the session layer).

## Non-goals

- Notifications / push alerts for unusual sign-ins — future RFC.
- "Remember this device" / trusted-device tokens — out of scope.
- Full redesign of the settings page — out of scope; the four-group
  consolidation is sufficient.

## Acceptance criteria

- [ ] Admin nav has no "Security" link. A user-menu dropdown appears
  in the top-right with "My account" and "Sign out."
- [ ] Admin nav label reads "Apps" (not "Clients").
- [ ] Settings tabs consolidated to 4; all settings remain accessible.
- [ ] `/me/security/overview` shows "You last signed in on {date}."
  when a previous session exists.
- [ ] All changes are pure HTML/CSS/i18n; no new JavaScript.
- [ ] CI invariants unchanged.
