//! Backup public types.

use serde::{Deserialize, Serialize};

/// Magic bytes at the head of an encrypted envelope. Lets `restore`
/// distinguish encrypted from plain at a single read of the first 8
/// bytes, without the operator having to remember which kind they
/// supplied.

/// Argon2id parameters for passphrase → AEAD key derivation.
/// 64 MiB / 3 iterations / 1 thread is well above the 19 MiB minimum
/// recommended by OWASP for password storage and well below anything
/// that would push backup creation past a couple of seconds on
/// reasonable hardware.

/// Provenance metadata written into every backup. `restore` consults
/// `format_version` and `schema_version` before doing anything
/// destructive; everything else is for the operator to read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format_version: u32,
    pub sui_id_version: String,
    pub schema_version: i64,
    pub created_at: String,
    pub hostname: String,
    pub issuer: String,
}

#[derive(Debug, Default, Clone)]
pub struct BackupOptions {
    /// When `Some`, the backup is encrypted under a key derived from
    /// the passphrase. When `None`, a plain tarball is produced.
    pub passphrase: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RestoreOptions {
    pub force: bool,
    /// Required when the backup file is encrypted. Optional otherwise
    /// (a plain tarball is accepted with `passphrase = None`).
    pub passphrase: Option<String>,
}

/// Result of `verify-backup` — purely informational.
#[derive(Debug, Clone)]
pub struct VerifyReport {
    pub manifest: Manifest,
    pub encrypted: bool,
    /// Total bytes of the tar (post-decrypt if encrypted).
    pub tar_bytes: usize,
    /// Bytes of the inner SQLite snapshot.
    pub db_bytes: usize,
    /// Whether the master key entry is present.
    pub key_present: bool,
}
