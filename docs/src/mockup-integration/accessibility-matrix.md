# Accessibility Matrix (RFC-MI-080 · v0.57.0)

Covers all screen groups migrated in the mockup integration arc.
Status: ✅ pass · ⚠️ note · ❌ blocker

## Key: ABDD checks per screen

| Check | Meaning |
|---|---|
| Landmark | Correct `<header role="banner">`, `<main id="main-content">`, `<footer role="contentinfo">` |
| Skip link | `<a class="skip-link" href="#main-content">` is first focusable element |
| Headings | Heading hierarchy is logical (no skipped levels) |
| Labels | Every input has a `<label>` or `aria-label` |
| Errors | Errors use `role="alert"` (FlashKind::Error); status uses `role="status"` |
| Non-color | Status is conveyed by text/shape/weight, not colour alone |
| Focus | Visible `:focus-visible` ring on all interactive elements |
| Live | Dynamic content uses appropriate `aria-live` or `role="alert"` |

---

## Setup

| Screen | Landmark | Skip link | Headings | Labels | Errors | Non-color | Focus | Notes |
|---|---|---|---|---|---|---|---|---|
| `/setup` (welcome) | ✅ | ✅ v0.57.0 | ✅ | ✅ | ✅ | ✅ step indicator uses badge+text | ✅ | Step indicator: badge number + text label (not colour-only) |
| `/setup/admin` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| `/setup/lang` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ lang picker uses border+text | ✅ | Active lang: `border-color` + `color` + text label |
| `/setup/hibp` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| `/setup/done` | ✅ | ✅ | ✅ | n/a | n/a | n/a | ✅ | |

## Login / Auth / MFA / Reset / Step-up

| Screen | Landmark | Skip link | Headings | Labels | Errors | Non-color | Focus | Notes |
|---|---|---|---|---|---|---|---|---|
| `/admin/login` | ✅ | ✅ | ✅ | ✅ for=id | ✅ `role="alert"` | ✅ text error message | ✅ | Anti-enumeration wording unchanged |
| `/admin/login` (MFA challenge) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| `/me/security/mfa` (TOTP setup) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ QR + secret text | ✅ | QR uses `role="img" aria-label` |
| `/forgot-password` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Generic response (anti-enumeration) |
| `/reset-password` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Token-expiry handled gracefully |
| `/me/step-up` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Purpose text visible to user |

## Dashboard

| Screen | Landmark | Skip link | Headings | Labels | Errors | Non-color | Focus | Notes |
|---|---|---|---|---|---|---|---|---|
| `/admin` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ warning uses callout border+text | ✅ | Warning: `.callout--warning` uses border + text (v0.52.0) |

## Users / Clients / Settings / Audit

| Screen | Landmark | Skip link | Headings | Labels | Errors | Non-color | Focus | Notes |
|---|---|---|---|---|---|---|---|---|
| `/admin/users` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| `/admin/users/{id}` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ danger-zone uses border+text | ✅ | Danger zone: `⚠ Danger Zone` text label (v0.54.0) |
| `/admin/clients` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| `/admin/clients/{id}/edit` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| `/admin/settings/*` | ✅ | ✅ | ✅ `route-tabs` aria-current | ✅ | ✅ | ✅ | ✅ | |
| `/admin/audit` | ✅ | ✅ | ✅ | ✅ filter label | ✅ | ✅ chain-ok uses badge+text | ✅ | Filter input has `<label for="audit-q">` |
| `/admin/signing-keys` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ | |

## Self-service security

| Screen | Landmark | Skip link | Headings | Labels | Errors | Non-color | Focus | Notes |
|---|---|---|---|---|---|---|---|---|
| `/me/security/overview` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ status uses badge+text | ✅ | |
| `/me/security/password` | ✅ | ✅ v0.55.0 | ✅ | ✅ | ✅ | ✅ | ✅ | Tab strip added v0.55.0 |
| `/me/security/mfa` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ TOTP count = text | ✅ | Recovery codes use text count; `.badge--danger` is supplementary |
| `/me/security/passkeys` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| `/me/security/sessions` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ scope label text | ✅ | "This device" / revoke-all-others labels identify scope |
| `/me/security/language` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |

## OIDC Consent

| Screen | Landmark | Skip link | Headings | Labels | Errors | Non-color | Focus | Notes |
|---|---|---|---|---|---|---|---|---|
| `/oauth2/consent` | ✅ | ✅ | ✅ | n/a | n/a | ✅ scope text descriptions | ✅ | Both Approve/Deny are `<button>` — keyboard reachable (v0.56.0) |

## Confirmations

| Screen | Landmark | Skip link | Headings | Labels | Errors | Non-color | Focus | Notes |
|---|---|---|---|---|---|---|---|---|
| `/admin/users/{id}/delete-confirm` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ reversibility badge | ✅ | `.reversibility-badge--permanent` uses text + colour |
| `/admin/users/{id}/disable-confirm` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ | |
| `/admin/users/{id}/mfa-reset-confirm` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ | |
| `/admin/clients/{id}/delete-confirm` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ | |
| `/admin/signing-keys/{id}/delete-confirm` | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ | |

## Error pages

| Screen | Landmark | Skip link | Headings | Labels | Errors | Non-color | Focus | Notes |
|---|---|---|---|---|---|---|---|---|
| 404 Not Found | ✅ | ✅ | ✅ | n/a | n/a | n/a | ✅ | Back link always present |
| 403 Forbidden | ✅ | ✅ | ✅ | n/a | n/a | n/a | ✅ | Context-aware back link (401 → login) |
| 500 Internal Error | ✅ | ✅ | ✅ | n/a | n/a | n/a | ✅ | |

---

*Generated: RFC-MI-080 v0.57.0. Review should be repeated after any UI change.*
