//! `server_settings` table — singleton row, see migration 0016.
//!
//! Holds process-wide settings configurable by an admin without a
//! restart. Today this is just `default_lang`; future settings
//! (UI theme defaults, password-policy knobs, etc) extend the row
//! without needing a new migration.
//!
//! The row is keyed on the literal string `'singleton'` and is
//! INSERTed as part of migration 0016 with conservative defaults,
//! so [`get`] is `Result<ServerSettingsRow>` rather than
//! `Result<Option<…>>` — the row always exists once migrations
//! have run.

use crate::{models::ServerSettingsRow, Database, StoreError, StoreResult};
use chrono::{DateTime, Utc};
use rusqlite::params;

const SINGLETON_ID: &str = "singleton";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ServerSettingsRow> {
    Ok(ServerSettingsRow {
        default_lang: row.get(1)?,
        created_at: row.get::<_, DateTime<Utc>>(2)?,
        updated_at: row.get::<_, DateTime<Utc>>(3)?,
    })
}

const SELECT_COLUMNS: &str = "id, default_lang, created_at, updated_at";

/// Fetch the singleton server-settings row. Migration 0016 inserts
/// the default row, so post-migration this never returns NotFound.
pub fn get(db: &Database) -> StoreResult<ServerSettingsRow> {
    db.with_conn(|conn| {
        conn.query_row(
            &format!("SELECT {SELECT_COLUMNS} FROM server_settings WHERE id = ?1"),
            [SINGLETON_ID],
            map_row,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound,
            other => StoreError::from(other),
        })
    })
}

/// Update the server default UI language. `lang` is a BCP-47 tag
/// — application-layer validation should ensure it is one of
/// `Locale::ALL` before calling.
pub fn update_default_lang(
    db: &Database,
    lang: &str,
    now: DateTime<Utc>,
) -> StoreResult<()> {
    db.with_conn(|conn| {
        let n = conn.execute(
            "UPDATE server_settings SET default_lang = ?1, updated_at = ?2 WHERE id = ?3",
            params![lang, now, SINGLETON_ID],
        )?;
        if n == 0 {
            // Should never happen — migration 0016 inserts the
            // row. If it does, the database is in a broken state
            // and we'd rather surface that than silently re-INSERT.
            Err(StoreError::NotFound)
        } else {
            Ok(())
        }
    })
}
