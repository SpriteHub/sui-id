# RFC 064 — Empty / error state primitives

**Status.** Implemented (v0.46.0)
**Priority.** P1 — Phase E (v0.46.0)
**Tracks.** RFC 044 (state words vocabulary contract) — that RFC
established the *terms*; RFC 064 materialises them as actual Leptos
components so the vocabulary cannot drift back into
free-form prose.
**Touches.** `crates/sui-id-web/src/components.rs` (new render
helpers), `crates/sui-id-web/src/pages.rs` (15+ call sites that
currently spell out "No X yet" inline). Depends on RFC 061 (subtle
tokens) — `EmptyState` uses `--surface-subtle`.

## Background

The codebase has ~15 places where an empty list / no-results / error
state is rendered inline. Each spells it out by hand:

- `<p class="muted">No users yet.</p>`
- `<p class="muted">No active sessions.</p>`
- `<tr><td colspan=5 class="muted">No clients registered.</td></tr>`
- ...

RFC 044 established the **state words** contract: each state has a
canonical i18n key (`empty_state_users_yet`, `empty_state_clients_yet`,
etc.) and a canonical phrasing. The vocabulary is in `strings.rs`;
the call sites use it. But each call site reinvents the surrounding
markup — paragraph, paragraph-with-icon, table-row, card. The
result: when a designer changes "we want empty states to have a
suggested next action," there's no one place to edit.

## Goal

Two Leptos render helpers in `components.rs`:

- `empty_state(EmptyStateProps) -> impl IntoView`
- `error_state(ErrorStateProps) -> impl IntoView`

Each takes a small data struct. Every empty/error render in
`pages.rs` calls one of these helpers. New empty states added in
future code are automatically consistent.

## Design

### `EmptyStateProps`

```rust
pub struct EmptyStateProps {
    /// Canonical short message ("No users yet").
    /// Resolved by the caller from `t.empty_state_*` (RFC 044).
    pub message: String,
    /// Optional helper sub-text ("Add one to start onboarding").
    pub hint: Option<String>,
    /// Optional primary call-to-action.
    pub action: Option<EmptyStateAction>,
    /// Compact mode for use inside a table cell (no big padding).
    pub compact: bool,
}

pub struct EmptyStateAction {
    pub href: String,
    pub label: String,
}
```

### `empty_state` render

```rust
pub fn empty_state(props: EmptyStateProps) -> impl IntoView {
    let cls = if props.compact { "empty-state empty-state--compact" }
              else { "empty-state" };
    view! {
        <div class=cls>
            <p class="empty-state__message">{props.message}</p>
            {props.hint.map(|h| view! { <p class="empty-state__hint muted">{h}</p> })}
            {props.action.map(|a| view! {
                <a href=a.href class="button secondary">{a.label}</a>
            })}
        </div>
    }
}
```

### CSS

```css
.empty-state {
  background: var(--surface-subtle);
  border: 1px dashed var(--border-muted);
  border-radius: var(--radius-md);
  padding: var(--space-5);
  text-align: center;
  color: var(--fg-muted);
}
.empty-state--compact {
  padding: var(--space-3);
  border-style: solid;
  text-align: left;
}
.empty-state__message {
  font-size: var(--font-size-body);
  margin: 0 0 var(--space-2);
}
.empty-state__hint {
  font-size: var(--font-size-caption);
  margin: 0 0 var(--space-3);
}
```

The `compact` variant is for table-row fallback (e.g. an empty
clients table); the full variant has more padding and a dashed
border for "this is a placeholder, not data."

### `ErrorStateProps`

```rust
pub struct ErrorStateProps {
    pub message: String,
    pub detail: Option<String>,
    pub retry: Option<EmptyStateAction>,
}
```

Uses `.card--warn` from RFC 062 as the wrapper (so error states
have a real visual signal, not just muted text).

### Migration scope

`pages.rs` call sites to migrate:

| File:line | Current | New |
|-----------|---------|-----|
| pages.rs `render_users` (no users) | inline `<p class="muted">` | `empty_state` |
| pages.rs `render_clients` (no clients) | inline | `empty_state` |
| pages.rs `render_signing_keys` (no retired keys) | inline | `empty_state(compact=true)` |
| pages.rs `render_me_sessions` (no other sessions) | inline | `empty_state` |
| pages.rs `render_me_passkey` (no passkeys) | inline | `empty_state` with register CTA |
| pages.rs `render_audit` (no rows in window) | inline | `empty_state` |
| pages.rs `render_dashboard` recent events (none) | inline | `empty_state(compact=true)` |
| pages.rs `render_user_detail` sessions (none) | inline | `empty_state(compact=true)` |
| ... | ... | ... |

The full sweep is mechanical. The migration commit replaces
~30 lines of duplicated markup with consistent helper calls.

## Test plan

1. Render each affected page with the empty condition triggered
   (no users, no clients, no sessions, etc.). Verify all use the
   same visual language.
2. Render an error state (e.g. db query failure surface). Verify
   `.card--warn` wrapper.
3. Verify i18n keys still flow through `t.empty_state_*` —
   `empty_state` doesn't bypass the vocabulary contract.
4. Manual: side-by-side with v0.45.0 — empty pages now have a
   dashed-bordered placeholder block, not just a faded line of text.

## Rollout

Single release. Visual change is consistent strengthening — no
empty state regresses. Operators on a never-populated install will
see clearer "this section has nothing yet" signposting.

## Future work

- An `LoadingState` primitive for placeholder shimmer. v1.0 doesn't
  need this (server-rendered, no spinners), so deferred.
- A `<NoResults>` variant distinct from `<EmptyState>` — "no users
  match this filter" reads differently from "no users exist." If
  this distinction becomes important, add the variant; for now,
  the same component covers both.
