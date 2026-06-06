# Implementation Notes — sui-id Mockup

Practical notes for integrating the mockup into the real service.
Read this **after** `HANDOFF.md`.

---

## 1. Workspace and crates

The mockup ships as a three-crate Cargo workspace:

```
sui-id-web-mockup/
├── crates/
│   ├── sui-id-core/      ← Domain types + service traits + Error
│   ├── sui-id-mail/      ← Typed mail contexts + template registry
│   └── sui-id-web/       ← Leptos SSR handlers + in-memory mock impls
├── docs/
└── rfcs/done/            ← 21 design RFCs (load-bearing)
```

The intended path for the real implementation:

- **`sui-id-core`** is **dependency-light and reusable.** Adopt it
  as-is. Domain types, error variants, and service traits stay
  identical.
- **`sui-id-mail`** is **template + renderer code.** Adopt the
  typed-context approach and the registry shape. The renderer
  (minijinja) is the implementation's choice.
- **`sui-id-web`** has two halves: the **handlers** (UI code, adopt)
  and the **`mock/` directory** (in-memory impls — replace with real
  backend).

### Recommended target layout

```
sui-id/
├── crates/
│   ├── sui-id-core/          ← adopted unchanged
│   ├── sui-id-mail/          ← adopted; add lettre transport
│   ├── sui-id-store-sqlite/  ← NEW — real impls of the 9 traits
│   ├── sui-id-store-postgres/← NEW — optional, same shape
│   └── sui-id-web/           ← handlers from mockup; drop the mock/ dir
└── bin/
    └── sui-id/
        └── main.rs           ← AppState::new_with_sqlite(...)
```

---

## 2. The trait seam — what to keep, what to extend

Nine traits in `sui-id-core/src/traits/`. Each is `Send + Sync`
and uses `#[async_trait]`. The full surface as of v0.4.6:

### UserService

```rust
async fn list(&self, filter: UserFilter) -> Result<Vec<User>>;
async fn get(&self, id: &UserId) -> Result<Option<User>>;
async fn create(&self, new: NewUser) -> Result<UserId>;
```

**Likely additions for the real impl**:
- `suspend(&UserId) -> Result<()>`
- `resume(&UserId) -> Result<()>`
- `delete_logical(&UserId) -> Result<()>`
- `set_preferred_lang(&UserId, Locale) -> Result<()>`
- `change_password(&UserId, new_hash: String) -> Result<()>`
- Pagination on `list` (cursor or page+size)

### ClientService

```rust
async fn list(&self) -> Result<Vec<Client>>;
async fn get(&self, id: &ClientId) -> Result<Option<Client>>;
```

**Likely additions**:
- `create`, `update`, `delete`
- `rotate_secret(&ClientId) -> Result<NewSecret>`

### SessionService

```rust
async fn list_for_user(&self, user: &UserId) -> Result<Vec<Session>>;
async fn store_ticket(&self, action_key: String, params: BTreeMap<String,String>, return_to: String) -> Result<String>;
async fn peek_ticket(&self, id: &str) -> Result<Option<StepUpTicket>>;
async fn consume_ticket(&self, id: &str) -> Result<Option<StepUpTicket>>;
```

**Likely additions**:
- `revoke(&SessionId) -> Result<()>`
- `revoke_all_for_user(&UserId) -> Result<u32>`
- `current_session_id(&self) -> ...` — implementation-defined

### AuditService

```rust
async fn list_recent(&self, limit: usize) -> Result<Vec<AuditEntry>>;
async fn chain_status(&self) -> Result<ChainStatus>;
async fn last_verified(&self) -> Result<String>;
```

**Likely additions**:
- `verify_chain(&self) -> Result<ChainStatus>` — runs the
  cryptographic verification
- `list_for_target(...)` — for showing audit excerpts on detail
  pages
- `emit(&self, row: NewAuditRow) -> Result<AuditId>` — used by
  other services internally

### SettingsService

```rust
async fn get(&self, key: &str) -> Result<Option<String>>;
async fn set(&self, key: &str, value: &str) -> Result<()>;
```

**Likely additions**:
- `get_all(&self, prefix: Option<&str>) -> Result<Vec<(String, String)>>`
- Typed helpers (e.g. `hibp_mode() -> HibpMode`)

### KeyService

```rust
async fn list(&self) -> Result<Vec<SigningKey>>;
```

**Likely additions**:
- `publish(&self) -> Result<KeyId>`
- `activate(&KeyId) -> Result<()>`
- `retire(&KeyId) -> Result<()>`
- `delete(&KeyId) -> Result<()>`
- `current_active(&self) -> Result<SigningKey>` — for actual JWT
  signing

### MfaService

```rust
async fn status_for_user(&self, user: &UserId) -> Result<MfaStatus>;
```

**Likely additions**:
- `enroll_totp(&UserId, secret: String) -> Result<()>`
- `verify_totp(&UserId, code: &str) -> Result<bool>`
- `disable(&UserId) -> Result<()>`
- `regenerate_recovery_codes(&UserId) -> Result<Vec<String>>`
- Passkey methods (`register`, `verify`, `list`, `delete`)

### MailService

```rust
async fn send(&self, template_key: &str, recipient: &str, context_json: &str) -> Result<()>;
```

The current shape is **sufficient as a seam**. The implementation
substitutes the no-op with a real `lettre`-backed sender. The
context is JSON because templates take typed contexts and the
trait doesn't need to know them — only the renderer does.

### HibpService

```rust
async fn check(&self, password: &str) -> Result<HibpResult>;
```

The implementation must hash locally (SHA-1) and only transmit the
prefix to the HIBP API, per the k-anonymity contract. **Never
transmit plaintext.** The `password` parameter never leaves the
process.

---

## 3. AppState wiring

The current `AppState` in `crates/sui-id-web/src/app_state.rs`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub users:    Arc<dyn UserService>,
    pub clients:  Arc<dyn ClientService>,
    pub sessions: Arc<dyn SessionService>,
    pub audit:    Arc<dyn AuditService>,
    pub settings: Arc<dyn SettingsService>,
    pub keys:     Arc<dyn KeyService>,
    pub mfa:      Arc<dyn MfaService>,
    pub mail:     Arc<dyn MailService>,
    pub hibp:     Arc<dyn HibpService>,
    pub now:      Arc<dyn Clock>,
}

impl AppState {
    pub fn new_with_mock() -> Self { ... }  // current
}
```

For the real implementation, **add** a new constructor:

```rust
impl AppState {
    pub async fn new_with_sqlite(db_path: &str) -> Result<Self> {
        let pool = SqlitePool::connect(db_path).await?;
        Ok(Self {
            users:    Arc::new(SqliteUserService::new(pool.clone())),
            clients:  Arc::new(SqliteClientService::new(pool.clone())),
            // ... etc.
            mail:     Arc::new(LettreMailService::new(...)),
            now:      Arc::new(SystemClock),
        })
    }
}
```

**Handlers don't change.** The binary chooses the constructor:

```rust
let state = if cli.use_mock {
    AppState::new_with_mock()
} else {
    AppState::new_with_sqlite(&cli.db_path).await?
};
let app = build_router(state);
```

---

## 4. Handler structure (do not over-abstract)

Each handler is a free function in `crates/sui-id-web/src/handlers/`.
Pattern:

```rust
pub async fn list(
    State(state): State<AppState>,
    LocaleExtractor(loc): LocaleExtractor,
    ThemeExtractor(theme): ThemeExtractor,
) -> Html<String> {
    let users = state.users.list(UserFilter::all()).await.unwrap_or_default();
    render_admin(loc, theme, "/admin/users", AdminNav::Users, "admin", move |s| {
        view! { /* ... */ }
    })
}
```

Extractor order matters: **State first, then `FromRequestParts`
extractors, then `FromRequest` (Form / body) extractors last.**

Each form has its own deserialiser struct (`AdminForm`,
`SecurityForm`, `StepUpForm`). **Do not** merge into a generic
form pipeline. The redundancy is intentional — each form's fields
are validated by its own shape.

---

## 5. Render functions

Two shells:

- **`render_simple`** — narrow centred shell. Used for setup, login,
  MFA, forgot-password, OIDC consent, step-up, confirmation, system
  errors.
- **`render_admin`** — full admin shell with sidebar. Used for
  `/admin/*` and `/me/*`.

Both take a closure of type `FnOnce(&'static Strings) -> impl IntoView`
that produces the page-specific content. The shells handle the
shared chrome (header, footer, sidebar, dev banner if applicable).

**Do not flatten the shells into the handlers.** They're called
from many places; the common chrome (banner, locale switcher, theme
switcher) is the consistency mechanism.

---

## 6. i18n strings

All UI strings live in `crates/sui-id-web/src/i18n.rs` as a
language-resolved `&'static Strings` table. There are two
languages: `ja` (Japanese, default) and `en` (English).

Adding a string:

1. Add the field to `pub struct Strings`.
2. Add the Japanese value to `JA: Strings = ...`.
3. Add the English value to `EN: Strings = ...`.
4. Reference as `s.your_field` in the view.

**Anti-enumeration wording is normative.** When translating
`auth_login_failed`, `auth_mfa_failed`, `forgot_password_done`, etc.,
preserve the property that they don't disclose whether the account
exists. This applies per locale.

---

## 7. Audit emission — service responsibility

Per RFC 020 §"audit emission is the service's responsibility":

```rust
// CORRECT
async fn suspend(&self, user_id: &UserId) -> Result<()> {
    self.set_status(user_id, UserStatus::Suspended).await?;
    self.audit.emit(NewAuditRow {
        event: "user.suspend",
        target: user_id.to_string(),
        target_kind: TargetKind::User,
        ...
    }).await?;
    Ok(())
}

// INCORRECT
async fn handler(State(state): State<AppState>) {
    state.users.suspend(&id).await?;
    state.audit.emit(...).await?;  // ← handler should not do this
}
```

The implementation must structurally enforce this. Audit emission
inside service methods makes audit **inseparable** from the
state change — there's no path that mutates without auditing.

For the mock impl in `sui-id-web/src/mock/users.rs`, audit emission
is skipped because crossing the `Arc<dyn AuditService>` boundary
from one service impl to another is awkward. The real SQL backend
will own both tables and can write both in a single transaction.

---

## 8. Step-up action registry

Defined in `crates/sui-id-web/src/handlers/stepup.rs` as
`REGISTRY: &[DangerousAction]`. Each entry is:

```rust
DangerousAction {
    key: "user.suspend",
    label: |s| s.action.user_suspend_label,
    impact: |s| s.action.user_suspend_impact,
    class: ReversibilityClass::Reversible,
}
```

Adding a new dangerous action requires:

1. Add an entry to `REGISTRY`.
2. Add `label` and `impact` strings to `i18n.rs` (both locales).
3. The handler that wants to trigger it links to
   `/stepup?action=<key>&return_to=<path>`.

The registry is the **single source of truth** for what actions
can step-up. A POST to `/stepup` with an unknown action key
silently redirects to `/admin`.

---

## 9. Routing — what the mockup commits to

```rust
// from crates/sui-id-web/src/router.rs
.route("/", get(root_redirect))
.route("/setup",          get(setup::welcome).post(setup::welcome_submit))
.route("/setup/admin",    get(setup::admin_form).post(setup::admin_submit))
.route("/setup/security", get(setup::security_form).post(setup::security_submit))
.route("/setup/done",     get(setup::done))
.route("/login",  get(auth::login).post(auth::login_submit))
.route("/mfa",    get(auth::mfa).post(auth::mfa_submit))
.route("/stepup", get(stepup::stepup).post(stepup::stepup_submit))
.route("/confirm/{token}", ...)
.route("/forgot-password", ...)
.route("/admin",                get(admin::dashboard))
.route("/admin/users",          get(users::list))
.route("/admin/users/{id}",     get(users::detail))
.route("/admin/clients",        get(clients::list))
.route("/admin/clients/{id}",   get(clients::detail))
.route("/admin/security",       get(security::security))
.route("/admin/settings",       get(settings::settings).post(settings::settings_submit))
.route("/admin/audit",          get(audit::audit))
.route("/me/security", get(me::security))
.route("/authorize",   get(oidc::authorize))
.route("/consent",     get(oidc::consent).post(oidc::consent_submit))
// + system error routes + theme/locale routes
```

The route names are **stable design commitments**. The
`/me/security?tab=...` pattern (single page, query param chooses tab)
vs separate `/me/security/mfa`, `/me/security/sessions` is a
deliberate choice — see RFC 002 for the rationale.

---

## 10. Theming

CSS lives in `crates/sui-id-web/src/theme.rs` as a single inlined
stylesheet served with every page. The theme is decided server-side
from the `sui_id_theme` cookie (`auto` / `light` / `dark`), then
falls back to `prefers-color-scheme`.

**Don't move CSS to a separate file unless you also move to a
hashed-URL stylesheet** (cache busting). The inlined approach is
deliberately simple and works without JS.

Colour tokens are defined as CSS variables. Adding a new colour:

1. Add the var to `:root` (light values).
2. Add the override in `html[data-theme="dark"]` (dark values).
3. Reference as `var(--your-token)` in component CSS.

---

## 11. SSR and JavaScript

The mockup is **pure SSR**, no hydration. Every page is a complete
HTML document. Forms post; pages reload.

Two places where JS is unavoidable:

- **WebAuthn / passkey registration** (RFC 013) — the
  `navigator.credentials` API only exists in the browser.
  The mockup ships a `<noscript>` fallback offering TOTP enrollment
  instead. Real implementation must preserve this.
- **HIBP password-strength preview** — out of scope for mockup;
  if added later, must degrade gracefully.

The mockup **does not** use any JS for:

- Form validation (server-side after submit)
- Toggling UI state (server-side via redirects)
- Navigation (anchor tags)
- Modals (we don't have modals; we have separate routes)

Preserve this baseline. If the real implementation adds JS, the
no-JS path must still work.

---

## 12. Cookies in use

| Cookie | Path | Use | Set when |
| --- | --- | --- | --- |
| `sui_id_theme` | `/` | Theme preference | User clicks theme switcher |
| `sui_id_lang` | `/` | Locale preference | User clicks language switcher |
| `sui_id_setup` | `/setup` | Setup token gate | Bootstrap (out-of-band) |
| `sui_id_setup_uid` | `/setup` | Threads new admin UID through wizard | Setup admin step (v0.4.6+) |
| `sui_id_session` | `/` | Auth session | Login success (implementation-specific) |

The mockup doesn't manage real auth sessions — that's the
implementation's job. But the **cookie hygiene** (HttpOnly, SameSite,
path-scoping) is documented above and should be preserved.

---

## 13. Testing strategy

The mockup ships with:

- **52 unit / integration tests** across the three crates.
- **21 route walk tests** in `tests/routes.rs` that exercise every
  route at GET / POST level.
- **Per-trait unit tests** for each in-memory mock implementation.

Recommended additions for the real implementation:

- **Snapshot tests** of rendered HTML for key flows (login,
  consent, audit) — to catch unintended layout regression.
- **Integration tests with a real SQLite backend** mirroring the
  in-memory test set.
- **Property tests** on the audit chain (immutability under
  insertion order) per RFC 016.

---

## 14. Build commands

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Run the mockup binary (mock data)
cargo run -p sui-id-web -- --mock-init

# Run with setup gate open (dev mode)
cargo run -p sui-id-web -- --dev

# Configure port
PORT=3000 cargo run -p sui-id-web -- --mock-init
```

The mockup builds clean in **Rust 1.91** edition 2024 with
`unsafe_code = "forbid"`.

---

## 15. Things to do early (and things to defer)

### Do early

- Adopt the visual system (`theme.rs`). Visual regressions surface
  immediately on integration.
- Adopt `sui-id-core` and start mapping the backend to its traits.
  This is the long pole.
- Adopt the step-up + confirmation flow infrastructure. Wire one
  destructive action through it (e.g. user suspend) and validate
  the pattern.
- Adopt the system error pages with investigation IDs. Support
  benefits immediately.

### Defer

- Optimising the rendered HTML size — Leptos SSR + inlined CSS is
  fine for self-host scale.
- Adding caching headers — every page is dynamic; static caching
  is wrong here.
- Adding client-side state (Redux / Zustand / etc.) — the mockup
  is intentionally JS-free; do not add a SPA layer.
- Real-time updates (SSE / WebSocket) — see Q5 in `OPEN_ISSUES.md`.

### Don't do at all

- Don't merge `/admin/users` and `/admin/users/{id}` into one
  master-detail SPA page. The list → detail → action pattern is
  load-bearing.
- Don't modalize the step-up confirmation. The dedicated route is
  the safety boundary.
- Don't add a "remember me" checkbox to login without a security
  review.
- Don't move strings to a runtime translation service. Compile-time
  `&'static Strings` is the simplicity guarantee.

---

## 16. Where to ask questions

- **Mockup design intent** → consult the RFC of the relevant
  feature (`rfcs/done/`).
- **Wording rules** → RFC 003 + `i18n.rs` source.
- **Trait shape** → RFC 020 + `crates/sui-id-core/src/traits/`.
- **Anything else** → escalate per `HANDOFF.md` §14.

Good luck with the integration. The mockup's job is done; the
product's begins.
