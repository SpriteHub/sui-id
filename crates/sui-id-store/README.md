# sui-id-store

Persistence layer for [sui-id](https://github.com/nabbisen/sui-id). Owns the
SQLite connection, runs schema migrations on startup, and exposes thin
repository functions for the domain layer in `sui-id-core`.

## Encryption model

Sensitive columns are sealed with XChaCha20-Poly1305 using a master key
kept *outside* the database. This avoids the heavy dependency tree of
SQLCipher while still preventing a stolen `.sqlite` file from yielding
plaintext refresh tokens or signing keys.

The master key never enters the database file; it is supplied externally
via the `SUI_ID_MASTER_KEY` environment variable or a separate key file.

## Status

This crate is an implementation detail of sui-id. Its API may change between
minor versions. For a working OIDC provider, install the binary instead:

```bash
cargo install sui-id
```

## License

Apache-2.0.
