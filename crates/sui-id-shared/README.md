# sui-id-shared

Shared types and DTOs used across the [sui-id](https://github.com/nabbisen/sui-id)
workspace.

This crate is an implementation detail of sui-id and intentionally has a
narrow surface area: typed identifiers (`UserId`, `ClientId`, …), the
public-facing JSON API DTOs, and the `ApiError` envelope. It does not
contain domain logic, storage, or HTTP code.

You generally do not depend on this crate directly. Install the binary
instead:

```bash
cargo install sui-id
```

See the [project README](https://github.com/nabbisen/sui-id) for an overview.

## License

Apache-2.0.
