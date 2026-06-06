# Screen Inventory — sui-id Mockup

Every screen in the mockup, in route order. Use this as a checklist
when integrating, or as a reference when wiring routes.

For each screen:

- **Route** — the HTTP path
- **Role** — what this screen is for
- **Primary** — the one thing the user should be able to do here
- **Primary data** — what gets rendered
- **Danger zone?** — does it surface destructive actions?
- **Shell** — `simple` (centred narrow shell) or `admin` (sidebar + content)

---

## Entry routes

### `/`
- **Role**: Friendly first impression.
- **Behaviour**: 303 redirect to `/login`.
- **Shell**: n/a.

### `/.well-known/openid-configuration`
- **Role**: OIDC discovery placeholder.
- **Status**: **Stub only.** Returns a static JSON shape.
- **Implementation note**: real OIDC implementation owns this.

---

## Setup (RFC 004)

### `/setup`
- **Role**: First wizard step. Gate decides whether to render the
  wizard, the closed card, the locked page, or the dev-disclosed
  variant.
- **Primary**: continue to admin creation.
- **Shell**: simple.
- **Gate states**:
  - `Closed` — install already initialised. Renders "Setup is closed".
  - `Locked` — no/invalid token cookie. Renders "Setup is locked" with
    token instructions.
  - `Allowed` — valid token cookie. Renders welcome wizard step.
  - `AllowedDev` — `--dev` mode. Renders welcome + dev-disclosure
    banner.

### `/setup/admin` (`GET` + `POST`)
- **Role**: Create the first admin user.
- **Primary**: form with username / email / password.
- **POST behaviour**: `state.users.create(NewUser)`, drops a
  path-scoped `sui_id_setup_uid` cookie carrying the new UID for the
  next step, redirects to `/setup/security`.
- **Shell**: simple.

### `/setup/security` (`GET` + `POST`)
- **Role**: Capture HIBP mode and default locale.
- **Primary**: two-field form (hibp: off/warn/enforce, lang: ja/en).
- **POST behaviour**: writes settings via `SettingsService::set`,
  fetches the new admin via the cookie, fires the `admin.first_admin`
  welcome email via `MailService::send`, calls
  `config::mark_initialised()`, clears both setup cookies, redirects
  to `/setup/done`.
- **Shell**: simple.

### `/setup/done`
- **Role**: Confirmation.
- **Primary**: "Sign in" link to `/login`.
- **Shell**: simple.

---

## Authentication

### `/login` (`GET` + `POST`)
- **Role**: Username + password sign-in.
- **Primary**: form submission.
- **Failure**: inline banner with generic "Sign-in failed" (no
  enumeration).
- **Success**: 303 to `/mfa` if MFA required, else to `return_to` or
  `/admin`.
- **Shell**: simple.

### `/mfa` (`GET` + `POST`)
- **Role**: TOTP code or recovery code entry.
- **Primary**: 6-digit code field.
- **Recovery link**: visible at all times.
- **Failure**: generic "Code did not match".
- **Shell**: simple.

### `/forgot-password` (`GET` + `POST`)
- **Role**: Request a password reset email.
- **POST behaviour**: 303 to `/forgot-password/sent` regardless of
  whether the email exists (anti-enumeration).
- **Shell**: simple.

### `/forgot-password/sent`
- **Role**: Confirmation page.
- **Primary**: "Check your email" message; resend link.
- **Shell**: simple.

### `/forgot-password/reset` (`GET` + `POST`)
- **Role**: Set new password using the reset token.
- **Primary**: password + confirm-password fields.
- **Failure paths**:
  - Token expired / invalid → generic "This link is no longer valid".
  - Password fails HIBP (enforce) → inline error.
  - Password fails HIBP (warn) → inline warning; submit still works.
- **Shell**: simple.

### `/forgot-password/reset/done`
- **Role**: Success confirmation.
- **Primary**: "Sign in with new password" link.
- **Shell**: simple.

---

## Authorization (OIDC) — stubbed protocol, finished UX

### `/authorize`
- **Role**: OIDC authorize endpoint UX shell.
- **Primary**: redirect to `/login` (if unauthenticated) or
  `/consent` (if authenticated). The mockup short-circuits straight
  to consent in dev.
- **Shell**: simple.
- **Status**: protocol details are **stubbed**; real OIDC
  implementation owns the parameter validation, code issuance, etc.

### `/consent` (`GET` + `POST`)
- **Role**: User-readable consent screen.
- **Primary**: itemised scope list, approve / deny buttons.
- **POST behaviour**: on approve → 303 to `redirect_uri` with `code`;
  on deny → 303 to `redirect_uri` with `error=access_denied`.
- **Shell**: simple.

---

## Admin shell

All routes below use the **admin shell** (sidebar + content), require
authentication, and have sidebar `aria-current="page"` set on the
active section.

### `/admin`
- **Role**: Dashboard.
- **Primary**: four stat cards (users / clients / last login / last
  audit row).
- **Primary data**: counts + a recent-audit excerpt.
- **Shell**: admin.

### `/admin/users`
- **Role**: User list.
- **Primary**: searchable / filterable table.
- **Primary data**: username, email, status, MFA enabled, last seen.
- **Tertiary action**: "+ Create user" (mockup-only stub).
- **Shell**: admin.

### `/admin/users/{id}`
- **Role**: User detail.
- **Primary data**: name, email, status badge, last-seen, MFA state.
- **Danger zone**: ✅ Suspend / Resume / Delete.
- **Shell**: admin.

### `/admin/clients`
- **Role**: OIDC client list.
- **Primary**: table of clients.
- **Primary data**: name, client ID, type (Public/Confidential),
  redirect-URI count, allowed scopes.
- **Shell**: admin.

### `/admin/clients/{id}`
- **Role**: Client detail.
- **Primary data**: redirect URIs, post-logout URIs, allowed scopes,
  client secret (Confidential only; rotation action).
- **Danger zone**: ✅ Delete.
- **Shell**: admin.

### `/admin/security`
- **Role**: Signing keys (RFC 017) + session policy + passkey policy.
- **Primary**: signing-key lifecycle table.
- **Per-key actions**: Activate (Pending → Active), Retire (Active →
  Retired), Delete (Retired + retention elapsed).
- **Global action**: Publish new key.
- **Shell**: admin.

### `/admin/settings?tab=...`
- **Role**: Server configuration (RFC 010).
- **Tabs**: `basic`, `security`, `auth`, `email`, `logs`, `other`.
- **Primary**: the form for the current tab.
- **Save behaviour**: "Review changes" button → step-up →
  `/confirm/{token}` → execute → 303 back to the same tab with a
  success banner.
- **Shell**: admin.

### `/admin/audit`
- **Role**: Audit log viewer (RFC 016).
- **Primary**: the audit row table.
- **Header data**: chain status badge, last-verified time, "Verify
  chain now" button (requires step-up).
- **Filter row**: event, actor, when, search.
- **Tertiary**: Export NDJSON.
- **Shell**: admin.

### `/admin/stepup`
- **Role**: Alias of `/stepup` so admin-shell links can keep their
  `/admin/...` prefix without a context switch.
- **Shell**: admin (header only; same step-up content).

---

## Self-service

### `/me/security?tab=...`
- **Role**: Authenticated user's own account controls.
- **Tabs**: `overview`, `password`, `mfa`, `passkey`, `sessions`,
  `language`.
- **Per-tab content**:
  - **overview**: account summary card.
  - **password**: change password form; HIBP feedback inline.
  - **mfa**: status (data-driven from `MfaService::status_for_user`);
    if enrolled, shows recovery codes remaining + disable / regenerate
    buttons; if not enrolled, "Set up TOTP" CTA; enrollment wizard
    accessible via `?enroll=totp&step=1|2|3`.
  - **passkey**: list of registered passkeys + add button.
  - **sessions**: list of active sessions; current device cannot
    revoke itself (RFC 014).
  - **language**: ja / en radio.
- **Destructive actions** (disable MFA, remove passkey, revoke
  session, revoke all): route through step-up.
- **Shell**: admin (sidebar shows "Me" section active).

---

## Step-up + confirmation (RFC 007)

### `/stepup` (`GET` + `POST`)
- **Role**: Re-authentication form before a destructive action.
- **Primary**: re-auth form (password / TOTP).
- **POST behaviour**: validates re-auth → `SessionService::store_ticket`
  → 303 to `/confirm/{token}`.
- **Shell**: simple.

### `/confirm/{token}` (`GET` + `POST`)
- **Role**: Impact-summary confirmation.
- **GET behaviour**: `SessionService::peek_ticket` (non-destructive);
  renders the action name + impact summary; "Proceed" + "Cancel"
  buttons.
- **POST behaviour**: `SessionService::consume_ticket` (one-shot);
  on success → execute + 303 to `return_to`; on miss → 303 to
  `/admin` fallback.
- **Shell**: simple.

---

## System error pages (RFC 008)

All include the investigation ID prominently.

### `/400`
- **Role**: Bad request envelope.
- **Use**: invalid form input not caught inline; malformed OIDC params.

### `/403`
- **Role**: Forbidden.
- **Use**: action requires privilege the actor doesn't hold.

### `/404` (fallback)
- **Role**: Not found.
- **Use**: any unmatched route.

### `/410`
- **Role**: Gone — used for expired step-up tickets.
- **Use**: `/confirm/{token}` with expired or unknown token.
- **Action**: link to "Request again" pointing to the originating
  `/stepup?action=...`.

### `/429`
- **Role**: Too many requests.
- **Use**: rate-limited operation.

### `/500`
- **Role**: Server error.
- **Use**: unexpected backend failure.
- **Wording**: includes an investigation ID for support handoff.

---

## Theme + locale (out-of-band)

### `/theme/{auto|light|dark}?return_to=...`
- **Role**: Set theme cookie, redirect back.
- **Cookie**: `sui_id_theme`, path `/`, no JS.

### `/lang/{ja|en}?return_to=...`
- **Role**: Set locale cookie, redirect back.
- **Cookie**: `sui_id_lang`, path `/`, no JS.

---

## Mapping to mock service traits

| Screen group | Traits used |
| --- | --- |
| `/setup/*` | `UserService::create`, `SettingsService::set`, `MailService::send` |
| `/login`, `/mfa`, `/forgot-password*` | (auth path — not yet on the seam in the mockup) |
| `/authorize`, `/consent` | (stub — protocol layer) |
| `/admin/users*` | `UserService::list`, `UserService::get` |
| `/admin/clients*` | `ClientService::list`, `ClientService::get` |
| `/admin/security` | `KeyService::list` |
| `/admin/settings` | `SettingsService::get`, `SettingsService::set` (via step-up) |
| `/admin/audit` | `AuditService::list_recent`, `chain_status`, `last_verified` |
| `/me/security?tab=mfa` | `MfaService::status_for_user` |
| `/me/security?tab=sessions` | `SessionService::list_for_user` |
| `/me/security?tab=password` | `HibpService::check` |
| `/stepup`, `/confirm/{token}` | `SessionService::{store,peek,consume}_ticket` |

---

## Screen count summary

| Group | Count |
| --- | --- |
| Setup | 4 |
| Authentication | 6 (login, mfa, forgot-password ×4) |
| Authorization | 2 |
| Admin | 8 (dashboard + users ×2 + clients ×2 + security + settings + audit) |
| Self-service | 1 page × 6 tabs |
| Step-up | 2 |
| System errors | 5 |
| Theme / locale | 2 |
| **Total distinct routes** | **35** |
