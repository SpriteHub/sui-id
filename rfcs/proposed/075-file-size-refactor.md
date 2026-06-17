# RFC 075 — File-size refactor

**Status.** Proposed
**Priority.** P3 — non-blocking for v1.0, but the three affected files
exceed the project's 500-ELOC guidance and will become harder to navigate
as the codebase matures. Addressing them now (before v1.0 locks the
reference shape of the codebase) is preferable to deferring indefinitely.
**Tracks.** Verification-soak maintenance.
**Touches.** `crates/sui-id/src/backup.rs` → `backup/`,
`crates/sui-id/src/main.rs` → `cli.rs` extracted,
`crates/sui-id-core/src/admin.rs` → `admin/`. No behaviour changes;
no new public API; no schema changes. Pure refactor.

---

## Background

The project's file-splitting policy (development guidelines §File
Separation) states: consider splitting at 300 effective LoC, strongly
recommended at 500. Three source files exceed 500 production LoC as of
v0.61.0:

| File | Total LoC | Production LoC | Notes |
|---|---|---|---|
| `sui-id/src/backup.rs` | 1,063 | 626 | 437 in inline `#[cfg(test)]` |
| `sui-id/src/main.rs` | 825 | 825 | Mixes entry-point, CLI arg parsing, subcommand handlers, help text |
| `sui-id-core/src/admin.rs` | 785 | 785 | Three distinct domains (users, clients, signing keys) in one file |

All three files are individually cohesive — they do not violate single
responsibility in a deep sense — but they have grown to the point where
finding a specific function requires knowing roughly where in the file it
lives. The refactor is a mechanical split: no function bodies change.

---

## Design

### 1. `backup.rs` → `backup/`

#### Current structure (within the single file)

| Lines | Content |
|---|---|
| 1–71 | Imports, constants (`FORMAT_VERSION = 1`) |
| 72–130 | Public types: `Manifest`, `BackupOptions`, `RestoreOptions`, `VerifyReport` |
| 131–227 | `run_backup()` |
| 228–270 | `run_restore()` |
| 271–304 | `run_verify()` |
| 305–513 | Internal helpers: `is_encrypted`, `parse_backup`, `check_manifest_compatibility`, `encrypt_envelope`, `decrypt_envelope`, `derive_key`, `hostname_or_unknown`, `write_atomic`, `tempfile_dir` |
| 514–626 | Minimal hand-rolled ustar tar writer/reader: `write_tar_entry`, `write_tar_terminator`, `write_octal`, `read_tar`, `read_octal` |
| 627–1063 | `#[cfg(test)] mod tests { … }` |

#### Target structure

```
crates/sui-id/src/backup/
├── mod.rs        Public API re-exports only; ~30 lines
├── types.rs      Manifest, BackupOptions, RestoreOptions, VerifyReport; ~60 lines
├── ops.rs        run_backup, run_restore, run_verify; ~175 lines (public surface)
├── crypto.rs     encrypt_envelope, decrypt_envelope, derive_key, is_encrypted; ~140 lines
├── fs.rs         hostname_or_unknown, write_atomic, tempfile_dir; ~60 lines
├── tar.rs        Minimal ustar writer/reader; ~115 lines
└── tests.rs      All existing tests moved here; ~437 lines
```

**`backup/mod.rs`** — contains only re-exports and a module-level
doc comment explaining the backup design:

```rust
//! Encrypted backup and restore for sui-id (RFC 040).
//!
//! # Archive format
//! …
pub use ops::{run_backup, run_restore, run_verify};
pub use types::{BackupOptions, Manifest, RestoreOptions, VerifyReport};
pub(super) use crypto::{decrypt_envelope, derive_key, encrypt_envelope, is_encrypted};
pub(super) use fs::{hostname_or_unknown, tempfile_dir, write_atomic};
pub(super) use tar::{read_tar, write_tar_entry, write_tar_terminator};
pub const FORMAT_VERSION: u32 = 1;

mod types;
mod ops;
mod crypto;
mod fs;
mod tar;
#[cfg(test)] mod tests;
```

**`backup/types.rs`** — `Manifest`, `BackupOptions`, `RestoreOptions`,
`VerifyReport`. No functions; purely data types with their `derive`s and
doc comments. All four are `pub` (used by the CLI in `main.rs`).

**`backup/ops.rs`** — `run_backup`, `run_restore`, `run_verify`. Uses
`types`, `crypto`, `fs`, and `tar` via `super::*`. The three functions
share no mutable state; no further split of ops is warranted.

**`backup/crypto.rs`** — `is_encrypted`, `encrypt_envelope`,
`decrypt_envelope`, `derive_key`. All four are private to the `backup`
module (`pub(super)`). This is the only file in the split that touches
cryptographic primitives; isolating it makes security review simpler.

**`backup/fs.rs`** — `hostname_or_unknown`, `write_atomic`,
`tempfile_dir`. OS-interaction helpers that have nothing to do with the
backup format logic. `pub(super)`.

**`backup/tar.rs`** — The minimal hand-rolled ustar write/read
implementation (~115 lines). No dependency on `crypto` or `fs`. Can be
read and tested independently. `pub(super)`.

**`backup/tests.rs`** — The existing `#[cfg(test)] mod tests` body,
verbatim. Requires `use super::*` and the existing test utilities.

#### `main.rs` callsite change

`main.rs` currently imports from `crate::backup`. After the split,
`crate::backup` remains the module path (via `backup/mod.rs`), so
**`main.rs` needs zero changes** — the `run_backup`, `run_restore`,
`run_verify`, `BackupOptions`, `RestoreOptions` imports are all
re-exported from `mod.rs`.

---

### 2. `main.rs` → extract `cli.rs`

#### Current structure

| Lines | Content |
|---|---|
| 1–15 | Imports |
| 16–60 | `async fn main()` — arg routing, top-level error handler |
| 61–80 | `fn find_subcommand()` — CLI arg parsing helper |
| 81–158 | `async fn serve()` — production server startup |
| 159–297 | `async fn serve_dev()` — dev-mode startup (seeds DB, relaxed settings) |
| 298–408 | `fn run_backup_subcommand`, `fn run_restore_subcommand`, `fn run_verify_backup_subcommand` — back-end wrappers for the three backup CLI commands |
| 387–407 | `fn run_admin_subcommand()` — dispatches `unlock-user`, `rotate-key`, etc. |
| 409–651 | `async fn run_admin_unlock_user`, `async fn run_admin_rotate_key` — heavy admin DB operations |
| 652–699 | `fn read_passphrase`, `fn parse_config_path`, `fn parse_named_path` — shared CLI utilities |
| 700–806 | `fn print_help()` — full multi-section help text |
| 807–825 | `async fn shutdown_signal()` — OS signal handling |

#### Target structure

```
crates/sui-id/src/
├── main.rs   Entry-point only: main(), shutdown_signal(), find_subcommand(); ~90 lines
└── cli.rs    All CLI subcommands + help text + CLI utilities; ~735 lines
```

**`main.rs`** retains:
- `async fn main()` — routing and top-level error printing.
- `fn find_subcommand()` — tiny helper used only by `main`.
- `async fn shutdown_signal()` — Tokio signal handler used only in `serve`.
- `async fn serve()` + `async fn serve_dev()` — startup paths. These could
  move to `startup.rs`, but that file already exists and handles the server
  construction. The *invocation* of startup lives in `main.rs`; moving it
  would not meaningfully reduce `main.rs`. Leave for a later pass.

Actually, `serve` and `serve_dev` total ~220 lines. Moving them to `cli.rs`
puts the startup path with the CLI tooling, which is a weaker grouping
than leaving them in `main.rs`. The cleaner boundary is:

| Moves to `cli.rs` | Stays in `main.rs` |
|---|---|
| All `run_*_subcommand` functions (backup, restore, verify, admin) | `async fn main()` |
| `async fn run_admin_unlock_user` | `fn find_subcommand()` |
| `async fn run_admin_rotate_key` | `async fn serve()` |
| `fn read_passphrase` | `async fn serve_dev()` |
| `fn parse_config_path` | `async fn shutdown_signal()` |
| `fn parse_named_path` | imports |
| `fn print_help()` | |

This gives:
- `main.rs` ~320 lines (still long, but the dominant content is the
  `serve_dev` seed logic which is intrinsically long).
- `cli.rs` ~510 lines.

Both drop below 500 LoC production.

**`cli.rs`** — `pub(crate) mod cli` or a `pub(super)` module. Functions
within are `pub(crate)` so `main.rs` can call them after `mod cli`.

**`main.rs` change**: add `mod cli;` at the top; replace the function
bodies with `cli::run_backup_subcommand(args)`, etc.

---

### 3. `admin.rs` (core) → `admin/`

#### Current structure

| Lines | Content | Domain |
|---|---|---|
| 1–22 | Imports | — |
| 23–46 | `audit_ok`, `audit_with_note` | Shared audit helpers |
| 47–60 | `require_admin` | Auth guard |
| 61–342 | `CreateUserSpec`, `create_user`, `list_users`, `set_user_disabled`, `delete_user`, `MfaResetReport`, `admin_reset_mfa`, `reset_user_password` | **User operations** |
| 343–694 | `CreatedClient`, `CreateClientSpec`, `create_client`, `set_client_allowed_scopes`, `set_client_post_logout_redirect_uris`, `update_client_basic`, `get_client`, `list_clients`, `update_client`, `set_client_disabled`, `delete_client`, `rotate_client_secret` | **Client operations** |
| 609–693 | `list_signing_keys`, `rotate_signing_key`, `delete_signing_key` | **Signing key operations** |
| 695–785 | `validate_redirect_uri`, private helpers | Shared helpers |

#### Target structure

```
crates/sui-id-core/src/admin/
├── mod.rs          Shared helpers + re-exports; ~80 lines
├── users.rs        User admin operations; ~285 lines
├── clients.rs      Client admin operations; ~360 lines
└── signing_keys.rs Signing key admin operations; ~85 lines
```

**`admin/mod.rs`** — contains the shared pieces that all three
submodules depend on:

```rust
//! Admin-layer domain functions (RFC 075, v0.62.0).
//!
//! Split into three submodules by resource type. All entry-points
//! that were previously `pub` in the flat `admin.rs` remain `pub`
//! here via re-exports.

mod users;
mod clients;
mod signing_keys;

pub use users::{
    CreateUserSpec, MfaResetReport,
    create_user, list_users, set_user_disabled,
    delete_user, admin_reset_mfa, reset_user_password,
};
pub use clients::{
    CreatedClient, CreateClientSpec,
    create_client, get_client, list_clients,
    update_client, update_client_basic,
    set_client_allowed_scopes, set_client_post_logout_redirect_uris,
    set_client_disabled, delete_client, rotate_client_secret,
};
pub use signing_keys::{
    list_signing_keys, rotate_signing_key, delete_signing_key,
};

pub(crate) use self::require_admin;

use crate::db::Database;
use crate::errors::CoreResult;
use sui_id_shared::ids::UserId;

/// Guard: return Forbidden if `user_id` does not hold the admin role.
/// Used by every admin-layer function as the first call.
pub async fn require_admin(db: &Database, user_id: UserId) -> CoreResult<()> { … }

/// Append an audit record with result = "ok" and no note.
pub(crate) async fn audit_ok(…) { … }

/// Append an audit record with result = "ok" and a free-text note.
pub(crate) async fn audit_with_note(…) { … }
```

**`admin/users.rs`** — all user admin functions. Imports from `super`
for `audit_ok`, `audit_with_note`, `require_admin`.

**`admin/clients.rs`** — all client admin functions. Imports
`validate_redirect_uri` (stays private within this file; it was a
`fn` helper in `admin.rs` used only by client functions).

**`admin/signing_keys.rs`** — the three signing-key functions, which
are already self-contained and have no shared state with users or clients.

#### `admin.rs` import callsite impact

Every crate that imports from `sui_id_core::admin` imports a name like
`sui_id_core::admin::create_user`. Because `admin/mod.rs` re-exports
everything that was previously `pub` in `admin.rs`, **zero callsites
outside the crate change**. The internal refactor is invisible across
the crate boundary.

---

## Implementation plan

Each split is independent and can be reviewed separately. Suggested order:

1. `admin.rs` → `admin/` — purely internal to `sui-id-core`; smallest
   blast radius; zero callsite changes.
2. `backup.rs` → `backup/` — internal to `sui-id`; zero callsite changes
   in `main.rs` thanks to re-exports.
3. `main.rs` → extract `cli.rs` — visible change to `main.rs` callers
   but `main.rs` is the binary entry-point, so no library callers are affected.

---

## Testing strategy

All existing tests move verbatim with their enclosing module. After the split:

- `backup/tests.rs` — `use super::*` gives access to all three
  submodule contents via `mod.rs` re-exports.
- The `admin/` split has no inline tests in the current `admin.rs`; all
  admin tests live in `sui-id-core/src/tests.rs` or `tests/` submodules and
  reference the public API, which is unchanged.
- `cli.rs` has no tests; CLI integration tests remain in
  `crates/sui-id/tests/`.

Expected outcome: **same test counts, all passing**. CI invariants unchanged.

## Acceptance criteria

- [ ] Each of the three resulting `mod.rs` files is ≤ 100 lines.
- [ ] No file in the split exceeds 400 production LoC.
- [ ] `cargo check --workspace` clean after each individual split.
- [ ] All existing tests pass (same counts, no new tests required).
- [ ] All callsites outside the refactored crate are unchanged
  (verified by confirming zero diff in callers).
- [ ] CI invariants unchanged.

## Non-goals

- No behaviour changes — function bodies are moved verbatim.
- No new public API surface.
- No doc-comment rewrites beyond adjusting module-level comments.
- `serve()` and `serve_dev()` are left in `main.rs` for this RFC; a
  further pass (if warranted) is a separate RFC.
