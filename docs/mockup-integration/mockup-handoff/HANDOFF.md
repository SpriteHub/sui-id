# sui-id UI/UX Mockup Handoff

**Document version**: 1.0
**Mockup version covered**: `v0.4.8`
**Audience**: `sui-id` implementation team — engineers, architects, project manager, security reviewer.

---

## 1. Overview

This document hands the `sui-id` UI/UX mockup off to the team building the
real product. The mockup is a runnable Rust + Leptos SSR application that
covers every user-facing surface of the IDaaS — setup, login, MFA,
authorization, admin console, audit log, account self-service, and the
system error pages — implemented to the level of detail where layout,
copy, information flow, and security-sensitive interaction patterns are
all settled.

The mockup is **not** a finished product. It uses in-memory mock data, a
no-op mail transport, and stub OIDC endpoints. What it provides is a
**stable specification of what the UI should look like, how the user
should move through it, and where the safety boundaries are.** The
implementation team's job is to fold this into the running service
without losing those properties.

**Quick navigation:**

| Need to know… | Section |
| --- | --- |
| What this is and how to treat it | §2 |
| Why it exists and what it solves | §3 |
| User journeys it expects to support | §5 |
| Screen list and what each one is for | §7 |
| What to do when mockup ≠ implementation | §14 |
| What's still undecided | §12 |
| First-week integration plan | §15 |

---

## 2. Positioning of the Mockup

### 2.1 What the mockup is

- A **design consolidation** of every user-facing surface in `sui-id`.
- A **runnable reference** — every screen renders, every navigation link
  works, every form posts, every step-up flow completes. The
  implementation team can boot it (`cargo run -- --mock-init`), click
  through it, and copy URLs to reproduce any state.
- A **specification artefact** for screen layout, information flow,
  empty states, error states, and microcopy.
- A **trait-shaped seam** (`sui-id-core`) for nine domain services
  that the real product can implement against (§15.2).

### 2.2 What the mockup is **not**

- It is **not** the source of truth for backend behaviour. Where the
  mockup's mock implementation diverges from real backend semantics
  (audit hash chaining, real password hashing, real HIBP queries, real
  email transport, real OIDC token issuance), the implementation
  decides — the mockup only documents the **UI contract** around those
  behaviours.
- It is **not** authoritative on routing structure if the real product
  has reasons to differ. The handler URLs (`/admin/users`,
  `/me/security?tab=mfa`, etc.) are recommendations grounded in design,
  not external commitments.
- It is **not** a complete OIDC implementation. The `/authorize`,
  `/consent`, and `/.well-known/openid-configuration` routes render
  the UX but are stub-only.

### 2.3 How to use it

| Layer | Treat the mockup as… |
| --- | --- |
| Screen layout, information density, primary/secondary action placement | **Authoritative reference** |
| User flows (setup → admin, login → MFA → consent, step-up → confirm) | **Authoritative reference** |
| Microcopy (Japanese + English) | **Default starting point** — edit for legal/locale needs |
| Visual system (spacing, type, colour semantics) | **Authoritative** (§9) |
| Routing / URL design | **Strong recommendation** |
| In-memory mock data | **Illustrative only** |
| OIDC protocol details | **Stub** — implementation owns the spec |
| Hash-chain crypto, password KDF, HIBP API integration | **Out of scope for mockup** — real implementation owns it |

### 2.4 Three categories of design intent

The mockup mixes three levels of certainty. The implementation team
should keep these distinct when reading any screen:

1. **Confirmed design intent** — settled by RFC, multiple screens
   depend on it. Changing it ripples. Example: the step-up →
   confirmation → return-to-origin pattern (RFC 007).
2. **Proposed interaction patterns** — implemented one way in the
   mockup, but the choice is replaceable. Example: the six-tab
   settings page layout vs. one-page-per-tab.
3. **Areas requiring product-side judgement** — explicitly open
   (see §12).

---

## 3. Background and Problems Solved

### 3.1 Why the mockup was commissioned

`sui-id` is a **self-hostable OSS IDaaS** in Rust. Its target operator
is technically literate (it's an OSS service they self-host) but **not
necessarily a security specialist**. Before the mockup work, the
backend implementation was advancing faster than the user-facing
surface, with concrete UX risks:

- **Setup confusion**: a setup token printed to stderr with no
  surrounding UI flow. Operators didn't know whether setup was
  open, closed, or partially done.
- **Cognitive load on security settings**: dozens of toggles in a
  flat list. Operators couldn't tell which were safe to change live.
- **Auth flow opacity**: login + MFA + OIDC consent each rendered
  in isolation. No shared shell, no continuity, easy to lose context
  mid-authorization.
- **Admin grouping**: users, clients, audit, signing keys, settings
  were not visibly grouped, so operators paged through unrelated
  sections to find one control.
- **Dangerous actions in line**: "delete user", "rotate signing key",
  "revoke all sessions" sat next to read-only operations with no
  separation.
- **Audit log presented as raw rows**: no chain status, no event
  taxonomy, no investigation ID surface.

### 3.2 What the mockup solved

For each problem area, the mockup ships a design response:

| Problem before | Design response | Intended improvement |
| --- | --- | --- |
| Setup token print-to-stderr | 4-step wizard with cookie-gated token (RFC 004) | Operator always knows where they are; one screen, one decision |
| Flat settings page | Six tabs (basic / security / auth / email / logs / other), each with one focused task (RFC 010) | Per-task screens; "review changes" before save |
| Auth flow opacity | Shared simple-shell layout for login / MFA / consent; consistent header/footer | Continuity across the three sub-screens |
| Admin grouping | Admin sidebar with five sections + self-service group | Stable left-anchor navigation |
| Dangerous actions in line | **Danger zone** at the bottom of each detail page; **step-up + impact summary** required for execution (RFC 007) | Physical separation + an explicit confirmation that names the impact |
| Raw audit rows | Chain-status badge, event taxonomy, investigation-ID column, cascade-row references (RFC 016) | Operators can tell at a glance if anything is wrong |
| No system error UX | Five system pages (404, 410, 429, 500) with the **investigation ID** prominently displayed (RFC 008) | Support handoff has a single reference number |
| One language only | Japanese + English with cookie-based switcher + `Accept-Language` fallback | Self-host audience often non-English-first |
| Light theme only | Three-mode theme switch (auto / light / dark) (RFC 009) | Reduces friction for late-night ops work |

### 3.3 Design principles preserved across every surface

Two principles are non-negotiable and explain most of the mockup's
choices. Implementation must preserve them.

#### 3.3.1 Accessible by Default and by Design (ABDD)

Every screen ships with: semantic HTML, keyboard reachability without
custom JS, `aria-current="page"` on active nav, `aria-live` on
toast/banner regions, contrast ratios meeting WCAG AA, and **no
colour-only signalling** — every status uses an icon or label
alongside the colour. See §10 for the full requirements.

#### 3.3.2 Minimalism — one place, one thing

A screen, a panel, or a component does **one** task. If the mockup
ever puts two unrelated tasks on one screen, that's a bug, not a
constraint to copy. This is why:

- the setup wizard is four steps, not one form;
- the settings page splits into six tabs;
- destructive actions live in a separate danger zone;
- step-up confirmation is a dedicated route, not a modal.

When integrating, **resist the urge to consolidate**. The redundancy
is the design.

---

## 4. Design Principles

| Principle | What it means here | Where you'll see it |
| --- | --- | --- |
| **Safety first** | Destructive actions need step-up + confirmation. Errors fail closed. | Step-up flow, HIBP fail-closed (RFC 012), MFA disable warning |
| **Continuity** | The user always knows where they are and where they came from. | Sidebar `aria-current`, breadcrumb-equivalent page titles, `return_to` query param, consistent shells |
| **Honest empty/error states** | Never hide failure behind a success page. | All forms have inline error variant; system errors expose the investigation ID |
| **No silent state changes** | Every write produces an audit row; every settings change goes through "review changes". | Audit page chain status, settings review banner |
| **Boring is good** | No flashy interactions. Confidence comes from predictability. | No spinners on instant operations, no animations on critical actions |

---

## 5. Main User Scenarios

Five flows the implementation must preserve end-to-end. Each names the
actor, start point, main steps, success state, and the UX concerns
that drove the design.

### 5.1 Initial setup path

| Field | Value |
| --- | --- |
| **Actor** | First-time operator after fresh install |
| **Start** | Bare server, no users, setup-token printed in launch logs |
| **Steps** | (1) Visit `/setup` with token cookie set → welcome card. (2) `/setup/admin` → username/email/password form. (3) `/setup/security` → HIBP mode + default language choice. (4) `/setup/done` → confirmation, login link. |
| **Success state** | Admin user exists; install marked initialised; welcome email queued; gate closes (further visits to `/setup` show "Setup is closed"). |
| **UX concerns** | Operator must understand each step in isolation; password policy visible; token cookie scoped to `/setup`; dev mode (`--dev`) bypasses the gate but discloses it visibly. |

### 5.2 Administrator daily operations

| Field | Value |
| --- | --- |
| **Actor** | Authenticated admin |
| **Start** | `/admin` dashboard |
| **Steps** | Sidebar navigation between Users / Clients / Security / Settings / Audit. Each section has list → detail → action. Detail pages have a top-aligned read surface and a bottom **danger zone** for destructive actions. |
| **Success state** | Whatever the operator came to do is done, with an audit row recorded. |
| **UX concerns** | Sidebar always present and left-anchored (RFC v0.4.8); search/filter row at the top of each list; bulk operations explicitly out of scope; admin cannot self-suspend. |

### 5.3 OIDC authorization path

| Field | Value |
| --- | --- |
| **Actor** | End user being authorized into a third-party client |
| **Start** | `/authorize?client_id=...&redirect_uri=...&...` |
| **Steps** | (1) If unauthenticated → `/login` → optionally `/mfa`. (2) Return to `/authorize`. (3) Render `/consent` with scopes itemised. (4) On approve → 303 to `redirect_uri` with code; on deny → 303 with `error=access_denied`. |
| **Success state** | `redirect_uri` reached with appropriate query params; audit row recorded for grant/deny. |
| **UX concerns** | Consent screen names every scope in user-readable terms; the client's display name and homepage URL must be shown; PKCE-only enforcement is silent (no UI for choosing flow). |

### 5.4 User authentication / MFA

| Field | Value |
| --- | --- |
| **Actor** | End user signing in |
| **Start** | `/login` |
| **Steps** | (1) Username/password. (2) On success, if MFA enabled → `/mfa` (TOTP code or recovery code). (3) On success → original destination or `/admin`. |
| **Failure paths** | Wrong password / wrong code → generic "invalid credentials" (no user enumeration); locked account → generic "cannot sign in"; MFA unavailable → recovery-code link visible. |
| **UX concerns** | Error wording cannot disclose whether the username exists. Recovery-code use is visibly counted (e.g. "8 / 10 remaining" on the MFA self-service page) so users notice consumption. |

### 5.5 Critical operation / step-up confirmation

| Field | Value |
| --- | --- |
| **Actor** | Admin or end user about to do something irreversible |
| **Start** | A "danger zone" button on a detail page (e.g. `/admin/users/{id}`'s "Suspend user", or `/me/security?tab=mfa`'s "Disable MFA") |
| **Steps** | (1) Button is a link to `/stepup?action=X&return_to=Y`. (2) Re-auth form (password or MFA). (3) On success → POST → 303 to `/confirm/{token}`. (4) Confirmation page renders the **impact summary** (RFC 007). (5) On confirm → execute, append audit row, 303 to `return_to`. |
| **Success state** | Action executed; audit row written; user back at the originating page with a success banner. |
| **UX concerns** | The impact summary is the **last point of bail-out**. It must enumerate consequences (e.g. "this revokes 3 active sessions, including this one"). Tickets are one-shot — a replayed `POST /confirm/{token}` falls back to `/admin` rather than re-executing. |

---

## 6. Information Architecture

### 6.1 Information hierarchy per screen

Every screen has exactly **one** primary concern. The implementation
should keep this discipline:

| Screen | Primary | Secondary | Contextual | Tertiary actions |
| --- | --- | --- | --- | --- |
| Dashboard | Stat cards (users, clients, last login, last audit row) | Recent audit excerpt | Quick links | — |
| Users list | The table of users | Search/filter row | "+ Create user" button | View per row |
| User detail | Name/email/status header | Last seen, MFA state, session count | Audit entries referencing this user | Suspend / Resume / Delete in danger zone |
| Clients list | The table of OIDC clients | Search/filter row | "+ Create client" | View per row |
| Client detail | Redirect URIs, scopes, type | Client ID, secret display (rotate) | Token endpoint auth method | Rotate secret, Delete in danger zone |
| Audit log | The table | Chain status badge, last verified time | Filter row, secret-hidden hint | Export NDJSON, Verify chain |
| Settings (per tab) | The form for that tab | Inline help text per field | Tab nav | "Review changes" → step-up |
| Self-service (`/me/security`) | The active tab's content | Tab nav | — | Per-tab actions; danger zone actions go through step-up |

### 6.2 Context preservation

The user never loses orientation because of the following invariants:

1. **Sidebar always shows the active section** via `aria-current="page"`.
2. **Page titles match the sidebar label** so the operator can verify
   their location.
3. **All step-up flows carry `return_to`** — the user comes back to the
   exact page they left.
4. **The setup-token cookie is path-scoped to `/setup`** — it cannot
   bleed into other surfaces.
5. **The setup gate has four explicit states** (open / closed / locked
   / dev-disclosed); the operator is never guessing which one applies.
6. **Form submissions either succeed visibly or render the same form
   with an inline error** — never a blank redirect.

### 6.3 Screen relationship map

```
                    ┌──────────────────────────────────┐
                    │                                  │
              ┌─────► /setup ─► /setup/admin ─►        │
              │      /setup/security ─► /setup/done    │
   fresh      │                                        │
   install ───┤                                        │
              │                                        │
              └──────►  (gate closes) ─────────────────┘
                                       │
                                       ▼
                                    /login ──► /mfa ──► /admin (dashboard)
                                       ▲                  │
   external                            │                  │ sidebar
   relying ────► /authorize ──► /login (if needed)        │
   party                       └► /mfa ──► /consent       ├─► /admin/users ──► /admin/users/{id}
                                            │             ├─► /admin/clients ──► /admin/clients/{id}
                                            ▼             ├─► /admin/security
                                       redirect_uri       ├─► /admin/settings (six tabs)
                                                          └─► /admin/audit

                                                          /me/security (six tabs: overview / password / mfa / passkey / sessions / language)

                                  (any danger button) ──► /stepup ──► /confirm/{token} ──► return_to
```

Three structural rules embedded in the map:

- **Setup is a one-way street.** Once `/setup/done` is reached, the
  gate closes; subsequent `/setup` hits show the closed card.
- **The OIDC authorize path joins the auth path mid-flow.** It does
  not duplicate `/login` or `/mfa`.
- **The step-up loop is the **only** way dangerous actions execute.**
  No handler should execute a destructive operation outside of this
  loop.

---

## 7. Screen Groups and Responsibilities

A condensed inventory. Full screen-by-screen detail is in
`SCREEN_INVENTORY.md`.

### 7.1 Setup (4 screens)

`/setup`, `/setup/admin`, `/setup/security`, `/setup/done`. One task
per screen. Setup-token gate (RFC 004) determines whether each screen
renders the wizard step, the closed card, the locked page, or the
dev-disclosed variant. Should **not** mix with login / admin
navigation — these screens have a minimal "simple shell" with no
sidebar.

### 7.2 Authentication (3 screens)

`/login`, `/mfa`, `/forgot-password` (+ `/forgot-password/sent`,
`/forgot-password/reset`, `/forgot-password/reset/done`). All on the
simple shell. Wording is **deliberately generic** to prevent user
enumeration (§11.4).

### 7.3 Authorization (2 screens, stubbed)

`/authorize`, `/consent`. The mockup shows the consent screen layout
and the redirect-back behaviour on approve/deny. The discovery stub
at `/.well-known/openid-configuration` is a placeholder.

### 7.4 Admin (5 sections)

Lives in the admin shell (sidebar + content). Each section is a
list → detail → action loop:

| Section | List | Detail | Notes |
| --- | --- | --- | --- |
| Dashboard | `/admin` (single page) | — | Stat cards + recent activity |
| Users | `/admin/users` | `/admin/users/{id}` | Danger zone: Suspend / Resume / Delete |
| Clients | `/admin/clients` | `/admin/clients/{id}` | Confidential vs Public; secret rotation; PKCE-only |
| Security | `/admin/security` (single page) | — | Signing keys lifecycle (RFC 017); session policy; passkey policy |
| Settings | `/admin/settings?tab=...` | — | Six tabs; review-before-save; all writes go through step-up |
| Audit | `/admin/audit` | — | Chain status, event taxonomy, filter, NDJSON export |

### 7.5 Self-service (`/me/security`, 1 page, 6 tabs)

Tabs: overview / password / mfa / passkey / sessions / language. Each
tab is a single focused task. Destructive actions (disable MFA,
remove passkey, revoke session) route through step-up.

### 7.6 Step-up + confirmation (2 screens)

`/stepup` (re-auth form), `/confirm/{token}` (impact summary). The
**only** path through which destructive actions execute.

### 7.7 System error pages (5 routes)

`/400`, `/403`, `/404`, `/410` (expired ticket), `/429`, `/500`. All
include the **investigation ID** so support has one number to track.
The 404 fallback renders the same component.

---

## 8. Interaction and Behaviour Rules

### 8.1 Button and link states

All interactive elements must expose these states:

| State | Treatment |
| --- | --- |
| Default | As shown in mockup |
| Hover | Slight background tint (`--state-hover` token) |
| Focus | **Visible focus ring** — required for keyboard nav; never `outline: none` without replacement |
| Active | Same as hover or slight press-down treatment |
| Disabled | Reduced opacity + `aria-disabled="true"` + cursor: not-allowed; not just visual |
| Loading | For async submits: button shows "Working…" text, becomes disabled |

Special:

- **Danger buttons** (`btn--danger`, `btn--danger-outline`): always
  in a `<section class="danger-zone">` or carry a step-up redirect.
- **Step-up buttons**: render as warning colour (not danger),
  signalling "this needs re-auth" rather than "this destroys data".

### 8.2 Validation and feedback

| Trigger | Where the message appears | Wording |
| --- | --- | --- |
| Field too short / wrong format | Inline under the field, on submit | Specific ("Email format invalid") |
| Password fails policy | Inline under the password field | List of failed rules |
| HIBP hit (enforce mode) | Inline error replacing submit | "This password appears in known breach datasets" |
| HIBP hit (warn mode) | Inline warning, submit still works | "This password is known. We strongly recommend changing it." |
| HIBP service down | Inline error, submit blocked | "Cannot verify password safety. Try again later." (fail-closed per RFC 012) |
| Login fails | Generic banner above form | "Sign-in failed" — never "user not found" or "wrong password" |
| Step-up succeeds | Banner on the return-to page | "Action completed" (with the action name) |
| Settings saved | Banner on the same tab | "Saved" — with link to audit log if relevant |
| System error | Full-page error component | Title + investigation ID + plain-English what-to-do |

### 8.3 Transition continuity

The mockup is **deliberately animation-free**. The shared shell
across login / MFA / consent provides perceived continuity through
layout, not motion. The implementation should preserve this: no
loading spinners on operations that complete in <200ms; no fade
transitions on page swaps; no skeleton screens — render the real
content or a typed error.

---

## 9. Visual System Rules

### 9.1 Spacing rhythm

The mockup uses a **4 px base unit** with multipliers at 4, 8, 12, 16,
20, 24. Variables exposed in `theme.rs`:

| Token (CSS var) | Use |
| --- | --- |
| Component internal padding | `12px` (compact) or `24px` (card) |
| Between sibling cards | `16px` |
| Between major sections | `24px` |
| Page outer padding (admin) | `24px` |
| Page outer padding (admin, mobile ≤800 px) | `16px` |
| Grid gap (sidebar / content) | `24px` |

### 9.2 Typography levels

| Level | Use | CSS class / element |
| --- | --- | --- |
| H1 | Page title — one per page | `<h1>` |
| H2 | Card titles, section dividers | `<h2 class="card__title">` |
| H3 | Sub-sections inside a card | `<h3>` |
| Body | Default paragraph text | `<p>` |
| Helper / hint | Field hints, subtle notes | `<small>`, `.text-muted` |
| Code | Identifiers (UIDs, KIDs, ticket IDs) | `<code>` |
| Caption | Table headers, badges | `<th>`, `.badge` |

### 9.3 Colour semantics

Colours **carry meaning** and must not be the sole signal (see §10.4).

| Colour role | Semantic meaning | Example uses |
| --- | --- | --- |
| **Accent / primary** | Primary CTA, current navigation | "Save", "Sign in", `aria-current="page"` |
| **Success** | Action completed safely | Active user badge, audit row OK, MFA enabled |
| **Warning** | Step-up required, attention needed | Step-up buttons, retired-but-locked signing key |
| **Danger** | Irreversible / destructive | Suspend user, delete client, MFA disable |
| **Info** | Neutral context, informational | Public client badge, HIBP warn-mode hint |
| **Neutral / muted** | Background, secondary text, disabled | Form hints, subtle subtitles |
| **Focus** | Keyboard focus indicator | Form-control focus border, button focus ring |

The four-mode theme system (`auto` / `light` / `dark`, plus the
automatic following of `prefers-color-scheme`) is in RFC 009. Every
colour role has a paired light/dark token; never hardcode hex.

---

## 10. Accessibility Requirements (ABDD)

ABDD is a non-negotiable constraint of the product. The implementation
must preserve every accessibility property the mockup ships with.

### 10.1 Semantic structure

The mockup uses semantic HTML throughout. The implementation must
not regress this when refactoring components:

| Element | Use |
| --- | --- |
| `<header>` | Top app bar |
| `<nav aria-label="…">` | Every navigation region (sidebar, settings tabs, language switch) |
| `<main>` | Single per page |
| `<section>` | Content groups inside main |
| `<form>` | Every interactive form (no `<div>`-wrapped fakes) |
| `<table>` | Tabular data (users, clients, audit, sessions, keys, passkeys) |
| `<button>` vs `<a>` | Action vs navigation. Submit buttons in forms; links for navigation |
| `<dialog>` | Reserved — the mockup avoids modal dialogs (prefers separate routes) |
| `aria-live="polite"` | Banner / toast region for transient messages |
| `aria-live="assertive"` | Reserved for true emergencies (we don't use it yet) |
| `role="status"` | Chain-verification status badge |
| `role="alert"` | The dev-mode disclosure banner |

### 10.2 Keyboard navigation

| Requirement | Where applied |
| --- | --- |
| Tab order follows document order | All forms |
| Visible focus ring always | All buttons, links, inputs |
| First focusable element in form is the first field, not a tab nav | Settings, Self-service |
| Escape closes — | Nothing, currently (no modals) |
| Enter submits the form | All forms |
| Step-up confirmation requires explicit button press | Cannot be triggered by Enter on a different field |

### 10.3 Screen reader behaviour

| Event | Announcement mechanism |
| --- | --- |
| Validation error | Inline error text is the field's `aria-describedby` target |
| Form-level error | Banner above form has `aria-live="polite"` |
| Successful save | Same banner region |
| Destructive action confirmation | Confirmation page is a full route; `<h1>` describes the action; impact summary is in a `<ul>` |
| Login failure | Generic banner, `aria-live="polite"` |
| MFA challenge | Page title and `<h1>` name the challenge type |
| Authorization outcome | Consent submit redirects; the success/denial is communicated by the destination page, not by a transient toast |

### 10.4 Never colour-only

Every status communicated by colour must also be communicated by:

- a **label** (visible text), or
- an **icon** with a visible label, or
- a **structural element** (badge with text inside, position in the
  page).

This is mandatory. Audit any new screen against it.

---

## 11. Security-Sensitive UX and Edge Cases

### 11.1 Wording rules to prevent user enumeration

- Login failure: "Sign-in failed" — never "user not found" or "wrong
  password".
- Password reset request: same response whether the email exists or
  not — "If an account exists, we've sent a reset link."
- Forgot-password reset-token expired / invalid: generic "This link
  is no longer valid" — never distinguish expired vs malformed.
- MFA failure: "Code did not match" — never reveal whether the user
  is enrolled.

### 11.2 Fail-closed behaviours

| Operation | When the dependent service is down | Behaviour |
| --- | --- | --- |
| HIBP check (RFC 012) | API unreachable | Reject the password regardless of mode |
| Signing key publish (RFC 017) | Existing keys can't be reached | Block the operation, show inline error |
| Audit emission | DB unreachable | Block the parent operation (cannot succeed without audit row) |
| Session list (`/me/security?tab=sessions`) | Backend unreachable | Render empty state with explicit "couldn't load" banner — never silently show no sessions |

### 11.3 Step-up coverage

Every destructive operation flows through step-up. The action key
space is documented in RFC 007 §"DANGEROUS_ACTIONS":

- `user.suspend`, `user.resume`, `user.delete`
- `client.secret.rotate`, `client.delete`
- `signing_key.publish`, `signing_key.activate`,
  `signing_key.retire`, `signing_key.delete`
- `me.mfa.disable`, `me.mfa.regen_recovery`, `me.passkey.delete`,
  `me.session.revoke`, `me.sessions.revoke_all`
- `settings.update.basic`, `settings.update.security`,
  `settings.update.auth`, `settings.update.email`,
  `settings.update.logs`
- `sessions.revoke_all` (admin-side)

A handler that performs any of these **outside** of the step-up loop
is a bug. The implementation should structurally enforce this — e.g.
make `consume_ticket` the only way to obtain the action context.

### 11.4 Edge-case screens (must be implemented)

| Case | UI |
| --- | --- |
| Setup gate closed (already initialised) | "Setup is closed" card with link to `/login` |
| Setup gate locked (no/invalid token) | "Setup is locked" page, instructions to retrieve the token |
| Step-up ticket expired | `/410` Gone page with "request again" link to original action |
| Account locked | `/login` returns to itself with the generic-failure banner; no special account-locked screen (user enumeration risk) |
| MFA method unavailable | Recovery-code link visible on `/mfa` |
| Recovery code use | Counted; "8 / 10 remaining" visible on `/me/security?tab=mfa` |
| Authorize: client unknown | Render the system 400 page with `error=invalid_client` |
| Authorize: redirect_uri mismatch | Same — never redirect to a non-allowlisted URI |
| Authorize: deny | 303 to `redirect_uri?error=access_denied` |
| Session timeout | Next request lands on `/login`; the original URL is preserved as `?return_to=...` |

---

## 12. Constraints and Open Questions

This section is **intentionally explicit**. Each item is something
the implementation team should not silently decide.

### 12.1 Confirmed constraints

These are settled. Do not relitigate without RFC-level discussion.

| Constraint | Source |
| --- | --- |
| OIDC clients are PKCE-only; no implicit / hybrid flow | RFC contract |
| Step-up is required for every destructive action | RFC 007 |
| Audit chain is append-only; verification is a UI-visible operation | RFC 016 |
| Setup token is one-shot, path-scoped, single-use | RFC 004 |
| Email rendering uses typed contexts (no string-template handlers) | RFC 011 |
| Theme + locale are cookie-set, never localStorage / JS-set | RFCs 009, mockup convention |
| ABDD principles apply to every new screen | Product constraint |

### 12.2 Open questions (decision-owner identified)

#### Q1 — MFA enable/disable surface

- **Why it is open**: the mockup puts MFA enable / disable on
  `/me/security?tab=mfa`, but the admin may also need to disable MFA
  for a user who has lost their device.
- **Mockup assumption**: admin uses a recovery flow on the user's
  detail page, not a "force-disable MFA" button.
- **Implementation reality**: TBD — backend may or may not expose
  admin-side MFA reset.
- **Recommended next decision owner**: **Security Reviewer**.

#### Q2 — Settings step-up granularity

- **Why it is open**: every settings tab currently routes through
  `settings.update.<tab>`. Some changes (e.g. `service_name`) are
  cosmetic; requiring step-up may be over-protective.
- **Mockup assumption**: all settings changes go through step-up.
- **Implementation reality**: TBD.
- **Recommended next decision owner**: **Product Manager** with
  **Security Reviewer**.

#### Q3 — Audit log retention and export

- **Why it is open**: the mockup shows "Export NDJSON" but does not
  define retention policy or export filtering.
- **Mockup assumption**: full log is available, exporter is unstated.
- **Implementation reality**: depends on backend storage.
- **Recommended next decision owner**: **Architect**.

#### Q4 — Bulk operations

- **Why it is open**: every list view supports single-row actions
  only. Operators may want bulk-suspend, bulk-delete, etc.
- **Mockup assumption**: out of scope; one user, one action.
- **Implementation reality**: not currently designed.
- **Recommended next decision owner**: **Product Manager** — explicit
  product-scope decision.

#### Q5 — Real-time updates

- **Why it is open**: the mockup is pure SSR with no live updates.
  Audit log doesn't auto-refresh; session list doesn't auto-refresh.
- **Mockup assumption**: refresh is manual.
- **Implementation reality**: a refresh button suffices; SSE/WS is
  out of scope.
- **Recommended next decision owner**: **Architect** (defer; SSR is
  fine).

#### Q6 — Localisation scope

- **Why it is open**: the mockup ships Japanese + English. A third
  language addition is mechanical but the data flow (locale →
  template registry → email body) is settled (RFC 011).
- **Mockup assumption**: ja/en only.
- **Implementation reality**: extending is non-blocking but should
  be planned.
- **Recommended next decision owner**: **Product Manager**.

#### Q7 — Self-service password change vs admin reset

- **Why it is open**: `/me/security?tab=password` lets the user
  change their password. Admin-side password reset is not in the
  mockup.
- **Mockup assumption**: admin uses the standard forgot-password
  flow, possibly initiated on behalf of the user.
- **Implementation reality**: TBD.
- **Recommended next decision owner**: **Security Reviewer**.

### 12.3 Items the mockup conceptually placed only (do not implement yet)

| Item | Where it appears | Status |
| --- | --- | --- |
| OIDC token endpoint, userinfo, JWKS | Discovery stub references them | Conceptual — implementation extends backend |
| Real HIBP API integration | `/me/security?tab=password` references it | Conceptual — needs API key config |
| Mail transport (SMTP) | `MailService::send` is no-op | Conceptual — `lettre` integration planned |
| Audit hash-chain verification (cryptographic) | Chain-status badge | UI exists; cryptographic verification is a backend concern |
| Passkey (WebAuthn) registration ceremony | `/me/security?tab=passkey` | UI complete; JS + backend ceremony is a separate effort |
| Backup / restore CLI | Not in UI | Out of scope |

---

## 13. Differences vs Current Implementation

The implementation team must compare the mockup against the live
backend on the following axes and produce a **delta document** before
beginning integration. The mockup team cannot do this comparison
because we do not have a current view of the running service.

For each delta the implementation team finds, classify it:

| Class | What it means | Resolution path |
| --- | --- | --- |
| **A — UI adaptation** | Backend already supports the operation; only UI wiring needed | Implementation team owns; integrate directly |
| **B — Data/state extension** | Backend has the data but no exposed endpoint or wrong shape | Light backend work + UI; involve **Architect** |
| **C — Backend capability gap** | Backend doesn't do this yet (e.g. audit hash chain, real HIBP) | Schedule with **Architect**; mockup is the spec |
| **D — Conflicts with current implementation** | Mockup proposes a flow the backend currently can't accommodate | **Escalate** per §14; mockup may need revision, or backend, or both |

Suggested first-pass deltas to expect (without seeing the backend):

- **Authentication state**: the mockup assumes session cookies; the
  backend's session model may differ.
- **Setup token shape**: the mockup uses a short opaque token in a
  cookie; the backend may use a different mechanism.
- **Audit row schema**: the mockup uses RFC 016's taxonomy; if the
  backend uses different event names, the audit page filter needs
  to be re-keyed.
- **Signing key states**: the mockup uses pending / active / retired;
  the backend may have different state names.
- **HIBP modes**: the mockup uses off / warn / enforce; the backend
  may have only enable / disable.
- **OIDC stub**: the mockup's `/authorize` and `/consent` are
  placeholders; the backend likely has a real implementation. The
  UI shape (consent page layout) is the contribution; the protocol
  is the backend's.

---

## 14. Escalation and Decision Policy

### 14.1 General rule

If the implementation team finds **any** discrepancy between the
mockup, the current implementation, the development specification,
or the project's security constraints, they **must not** resolve it
by self-interpretation alone.

### 14.2 Escalation procedure

1. **Identify** the discrepancy clearly — write it down, include the
   mockup screen reference (route URL) and the relevant
   implementation code reference if any.
2. **Classify** it as one of:
   - UX issue (visual or flow difference, no security impact)
   - Security issue (could change the security posture)
   - Product scope issue (the feature is in question)
   - Technical feasibility issue (backend cannot support as designed)
3. **Record** it in the issue tracker with the classification.
4. **Consult** the appropriate decision-maker:

| Classification | First consult |
| --- | --- |
| UX | Product Manager |
| Security | Security Reviewer (mandatory) |
| Product scope | Product Manager |
| Technical feasibility | Architect |

### 14.3 Resolution priority

When two principles conflict, resolve in this order:

1. **Security** — never compromised
2. **Robustness** — fail-closed, predictable
3. **Maintainability** — code the next maintainer can read
4. **Standards compliance** — OIDC / OAuth / WebAuthn / RFC fidelity
5. **Usability** — the mockup's intent
6. **Visual preference** — the mockup's appearance

A common case: a security review may demand a wording change that
weakens the UX intent. **The security wording wins.**

### 14.4 Temporary implementation rule

If implementation must proceed before a full decision is made:

- Preserve safety — never weaken a security boundary as a stopgap.
- Preserve clarity — don't add features that aren't decided.
- **Mark the area visibly in code** with a `// TODO(handoff-Q<n>)` comment
  referencing the open question from §12.2.
- Choose the **most reversible** option. Adding a feature later is
  easier than removing one.

---

## 15. Implementation Recommendations

### 15.1 Recommended integration order

A first-week plan that minimises risk:

1. **Day 1 — Adopt the visual system.** Copy `theme.rs` (or its
   tokens) into the running service. Verify dark mode + locale
   switching. This is the least controversial slice and unblocks
   visual review.
2. **Day 2 — Adopt the shells.** `render_simple`, `render_admin`,
   header / sidebar / footer. Login page now visually matches the
   mockup.
3. **Days 3–5 — Adopt the admin shell pages in dependency order.**
   Dashboard → Users → Clients → Audit. Each page is read-only,
   so this is low risk.
4. **Week 2 — Step-up flow.** Adopt `/stepup` + `/confirm/{token}`
   and route the first destructive action (e.g. user suspend)
   through it.
5. **Week 3 — Settings + self-service.** These are write-heavy and
   need backend support. Stage tab-by-tab.
6. **Week 4 — Setup wizard.** Replace the existing token-print
   bootstrap. This is the last to land because it's a one-shot UX.

### 15.2 The trait seam (`sui-id-core`)

The mockup is structured around a trait-based service seam
(RFC 020). This is the recommended **integration interface** for the
implementation team:

```rust
// sui-id-core exposes nine traits, all `Send + Sync` + `#[async_trait]`:
trait UserService    { async fn list, get, create, … }
trait ClientService  { async fn list, get, … }
trait SessionService { async fn list_for_user, store_ticket, peek_ticket, consume_ticket, … }
trait AuditService   { async fn list_recent, chain_status, last_verified, … }
trait SettingsService{ async fn get, set, … }
trait KeyService     { async fn list, … }
trait MfaService     { async fn status_for_user, … }
trait MailService    { async fn send, … }
trait HibpService    { async fn check, … }
```

The implementation team can:

- **Keep the trait shapes** and implement them against the real
  backend (`sui-id-store-sqlite`-style crate).
- **Extend the trait method sets** when the real backend exposes
  capabilities the mockup didn't need (e.g. pagination cursors).
- **Replace `AppState::new_with_mock()`** with
  `AppState::new_with_sqlite()` (or equivalent) — handlers don't
  change.

If the implementation team prefers a different seam, that's fine —
but **preserve the boundary**. The mockup's contribution here is the
*shape* of "what handlers need from the backend" expressed as 9
small traits.

### 15.3 Don't over-abstract

The mockup is intentionally **small and concrete**. Some patterns
look "ripe for generalisation" but the mockup left them concrete on
purpose:

- **Each handler is a function, not a trait method.** Resist
  turning handlers into typed builders or layered handler traits.
- **Each form has its own deserialiser struct** (`AdminForm`,
  `SecurityForm`, `StepUpForm`). Don't merge into a generic
  `Form<T>` infrastructure.
- **Each page has its own render function.** Components exist
  (`badge`, `callout`, `step_indicator`) but page-level structure
  is deliberately handwritten.

### 15.4 SSR / hydration awareness

The mockup is **pure SSR** — every page is a complete HTML document
delivered by the server. There is **no client-side hydration**.
Forms post; pages reload. This is a deliberate choice — it eliminates
hydration-mismatch risk and keeps the security boundary simple.

If the implementation team wants partial hydration (Leptos islands,
htmx, etc.), preserve the SSR-first behaviour as the baseline. The
mockup's design works with JavaScript disabled.

### 15.5 Asynchronous / loading behaviour

Per §8.3, the mockup avoids spinners. For operations that genuinely
take >200ms (signing key publish, email send, HIBP check), the
implementation should:

- Render the previous page's success banner *after* the operation
  completes (synchronous full-form-post is fine).
- For genuinely long operations (e.g. bulk audit verification),
  show the same page with a progress section that polls or
  refreshes; do **not** introduce a modal spinner.

### 15.6 Mapping mockup routes to expected backend modules

Suggested, not prescriptive:

| Mockup route(s) | Backend module(s) likely involved |
| --- | --- |
| `/setup/*` | bootstrap / initial-admin module |
| `/login`, `/mfa`, `/forgot-password*` | auth / session module |
| `/authorize`, `/consent` | OIDC authorize endpoint |
| `/admin/users*` | user management |
| `/admin/clients*` | OIDC client management |
| `/admin/security` | signing-key store + session policy |
| `/admin/settings` | settings store |
| `/admin/audit` | audit log + chain verification |
| `/me/security*` | user self-service |
| `/stepup`, `/confirm/{token}` | step-up + ticket store |

---

## 16. Appendices

### 16.1 Companion documents

This handoff comes as a **package**. The companion files give the
implementation team faster lookup paths for specific concerns:

| Document | Purpose |
| --- | --- |
| `HANDOFF.md` (this file) | Main explanatory document |
| `SCREEN_INVENTORY.md` | Every screen with route, role, primary action, primary data |
| `FLOW_SUMMARY.md` | The five user flows from §5, with sequence diagrams |
| `OPEN_ISSUES.md` | The §12.2 open questions, formatted for direct issue-tracker import |
| `IMPLEMENTATION_NOTES.md` | Practical notes — file structure, build commands, the trait seam in code |

### 16.2 Glossary

| Term | Meaning |
| --- | --- |
| **ABDD** | Accessible by Default and by Design |
| **IDaaS** | Identity (and access management) as a Service |
| **Step-up** | Re-authentication required before a destructive action |
| **Impact summary** | The page rendered between step-up success and action execution; names the consequences |
| **Investigation ID** | An opaque ID shared by a system error page and the matching audit row |
| **Setup gate** | The state machine that decides whether `/setup/*` renders the wizard, the closed card, the locked page, or the dev-disclosed variant |
| **Danger zone** | The bottom section of detail pages where destructive actions live, visually and physically separated from read-only operations |
| **Service trait** | One of the 9 `Send + Sync` async traits in `sui-id-core` |
| **Mock impl** | The in-memory implementation of a service trait, used by the mockup binary |

### 16.3 RFC index

The mockup is documented by 21 RFCs in `rfcs/done/`. The most
load-bearing for implementation:

| RFC | Subject | Why it matters for implementation |
| --- | --- | --- |
| 002 | Promotion path | The swap-in-place plan for replacing mock with real impl |
| 003 | State copy (microcopy) | The wording bank — translate but do not invent |
| 004 | Setup token gate | The four-state gate logic |
| 007 | Step-up + confirmation | The mandatory pattern for destructive actions |
| 008 | System error pages | The 4xx / 5xx envelope + investigation ID |
| 011 | Email templates | Typed contexts; the renderer is the implementation's choice |
| 012 | HIBP UI surface | Three modes + fail-closed |
| 014 | Session lifecycle UI | FIFO eviction, current-device cannot revoke itself |
| 016 | Audit event taxonomy | The dot-separated event names; cascade rows |
| 017 | Signing key rotation | The four-state lifecycle + retention window |
| 020 | Backend integration seam | The trait surface |

The full RFC set is in `rfcs/done/`.

### 16.4 Mockup runtime quick reference

```bash
# Boot with mock data, mark setup initialised
./target/debug/sui-id-web --mock-init

# Boot in dev mode (setup gate disclosed but allows access)
./target/debug/sui-id-web --dev

# Default port
PORT=3000 ./target/debug/sui-id-web

# Routes
curl http://localhost:3000/login
curl http://localhost:3000/admin/audit
curl "http://localhost:3000/me/security?tab=mfa"
```

### 16.5 Final checklist for implementation team

Before considering a screen "integrated":

- [ ] Renders the same primary action in the same position as the mockup
- [ ] Preserves the danger-zone separation (if any)
- [ ] Has visible focus rings on all interactive elements
- [ ] Has `aria-current` / `aria-live` / `role` attributes where the mockup does
- [ ] Routes destructive actions through `/stepup`
- [ ] Has both ja and en strings
- [ ] Renders correctly in light and dark mode
- [ ] Renders correctly at ≤560 px (responsive tables, single-column admin layout)
- [ ] Inline errors render at the right place; no silent failures
- [ ] Includes the investigation ID on any error page

If any item fails, treat it as a §14 escalation.

---

**End of HANDOFF.md.** Companion documents follow.
