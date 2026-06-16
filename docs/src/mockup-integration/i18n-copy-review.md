# i18n Copy Review (RFC-MI-080 · v0.57.0)

Verifies that all user-facing strings in the migrated screens are
localised and not leaking raw key names.

## Method

- `text-leaks` CI invariant: zero occurrences of `>t.some_key<` in
  compiled output. **Passing at v0.57.0: 0 leaks.**
- Manual survey: checked each screen group's render functions for
  bare string literals in `view!` macros.
- All three locale files (en, ja, zh) compiled successfully.

## Key additions during the MI arc

| RFC | Keys added | Locales |
|---|---|---|
| RFC-MI-000 (Phase 0) | ~58 net-new keys for mockup delta | en, ja, zh |
| RFC-MI-022 (Phase 2) | `me_tab_password` | en, ja, zh |
| RFC-MI-050/051 (Phase 5) | `danger_zone_title` | en, ja, zh |
| RFC-MI-070 (Phase 7) | `consent_scope_{openid,profile,email,offline_access}_desc` (×4) | en, ja, zh |
| RFC-MI-080 (Phase 8) | `a11y_skip_to_main` | en, ja, zh |

## Screen-by-screen bare-literal audit

No screen group uses bare string literals for user-visible text.
All visible copy routes through `t.some_key` lookups.

Exception: error-page fallback strings (e.g. "Internal Server Error")
are known English-only bare literals. These are in the error-handler
layer and are not part of the mockup-integration scope.

## Copy consistency

- "Danger Zone" → `t.danger_zone_title` (consistent across user and client pages)
- "Verify your identity" scope label → `t.consent_scope_openid` (description separate)
- Tab labels: `t.me_tab_{overview,password,mfa,passkey,sessions,language}` — all six present
- Settings tab labels: `t.settings_tab_{basic,security,authentication,logs,email,advanced}` — all six present

## Open items

None. All copy from migrated screens is localised.

---

*Generated: RFC-MI-080 v0.57.0.*
