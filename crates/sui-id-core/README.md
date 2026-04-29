# sui-id-core

[![crates.io](https://img.shields.io/crates/v/sui-id-core?label=rust)](https://crates.io/crates/sui-id-core)
[![Rust Documentation](https://docs.rs/sui-id-core/badge.svg?version=latest)](https://docs.rs/sui-id-core)
[![Dependency Status](https://deps.rs/crate/sui-id-core/latest/status.svg)](https://deps.rs/crate/sui-id-core)
[![License](https://img.shields.io/github/license/nabbisen/sui-id-core)](https://github.com/nabbisen/sui-id-core/blob/main/LICENSE)

Authentication and authorization core for
[sui-id](https://github.com/nabbisen/sui-id). Provides:

- Argon2id password hashing and verification.
- Ed25519 (EdDSA) JWT signing and verification.
- OAuth 2.0 Authorization Code with mandatory PKCE (S256).
- OAuth 2.0 Refresh Token grant with rotation on every use.
- OpenID Connect Discovery and JWKS document construction.
- Admin session lifecycle and the first-run setup state machine.

This crate has no knowledge of HTTP. It speaks in terms of `sui-id-store`
and pure data; the HTTP wiring lives in the `sui-id` binary crate.

## Status

This crate is an implementation detail of sui-id. Its API may change between
minor versions. For a working OIDC provider, install the binary instead:

```bash
cargo install sui-id
```

