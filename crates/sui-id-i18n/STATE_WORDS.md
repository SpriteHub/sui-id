# sui-id-i18n — state word conventions

This file is the translator and i18n-maintainer reference for the five
UI states defined in `docs/src/contributing/state-contract.md`.

## Key naming conventions

| State | Key suffix / prefix | Example |
|---|---|---|
| empty | `{section}_empty` | `profile_passkeys_empty` |
| error (flash) | `{action}_failed_flash` | `password_change_failed_flash` |
| error (inline) | `{field}_invalid` | `username_invalid` |
| error (page) | `error_{status}_{title\|lede}` | `error_not_found_title` |
| success | `{action}_{verb}ed_flash` | `user_created_flash` |
| loading | `loading` or `webauthn_in_progress` | |
| disabled hint | `{feature}_disabled_hint` | `smtp_test_disabled_hint` |

## Translation principles

- **empty**: state what is absent, suggest the next action if possible.
- **error**: be short and neutral. Never expose internal errors. Include
  "contact administrator" only for 5xx errors.
- **success**: past tense of the action. One sentence.
- **loading**: present progressive. Short.
- **disabled hint**: one sentence. Active voice. "Becomes available
  after X is configured."

## Three-locale coverage requirement

Every key added to `strings.rs` must appear in `ja.rs`, `en.rs`, and
`zh.rs`. The Rust compiler enforces exhaustiveness via the struct
literal syntax — missing fields cause a compile error.

## Technical identifiers

Do not translate proper nouns: `TOTP`, `WebAuthn`, `PKCE`, `JWKS`,
`Ed25519`, `Argon2id`, `HIBP`, `OIDC`, `SMTP`, `STARTTLS`. These are
stable identifiers used in documentation and RFCs; preserving them
helps operators look them up.

## Reference locale: Japanese

The canonical reference locale is `ja.rs`. When adding a new key, write
the Japanese translation first, then derive English and Chinese. This
order keeps the product's primary audience at the centre of every copy
decision.
