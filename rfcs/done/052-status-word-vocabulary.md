# RFC 052 — Status word vocabulary unification

**Status.** Implemented (v0.43.0)
**Priority.** P1 — Phase B (v0.43.0)
**Tracks.** Closes the operational half of RFC 044 (the
state-words contract). Reduces the surface RFC 051's
`text-leaks-pages` CI check has to police.
**Touches.** `crates/sui-id-web/src/pages.rs` (call sites),
`crates/sui-id-web/src/components.rs` (the helper module's
new entries), `crates/sui-id-i18n/src/strings.rs` and the
locale files.

## Summary

The same six status words — `active`, `disabled`, `deleted`,
`in use`, `retired`, `published` — appear as hardcoded
English literals at 24+ call sites across `pages.rs`. RFC
044 codified the vocabulary in `STATE_WORDS.md` but did not
enforce it in code. This RFC adds a single typed helper
`status_badge(t, kind)` that returns a localised badge view
with the consistent CSS class, replaces every duplicate
site with one call, and gives the vocabulary a single
authoritative source of truth.

## Background

The status badges currently render as:

```rust
// pages.rs:1282–1288 (clients table row)
let status_badge = if is_deleted {
    view! { <span class="badge badge--danger">"deleted"</span> }.into_any()
} else if is_disabled {
    view! { <span class="badge badge--warn">"disabled"</span> }.into_any()
} else {
    view! { <span class="badge badge--ok">"active"</span> }.into_any()
};
```

The same triplet appears in `user_row_view`, with
near-identical code. `signing_keys_row_view` does the
same with `in use` / `retired` / `published`. Each call
site has copy-pasted the badge class assignment and the
hardcoded English text. Any future locale support has to
walk all 24+ sites to translate; RFC 044's state contract
has no compiler-level reach.

## Goals

1. One source of truth for the status-word vocabulary in
   `Strings`, with translations in ja / en / zh.
2. One helper, `status_badge(t, kind)`, used everywhere.
3. No regression in CSS class output — the same `badge`,
   `badge--ok`, `badge--warn`, `badge--danger`, `badge--info`
   classes are still produced.
4. The empty-state literal placeholders (`"-"`, `"(any)"`,
   `"(falls back to redirect_uris)"`, `"(no email)"`) also
   move to typed `Strings` fields. Treated together because
   the "empty" state is a sibling of the status state under
   the state-words contract.

## Detailed design

### Part A — typed status enum

A small `StatusKind` enum encodes the badge mapping
deterministically:

```rust
// crates/sui-id-web/src/components.rs

/// Status badge kind. Each variant carries a fixed CSS
/// class and points at one Strings field.
pub enum StatusKind {
    Active,         // → badge--ok,     t.status_active
    Disabled,       // → badge--warn,   t.status_disabled
    Deleted,        // → badge--danger, t.status_deleted
    Pending,        // → badge--info,   t.status_pending
    InUse,          // → badge--ok,     t.status_in_use
    Retired,        // → badge--muted,  t.status_retired
    Published,      // → badge--ok,     t.status_published
    Healthy,        // → badge--ok,     t.status_healthy
    Unhealthy,      // → badge--danger, t.status_unhealthy
}

pub fn status_badge(t: &'static sui_id_i18n::Strings, kind: StatusKind)
    -> impl leptos::IntoView
{
    let (class, text) = match kind {
        StatusKind::Active     => ("badge badge--ok",     t.status_active),
        StatusKind::Disabled   => ("badge badge--warn",   t.status_disabled),
        StatusKind::Deleted    => ("badge badge--danger", t.status_deleted),
        StatusKind::Pending    => ("badge badge--info",   t.status_pending),
        StatusKind::InUse      => ("badge badge--ok",     t.status_in_use),
        StatusKind::Retired    => ("badge badge--muted",  t.status_retired),
        StatusKind::Published  => ("badge badge--ok",     t.status_published),
        StatusKind::Healthy    => ("badge badge--ok",     t.status_healthy),
        StatusKind::Unhealthy  => ("badge badge--danger", t.status_unhealthy),
    };
    view! { <span class=class>{text}</span> }
}
```

Adding a `--muted` badge variant is a small `components.rs`
change in the same RFC; it covers `retired` cleanly without
hand-rolled inline styles.

### Part B — empty / placeholder vocabulary

The empty-state literals are grouped under a single naming
convention:

```rust
// strings.rs additions
pub empty_dash:                &'static str,  // "-"
pub empty_any:                 &'static str,  // "(any)"
pub empty_none:                &'static str,  // "(none)"
pub empty_falls_back:          &'static str,  // "(falls back to …)"
pub empty_no_email:            &'static str,  // "(no email)"
pub empty_not_set:             &'static str,  // "(not set)"
```

Locale samples:

```rust
// ja.rs
empty_dash:        "—",
empty_any:         "（すべて）",
empty_none:        "（なし）",
empty_falls_back:  "（redirect_uris にフォールバック）",
empty_no_email:    "（メールアドレスなし）",
empty_not_set:     "（未設定）",

// en.rs
empty_dash:        "—",
empty_any:         "(any)",
empty_none:        "(none)",
empty_falls_back:  "(falls back to redirect_uris)",
empty_no_email:    "(no email)",
empty_not_set:     "(not set)",

// zh.rs
empty_dash:        "—",
empty_any:         "（全部）",
empty_none:        "（无）",
empty_falls_back:  "（回退到 redirect_uris）",
empty_no_email:    "（无邮箱）",
empty_not_set:     "（未设置）",
```

The em-dash `—` is intentional: U+2014 is one character that
renders consistently in all CJK and Latin fonts, and is the
neutral standard for "missing value" in tabular UIs. The
ASCII hyphen `-` previously used was both shorter and easier
to confuse with a real value.

### Part C — call-site rewrites

Each duplicated site becomes a single call:

```rust
// Before — pages.rs:1282–1288
let status_badge = if is_deleted {
    view! { <span class="badge badge--danger">"deleted"</span> }.into_any()
} else if is_disabled {
    view! { <span class="badge badge--warn">"disabled"</span> }.into_any()
} else {
    view! { <span class="badge badge--ok">"active"</span> }.into_any()
};

// After
let kind = if c.is_deleted { StatusKind::Deleted }
           else if c.is_disabled { StatusKind::Disabled }
           else { StatusKind::Active };
let status_badge = status_badge(t, kind);
```

Empty-state literals follow the same pattern:

```rust
// Before
let scopes_display = if c.allowed_scopes.trim().is_empty() {
    "(any)".to_string()
} else {
    c.allowed_scopes.clone()
};
let logout_display = if logout_count == 0 {
    "(falls back to redirect_uris)".to_string()
} else {
    format!("{logout_count} URI(s)")
};

// After
let scopes_display = if c.allowed_scopes.trim().is_empty() {
    t.empty_any.to_string()
} else {
    c.allowed_scopes.clone()
};
let logout_display = if logout_count == 0 {
    t.empty_falls_back.to_string()
} else {
    // Number formatting goes via crates/sui-id-i18n/src/formatters.rs
    // (existing): the format also becomes locale-aware via a fmt template.
    (t.clients_logout_uri_count_template)(logout_count)
};
```

### Part D — call-site coverage

Sites to update (per the gap-analysis report):

| Source                                             | What                                         |
|----------------------------------------------------|----------------------------------------------|
| `pages.rs::client_row_view`                        | 3 status branches → `StatusKind`             |
| `pages.rs::render_client_edit`                     | 1 status                                     |
| `pages.rs::user_row_view`                          | 3 status branches                            |
| `pages.rs::render_user_detail`                     | 2 status branches                            |
| `pages.rs::signing_key_row_view`                   | 3 status (in use / retired / published)      |
| `pages.rs::render_dashboard`                       | service-ok badge                             |
| `pages.rs::settings_basic`                         | enabled/disabled badges                      |
| `pages.rs::settings_security`                      | enabled/disabled badges                      |
| `pages.rs::settings_email`                         | configured/not configured badges             |
| `pages.rs::clients` `kv_text` / `kv_code` for `-`  | empty-dash                                   |
| `pages.rs::user_detail` `(no email)` / `(not set)` | empty-no-email / empty-not-set               |
| `pages.rs::clients` `(any)` / `(falls back to …)`  | empty-any / empty-falls-back                 |

Approximately 24 status sites + 14 empty-state sites = 38
mechanical call-site changes.

### Part E — interaction with RFC 051

RFC 051's strict JA-character grep would flag the previous
hardcoded JA literals; RFC 052 *replaces* them with typed
keys, so the grep stays clean.

Ordering: RFC 052 can land **inside** RFC 051's branch, or
as a fast-follow PR. The two are written separately because
RFC 052's helper has independent value (it survives even if
RFC 051 is split across multiple sub-PRs).

## Test plan

- **Compile-time**: `Strings` exhaustiveness covers the new
  fields automatically.
- **Unit test** (new) in `sui-id-web/src/tests.rs`:
  ```rust
  #[test]
  fn status_badge_classes() {
      use sui_id_i18n::Locale;
      let t = Locale::En.strings();
      for kind in [StatusKind::Active, StatusKind::Deleted, …] {
          let html = leptos::SsrTester::render(|| status_badge(t, kind));
          assert!(html.contains("badge"));
          // class shape: always "badge X" with X in the allowed set
      }
  }
  ```
- **E2E**: the existing clients-list E2E test asserts the
  status badge text appears for one row. Update the assertion
  to the new English value `"Active"` / `"Disabled"` /
  `"Deleted"` (capitalised, since that is conventional badge
  styling — see Part F).
- **Visual smoke**: open the clients list and the users
  list in each locale; confirm badges read correctly.

### Part F — capitalisation convention

The previous English literals were lower-case (`"active"`).
The state-words contract doesn't specify case, but
badges read better capitalised. This RFC standardises on
"Sentence case" for English status words:
`Active` / `Disabled` / `Deleted` / `In use` / `Retired` /
`Published`. The JA and ZH renderings carry no case.

## Security considerations

None.

## Migration risk

Low. The CSS classes produced are identical to today's;
only the text content changes. The new `badge--muted` class
is added to `components.rs`; existing badge classes stay.

## Estimated effort

- Part A (enum + helper + new badge variant): 1 hour
- Part B (Strings + ja/en/zh fields): 1 hour
- Part C/D (38 call-site rewrites): 2 hours
- Part E/F (capitalisation review): 30 min
- Tests + visual smoke: 1.5 hours

**~6 hours total.**

## Version impact

Patch bump within v0.43.0 alongside RFC 051.

## Open questions

1. **Locale-aware count templates**. The
   `clients_logout_uri_count_template` cited in Part C uses
   the templated-string pattern from RFC 051 Part C. If
   that pattern is not yet merged when this RFC ships, the
   call site can fall back to `format!("{} URI(s)", n)`
   temporarily. (English-only, but consistent with
   what's there today.) Resolve by sequencing 051 before
   052 in the PR queue.
2. **`empty_dash = "—"`** changes the visual character.
   Mostly cosmetic improvement; confirm with the maintainer
   that this is the desired direction. Reverting to `-` is
   a single-line change.
