# No-JS Operation Matrix (RFC-MI-080 · v0.57.0)

All core identity flows must work with JavaScript disabled.
Status: ✅ works · ⚠️ degraded (gracefully) · ❌ blocked

## Principles

- All forms use `method="post"` — no JS required for submission.
- CSRF tokens are server-rendered in every form (since RFC-MI-021 v0.51.0).
- Theme toggle (localStorage) degrades to `prefers-color-scheme` — acceptable.
- Copy-to-clipboard buttons (`copy.js`) degrade silently — content still visible.
- WebAuthn (passkey) requires JS by design — noted but not a blocker for
  password-based flows.

## Core flows

| Flow | No-JS result | Notes |
|---|---|---|
| Admin login (password) | ✅ | Standard POST form |
| Admin login (TOTP challenge) | ✅ | Standard POST form |
| Admin sign out | ✅ | Server-rendered `_csrf` in form since v0.51.0; `logout-csrf.js` removed |
| Forgot password request | ✅ | Standard POST form |
| Password reset | ✅ | Standard POST form |
| Step-up (password) | ✅ | Standard POST form |
| Setup wizard (all steps) | ✅ | Standard POST forms |
| User list, detail (read-only) | ✅ | Pure GET, SSR |
| Client list, edit form | ✅ | Standard POST form |
| Settings (all tabs) | ✅ | Standard POST forms |
| Audit log (filter) | ✅ | Standard GET form |
| Change password | ✅ | Standard POST form |
| MFA enrolment (TOTP) | ✅ | QR + secret both rendered server-side |
| Session revocation | ✅ | Standard POST form |
| Language preference | ✅ | Standard POST form |
| Dangerous confirmations | ✅ | Standard POST forms with CSRF |
| OIDC consent (approve/deny) | ✅ | Standard POST forms with CSRF |
| OIDC authorization redirect | ✅ | Browser redirect, no JS |

## Degraded-gracefully (non-blocking)

| Feature | Without JS | Impact |
|---|---|---|
| Theme toggle | Respects `prefers-color-scheme`; choice not persisted | Low — sensible default |
| Copy buttons | Button renders; clipboard copy silently fails | Low — content still readable/selectable |
| Passkey registration | ❌ blocked | WebAuthn requires JS; password flows remain available |
| Passkey authentication | ❌ blocked | As above |

---

*Generated: RFC-MI-080 v0.57.0.*
