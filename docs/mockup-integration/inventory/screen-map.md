# Screen Map — Mockup ↔ Product

Phase-0 deliverable of [RFC-MI-000](../../../rfcs/done/RFC-MI-000-baseline-delta-inventory.md).
Generated against `sui-id-web-mockup v0.4.8` ↔ `sui-id v0.49.0`.

## Reading the table

- **Mockup screen** — the route as declared in
  `mockup-tmp/crates/sui-id-web/src/router.rs`.
- **Product route** — the route as declared in
  `crates/sui-id/src/router.rs` (v0.49.0 codebase).
- **`render_*`** — the public render function from
  `crates/sui-id-web/src/lib.rs` that produces the page body, or `—`
  if the product produces the body in a handler.
- **Handler module** — file under `crates/sui-id/src/handlers/`.
- **Shell** — which Leptos shell wraps the page (`AuthShell` /
  `Shell`).
- **Auth req.** — `none` (public), `setup-token` (setup wizard),
  `user` (any authenticated user), `admin` (admin privilege required).
- **CSRF on POST** — does the page emit a form whose POST is CSRF-protected?
- **Status** — one of the five values RFC-MI-000 §7 mandates:
  - `ready-to-integrate` — mockup intent is structurally compatible
    with the product surface; visual adaptation only.
  - `needs-visual-adaptation` — same as ready, called out separately
    when the mockup's IA or layout differs noticeably from the product
    page; the integration work is visual only.
  - `requires-handler-change` — handler-side code must change (route
    rename, new parameter, new field in the Data struct).
  - `requires-backend-review` — touches a contract owned by a
    non-web crate (`sui-id-core`, `sui-id-store`, OIDC engine) and
    needs that crate's owner to sign off.
  - `do-not-implement-yet` — mockup-only construct that the product
    intentionally does not surface; defer or reject.

## Entry routes

| Mockup screen | Product route | `render_*` | Handler module | Shell | Auth req. | CSRF on POST | Status |
|---|---|---|---|---|---|---|---|
| `/` | `/` | — | `handlers::index::root` | — | none | n/a | ready-to-integrate |
| `/.well-known/openid-configuration` | `/.well-known/openid-configuration` | — | `handlers::oidc::discovery` | n/a (JSON) | none | n/a | mockup is a stub; product is the real implementation — **do-not-implement-yet** (no mockup intent to absorb) |
| n/a | `/healthz` | — | `handlers::index::healthz` | n/a | none | n/a | product-only; mockup does not surface (keep as-is) |

## Setup wizard

The mockup has a **three-step** wizard (`welcome → admin → security
→ done`). The product has a **four-step** wizard (`welcome → admin →
lang → hibp → done`). The mockup's `/setup/security` collapses
language + HIBP into one screen.

| Mockup screen | Product route | `render_*` | Handler module | Shell | Auth req. | CSRF on POST | Status |
|---|---|---|---|---|---|---|---|
| `/setup` (Closed / Locked / Allowed / AllowedDev states) | `/setup` | `render_setup_welcome` | `handlers::setup::welcome_get` | `AuthShell` | setup-token via `?token=…` | n/a | requires-handler-change (mockup's four gate states are richer than the product's "init/no-init" branch — see RFC-MI-040) |
| `/setup/admin` GET+POST | `/setup/admin` GET+POST | `render_setup_admin` | `handlers::setup::admin_get` / `admin_post` | `AuthShell` | setup-token | ✓ | needs-visual-adaptation |
| `/setup/security` GET+POST (lang + HIBP combined) | `/setup/lang` GET+POST · `/setup/hibp` GET+POST | `render_setup_lang` · `render_setup_hibp` | `handlers::setup::lang_*` / `hibp_*` | `AuthShell` | setup-token | ✓ | **D-OPEN: combine or keep split?** Mockup combines; product splits. Migration plan §11 leaves this open. Default: keep product's two-screen split (smaller per-screen surface, lower form-failure blast radius). RFC-MI-040 must resolve. |
| `/setup/done` | `/setup/done` | `render_setup_done` | `handlers::setup::done_get` | `AuthShell` | n/a | n/a | needs-visual-adaptation |

## Authentication

| Mockup screen | Product route | `render_*` | Handler module | Shell | Auth req. | CSRF on POST | Status |
|---|---|---|---|---|---|---|---|
| `/login` GET+POST | `/admin/login` GET+POST | `render_login` | `handlers::admin::login_get` / `login_post` | `AuthShell` | none | ✓ | needs-visual-adaptation. **Route delta:** mockup uses `/login`, product uses `/admin/login`. Product path stays. |
| `/mfa` GET+POST | `/admin/login/mfa` GET+POST | `render_mfa_challenge` | `handlers::admin::mfa_challenge_get` / `mfa_challenge_post` | `AuthShell` | (pre-auth ticket cookie) | ✓ | needs-visual-adaptation |
| `/forgot-password` GET+POST | `/forgot-password` GET+POST | `render_forgot_password` | `handlers::forgot_password::forgot_password_get` / `_post` | `AuthShell` | none | ✓ | ready-to-integrate. **Guardrail:** anti-enumeration wording is non-negotiable. |
| `/forgot-password/sent` | (same `/forgot-password` page after POST; product flashes a neutral message) | `render_forgot_password_sent` | `handlers::forgot_password::forgot_password_post` (response branch) | `AuthShell` | none | n/a | needs-visual-adaptation. Mockup's separate `/sent` URL is not strictly necessary; the product's flash pattern is acceptable. RFC-MI-041 decides. |
| `/forgot-password/reset` GET+POST | `/reset-password` GET+POST | `render_reset_password` · `render_reset_password_invalid` | `handlers::forgot_password::reset_password_get` / `_post` | `AuthShell` | reset-token (URL param) | ✓ | needs-visual-adaptation. Route rename mockup→product: `/forgot-password/reset` → `/reset-password`. Keep product path. |
| `/forgot-password/reset/done` | (same `/reset-password` page after POST; "Sign in" link) | (response branch in `reset_password_post`) | `handlers::forgot_password::reset_password_post` | `AuthShell` | none | n/a | needs-visual-adaptation |

## OIDC consent flow

| Mockup screen | Product route | `render_*` | Handler module | Shell | Auth req. | CSRF on POST | Status |
|---|---|---|---|---|---|---|---|
| `/authorize` | `/oauth2/authorize` | — (302 redirect) | `handlers::oidc::authorize` | n/a | user | n/a | ready-to-integrate. Product enforces full OIDC parameter validation; mockup is a stub. |
| `/consent` GET+POST | `/oauth2/authorize` (re-enters with `consent_required=1`) | `render_consent` | `handlers::oidc::authorize` (consent branch) | `AuthShell` | user | ✓ | needs-visual-adaptation. RFC-MI-070 covers the UX delta (scope explanation, anti-phishing clarity, button hierarchy). **Guardrail:** approve / deny must not silently change protocol semantics (Authorization Code + PKCE only). |
| `/code-issued` (mockup dev convenience) | (real product redirects directly to client's `redirect_uri`) | n/a | n/a | n/a | n/a | n/a | **do-not-implement-yet** — mockup-only debug page; the product issues the code and redirects per RFC 6749. |

## Admin shell — read-only surfaces

| Mockup screen | Product route | `render_*` | Handler module | Shell | Auth req. | CSRF on POST | Status |
|---|---|---|---|---|---|---|---|
| `/admin` | `/admin` | `render_dashboard` | `handlers::admin::dashboard` | `Shell` | admin | n/a (read-only) | needs-visual-adaptation. RFC-MI-030 covers four stat cards, sparkline + recent events. |
| `/admin/users` | `/admin/users` | `render_users` | `handlers::admin::users_get` (GET) · `users_create` (POST) | `Shell` | admin | ✓ (create form) | needs-visual-adaptation. Search/filter is mockup-only; deferred unless RFC-MI-031 promotes it. |
| `/admin/users/{id}` | `/admin/users/{id}` | `render_user_detail` | `handlers::admin::users_detail_get` | `Shell` | admin | ✓ (danger-zone forms) | needs-visual-adaptation. Status badges + danger zone — see dangerous-action-map. |
| `/admin/clients` | `/admin/clients` | `render_clients` | `handlers::admin::clients_get` (GET) · `clients_create` (POST) | `Shell` | admin | ✓ | needs-visual-adaptation |
| `/admin/clients/{id}` | `/admin/clients/{id}/edit` | `render_client_edit` | `handlers::admin::clients_edit_get` / `_post` | `Shell` | admin | ✓ | needs-visual-adaptation. **Route delta:** mockup uses `/admin/clients/{id}` for detail; product uses `/admin/clients/{id}/edit`. Product path stays. |
| `/admin/security` (signing keys + session policy combined) | `/admin/signing-keys` | `render_signing_keys` | `handlers::admin::signing_keys_get` | `Shell` | admin | ✓ (rotate, delete) | requires-handler-change. Mockup combines signing-key lifecycle with session-policy hints; the product surfaces these separately (signing-keys page + `/admin/settings/security` tab). RFC-MI-031 must keep them separate. |
| `/admin/audit` | `/admin/audit` | `render_audit` | `handlers::admin::audit_get` | `Shell` | admin | n/a | needs-visual-adaptation. RFC-MI-031 covers table density and copy-id behaviour. |
| `/admin/audit` (Export NDJSON button) | `/admin/audit.csv` | — | `handlers::admin::audit_csv_get` | n/a | admin | n/a | **D-OPEN: NDJSON vs CSV.** Mockup proposes NDJSON; product ships CSV. Migration plan does not require this change. **do-not-implement-yet** unless RFC-MI-031 motivates it. |

## Admin shell — settings tabs

Mockup uses `/admin/settings?tab=…`; product uses path-based
`/admin/settings/{tab}`. Tab names differ — see
`tab-routing-delta.md` for the rename table.

| Mockup tab | Product route | `render_*` | Handler | Shell | Auth | CSRF | Status |
|---|---|---|---|---|---|---|---|
| `?tab=basic` | `/admin/settings/basic` | `render_settings_basic` | `handlers::settings::basic_get` · `basic_lang_post` | `Shell` | admin | ✓ | needs-visual-adaptation |
| `?tab=auth` | `/admin/settings/authentication` | `render_settings_authentication` | `handlers::settings::authentication_get` | `Shell` | admin | (read-only) | needs-visual-adaptation. **Name delta:** mockup `auth` → product `authentication`. |
| `?tab=security` | `/admin/settings/security` | `render_settings_security` | `handlers::settings::security_get` · `idle_timeout_post` · `max_sessions_post` | `Shell` | admin | ✓ | needs-visual-adaptation |
| `?tab=email` | `/admin/settings/email` | `render_settings_email` | `handlers::settings::email_get` · `email_post` · `email_test` | `Shell` | admin | ✓ | needs-visual-adaptation |
| `?tab=logs` | `/admin/settings/logs` | `render_settings_logs` | `handlers::settings::logs_get` | `Shell` | admin | (read-only) | needs-visual-adaptation |
| `?tab=other` | `/admin/settings/other` | `render_settings_other` | `handlers::settings::other_get` | `Shell` | admin | (read-only) | needs-visual-adaptation |

## Self-service security tabs

Mockup uses `/me/security?tab=…`; product uses path-based
`/me/security/{tab}`. See `tab-routing-delta.md`.

| Mockup tab | Product route | `render_*` | Handler | Shell | Auth | CSRF | Status |
|---|---|---|---|---|---|---|---|
| `?tab=overview` | `/me/security/overview` | `render_me_overview` | `handlers::me_security::overview_get` | `Shell` | user | n/a | needs-visual-adaptation |
| `?tab=password` | `/me/security/password` | `render_me_security` | `handlers::me_security::password_change_get` · `password_change_post` | `Shell` | user | ✓ | needs-visual-adaptation |
| `?tab=mfa` | `/me/security/mfa` | `render_me_mfa` | `handlers::me_security::mfa_get` · `mfa_enroll_start` · `mfa_enroll_confirm` · `mfa_disable` · `mfa_regenerate_recovery` | `Shell` | user | ✓ | requires-handler-change. Mockup uses `?tab=mfa&enroll=totp&step=N` for the enrollment wizard; product splits enrolment into POSTs from the same tab. RFC-MI-060 confirms the product pattern stays. |
| `?tab=passkey` | `/me/security/passkeys` | `render_me_passkey` | `handlers::me_security::passkeys_get` · `passkey_register_start` · `passkey_register_complete` · `passkey_delete` · `passkey_rename_post` | `Shell` | user | ✓ | needs-visual-adaptation. **Name delta:** mockup singular `passkey` → product plural `passkeys`. |
| `?tab=sessions` | `/me/security/sessions` | `render_me_sessions` | `handlers::me_security::sessions_tab_get` · `revoke_one` · `revoke_all_others` | `Shell` | user | ✓ | needs-visual-adaptation |
| `?tab=language` | `/me/security/language` | `render_me_language` | `handlers::me_security::language_get` · `language_post` | `Shell` | user | ✓ | needs-visual-adaptation |
| `?tab=recovery` (recovery-codes view) | (folded into `/me/security/mfa`) | (render_me_mfa branch) | `handlers::me_security::mfa_get` | `Shell` | user | n/a | **do-not-implement-yet** as separate route. Recovery view stays inside the MFA tab per RFC 056 + RFC 055. |
| `?tab=totp` (during enrolment) | (folded into `/me/security/mfa` POST flow) | — | (handler progression) | `Shell` | user | ✓ | already handled by product's mfa enrol POSTs. |

## Step-up + confirmation

The mockup uses a **generic** `/stepup?action=…&return_to=…` →
`/confirm/{token}` pattern for every destructive operation. The
product uses **per-operation** confirmation routes (RFC 030 +
RFC 058). See `dangerous-action-map.md` for the full mapping.

| Mockup screen | Product route(s) | `render_*` | Handler | Shell | Auth | CSRF | Status |
|---|---|---|---|---|---|---|---|
| `/stepup?action=X&return_to=Y` GET+POST (generic) | `/me/security/step-up` GET+POST (step-up ticket only) **+** the operation-specific `/admin/<resource>/{id}/<action>-confirm` GET routes | `render_step_up` · `render_confirm_*` | `handlers::step_up::get` / `post` and `handlers::admin::*_confirm_get` | `Shell` | user (step-up) / admin (confirm) | ✓ | requires-handler-change. **Non-negotiable guardrail (D-02 + RFC-MI-051):** the generic `/confirm/{token}` route is rejected. Each dangerous operation keeps its named confirm route. The mockup's `/stepup?action=…` link target style is rewritten by the integration to point at the product's named confirm GET. |
| `/confirm/{token}` GET+POST (generic) | (rejected — see above) | — | — | — | — | — | **do-not-implement-yet.** Per migration plan §D-02 and RFC-MI-051. |
| `/admin/stepup` (alias) | `/me/security/step-up` | `render_step_up` | `handlers::step_up::get` / `post` | `Shell` | user | ✓ | needs-visual-adaptation |

## System error pages

| Mockup screen | Product behaviour | `render_*` | Handler | Shell | Status |
|---|---|---|---|---|---|
| `/400` | The product does not render a `/400` body; 400 responses come from individual handlers as plain text or JSON. | n/a | n/a | n/a | **do-not-implement-yet.** Mockup's `/400` is a design-deck convenience; the product's per-handler 400 emission stays. |
| `/403` | (no dedicated route; `403` responses are inline) | `render_error` (`ErrorKind::Forbidden`) | (per-handler) | `AuthShell` | needs-visual-adaptation |
| `/404` (fallback) | (axum router fallback) | `render_error` (`ErrorKind::NotFound`) | (router fallback) | `AuthShell` | needs-visual-adaptation |
| `/410` (expired step-up ticket) | (the product redirects expired step-up back to the originating page with a flash) | n/a | `handlers::step_up::get` (expired branch) | n/a | **do-not-implement-yet as separate route.** Behaviour-equivalent UX is the flash on the originating page. |
| `/429` (rate-limited) | (rate-limit middleware returns 429 with `Retry-After`) | `render_error` (`ErrorKind::TooManyRequests`) | (middleware) | `AuthShell` | needs-visual-adaptation |
| `/500` | (per-handler 500) | `render_error` (`ErrorKind::InternalError`) | (per-handler / panic catcher) | `AuthShell` | needs-visual-adaptation |

## Theme + locale cookies (out-of-band)

| Mockup route | Product equivalent | Status |
|---|---|---|
| `/theme/{auto,light,dark}?return_to=…` | (product uses `localStorage` via `theme-init.js`; no server route — RFC-MI-012 is the decision point) | requires-handler-change **only if** RFC-MI-012 chooses Option B (cookie-backed). Default is to preserve the existing client-side model. |
| `/lang/{ja,en}?return_to=…` | (product uses `?lang=xx` setter inside the relevant page; `sui_id_lang` cookie set by handler) | needs-visual-adaptation. The product already supports the cookie-set path through page-level handlers; the standalone `/lang/<code>` route is unnecessary. |

## Aggregate

Out of **35 mockup routes**:

- **ready-to-integrate**: 4
- **needs-visual-adaptation**: 21
- **requires-handler-change**: 5
- **requires-backend-review**: 0 (no mockup intent crosses the
  web-crate boundary in a way the existing `sui-id-core` and
  `sui-id-store` contracts do not already cover)
- **do-not-implement-yet**: 5 (`/.well-known/openid-configuration`
  stub semantics, `/code-issued`, generic `/confirm/{token}`, `/400`,
  `/410`, mockup-specific `?tab=recovery`)

No route is classified as `requires-backend-review`. The integration
is, by construction, a web-layer migration — every protocol contract,
data contract, and audit contract is owned by `sui-id-core` /
`sui-id-store` / the OIDC engine and is preserved unchanged.

## Decisions surfaced

The following decisions are listed in the migration plan's "Open
Decision Backlog" but become concrete here:

| ID | Surface | Decision needed |
|---|---|---|
| **screen-D1** | Setup wizard | Mockup combines lang + HIBP in `/setup/security`; product keeps them as separate `/setup/lang` and `/setup/hibp` screens. RFC-MI-040 must resolve. **Default:** keep product split. |
| **screen-D2** | `/forgot-password/sent` | Mockup splits sent confirmation into its own route; product flashes a neutral message in place. RFC-MI-041 must resolve. **Default:** keep product flash. |
| **screen-D3** | Admin client detail | Mockup uses `/admin/clients/{id}`; product uses `/admin/clients/{id}/edit`. RFC-MI-031 must resolve. **Default:** keep product path. |
| **screen-D4** | Audit export | Mockup proposes NDJSON; product ships CSV. RFC-MI-031 must resolve. **Default:** keep product CSV. |
| **screen-D5** | Recovery codes view | Mockup `?tab=recovery`; product folds inside `/me/security/mfa`. RFC-MI-060 must resolve. **Default:** keep product fold. |

All five defaults preserve the product surface. The mockup's UX
intent is absorbed visually within the existing route shape.
