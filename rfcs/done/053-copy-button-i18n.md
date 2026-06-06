# RFC 053 — Copy-button i18n contract

**Status.** Implemented (v0.43.0)
**Priority.** P1 — Phase B (v0.43.0)
**Tracks.** Closes a long-tail i18n leak that RFC 051's
body-text audit deliberately leaves out (since copy-button
text is mostly attribute text).
**Touches.** `crates/sui-id-web/src/pages.rs`,
`crates/sui-id-web/src/components.rs`,
`crates/sui-id-web/src/layout.rs`,
`crates/sui-id-i18n/src/strings.rs` and locale files.

## Summary

The `copy_btn(value, label)` helper in `pages.rs` takes
`label: &'static str` — typically a hardcoded English noun
like `"Client ID"`, `"JWKS URI"`, `"Client Secret"`. The
function bakes that label into the aria-label, the title
attribute, and the button text. As a result, twelve copy
buttons across the admin panel announce themselves in
English regardless of the user's locale. The clipboard
confirmation text `"✓ Copied"` (injected by `COPY_JS`) has
the same problem.

This RFC retypes `copy_btn` to take a `&'static str` from
the typed `Strings` table, rotates all callers, and threads
the locale into the inline JS so the confirmation text is
localised.

## Background

`copy_btn` today:

```rust
fn copy_btn(value: String, label: &'static str) -> impl IntoView {
    view! {
        <button
            type="button"
            class="copy-btn"
            data-copy=value
            aria-label=format!("Copy {label}")
            title=format!("Copy {label}")>
            "📋 Copy"
        </button>
    }
}
```

Callers pass English labels:

```
$ grep -n 'copy_btn(' crates/sui-id-web/src/pages.rs
… "Client ID" / "JWKS URI" / "Client Secret" / "redirect URI" …
```

The clipboard confirmation toast is the inline JS:

```js
btn.textContent = '\u2713 Copied';
```

Three leaks: (1) the `aria-label` / `title` (`"Copy <noun>"`),
(2) the button's own text `"📋 Copy"`, (3) the post-click
`"✓ Copied"`.

## Goals

1. The aria-label, title, and button text all read in the
   user's locale.
2. The post-click confirmation text reads in the user's
   locale, without re-fetching the page.
3. The helper signature is type-safe: callers cannot pass a
   raw string by accident.

## Detailed design

### Part A — typed helper

The new helper signature takes the localised noun by
function pointer into `Strings`, plus the surrounding
phrase via two more typed strings:

```rust
// crates/sui-id-web/src/components.rs

/// Render a copy-to-clipboard button.
///
/// `value` is the credential text to copy. `noun` is what
/// the button is copying (typed lookup into `Strings`),
/// used in the aria-label and title. The button text reads
/// from `t.copy_button_label`.
pub fn copy_btn(
    t: &'static sui_id_i18n::Strings,
    value: String,
    noun: &'static str,
) -> impl leptos::IntoView {
    // The aria/title phrase is e.g. "Copy Client ID" (en),
    // "Client ID をコピー" (ja). Built from a template field
    // so each locale controls the word order.
    let phrase = t.copy_button_aria_template.replace("{noun}", noun);
    view! {
        <button type="button" class="copy-btn"
                data-copy=value
                aria-label=phrase.clone()
                title=phrase>
            { t.copy_button_label }
        </button>
    }
}
```

Where `noun` is itself one of `t.copy_noun_client_id`,
`t.copy_noun_jwks_uri`, `t.copy_noun_client_secret`,
`t.copy_noun_redirect_uri`, etc. — strongly typed lookups
into `Strings`. The 8 distinct nouns currently in use
become 8 fields, one each.

### Part B — Strings additions

```rust
// strings.rs additions
pub copy_button_label:         &'static str,  // "📋 Copy" / "📋 コピー"
pub copy_button_label_done:    &'static str,  // "✓ Copied" / "✓ コピー済み"
pub copy_button_aria_template: &'static str,  // "Copy {noun}" / "{noun} をコピー"

pub copy_noun_client_id:       &'static str,
pub copy_noun_client_secret:   &'static str,
pub copy_noun_jwks_uri:        &'static str,
pub copy_noun_redirect_uri:    &'static str,
pub copy_noun_audit_id:        &'static str,    // RFC 046
pub copy_noun_setup_token:     &'static str,
pub copy_noun_recovery_code:   &'static str,
pub copy_noun_passkey_id:      &'static str,
```

Sample locale values (English):

```rust
copy_button_label:         "📋 Copy",
copy_button_label_done:    "✓ Copied",
copy_button_aria_template: "Copy {noun}",
copy_noun_client_id:       "Client ID",
copy_noun_client_secret:   "client secret",
copy_noun_jwks_uri:        "JWKS URI",
copy_noun_redirect_uri:    "redirect URI",
copy_noun_audit_id:        "audit row ID",
copy_noun_setup_token:     "setup token",
copy_noun_recovery_code:   "recovery code",
copy_noun_passkey_id:      "passkey ID",
```

Japanese:

```rust
copy_button_label:         "📋 コピー",
copy_button_label_done:    "✓ コピー済み",
copy_button_aria_template: "{noun} をコピー",
copy_noun_client_id:       "クライアント ID",
copy_noun_client_secret:   "クライアントシークレット",
copy_noun_jwks_uri:        "JWKS URI",
copy_noun_redirect_uri:    "リダイレクト URI",
copy_noun_audit_id:        "監査行 ID",
copy_noun_setup_token:     "セットアップトークン",
copy_noun_recovery_code:   "リカバリーコード",
copy_noun_passkey_id:      "パスキー ID",
```

Chinese:

```rust
copy_button_label:         "📋 复制",
copy_button_label_done:    "✓ 已复制",
copy_button_aria_template: "复制 {noun}",
copy_noun_client_id:       "Client ID",
copy_noun_client_secret:   "客户端密钥",
copy_noun_jwks_uri:        "JWKS URI",
copy_noun_redirect_uri:    "重定向 URI",
copy_noun_audit_id:        "审计条目 ID",
copy_noun_setup_token:     "初始化令牌",
copy_noun_recovery_code:   "恢复码",
copy_noun_passkey_id:      "通行密钥 ID",
```

Some nouns stay in Latin script in Chinese (`Client ID`,
`JWKS URI`) because they are protocol identifiers; the
state-words contract already allows preserving such proper
nouns.

### Part C — caller migration

Every `copy_btn(` call site updates. From:

```rust
copy_btn(cid, "Client ID")
copy_btn(sec, "Client Secret")
copy_btn("/.well-known/jwks.json".to_string(), "JWKS URI")
```

To:

```rust
copy_btn(t, cid, t.copy_noun_client_id)
copy_btn(t, sec, t.copy_noun_client_secret)
copy_btn(t, "/.well-known/jwks.json".to_string(), t.copy_noun_jwks_uri)
```

12 call sites total (count from `grep -c 'copy_btn(' pages.rs`
at v0.41.0).

### Part D — clipboard confirmation text

The inline `COPY_JS` in `layout.rs` hardcodes the toast
text:

```js
btn.textContent = '\u2713 Copied';
```

Two options to localise:

**Option D1 — Read from a `data-` attribute on the
button.** Each button carries `data-copy-done="…"` set by
SSR to the localised string. The JS reads that instead of
the hardcoded literal. Recommended.

```rust
// components.rs::copy_btn
view! {
    <button …
            data-copy=value
            data-copy-done=t.copy_button_label_done
            aria-label=phrase.clone()
            title=phrase>
        { t.copy_button_label }
    </button>
}
```

```js
// layout.rs::COPY_JS
var done = btn.getAttribute('data-copy-done') || '\u2713 Copied';
btn.textContent = done;
```

**Option D2 — Inject a JS object from SSR.** A small
`<script>` block in `Shell` writes
`window.suiIdL10n = { copied: '✓ コピー済み' }` and the JS
reads from there.

Option D1 is local to the button, doesn't pollute the
global namespace, and survives a button being re-rendered
client-side (a future possibility). RFC 053 picks D1.

### Part E — restore-on-leave

The current JS resets the button text after 1800 ms by
caching `orig = btn.textContent`. That still works without
modification — `orig` captures whatever locale-text the
SSR produced.

## Test plan

- **Compile-time**: `Strings` exhaustiveness covers the
  new fields.
- **Unit test** in `sui-id-web`:
  ```rust
  #[test]
  fn copy_btn_attributes() {
      let t = sui_id_i18n::Locale::En.strings();
      let html = render_ssr(|| copy_btn(t, "abc".into(), t.copy_noun_client_id));
      assert!(html.contains(r#"aria-label="Copy Client ID""#));
      assert!(html.contains(r#"data-copy-done="✓ Copied""#));
  }
  ```
- **E2E**: an existing clients-list E2E navigates to
  `/admin/clients` and finds a copy button. Extend to
  assert the button's aria-label contains the locale-
  appropriate phrase (`"Copy"` or `"コピー"`).
- **Manual** (Playwright/browser): click a copy button in
  each locale; observe the toast reads in the same locale;
  observe it restores after the timeout.

## Security considerations

None. The clipboard content (`value`) is unchanged. The new
`data-copy-done` attribute carries only the locale's
"Copied" word, which is not sensitive.

## Migration risk

Low. The helper changes from `copy_btn(value, label)` to
`copy_btn(t, value, noun)`; every call site updates in the
same commit. No other code reaches into the helper.

## Estimated effort

- Part A (helper signature + components.rs body): 30 min
- Part B (Strings additions + ja/en/zh): 1 hour
- Part C (12 call-site migrations): 30 min
- Part D (data-attribute + JS read): 30 min
- Tests + manual: 1 hour

**~3.5 hours total.**

## Version impact

Patch bump within v0.43.0 alongside RFCs 051 and 052.

## Open questions

None.
