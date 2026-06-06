# RFC 065 — `pages.rs` split per screen domain

**Status.** Implemented (v0.47.0)
**Priority.** P0 — Phase F (v0.47.0)
**Tracks.** Project spec §8.3 — files over 500 LOC recommend
splitting. `pages.rs` is 4170 LOC, 8× the recommend ceiling. Phase F
landing is the gate for v1.0-rc.
**Touches.** `crates/sui-id-web/src/pages.rs` (deleted), new
`crates/sui-id-web/src/pages/` directory with 12 child modules, plus
`crates/sui-id-web/src/lib.rs` re-export list (unchanged in shape).

## Background

`pages.rs` accumulated 4170 lines through ten releases worth of
screen additions. It contains:

- 36 `pub fn render_*` functions (one per admin/auth/setup/etc. screen)
- 7 private helper functions (`flash_banner`, `fmt_time`, `render`,
  `copy_btn`, `kv_row`, `setup_step_indicator`, `url_encode`)
- 4 row-view helpers (`user_row_view`, `client_row_view`,
  `audit_row_view`, `signing_key_row_view`)
- 25 `pub struct *Data` types feeding the renderers
- `Flash` / `FlashKind` types
- `MeTab` enum + `me_security_tabs` helper
- `ReversibilityKind` enum + `ConfirmScreenData` + `confirm_screen`
- `EmptyStateData` + `EmptyStateAction` + `empty_state` +
  `table_empty_row`

Editing one screen requires scrolling past nine others. Adding a
screen risks colliding with a helper that turned `pub` two years ago
to be reused. Every existing CI grep that scopes by file path has
to special-case `pages.rs` because half the codebase lives there.

## Goal

Split `pages.rs` into per-screen child modules under
`crates/sui-id-web/src/pages/`, mirroring the screen architecture
in the PDF (setup / auth / self-service / admin / OIDC / error).
No file under `pages/` exceeds 500 non-comment lines. The public
API (`sui_id_web::render_*`, `sui_id_web::*Data`, etc.) is
unchanged — the `lib.rs` re-export list keeps existing callers
working without modification.

## Design

### Module layout

```
crates/sui-id-web/src/
├── lib.rs                       # re-exports unchanged
└── pages/
    ├── mod.rs                   # declares submodules + private uses
    ├── common.rs                # private helpers: flash_banner, fmt_time,
    │                            # render, copy_btn, kv_row, url_encode
    │                            # pub: Flash, FlashKind, EmptyStateData,
    │                            # EmptyStateAction, empty_state,
    │                            # table_empty_row
    ├── setup.rs                 # 5 render_setup_* + setup_step_indicator
    ├── auth.rs                  # render_login, render_mfa_challenge,
    │                            # render_mfa_setup, render_step_up,
    │                            # render_forgot_password*, render_reset_password*,
    │                            # render_password_change
    ├── dashboard.rs             # render_dashboard + DashboardData + sparkline
    │                            # rendering
    ├── users.rs                 # render_users + render_user_detail +
    │                            # user_row_view + UserDetail*Data
    ├── clients.rs               # render_clients + render_client_edit +
    │                            # client_row_view + ClientEditData
    ├── audit.rs                 # render_audit + audit_row_view
    ├── signing_keys.rs          # render_signing_keys + signing_key_row_view
    ├── settings.rs              # 6 render_settings_* + Settings*Data
    ├── confirm.rs               # 5 render_confirm_* + ReversibilityKind +
    │                            # ConfirmScreenData + confirm_screen +
    │                            # reversibility_badge
    ├── me_security.rs           # 6 render_me_* + MeTab + me_security_tabs
    ├── oidc.rs                  # render_consent + ConsentData
    └── error.rs                 # render_error + status_phrase
```

### Estimated post-split LOC

| File | Estimated LOC | Within spec? |
|------|--------------:|:---:|
| pages/mod.rs | 30 | ✅ |
| pages/common.rs | ~200 | ✅ |
| pages/setup.rs | ~400 | ✅ |
| pages/auth.rs | ~800 | ✅ (could split further if needed; the auth surface is dense) |
| pages/dashboard.rs | ~350 | ✅ |
| pages/users.rs | ~400 | ✅ |
| pages/clients.rs | ~440 | ✅ |
| pages/audit.rs | ~120 | ✅ |
| pages/signing_keys.rs | ~280 | ✅ |
| pages/settings.rs | ~970 | ⚠ exceeds 500 — secondary split needed |
| pages/confirm.rs | ~260 | ✅ |
| pages/me_security.rs | ~700 | ⚠ exceeds 500 — secondary split needed |
| pages/oidc.rs | ~60 | ✅ |
| pages/error.rs | ~115 | ✅ |

Two files (settings, me_security) exceed 500 LOC. Two options:
1. Split each into a sub-directory: `pages/settings/{basic,security,
   authentication,logs,email,other}.rs` and `pages/me_security/{
   overview,mfa,sessions,passkey,language,security}.rs`.
2. Accept these two as exceptions, document why in mod.rs comments.

Recommendation: option 1 — there is no good reason to make one
exception while denying another. The sub-directories cost a few
extra `mod` lines but pay off in editability.

### Helper module: `common.rs`

The challenge: private helpers like `flash_banner` and `copy_btn` are
called from every screen. Moving them into `common.rs` and making
them `pub(super)` keeps them callable from sibling modules without
exposing them to external crates.

Public types that *must* stay `pub` because handlers reference them
(Flash, FlashKind, EmptyStateData, EmptyStateAction) move to
`common.rs` as `pub`. The `lib.rs` re-export rewires them through
`pub use pages::common::{...}`.

### Migration approach

Copy-then-rewire, one screen at a time:

1. Create `pages/` directory.
2. Create `pages/common.rs` first; move helpers + Flash + empty-state
   primitives. Run `cargo check` — expect every `pages.rs`
   render_* to still build because the helpers still exist (via
   re-export in old pages.rs).
3. Add `pages/mod.rs` declaring submodules. Delete `pages.rs` and
   replace it with `pub mod pages;` in `lib.rs`. The mod.rs
   re-exports each submodule's public surface.
4. For each screen in [setup, oidc, error, audit, dashboard,
   signing_keys, users, clients, confirm, settings, me_security,
   auth] — in that order, smallest first:
   - Create the corresponding `pages/{screen}.rs` file with
     the `render_*` functions, supporting `*Data` types, and
     private helpers used only by that screen.
   - Add `use crate::pages::common::*;` at the top.
   - Update `pages/mod.rs` to re-export the screen's public
     surface.
   - Run `cargo check`. Fix any "function not found" errors by
     finding which sibling module needs the public marker.
5. Final pass: `cargo check --workspace --tests` + unit suite.

### What the lib.rs change looks like

`pub use pages::{...}` stays. The only difference is that the path
resolves to `pages::dashboard::render_dashboard` instead of
`pages::render_dashboard`, but Rust's `pub use` is transparent —
external callers see no change.

## Test plan

1. After each screen migration: `cargo check -p sui-id-web` PASS.
2. After full migration: `cargo check --workspace --tests` PASS.
3. Unit suite: `cargo test --workspace --lib` — **215/215 PASS**
   (unchanged from v0.46.0; no logic changed).
4. Manual: render each admin page (dashboard, users, clients,
   audit, signing keys, settings tabs, me_security tabs, setup,
   login, mfa challenge, mfa setup, forgot password, reset
   password, step-up, confirm screens, consent, error). Verify
   visual parity with v0.46.0.

## Rollout

Single release. Pure code-structural change. No user-visible
effect. No data migration. No public API surface change because
the `lib.rs` re-export list is preserved verbatim.

## Risks

- **Helper visibility drift**: a helper now `pub(super)` is callable
  from sibling modules but not from `lib.rs` re-export. If any
  helper turns out to be needed externally, surface it through
  `pub use`.
- **Build-broken intermediate state**: during migration, both
  `pages.rs` and `pages/mod.rs` momentarily exist. Solved by
  deleting `pages.rs` before adding `pages/mod.rs` and keeping
  the screen migrations small.
- **Stale grep references in CI**: existing CI greps that scope by
  `pages.rs` need to scope to `pages/**.rs` instead. Three CI jobs
  (text-leaks, css-tokens, semantic-palette-parity) need
  audit.

## Future work

- RFC 067 inline-style discipline pass — depends on this RFC because
  the inline styles are scattered across all screens; easier to
  sweep when each screen has its own file.
- A `pub(crate)` visibility audit: helpers that are only used by
  `pages/` should be `pub(super)`, not `pub`.
