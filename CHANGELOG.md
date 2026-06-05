# Changelog

All notable changes to sui-id will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.34.0] — Unreleased

**Minor version bump.** RFC 002 adds a new locale (zh), a new public API
(`Formatters`), a new migration (0024), and a new field on `OutgoingMail` —
all breaking additions.

### RFC 002 — i18n scope expansion

Implements sub-threads B, C, D, E, and A from the RFC umbrella.

#### Sub-thread A — Chinese Simplified locale (`zh`)

`Locale::Zh` is now a fully supported locale. `STRINGS_ZH` provides
complete translations across all ~260 string fields. `FORMATTERS_ZH`
provides date/time/count formatters consistent with Mainland Chinese
conventions. `Locale::ALL` now contains three variants; all exhaustive
match guards that already iterate `ALL` pick up `Zh` without any
per-site change.

`Locale::parse("zh")` and `negotiate_from_accept_language("zh, ...")` now
return `Some(Locale::Zh)` — previously unknown.

#### Sub-thread B — `Formatters` struct

New `sui_id_i18n::Formatters` struct alongside `Strings`:

```rust
pub struct Formatters {
    pub fmt_date:      fn(DateTime<Utc>) -> String,
    pub fmt_time:      fn(DateTime<Utc>) -> String,
    pub fmt_date_time: fn(DateTime<Utc>) -> String,
    pub fmt_relative:  fn(at: DateTime<Utc>, now: DateTime<Utc>) -> String,
    pub fmt_count:     fn(u64) -> String,
}
```

- `Locale::formatters()` returns the locale-specific instance.
- **Ja**: `%Y年%-m月%-d日` dates; relative "3 時間前".
- **En**: `%-d %b %Y` dates; relative "3 hours ago" (singular-aware).
- **Zh**: `%Y年%m月%d日` dates; relative "3 小时前".
- `fmt_count` groups with commas (1,234,567) for all locales.

No ICU dependency. All formatter functions are plain `fn` pointers
(`&'static` compatible).

7 unit tests in `crates/sui-id-i18n/src/formatters.rs`.

#### Sub-thread C — Per-recipient locale for outbound mail

- **Migration 0024** adds a nullable `locale TEXT` column to
  `email_outbox`. The worker stores the BCP-47 tag resolved from the
  recipient's `preferred_lang` at enqueue time.
- `OutgoingMail` gains an `pub locale: Option<Locale>` field (defaults
  to `None` at all existing call sites).
- `OutboxMailSender::send` serialises the locale tag into the outbox row.

The worker now renders mail in the recipient's own language rather than
the requesting user's. Resolution order: recipient's `preferred_lang`
→ server default → Ja.

#### Sub-thread D — Audit event labels

30 new fields added to `Strings`, grouped under `// Audit event labels`:
`audit_event_auth_login_success`, `audit_event_user_create`, etc.
One additional field: `settings_tab_advanced` (RFC 023 renamed the
settings "Other" tab to "Advanced"; the i18n key was previously missing
in the typed `Strings` struct).

All three locales (Ja, En, Zh) have complete translations.

#### Sub-thread E — `Locale::direction()` + HTML `dir=` attribute

- `Locale::direction()` returns `"ltr"` or `"rtl"` (all current locales
  return `"ltr"`; RTL locales will override when added).
- `Shell` in `layout.rs` now sets `<html dir={direction}>` alongside
  `<html lang={tag}>`. No visual change for LTR locales; correct foundation
  for Arabic/Hebrew/Persian when they land.

### Test results

- `sui-id-i18n`: **12 tests pass** (7 formatter + 5 existing)
- `sui-id-store`: **33 tests pass**
- `sui-id-core`: **114 tests pass**
- `cargo check --workspace`: clean
- `cargo check -p sui-id --tests`: clean

---

## [0.33.0] — Previous release

**Minor version bump.** RFC 001 introduces a new DB migration (0023) and a
new in-process background worker, both of which affect the startup sequence.

### RFC 001 — Persistent email outbox + retry worker

Outgoing mail is no longer sent inline with the HTTP request that triggered
it. Instead, requests enqueue a row in the new `email_outbox` table and
return immediately; the `OutboxWorker` background task drains the queue
with exponential backoff.

#### What changed for operators

- **Reduced handler latency.** `/forgot-password` and password-change
  notifications no longer block on SMTP. The response returns immediately
  regardless of SMTP availability.
- **Automatic retry.** Failed deliveries are retried up to 5 times on the
  schedule: 30 s → 2 m → 10 m → 1 h → 6 h. After 5 attempts the row is
  marked `failed` and a `mail.outbox.permanent_failure` audit event is
  written.
- **Restart safety.** Any row in `sending` state when the process exits is
  reset to `queued` on the next startup by `requeue_stuck_sending`.
- **Encryption unchanged.** `recipient_enc` and `payload_enc` are sealed
  under the master key with dedicated AADs; both columns are added to the
  `admin rotate-key` reseal harness.

#### Schema

Migration **0023** adds:

```
email_outbox (id, state, template, recipient_enc, payload_enc,
              attempt_count, next_attempt_at, last_error,
              created_at, updated_at)
```

Partial index on `(next_attempt_at) WHERE state = 'queued'` for fast
scheduler polls.

#### New types and APIs (all in `sui-id-core` / `sui-id-store`)

- `sui_id_shared::ids::EmailOutboxId`
- `sui_id_store::models::{EmailOutboxState, EmailOutboxRow}`
- `sui_id_store::StoreError::InvalidData`
- `sui_id_store::repos::email_outbox::{enqueue, claim_one_eligible,
  mark_sent, record_failure, mark_permanently_failed,
  requeue_stuck_sending, reseal_all}`
- `sui_id_core::mail::outbox::{OutboxMailSender, OutboxWorker}`

#### Dev mode unchanged

`test_app()` / `test_app_with_mailer()` still use `InMemoryMailSender`
directly. The outbox path is production-only; tests observe mail via the
in-memory sender as before.

#### Tests

5 new unit tests in `sui-id-store`: `enqueue_and_claim_round_trip`,
`claim_respects_next_attempt_at`, `mark_sent_after_claim`,
`record_failure_increments_attempt_count`,
`requeue_stuck_sending_resets_old_rows`.

### Test results

- `sui-id-store`: **33 tests pass** (28 previous + 5 email_outbox)
- `sui-id-core`: **114 tests pass**
- `cargo check --workspace`: clean
- `cargo check -p sui-id --tests`: clean

---

## [0.32.0] — Previous release

### RFC 017 — UI/UX design contracts

Adds [`docs/ui-ux-contracts.md`](docs/ui-ux-contracts.md), the frozen
cross-cutting contract for the admin domain UI. Sections:

- **§ 1** Screen relation map (five-stream isolation)
- **§ 2** Screen responsibilities matrix
- **§ 3** Dangerous-operation UI pattern (step-up + explicit-verb confirm)
- **§ 4** State copy contract (loading / empty / success / error / disabled)
- **§ 5** Admin dashboard information policy
- **§ 6** Settings tab structure (six fixed tabs; Advanced tab isolates risky knobs)
- **§ 7** Client management UI constraints
- **§ 8** Audit log display rules
- **§ 9** Dev mode UI separation
- **§ 10** Accessibility implementation contract (focus ring, ARIA, keyboard)
- **§ 11** Text selection contrast (WCAG 2.1 SC 1.4.3 requirement)

Implementation RFCs (002, 003, 008, 010–012, 016, 023) reference this document
as their inherited contract. No code change.

### RFC 023 — Visual design system

Completes the CSS token and component system shipped to the binary in
`sui-id-web`. All changes are in `tokens.rs` and `components.rs`.

**tokens.rs additions:**

- **Motion tokens** — `--motion-instant/fast/base/slow` and
  `--motion-easing`. Components reference these for `transition-duration`;
  the `prefers-reduced-motion` override block zeros them automatically so
  no per-component duplication is needed.
- **Z-index scale** — `--z-below / --z-base / --z-raised / --z-overlay /
  --z-dropdown / --z-modal / --z-toast`. Named layers prevent magic numbers.
- **`@media (prefers-reduced-motion: reduce)`** block — zeros all motion
  tokens and applies `animation-duration: 0.01ms` globally.
- **`::selection` styles** — moved from components.rs to tokens.rs and
  explicitly meeting WCAG 2.1 SC 1.4.3 contrast requirements in both
  modes (light: ~13:1, dark: ~7:1).

**components.rs additions:**

- **Tab component** (`.tabs`, `.tabs__bar`, `.tab-btn`) — horizontal tab
  bar with motion-token transitions for Settings and similar multi-panel
  screens. `aria-selected="true"` drives the active indicator.
- **Dev-mode banner** (`.dev-banner`) — yellow ribbon displayed on every
  page when `--dev` is active, with `.dev-banner__bind-warn` for the
  non-loopback warning (RFC 017 § 9).
- **Motion-aware transitions** — `button`, `input`, `a` and related elements
  now reference `var(--motion-fast)` instead of hardcoded durations.
- **Reversibility badge** (`.reversibility-badge--recoverable` /
  `--permanent`) — coloured badge for dangerous-operation confirm screens
  (RFC 017 § 3). Colour is never the sole signal; badge text "Recoverable"
  / "Not recoverable" is always present.

### RFC 024 — Documentation consolidation

- **`CHANGELOG.md`** — now a thin index of current-release notes plus links
  to `docs/changelog/v0.30.md` (0.30.x history) and
  `docs/changelog/archive.md` (0.29.x and earlier). Reduces the root file
  from 5,304 lines to ~90.
- **`ROADMAP.md`** — compressed from 639 lines to 64 lines: an RFC index
  table, a near-term priority statement, a "completed" table, and a
  constraints section. Stale detail moved into the completed-RFC files.

---

## [0.31.0] — Previous release

**Minor version bump.** RFC 014 (hot-path caches) introduces a new cache
subsystem and changes the `AppState` constructor — both are breaking API
additions. RFC 028 (copy buttons, v0.30.1) ships in the same release.

### RFC 028 — Copy-to-clipboard for credential values (v0.30.1 → rolled in)

Adds `📋 Copy` buttons next to Client ID, client secret, User UUID, and
JWKS URI. The `clipboard-available` CSS class is set by a small inline JS
snippet when `navigator.clipboard` is present; buttons are hidden without
it (non-HTTPS contexts degrade cleanly).

### RFC 014 — Hot-path caches

Two request-critical DB reads are now served from in-process caches:

#### Cache 1 — Redirect-origin set (`RedirectOriginsCache`)

`/oauth2/token` CORS pre-flight previously queried every registered client
on every request to build the allowed-origins set. The cache is now
rebuilt once at startup and after every client mutation (create / update /
disable / delete). CORS checks call `caches.redirect_origins.contains(origin).await`
— a single `RwLock::read` instead of a DB round-trip.

#### Cache 2 — Active signing keys (`JwksCache`)

`verify_access_token` and `verify_id_token` previously loaded the
published-keys list from the DB on every call. The cache is rebuilt once
at startup and after every signing-key rotation or deletion. Hot paths
call `verify_access_token_cached` / `verify_id_token_cached`, which take
a snapshot of the key list from the cache.

#### Cache design

- Both caches are `tokio::sync::RwLock<T>` snapshots stored as `Arc<Caches>`
  in `AppState`.
- Writes hold the lock only during the in-memory update (microseconds).
- Rebuild on mutation is synchronous with the write: if the rebuild fails,
  the mutation still returns success but the cache keeps the previous
  snapshot and a `warn!` log is emitted.
- Cold start: caches are pre-populated during `startup::prepare()`. A
  startup rebuild failure yields an empty cache and a warn log; the next
  successful mutation re-syncs.

#### New public API

- `sui_id_core::cache::Caches` — combined cache handle, stored in `AppState`.
- `sui_id_core::cache::RedirectOriginsCache::contains(&self, origin) -> bool` (async)
- `sui_id_core::cache::JwksCache::snapshot(&self) -> Vec<CachedSigningKey>` (async)
- `tokens::verify_access_token_cached(caches, clock, token)` — hot-path variant.
- `tokens::verify_id_token_cached(caches, clock, token, accept_expired)` — hot-path variant.
- `signing_keys::list_active(db)` — new repo function (active keys only).

#### Breaking: `AppState::new` gains a `caches: Arc<Caches>` parameter

All construction sites (startup, tests, dev-mode, CLI sub-commands) updated.

#### Cache invalidation hooks

`admin::{create_client, update_client, update_client_basic, set_client_disabled,
delete_client}` all rebuild `redirect_origins` on success.
`admin::{rotate_signing_key, delete_signing_key}` rebuild `jwks` on success.
All accept `caches: &Caches` as a new final parameter.

#### Test updates

- 3 new unit tests in `cache.rs` (origin extraction, contains, snapshot).
- E2E tests updated throughout: `AppState::new` call sites, async helper
  functions, `db.with_conn` missing `.await`, mailer async methods,
  `move` closures for captured `user.id` / `stale`.

### Test results

- `sui-id-store`: 28 tests pass
- `sui-id-core`: 114 tests pass (111 previous + 3 cache tests)
- `cargo check --workspace`: clean
- `cargo check -p sui-id --tests`: clean (e2e test compilation)

---

---

## Older releases

| Version series | File |
|---|---|
| 0.30.x | [docs/changelog/v0.30.md](docs/changelog/v0.30.md) |
| 0.29.x and earlier | [docs/changelog/archive.md](docs/changelog/archive.md) |
