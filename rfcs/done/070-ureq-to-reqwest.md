# RFC 070 — ureq → reqwest migration

**Status.** Implemented (v0.57.1)
**Priority.** P3 — implemented alongside RFC 069 at user request.
**Tracks.** Dependency strategy; HIBP outbound HTTP.
**Touches.** `crates/sui-id-core/src/hibp.rs`, workspace `Cargo.toml`,
`crates/sui-id/src/handlers/setup.rs`.

## Implementation note (v0.57.1)

Implemented alongside RFC 069. The P3 "deferred" status was overridden
by explicit user request.

### Changes made

**`HibpClient` trait** — made async via `#[async_trait::async_trait]`:
```rust
// Before (sync, spawn_blocking required)
pub trait HibpClient: Send + Sync {
    fn check(&self, password: &str) -> HibpCheckOutcome;
}

// After (async, no wrapper needed)
#[async_trait::async_trait]
pub trait HibpClient: Send + Sync {
    async fn check(&self, password: &str) -> HibpCheckOutcome;
}
```

**`HttpHibpClient`** — rebuilt on `reqwest::Client`:
- `reqwest::Client` built at construction time with a 5-second timeout
- `client.get(&url).header(...).send().await?.text().await?`
  replaces the ureq agent builder + synchronous `.call()`
- No `spawn_blocking` needed; the handler's `enforce_hibp` call
  is simply `.await`ed

**`enforce_hibp`** — `client.check(password)` now properly awaited:
```rust
// Before: client.check(password) — sync call inside async fn (bug!)
// After:  client.check(password).await — correct
```
Note: the previous code was inadvertently blocking the tokio runtime
on the ureq HTTP request. This is now fixed as a side-effect.

**All test impls** (`InMemoryHibpClient`, `StubBreached`, `StubClean`,
`StubUnavailable`) updated with `#[async_trait::async_trait]` and
`async fn check`.

**`ureq`** removed from all Cargo.toml files.
**`reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }`**
and **`async-trait = "0.1"`** added to workspace and `sui-id-core`.

**`setup.rs`** — `spawn_blocking` wrapper comment removed; the handler
calls `enforce_hibp(...).await` directly (it always did await it;
now the inner check is also properly async).

### Outcome

- `cargo check --workspace` clean.
- **228/228 library tests pass**.
- `ureq` no longer appears anywhere in the dependency tree.
- HIBP check is now fully async: no thread pool usage for network I/O.

---

## Background

The project currently makes **one outbound HTTP call** — the Have I Been
Pwned (HIBP) k-anonymity prefix lookup in `hibp.rs`. For this single call,
`ureq 2.x` (synchronous, blocking, rustls-backed) was deliberately chosen
over `reqwest` (async, hyper-backed). The reasoning at time of writing:

1. **Cold path.** One GET per password-set operation; only runs during the
   setup wizard as of v0.24.0.
2. **Synchronous API is simpler at one call site.** `ureq::get(url).call()?`
   vs. constructing a `reqwest::Client`, awaiting the response, and so on.
3. **Lighter footprint.** ureq does not pull in `hyper`; it reuses `rustls`
   already in the tree via `wasm-smtp-tokio`.
4. **Blocking fits `spawn_blocking`.** The handler wraps the call in
   `tokio::task::spawn_blocking`, which is idiomatic for a one-off blocking
   I/O operation. Using `reqwest` async inside `spawn_blocking` would mean
   driving a `Future` inside a blocking thread, requiring nested runtime
   plumbing.

These reasons remain valid as long as HIBP is the only outbound call.

---

## Why this RFC exists now

Two trigger conditions are visible on the roadmap:

1. **ureq 3.x is a major breaking change.** ureq 2.12 → 3.x (MSRV 1.85,
   MSRV-compatible with our toolchain) redesigned the builder API. If we
   update ureq for its own sake, we are paying a migration cost that buys
   nothing new. At that migration cost point, it is worth asking whether
   reqwest should replace it instead.

2. **Outbound HTTP will grow.** RFC 004 (federation as upstream OIDC
   client) requires outbound OIDC discovery (`/.well-known/openid-configuration`
   GET) and token exchange (POST). Any webhook delivery feature, any
   external token introspection, and any back-channel logout notification
   add more call sites. At two or more async call sites the cost model
   inverts: reqwest's async client amortises its setup overhead and
   simplifies the calling code (no `spawn_blocking`).

---

## Decision criteria: when to migrate

Migrate from ureq to reqwest when **at least one** of the following is true:

- A second outbound HTTP call site is added to the codebase (see RFC 004,
  federation; any webhook RFC; back-channel logout).
- ureq is itself updated to 3.x, making the migration cost equivalent.
- The project adopts an HTTP client interface (trait object / mock) for
  testing, where reqwest is the obvious concrete implementation.

Do **not** migrate while HIBP is the only call site, because:
- reqwest (async) adds `hyper` + `hyper-util` + `http-body-util` to the
  dependency tree with no runtime benefit.
- The `spawn_blocking` pattern is not wrong; it is idiomatic Tokio for
  rare blocking calls.

---

## Design (when triggered)

### Cargo.toml

```toml
# Remove:
ureq = { version = "2", default-features = false, features = ["tls"] }

# Add:
reqwest = { version = "0.12", default-features = false,
            features = ["rustls-tls", "json"] }
```

`rustls-tls` reuses `rustls` already in the tree (same as ureq's current
TLS stack). `json` enables the `Response::json::<T>()` helper, which
replaces ureq's manual text-read + line-split pattern in hibp.rs.

### hibp.rs rewrite

The production `HttpHibpClient::check_password` method changes from:

```rust
// Before (ureq, synchronous, wrapped in spawn_blocking)
let agent = ureq::AgentBuilder::new()
    .timeout(Duration::from_secs(5))
    .build();
let body = agent.get(&url).call()?.into_string()?;
```

to:

```rust
// After (reqwest, async, no spawn_blocking needed)
let body = self.client
    .get(&url)
    .timeout(Duration::from_secs(5))
    .send()
    .await?
    .text()
    .await?;
```

`HttpHibpClient` gains a `reqwest::Client` field (constructed once, at
server startup, and cloned per handler invocation — reqwest clients are
`Clone + Send + Sync`).

### spawn_blocking removal

The `spawn_blocking` wrapper in `handlers/setup.rs` is removed. The handler
becomes a standard async function.

### HttpHibpClient test surface

The existing `MockHibpClient` used in tests is unaffected — it bypasses the
HTTP layer entirely. No test changes required unless the client constructor
signature changes.

---

## Migration boundary condition

If this RFC is triggered because a **new outbound HTTP feature** is being
added (e.g., RFC 004 federation), the right sequencing is:

1. This RFC (ureq → reqwest) as a prerequisite or in the same release.
2. The triggering feature RFC adds its call sites on top of reqwest.
3. `ureq` is removed from `Cargo.toml` and the lock.

If triggered by ureq 3.x upgrade cost alone (no new feature), ship this RFC
standalone before the new feature lands.

---

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| reqwest + hyper increase binary size / compile time | Low–medium | Acceptable trade-off once a second call site exists; compile time impact is modest with LLD and incremental builds |
| TLS handshake behaviour differs from ureq | Very low | Both use rustls with the same root store; behaviour is identical |
| reqwest::Client is accidentally dropped (per-request construction) | Medium | Enforce via constructor injection into the `AppState` — the same pattern already used for database pools |
| HIBP test coverage gaps | Low | Mock is already in place; integration test for the real client is also present |

---

## Acceptance criteria

- [ ] `ureq` is removed from `Cargo.toml`; `reqwest` added with
  `rustls-tls` feature.
- [ ] `hibp.rs` `HttpHibpClient::check_password` is async; `spawn_blocking`
  wrapper removed from `handlers/setup.rs`.
- [ ] `reqwest::Client` is constructed once at startup and stored in
  `AppState` (or the HIBP client wrapper).
- [ ] All existing HIBP tests pass (mock client path unchanged).
- [ ] `cargo check --workspace` clean; 228/228 library tests pass.
- [ ] `reqwest` is not duplicated in `Cargo.lock` (single instance, not two
  conflicting major versions).
