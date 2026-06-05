# RFC 051 — Per-screen i18n completeness audit

**Status.** Proposed
**Priority.** P0 — primary work of Phase B (v0.43.0)
**Tracks.** UI/UX correctness baseline; closes the
"未完了 : admin dashboard / users / clients / signing keys /
audit log / settings Auth / Logs / Email / Other / 404 /
500 / rate-limited / mail templates" list from the
`a11y + i18n v0.29.x` slide.
**Touches.** `crates/sui-id-web/src/pages.rs`,
`crates/sui-id-i18n/src/strings.rs`,
`crates/sui-id-i18n/src/{ja,en,zh}.rs`,
`.github/workflows/ci.yml`.

## Summary

After RFC 050 lands, the application chrome is locale-aware
but the page bodies are not. `pages.rs` contains dozens of
hardcoded Japanese strings (form labels, hints, button
captions, section headings) and hardcoded English strings
(table headers, status text, empty-state placeholders).
This RFC inventories every leak per render function and
replaces each with a typed `Strings` field. The companion
CI guard added here catches new leaks on PR.

## Background

A pass over `pages.rs` at v0.41.0 shows the leakage shape
varies by page domain:

- **Setup wizard** (`render_setup_*`) — almost fully i18n'd.
  Authored first; the discipline was in place.
- **Auth screens** (login, MFA challenge, forgot/reset) —
  mostly i18n'd, scattered residual literals.
- **Dashboard** — mixed JA/EN literals, ~10 sites.
- **Users / Clients / Client edit** — heavy JA literals on
  forms, EN literals on table headers and status badges.
- **Settings (5 tabs)** — partial; section headings and
  field hints often hardcoded.
- **Self-service `/me/security/*`** — mid-coverage; tab
  labels and empty-state copy often hardcoded.
- **Confirm screens** (`render_confirm_*`) — mostly i18n'd
  via the existing `confirm_*` keys.
- **Consent screen** (`render_consent`, RFC 038) — well
  i18n'd.
- **Error pages** (`render_error`, RFC 042) — well i18n'd.

The total count is hard to enumerate without grep; this RFC
captures it precisely in the audit step below.

The state contract in
`docs/src/contributing/state-contract.md` and
`crates/sui-id-i18n/STATE_WORDS.md` (RFC 044) already
defines the vocabulary; what's missing is enforcement and
the fields themselves.

## Goals

1. Every visible text in every `render_*` function flows
   through `Locale::strings()`. No literal Japanese
   characters. No literal English noun-phrases.
2. The key vocabulary respects the existing state-words
   contract (RFC 044) — empty / error / success / loading /
   disabled prefixes follow the documented convention.
3. CI fails on PRs that introduce new hardcoded text into
   `pages.rs`.

## Detailed design

### Part A — the audit pass

For each render function in `pages.rs`, walk it line by line
and identify every literal text node. The work order matches
the gap-analysis severity:

| Render function family       | Sites (rough est.) | Notes                                |
|-----------------------------|-------------------:|--------------------------------------|
| `render_dashboard`          | ~12                | EN/JA mixed; stat labels, endpoints  |
| `render_clients`            | ~25                | full create-form hardcoded JA        |
| `render_client_edit`        | ~15                | section headings + form hints        |
| `render_users`              | ~10                | create-form labels, table th         |
| `render_user_detail`        | ~12                | sections + status text               |
| `render_audit`              | ~8                 | filters + table th                   |
| `render_signing_keys`       | ~10                | rotate section + status              |
| `render_settings_*` (×5)    | ~30 total          | section headings + per-field hints   |
| `render_me_*` (×5)          | ~20 total          | tab content + empty states           |
| `render_login`              | ~3                 | residual links/captions              |
| `render_mfa_challenge`      | ~4                 | option labels                        |
| `render_step_up`            | ~3                 | option labels                        |
| `render_forgot_password*`   | ~2                 | residual hints                       |
| `render_reset_password*`    | ~3                 | residual hints                       |
| `render_consent`            | 0                  | clean                                |
| `render_error`              | 0                  | clean                                |
| `render_confirm_*` (×5)     | ~5                 | residual button labels               |

Each newly required key is added to `strings.rs` and
supplied in `ja.rs`, `en.rs`, `zh.rs`. Key naming follows
the contract in `STATE_WORDS.md`:

- Section headings: `{page}_section_{name}` (e.g.
  `clients_section_create`, `dashboard_section_activity`).
- Form labels: `{page}_label_{field}` (e.g.
  `clients_label_app_name`, `clients_label_redirect_uris`).
- Form hints: `{page}_hint_{field}` (e.g.
  `clients_hint_redirect_uris`).
- Buttons: `{page}_button_{verb}` (e.g.
  `clients_button_register`).
- Captions / inline text: `{page}_caption_{what}`.
- Empty states: `{section}_empty` (state-contract rule).
- Success flashes: `{action}_{verb}ed_flash` (state-
  contract rule).

Existing keys are re-used wherever possible (per the
HANDOFF v0.41.0 § 5.8 "既存キー再利用" principle). A new
key is added only when no existing key carries the right
sense.

### Part B — refactor outline

Each render function is reorganised so the `t = lang.strings()`
binding sits at the top and every text node references `t.*`.
This is what most render functions already do partially; the
work is finishing the pass.

Pattern (using `render_clients` as the demonstrator):

```rust
pub fn render_clients(
    clients: Vec<ClientSummary>,
    flash: Option<Flash>,
    new_secret: Option<(String, String)>,
    csrf_token: String,
    dev_mode: bool,
    lang: sui_id_i18n::Locale,
) -> String {
    render(move || {
        let t = lang.strings();
        let n = clients.len();
        view! {
            <Shell title=t.clients_title.to_string() show_nav=true
                   current=Some("clients".to_string())
                   dev_mode=dev_mode lang=lang>
                <header class="page-header">
                    <h1 class="page-header__title">{t.clients_title}</h1>
                    <p class="page-header__lede">
                        { t.clients_lede }
                        " "
                        { (t.clients_count_caption)(n) }   // see Part C
                    </p>
                </header>
                {flash_banner(flash)}
                {new_secret.map(|(cid, sec)| view! {
                    <div class="flash warn" role="status">
                        <strong>{t.clients_secret_once_banner}</strong>
                        …
                    </div>
                })}
                <section>
                    <h2>{t.clients_section_create}</h2>
                    <div class="card">
                        <form method="post" action="/admin/clients" class="stack">
                            <input type="hidden" name="_csrf" value=csrf_token.clone()/>
                            <div class="field">
                                <label class="field__label" for="c-name">
                                    {t.clients_label_app_name}
                                </label>
                                <input id="c-name" name="name" type="text" required=true/>
                            </div>
                            <div class="field">
                                <label class="field__label" for="c-uris">
                                    {t.clients_label_redirect_uris}
                                </label>
                                <textarea id="c-uris" name="redirect_uris" required=true rows="3"/>
                                <span class="field__hint">{t.clients_hint_redirect_uris}</span>
                            </div>
                            …
                            <button type="submit">{t.clients_button_register}</button>
                        </form>
                    </div>
                </section>
                …
            </Shell>
        }
    })
}
```

The render function gains no parameters and changes no
public API; the entire change is internal.

### Part C — interpolated text

A handful of leaking strings are not constants but
formatted phrases:

```rust
format!(" 現在 {client_count} 件。")
format!("Hello, {}. {}", admin_username, t.dashboard_lede)
```

The project already uses a "method on `Strings`" pattern
for these (see `strings.rs` doc comment lines 14–18:
*"When a string has variable interpolation, expose it as a
method below the struct"*). RFC 051 follows that pattern:

```rust
// strings.rs
impl Strings {
    pub fn clients_count_caption(&self, n: usize) -> String {
        // Each locale's STRINGS_* defines a fmt template field,
        // and this method formats it with the count.
        self.clients_count_template.replace("{n}", &n.to_string())
    }

    pub fn dashboard_greeting(&self, username: &str) -> String {
        self.dashboard_greeting_template
            .replace("{username}", username)
    }
}
```

With locale-side templates:

```rust
// ja.rs
clients_count_template:        "現在 {n} 件。",
dashboard_greeting_template:   "こんにちは、{username} さん。",

// en.rs
clients_count_template:        "{n} registered.",
dashboard_greeting_template:   "Hello, {username}.",

// zh.rs
clients_count_template:        "目前 {n} 个。",
dashboard_greeting_template:   "您好，{username}。",
```

The simple `replace` is adequate; we are not building a
plural/format engine. If a future RFC needs ICU MessageFormat
plurals, it can replace this helper without changing the
call sites.

A formatters module already exists at
`crates/sui-id-i18n/src/formatters.rs`. Time and number
formatters live there. The text-template helpers above
can live alongside if their volume grows.

### Part D — CI invariant

Add a `text-leaks-pages` step. The grep is conservative —
it flags only patterns very unlikely to be intentional
inside Leptos `view!` macro children:

```yaml
  text-leaks-pages:
    name: text-leak invariants — pages.rs
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - name: No hardcoded JA characters in pages.rs
        run: |
          set -e
          # CJK Unified Ideographs + Hiragana + Katakana ranges
          # in any literal string between view! tags.
          found=$(grep -nP '"[^"]*[\p{Han}\p{Hiragana}\p{Katakana}][^"]*"' \
                    crates/sui-id-web/src/pages.rs || true)
          if [ -n "$found" ]; then
            echo "::error::Hardcoded Japanese characters in pages.rs."
            echo "Move text to crates/sui-id-i18n/src/strings.rs."
            echo "$found"
            exit 1
          fi
      - name: No CamelCase English noun phrases in stat labels (heuristic)
        # We deliberately do NOT lint every English string -- many are
        # technical identifiers (URLs, JWKS, OIDC, etc) that should stay
        # untranslated. The targeted heuristic below flags only span/label
        # element bodies that are 2+ English words wrapped in quotes.
        run: |
          set -e
          # Matches things like: <span class="…">"Two Words Or More"</span>
          found=$(grep -nE '"\s*[A-Z][a-z]+\s+[A-Z]?[a-z]+\s*"' \
                    crates/sui-id-web/src/pages.rs \
                  | grep -v 'aria-label=' \
                  | grep -v 'title=' \
                  | grep -v 'allowed_scopes' || true)
          if [ -n "$found" ]; then
            echo "::warning::Possible hardcoded English label in pages.rs."
            echo "$found"
          fi
```

The first check is strict — Japanese characters in
`pages.rs` are unambiguously a leak after this RFC. The
second is advisory; it surfaces likely-but-not-certain
English leaks for human review. The maintainer can decide
later whether to harden the second check based on false-
positive rate.

`aria-label=` and `title=` are deliberately excluded from
the strict EN check here because RFC 054 handles those in
a separate, more thorough pass. (RFC 051's scope is element
*body* text; RFC 054 covers attribute text.)

### Part E — locale value review

A native reviewer (one per locale) should pass over the
final `ja.rs`, `en.rs`, `zh.rs` once the audit lands. This
catches phrasing issues that no grep would: a translation
that is technically correct but reads awkwardly to a native
operator. The state-contract gives the vocabulary; the
review polishes it.

This RFC does not gate the merge on full review; it gates
on technical completeness (every key supplied, every site
fixed). Polish edits land as follow-up patches.

## Test plan

- **Unit test** (compile-time): `Strings` exhaustiveness
  ensures every key has all three locale values. Already
  guaranteed by the project's existing pattern.
- **CI**: the new `text-leaks-pages` job fails on a draft
  commit that reintroduces a JA literal; passes on the main
  fix branch.
- **E2E** (extend existing tests): for one screen per
  domain (dashboard, clients, users, audit, settings/auth,
  /me/security/mfa), set `Cookie: sui_id_lang=ja`,
  `=en`, `=zh` in turn and assert a known translated phrase
  appears in the HTML. Six screens × three locales = 18
  small assertions, in one new test file
  `tests/e2e/rfc051_i18n_coverage.rs`.
- **Manual spot check**: open the project in dev mode in
  each locale; click through every nav link; visually
  confirm the page bodies switch with the locale.

## Security considerations

None. Translation strings are `&'static`, no user input is
formatted as code, and the template-replacement helper
(Part C) does not interpret control characters.

## Migration risk

Low. The render functions keep their signatures. The
`Strings` struct grows by ~150 fields; the compiler
enforces all three locale files supply them.

A snapshot of the previous EN values is captured in the PR
description so reviewers can confirm semantic equivalence
where the EN string is being re-derived from the JA
(several previously-only-JA sites need first-time English
phrasing).

## Estimated effort

- Part A audit (line-by-line walk across 29 render
  functions): 1.5 days
- Part B refactor + key additions: 1 day
- Part C interpolation helper + the ~8 templated strings:
  0.5 day
- Part D CI snippet: 0.5 day
- Test plan (E2E + manual sweep): 0.5 day

**~4 working days total.**

This is the single largest RFC in the v0.42→v1.0-rc plan.
Splitting per page domain was considered and rejected — the
key-naming consistency benefits from one author doing one
pass, not five.

## Version impact

Minor bump candidate. The `Strings` table is a public API
surface (anyone consuming `sui-id-i18n` sees new fields).
The change is additive: existing field accessors are
unaffected.

## Out of scope

- **Email templates.** Per-recipient locale in
  `core::mail::outbox` is listed in the design memo's
  "未完了 / mail templates" bucket but lives outside `pages.rs`.
  Handled in a separate RFC (TBD; depends on RFC 001 outbox
  surface).
- **Status-word vocabulary.** RFC 052.
- **Copy-button labels.** RFC 053.
- **Aria-label / title attributes.** RFC 054.
- **Style discipline.** RFC 067.

Each is a separate Phase B / F RFC.
