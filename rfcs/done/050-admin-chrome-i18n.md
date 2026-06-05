# RFC 050 — Admin chrome i18n (Nav, Footer, ThemeToggle)

**Status.** Implemented (v0.42.0)
**Priority.** P0 — blocker for v0.42.0
**Tracks.** UI/UX correctness baseline; Phase A of the
v0.42 → v1.0-rc plan. Closes a contract item the
`a11y + i18n v0.29.x` slide flagged as "未完了."
**Touches.** `crates/sui-id-web/src/layout.rs`,
`crates/sui-id-i18n/src/strings.rs`,
`crates/sui-id-i18n/src/{ja,en,zh}.rs`,
`crates/sui-id/src/handlers/admin.rs` (signature change for
`Shell` / `AuthShell` callers).

## Summary

The application chrome — the nav rendered by `Shell`, the
footer tagline and a11y badges, the theme-toggle buttons —
contains hardcoded English and Japanese strings. The nav
i18n keys already exist (`nav_dashboard` through
`nav_logout`) and are never read; the footer and theme-toggle
have no i18n keys at all. As a result, regardless of which
locale the user has selected, every admin page renders the
same English menu and the same hardcoded Japanese footer
line. This RFC threads the resolved `Locale` through the
chrome, reads the existing `nav_*` keys, and adds the
missing `footer_*`, `a11y_*`, `theme_*` keys for the three
locales.

## Background

In `crates/sui-id-web/src/layout.rs`:

```rust
// L181 — Nav (every entry hardcoded English)
let items = [
    ("dashboard", "Dashboard", "/admin"),
    ("users",     "Users",     "/admin/users"),
    ("clients",   "Clients",   "/admin/clients"),
    ("signing-keys", "Keys",   "/admin/signing-keys"),
    ("audit",     "Audit",     "/admin/audit"),
    ("settings",  "Settings",  "/admin/settings"),
    ("profile",   "Profile",   "/admin/profile"),
];
// L210 — Sign out button: hardcoded English

// L231 — Footer tagline (hardcoded Japanese):
"🌱 sui-id · 静かで、凛として、やさしい ID 基盤を。"

// L234–L236 — a11y badges (hardcoded English):
"⌨ Keyboard" / "⊙ Screen reader" / "◐ Contrast"

// L255–L273 — theme toggle (hardcoded English):
"☀ Light" / "🖥 Auto" / "☾ Dark"
```

The corresponding i18n keys for the nav exist:

```
$ grep "pub nav_" crates/sui-id-i18n/src/strings.rs
    pub nav_dashboard:     &'static str,
    pub nav_users:         &'static str,
    pub nav_clients:       &'static str,
    pub nav_signing_keys:  &'static str,
    pub nav_audit:         &'static str,
    pub nav_settings:      &'static str,
    pub nav_profile:       &'static str,
    pub nav_logout:        &'static str,
```

And `ja.rs`, `en.rs`, `zh.rs` all supply values. No
production code reads them.

The footer tagline and theme-toggle labels have **no**
i18n keys; they were authored as literals from day one.

The slide labelled "a11y + i18n v0.29.x" in
`suiiduiuxdevelopmentsupportv0.29x.pdf` explicitly listed
*admin dashboard / users / clients* as "未完了" for i18n.
Phase B (RFC 051) handles the page-content half. This RFC
handles the chrome half, which has to land first because
every page passes through `Shell`.

## Goals

1. The nav, footer tagline, a11y badge titles and
   theme-toggle button labels all change when `Locale`
   changes. JA/EN/ZH are wired in this RFC; future locales
   inherit automatically via the typed `Strings` table.
2. The `Shell` component receives the resolved `Locale` and
   passes it down to `Nav`, `Footer`, `ThemeToggle`. No
   inferred locale from the document, no string-based
   plumbing.
3. The default emoji glyphs (🌱, ⌨, ⊙, ◐, ☀, 🖥, ☾) stay
   visible alongside the localised text — they are visual
   anchors, not the text itself.

## Detailed design

### Part A — extend the `Strings` table

Add the following fields to
`crates/sui-id-i18n/src/strings.rs` in the existing
"---- Navigation ----" / "---- Chrome ----" sections:

```rust
// ---- Chrome / footer ----
pub footer_tagline:        &'static str,
pub a11y_keyboard:         &'static str,
pub a11y_screen_reader:    &'static str,
pub a11y_contrast:         &'static str,

// ---- Theme toggle ----
pub theme_toggle_group:    &'static str,  // aria-label="Theme"
pub theme_toggle_light:    &'static str,
pub theme_toggle_auto:     &'static str,
pub theme_toggle_dark:     &'static str,
pub theme_toggle_light_title: &'static str,  // title="Light theme"
pub theme_toggle_auto_title:  &'static str,
pub theme_toggle_dark_title:  &'static str,

// ---- Nav aria-labels ----
pub nav_aria_main:         &'static str,  // aria-label="Main"
pub nav_aria_signout:      &'static str,  // aria-label="Sign out"
```

Supply values in `ja.rs`, `en.rs`, `zh.rs`. The compiler
forces all three to provide every field via the existing
exhaustive-struct-literal pattern; no field can be missed.

Sample (Japanese):

```rust
footer_tagline:        "🌱 sui-id · 静かで、凛として、やさしい ID 基盤を。",
a11y_keyboard:         "キーボード対応",
a11y_screen_reader:    "スクリーンリーダー対応",
a11y_contrast:         "コントラスト対応",
theme_toggle_group:    "テーマ",
theme_toggle_light:    "ライト",
theme_toggle_auto:     "自動",
theme_toggle_dark:     "ダーク",
theme_toggle_light_title: "ライトテーマ",
theme_toggle_auto_title:  "OS の設定に従う",
theme_toggle_dark_title:  "ダークテーマ",
nav_aria_main:         "メインナビゲーション",
nav_aria_signout:      "ログアウト",
```

Sample (English):

```rust
footer_tagline:        "🌱 sui-id · A quiet, dependable identity foundation.",
a11y_keyboard:         "Keyboard accessible",
a11y_screen_reader:    "Screen-reader friendly",
a11y_contrast:         "High contrast support",
theme_toggle_group:    "Theme",
theme_toggle_light:    "Light",
theme_toggle_auto:     "Auto",
theme_toggle_dark:     "Dark",
theme_toggle_light_title: "Light theme",
theme_toggle_auto_title:  "Follow system",
theme_toggle_dark_title:  "Dark theme",
nav_aria_main:         "Main navigation",
nav_aria_signout:      "Sign out",
```

Sample (Chinese):

```rust
footer_tagline:        "🌱 sui-id · 安静、可靠的身份认证基础。",
a11y_keyboard:         "支持键盘操作",
a11y_screen_reader:    "支持屏幕阅读器",
a11y_contrast:         "支持高对比度",
theme_toggle_group:    "主题",
theme_toggle_light:    "浅色",
theme_toggle_auto:     "自动",
theme_toggle_dark:     "深色",
theme_toggle_light_title: "浅色主题",
theme_toggle_auto_title:  "跟随系统",
theme_toggle_dark_title:  "深色主题",
nav_aria_main:         "主导航",
nav_aria_signout:      "登出",
```

Translator notes: the tagline carries the project's mood
intentionally (静か → quiet, dependable; 凛として is hard to
translate directly and is treated as conveyed by "quiet"
together with the project name). The Chinese rendering
prioritises *安静可靠* over a literal translation. The
maintainer or a native reviewer should sanity-check before
merge; the keys can be tuned without re-touching the
`layout.rs` plumbing.

### Part B — wire `Locale` through `Shell`

`Shell` already accepts `lang: Option<Locale>`. Make it
mandatory in this RFC (every call site already passes
something via `.unwrap_or_default()` further down). The
internal `Nav`, `Footer`, `ThemeToggle` components gain a
`lang: Locale` parameter and a `t = lang.strings()` head
binding.

```rust
#[component]
pub fn Shell(
    title: String,
    show_nav: bool,
    current: Option<String>,
    lang: sui_id_i18n::Locale,            // was Option<Locale>
    #[prop(optional)] dev_mode: Option<bool>,
    children: Children,
) -> impl IntoView {
    let stylesheet = format!("{}\n{}", TOKENS_CSS, COMPONENTS_CSS);
    let lang_tag = lang.tag();
    let dir_attr = lang.direction();
    view! {
        <html lang=lang_tag dir=dir_attr>
            <head>…</head>
            <body>
                {dev_mode.unwrap_or(false).then(…)}
                <header class="app-header">
                    <h1 class="app-header__brand">"sui-id"</h1>
                    {show_nav.then(|| view! {
                        <Nav current=current.clone() lang=lang csrf_token="".to_string() />
                    })}
                </header>
                <main class="app-main">{children()}</main>
                <Footer lang=lang />
            </body>
        </html>
    }
}
```

The `Nav` component reads `t.nav_*`:

```rust
#[component]
fn Nav(current: Option<String>, lang: sui_id_i18n::Locale, csrf_token: String)
    -> impl IntoView
{
    let t = lang.strings();
    let items = [
        ("dashboard",    t.nav_dashboard,    "/admin"),
        ("users",        t.nav_users,        "/admin/users"),
        ("clients",      t.nav_clients,      "/admin/clients"),
        ("signing-keys", t.nav_signing_keys, "/admin/signing-keys"),
        ("audit",        t.nav_audit,        "/admin/audit"),
        ("settings",     t.nav_settings,     "/admin/settings"),
        ("profile",      t.nav_profile,      "/admin/profile"),
    ];
    view! {
        <nav class="app-nav" aria-label=t.nav_aria_main>
            { items.into_iter().map(|(key, label, href)| {
                let aria = if current.as_deref() == Some(key) { Some("page") } else { None };
                view! { <a class="app-nav__link" href=href aria-current=aria>{label}</a> }
            }).collect::<Vec<_>>() }
            …
            <button type="submit" class="app-nav__link app-nav__signout"
                    aria-label=t.nav_aria_signout>
                {t.nav_logout}
            </button>
            …
        </nav>
    }
}
```

`Footer` reads `t.footer_tagline`, `t.a11y_*`. `ThemeToggle`
reads `t.theme_toggle_*`. The emoji glyphs are kept as
prefixes in the rendered text so the visual landmark
survives a locale switch — e.g.
`"⌨ {t.a11y_keyboard}"`.

### Part C — Profile link redirect note

When RFC 055 (Phase C) lands and consolidates
`/admin/profile` onto `/me/security/*`, the `"profile"` nav
entry will retarget. That is out of scope here; this RFC
keeps the existing `/admin/profile` URL.

### Part D — `AuthShell` symmetry

`AuthShell` (the centred narrow layout used by setup /
login / forgot-password) has the same `Footer` and the same
chrome but no Nav. Apply the same `Locale`-mandatory change
and the same Footer/ThemeToggle propagation.

### Part E — `view!` macro quirks

When the children of `<a>` or `<button>` are a typed
`&'static str` from `Strings`, Leptos accepts both
`{label}` and a bare `label` token — but per RFC 048 the
project standardises on `{…}` to make every interpolation
explicit. New code in this RFC follows that convention.

## Test plan

- **Unit test** in `sui-id-i18n`: assert all three
  `STRINGS_*` constants compile against the extended
  `Strings` definition. This is a compile-time test (struct
  literal exhaustiveness); no runtime assertion needed.
- **Snapshot test**, optional: render `Shell` with each of
  the three locales, snapshot the chrome HTML, diff. If the
  project already uses `insta` or similar this is cheap; if
  not, a manual end-to-end check suffices.
- **E2E**: existing dashboard E2E flow runs with `Locale::Ja`
  by default; add a small follow-up that sets the cookie
  `sui_id_lang=en` and asserts the nav contains `Dashboard` /
  `Users` text. (A single test covers the locale plumbing.)
- **Visual smoke**: open the dashboard in JA / EN / ZH,
  confirm the nav, footer tagline, a11y badges and theme
  toggle all change. Switch the theme to dark; the toggle
  state survives a reload (existing `localStorage`-based
  logic is unchanged).

## Security considerations

None. No new authority granted; no new privilege boundary
crossed. The `Strings` data is `&'static`, immutable, and
identical to the rest of the i18n surface.

## Migration risk

Low. The single API change is `Shell`'s `lang` parameter
going from `Option<Locale>` to `Locale`. Every existing
caller already passes `lang=lang` (when the caller has a
resolved locale) or `lang=Locale::default()` implicitly via
`.unwrap_or_default()`. The mechanical fix is replacing the
implicit `.unwrap_or_default()` with an explicit
`Locale::default()` at callers that have no resolved locale
in hand — none do today, since every `render_*` function in
`pages.rs` already binds `lang` from the function argument.

A grep across the workspace catches stragglers:

```
$ grep -rn 'Shell\s*\(\|AuthShell\s*\(' crates/ --include='*.rs' \
    | grep -v 'lang='
```

Expected to return empty after the migration.

## Estimated effort

- Part A (Strings + ja/en/zh fields): 1 hour
- Part B (Shell/Nav/Footer/ThemeToggle wiring): 2 hours
- Part C (skipped; covered by RFC 055)
- Part D (AuthShell symmetry): 30 minutes
- Test plan (E2E + visual smoke): 1.5 hours

**~5 hours total.**

## Version impact

Patch bump candidate; bundled into v0.42.0 with RFCs 048
and 049.

## Open questions

1. **Tagline tone in EN/ZH.** The JA tagline "静かで、凛として、
   やさしい ID 基盤を。" carries the project's mood. The EN/ZH
   renderings above are first-pass; a native reviewer should
   confirm before merge. Tuning the string does not affect
   the wiring.
2. **`prefers-reduced-data` for emoji.** The chrome uses
   emoji as visual landmarks. Some screen readers vocalise
   emoji verbosely. We rely on `title=` / explicit
   surrounding text for the a11y story, but an audit pass in
   Phase F could revisit. Out of scope here.
