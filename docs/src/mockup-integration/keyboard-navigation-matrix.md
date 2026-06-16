# Keyboard Navigation Matrix (RFC-MI-080 · v0.57.0)

Status: ✅ reachable · ⚠️ reachable with effort · ❌ unreachable

## Key controls expected

| Key | Expected action |
|---|---|
| Tab / Shift+Tab | Move between focusable elements |
| Enter / Space | Activate buttons, links, checkboxes |
| Skip link (Tab on page load) | Jump to `#main-content` (added v0.57.0) |

## Screen groups

| Screen group | Skip link | Nav reachable | All forms | All buttons | Route tabs | Notes |
|---|---|---|---|---|---|---|
| Setup wizard | ✅ | n/a | ✅ | ✅ | ✅ step indicator | Step indicator uses `<nav>` + `<a>` links |
| Login / MFA / step-up | ✅ | n/a | ✅ | ✅ | n/a | Auth flows use no nav |
| Forgot / reset password | ✅ | n/a | ✅ | ✅ | n/a | |
| Dashboard | ✅ | ✅ | n/a | ✅ metric links | n/a | Period-range tabs reachable via `<a>` links |
| Users list | ✅ | ✅ | n/a | ✅ | n/a | |
| User detail | ✅ | ✅ | n/a | ✅ danger-zone | n/a | Danger-zone buttons in dedicated section |
| Client list | ✅ | ✅ | n/a | ✅ | n/a | |
| Client edit | ✅ | ✅ | ✅ | ✅ save/cancel/delete | n/a | |
| Settings (all tabs) | ✅ | ✅ | ✅ | ✅ | ✅ route tabs | |
| Audit log | ✅ | ✅ | ✅ filter | ✅ | n/a | |
| Signing keys | ✅ | ✅ | n/a | ✅ | n/a | |
| Self-service security overview | ✅ | ✅ | n/a | n/a | ✅ route tabs | |
| Change password | ✅ | ✅ | ✅ | ✅ | ✅ route tabs | Tab strip added v0.55.0 |
| MFA tab | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Passkeys tab | ✅ | ✅ | n/a | ✅ (WebAuthn requires JS) | ✅ | |
| Sessions tab | ✅ | ✅ | ✅ | ✅ revoke | ✅ | |
| Language tab | ✅ | ✅ | ✅ | ✅ | ✅ | |
| OIDC consent | ✅ | n/a | ✅ | ✅ approve+deny | n/a | Both actions are `<button>` elements |
| Dangerous confirmations | ✅ | ✅ | ✅ | ✅ | n/a | All confirm/cancel are buttons in forms |
| Error pages | ✅ | ✅ | n/a | ✅ back link | n/a | |

---

*Generated: RFC-MI-080 v0.57.0.*
