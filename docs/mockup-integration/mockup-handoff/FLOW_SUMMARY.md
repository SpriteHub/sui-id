# Flow Summary — sui-id Mockup

The five user flows the implementation must preserve end-to-end. Each
flow shows actor, routes, state changes, and the UX intent that
drove the design.

---

## Flow 1 — Initial setup

```
[Operator after fresh install]
        │
        │  starts the server, reads setup token from stderr
        ▼
   Browser → /setup
        │
        │  gate check: sui_id_setup cookie present? install initialised?
        │
   ┌────┴────┬───────────┬────────────┐
   │         │           │            │
 Allowed  Closed       Locked    AllowedDev (--dev)
   │         │           │            │
   │      "Setup is   "Setup is   wizard +
   │       closed"     locked"    dev-disclosure
   │                                   │
   │                                   │ same path as Allowed
   └─────────┬─────────────────────────┘
             │
             ▼
        Welcome card
             │
             │  click "Continue"
             ▼
        /setup/admin  (GET)
             │
             │  fill username + email + password
             │  submit POST /setup/admin
             ▼
        state.users.create(NewUser)
             │
             │  cookie sui_id_setup_uid = u_<new>; path=/setup
             │  303 to /setup/security
             ▼
        /setup/security (GET)
             │
             │  pick HIBP mode + default locale
             │  submit POST /setup/security
             ▼
        state.settings.set("hibp_mode", ...)
        state.settings.set("default_lang", ...)
        state.users.get(uid_from_cookie)
        state.mail.send("admin.first_admin", ...)
        config::mark_initialised()
        clear both setup cookies
             │
             │  303 to /setup/done
             ▼
        /setup/done — "Sign in" link
             │
             ▼
        /login
             │
             │  (subsequent /setup hits now show Closed)
```

### UX intent

- **Operator never sees the token in URLs or logs after setup.** It
  lives only in a path-scoped cookie.
- **The wizard is one-way.** Cancelling and revisiting `/setup`
  before completion is supported (the gate stays Allowed), but
  completing `/setup/security` closes it permanently.
- **Dev mode is loud.** The `--dev` flag bypasses the token check
  but discloses the bypass via a banner. There is no way to be in
  dev mode and not see this.

---

## Flow 2 — Administrator daily operation

```
[Authenticated admin]
        │
        ▼
   /admin (dashboard)
        │
        │  sidebar nav (always present, left-anchored)
        │
   ┌────┼─────────────────────────────────┐
   │    │                                 │
   ▼    ▼                                 ▼
Users  Clients   Security  Settings    Audit
 │       │         │          │          │
 ▼       ▼         ▼          ▼          ▼
list   list      page       tabs       table
 │       │
 ▼       ▼
detail detail
 │       │
 │       │  (top: read surface)
 │       │
 │       │  (bottom: danger zone)
 │       ▼
 │     [Rotate secret] / [Delete]
 │       │
 ▼       │
[Suspend]/[Resume]/[Delete]
         │
         │  every danger button:
         │   href="/stepup?action=X&return_to=Y"
         ▼
       /stepup (see Flow 5)
```

### UX intent

- **The sidebar is the navigation backbone.** It never changes
  position across pages. `aria-current="page"` marks the active
  section.
- **Detail pages have a fixed information shape**: read surface on
  top, danger zone at the bottom. The physical separation is
  intentional — operators visually scan from top down and rarely
  reach the danger zone by accident.
- **List → detail → action is the universal pattern.** Operators
  learn it once.

---

## Flow 3 — OIDC authorization

```
[External relying party]
        │
        │  302 user to /authorize?
        │      client_id=...&redirect_uri=...&scope=...&state=...&code_challenge=...
        ▼
   /authorize (GET)
        │
        │  validate params (real impl);
        │  check session
        │
   ┌────┴────┐
   │         │
authenticated   not authenticated
   │             │
   │             ▼
   │         /login (return_to=/authorize?...)
   │             │
   │             ▼
   │         /mfa  (if enabled)
   │             │
   │             ▼
   │         back to /authorize
   ▼
 /consent (GET)
        │
        │  render: client name, scopes itemised in plain language,
        │  approve / deny buttons
        │
   ┌────┴────┐
   │         │
[Approve]  [Deny]
   │         │
   ▼         ▼
303 to redirect_uri    303 to redirect_uri
   ?code=...               ?error=access_denied
   &state=...              &state=...
   │
   ▼
audit row: client.consent.grant (or .deny)
```

### UX intent

- **The user never sees the OIDC machinery.** They see "App X wants
  to access Y, Z, W. Allow / Deny."
- **Scopes are itemised in user-readable strings.** Implementation
  must maintain a scope → human description map. Raw scope strings
  (`openid`, `profile`, `email`) are never shown alone.
- **Deny is a first-class option**, visually equivalent to approve.
  Not a small "no thanks" link.
- **The redirect back is the success signal.** No transient toast,
  because the user is already on a different domain.

---

## Flow 4 — User authentication + MFA

```
[End user]
        │
        ▼
   /login (GET)
        │
        │  username + password
        │  POST /login
        ▼
   ┌────┴────────────────────┐
   │                         │
authenticate                fail (no enumeration)
   │                         │
   ▼                         ▼
                       same page,
                       "Sign-in failed" banner
   │
   │  MFA required?
   │
   ┌────┴────┐
   │         │
  No       Yes
   │         │
   │         ▼
   │      /mfa (GET)
   │         │
   │         │  6-digit TOTP or recovery code
   │         │  recovery-code link visible
   │         │  POST /mfa
   │         ▼
   │      ┌──┴────────────────┐
   │      │                   │
   │   verify             fail (generic)
   │      │                   │
   │      ▼                   ▼
   │  recovery code?     "Code did not match"
   │      │
   │      ├─yes──► counter--
   │      │        (visible later on /me/security?tab=mfa)
   │      │
   │      ▼ no
   │  proceed
   │
   ▼
303 to return_to OR /admin
        │
        ▼
audit row: auth.login.success or auth.mfa.success
```

### UX intent

- **Error wording is identical** whether the username doesn't exist,
  the password is wrong, or the account is locked. Three different
  failure modes, one user-facing message.
- **MFA failure is also generic.** Never reveal whether the user
  is enrolled.
- **Recovery codes are visibly counted.** On `/me/security?tab=mfa`
  the user sees "8 / 10 remaining" so they notice consumption.
- **MFA is its own route, not a modal.** Because re-loading or
  cancelling should be possible without leaving any partial-auth
  state.

---

## Flow 5 — Step-up + confirmation (the critical-action path)

```
[Admin or end user]
        │
        │  clicks a danger-zone button
        │
        ▼
GET /stepup?action=<key>&return_to=<path>[&target=<id>]
        │
        │  validate action key against DANGEROUS_ACTIONS registry
        │  render re-auth form (password or MFA depending on policy)
        ▼
POST /stepup
        │
        │  ┌── action key unknown ──► 303 /admin (silently)
        │  │
        │  ├── re-auth fails ───────► same page with error
        │  │
        │  └── re-auth succeeds
        │         │
        │         ▼
        │      state.sessions.store_ticket(action_key, params, return_to)
        │         │
        │         │  returns tk_<random>
        │         ▼
        │      303 to /confirm/tk_<random>
        │
        ▼
GET /confirm/{token}
        │
        │  state.sessions.peek_ticket(token)  (non-destructive)
        │
        │  ┌── ticket missing/expired ──► 410 page
        │  │
        │  └── ticket fresh
        │         │
        │         ▼
        │      render the impact-summary page:
        │        - action name (h1)
        │        - action class (Reversible / Undoable now / Destructive / Irreversible)
        │        - impact list (e.g. "Revokes 3 sessions including this device")
        │        - [Cancel] (link to return_to) — [Proceed] (button)
        │
POST /confirm/{token}
        │
        │  state.sessions.consume_ticket(token)  (one-shot)
        │
        │  ┌── ticket missing/expired/already consumed ──► 303 /admin
        │  │
        │  └── ticket fresh
        │         │
        │         ▼
        │      execute the action against the relevant service
        │      append audit row (e.g. user.suspend / signing_key.retire)
        │         │
        │         ▼
        │      303 to ticket.return_to
        │
        ▼
return_to page (e.g. /admin/users/u_charlie)
        │
        │  banner: "Action completed"
        ▼
audit row visible at /admin/audit
```

### UX intent

- **The impact-summary page is the user's last chance to bail.** It
  must read like a confirmation, not a continuation. The action
  name is in `<h1>`; the impact is a `<ul>`; the buttons are visually
  asymmetric (Cancel is `btn--ghost`, Proceed is the action's
  class colour).
- **Tickets are one-shot.** A user who hits Back after success and
  re-submits gets a 303 to `/admin`, not a re-execution.
- **TTL is implementation-internal.** The user sees `Some(ticket)`
  (fresh) or 410 (expired/unknown) — they never see "expires in
  N minutes". This keeps the UI calm.
- **`return_to` is the continuity guarantee.** The user always
  comes back to the page that originated the action.

---

## Cross-flow concerns

### Session lifetime

- Sessions are server-side, cookie-id-keyed.
- Idle timeout shown on `/me/security?tab=sessions`.
- FIFO eviction (RFC 014): when the per-user session cap is reached,
  the oldest is evicted.
- The current device cannot revoke itself — it would log the user
  out of the very page they're on.

### Locale and theme

- Both are cookie-set, never JS-set, never localStorage.
- Switching locale or theme: 303 redirect back to `return_to`.
- Both work without JavaScript.
- Defaults: locale from `Accept-Language` then `ja`; theme from
  `prefers-color-scheme` then `auto`.

### Failure rendering

- **Inline errors** appear for field-level validation failures.
- **Banner errors** appear for form-level failures (e.g. "Sign-in
  failed").
- **System pages** (`/400` / `/403` / `/404` / `/410` / `/429` /
  `/500`) appear for envelope-level failures and always include an
  investigation ID.

### Audit emission

- **Every** state-changing operation produces an audit row.
- Audit emission is the service's responsibility (RFC 020 §"audit
  emission"). Handlers do not call `audit.emit(...)` directly; the
  write happens inside the service method.
- Cascade rows (one operation causing several, e.g.
  `user.suspend` → `session.revoke` for each session) are linked
  via the `cause` field.

---

## Diagram conventions used above

- `─►` action / transition
- `┌──┐ │ ▼` plain ASCII flow
- `[bracketed]` user choice or destination
- `state.X.Y()` a `sui-id-core` service-trait call

---

## Where flows live in code

| Flow | Handler file(s) |
| --- | --- |
| Setup | `handlers/setup.rs` |
| Login / MFA | `handlers/auth.rs` |
| Forgot password | `handlers/password_reset.rs` |
| Authorize / Consent | `handlers/oidc.rs` |
| Admin daily ops | `handlers/admin.rs`, `users.rs`, `clients.rs`, `security.rs`, `settings.rs`, `audit.rs` |
| Self-service | `handlers/me.rs` |
| Step-up + confirm | `handlers/stepup.rs` |
| Errors | `handlers/error.rs` |
