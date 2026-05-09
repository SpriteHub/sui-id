//! JSON-TEXT column validation helpers (RFC 021 § 5).
//!
//! Every repository write path that stores application-controlled data in a
//! SQLite TEXT column that the application later deserialises as JSON must
//! call [`require_valid_json`] before executing the INSERT/UPDATE. This
//! converts a future silent-corruption scenario into a typed error at the
//! write site.
//!
//! Reads do not need this guard: serde_json deserialisation already fails
//! loudly on bad JSON. The guard is a *pre-condition* check on writes.

use crate::errors::{StoreError, StoreResult};
use serde::de::DeserializeOwned;

/// Validate that `json_str` deserialises successfully as `T`.
///
/// Returns `Ok(())` when valid. Returns `Err(StoreError::CorruptJson)`
/// when the string is not valid JSON or does not match the expected
/// shape `T`. Callers use this to gate INSERT/UPDATE calls:
///
/// ```ignore
/// require_valid_json::<Vec<String>>(&redirect_uris_json, "clients.redirect_uris")?;
/// conn.execute("INSERT INTO clients ...")?;
/// ```
///
/// The `context` string identifies the column for diagnostics and is
/// embedded in the error message.
pub fn require_valid_json<T: DeserializeOwned>(
    json_str: &str,
    context: &'static str,
) -> StoreResult<()> {
    serde_json::from_str::<T>(json_str)
        .map(|_| ())
        .map_err(|e| StoreError::CorruptJson { context, source: e })
}
