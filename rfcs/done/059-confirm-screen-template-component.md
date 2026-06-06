# RFC 059 — Confirm-screen template component

**Status.** Implemented (v0.45.0)
**Priority.** P1 — Phase D (v0.45.0)
**Tracks.** Code consistency on the five confirm screens. Today they
re-implement the same Shell + auth-card + identity + impact + badge +
form scaffold. Drift between them is silent.
**Touches.** `crates/sui-id-web/src/components.rs` (new `confirm_screen`
function), `crates/sui-id-web/src/pages.rs` (5 `render_confirm_*`
functions collapse to ~10 LOC each).

## Background

There are five `render_confirm_*` functions in `pages.rs`:

| Function | LOC | Action |
|----------|----:|--------|
| `render_confirm_disable_user` | 54 | toggle user disabled |
| `render_confirm_delete_user` | 32 | delete user |
| `render_confirm_reset_mfa` | 32 | reset another user's MFA |
| `render_confirm_delete_client` | 32 | delete OIDC client |
| `render_confirm_delete_signing_key` | 33 | delete signing key |

Every one of them follows the same shape:

```
Shell title=... current=<nav> ...
  div.auth-card max-width:36rem
    h1 {title}
    p strong {target_identity}             // varies: username / client name / kid
    [p.muted {impact}]                     // optional (re-enable skips)
    p {reversibility_badge}                // ✓ recoverable / ⚠ irreversible
    [p.muted.caption {reversibility_text}] // optional
    form method=post action=<dynamic>
      input hidden _csrf
      [input hidden disabled=<new_state>]   // disable only
      input hidden _confirmed=1
      [textarea name=reason]                // disable only
      button.danger|btn submit {button_label}
      a.button.secondary href=<list> {cancel}
```

The drift risk is real: when RFC 049 changed `--max-width-card` token,
every confirm screen had to be updated individually. When RFC 045
added the reason textarea to disable, only that one function got the
field. Future copy-edits (e.g. cancel button text) require five
edits.

## Goal

A single `confirm_screen` component that takes the variable bits as
data; the five callers shrink to ~10 LOC each that build the data
struct and delegate rendering.

## Design

### Component signature

```rust
/// Shared confirm-screen scaffold for dangerous-action gates
/// (RFC 030 + RFC 059).
///
/// The five `render_confirm_*` functions delegate the page body to
/// this. Each caller still owns the data construction (which strings
/// to pull, which `current=` nav key to highlight); this just renders
/// the consistent skeleton.
pub fn confirm_screen(
    data: ConfirmScreenData,
    lang: sui_id_i18n::Locale,
) -> impl IntoView {
    // <auth-card> body only — caller wraps with Shell.
}

pub struct ConfirmScreenData {
    /// Page title (h1 and shell title).
    pub title: String,

    /// Identity-of-target line. The caller passes the visible name
    /// or identifier (e.g. username, client name, "kid (RS256)").
    /// Rendered as `<p><strong>{identity}</strong></p>`.
    pub identity: String,

    /// Optional impact line (`<p class="muted">{impact}</p>`).
    /// `None` skips the line entirely — used by the user re-enable
    /// case where there's no destructive impact to warn about.
    pub impact: Option<String>,

    /// Reversibility badge kind. `None` skips the badge.
    pub badge: Option<ReversibilityKind>,

    /// Optional small-print reversibility note.
    pub reversibility_text: Option<String>,

    /// Form action URL (`POST` target).
    pub action_url: String,

    /// CSRF token (string already resolved by the caller).
    pub csrf_token: String,

    /// Additional hidden inputs the action needs.
    /// For the disable form: `vec![("disabled".into(), "true".into())]`.
    /// Empty for delete/reset forms.
    pub extra_hidden: Vec<(String, String)>,

    /// If true, render the disable-reason textarea (RFC 045).
    pub include_reason_field: bool,

    /// Submit button label.
    pub button_label: String,

    /// True → `class="danger"`, false → `class="btn"`. The re-enable
    /// case is the only "btn" path.
    pub button_danger: bool,

    /// Cancel link URL.
    pub cancel_url: String,
}

pub enum ReversibilityKind { Recoverable, Irreversible }
```

The `_confirmed=1` hidden input is unconditional — every confirm
screen needs it. The component emits it automatically; callers don't
pass it.

### Caller shape (e.g. `render_confirm_delete_client`)

Before (32 LOC):

```rust
pub fn render_confirm_delete_client(
    data: ConfirmDeleteClientData,
    dev_mode: bool,
    lang: sui_id_i18n::Locale,
) -> String {
    render(move || {
        let t = lang.strings();
        let action = format!("/admin/clients/{}/delete", data.client_id);
        let badge = reversibility_badge(false, t);
        let name = data.client_name.clone();
        view! {
            <Shell title=t.confirm_delete_client_title.to_string() show_nav=true
                   current=Some("clients".to_string()) dev_mode=dev_mode lang=lang>
                <div class="auth-card" style="max-width:36rem">
                    <h1>{t.confirm_delete_client_title}</h1>
                    <p><strong>{name}</strong></p>
                    <p class="muted">{t.confirm_delete_client_impact}</p>
                    <p>{badge}</p>
                    <p class="muted" style="font-size:var(--font-size-caption)">
                        {t.confirm_delete_client_reversibility}
                    </p>
                    <form method="post" action=action class="row" style="gap:var(--space-2);margin-top:var(--space-4)">
                        <input type="hidden" name="_csrf" value=data.csrf_token />
                        <input type="hidden" name="_confirmed" value="1" />
                        <button type="submit" class="danger">{t.confirm_delete_client_button}</button>
                        <a href="/admin/clients" class="button secondary">{t.confirm_cancel}</a>
                    </form>
                </div>
            </Shell>
        }
    })
}
```

After (~15 LOC):

```rust
pub fn render_confirm_delete_client(
    data: ConfirmDeleteClientData,
    dev_mode: bool,
    lang: sui_id_i18n::Locale,
) -> String {
    render(move || {
        let t = lang.strings();
        let body = confirm_screen(ConfirmScreenData {
            title: t.confirm_delete_client_title.into(),
            identity: data.client_name.clone(),
            impact: Some(t.confirm_delete_client_impact.into()),
            badge: Some(ReversibilityKind::Irreversible),
            reversibility_text: Some(t.confirm_delete_client_reversibility.into()),
            action_url: format!("/admin/clients/{}/delete", data.client_id),
            csrf_token: data.csrf_token.clone(),
            extra_hidden: vec![],
            include_reason_field: false,
            button_label: t.confirm_delete_client_button.into(),
            button_danger: true,
            cancel_url: "/admin/clients".into(),
        }, lang);
        view! {
            <Shell title=t.confirm_delete_client_title.to_string() show_nav=true
                   current=Some("clients".to_string()) dev_mode=dev_mode lang=lang>
                {body}
            </Shell>
        }
    })
}
```

Net: 32 → 22 LOC, but the body is **structurally identical** to the
other four — drift impossible because there's only one definition.

### Why not collapse Shell too?

Shell takes `current=Some(<key>)` which varies per route. Putting that
inside `confirm_screen` would mean threading another parameter, and
the caller still needs to choose the key. Keeping Shell at the caller
is the cleanest split: the component owns the **dangerous-action
scaffold**, the caller owns the **page chrome integration**.

## Test plan

1. Unit: each of the 5 `render_confirm_*` outputs same HTML structure
   before and after (modulo whitespace). Existing e2e tests for
   confirm flows continue to pass without modification.
2. Manual: render each confirm screen in ja/en/zh and verify visual
   parity with v0.44.0.

## Rollout

Single release. No new routes, no schema migration, no user-visible
change. Pure code-structural improvement.

## Future work

- The `Shell + ConfirmScreen + body` pattern could be tightened
  further by giving `ConfirmScreen` a `current_nav` parameter and
  wrapping the Shell internally. Deferred: keeping the chrome at the
  caller is more flexible for the rare case where a confirm screen
  appears in a non-admin context (e.g. user self-confirm during MFA
  reset request flow, if RFC X ever adds one).
