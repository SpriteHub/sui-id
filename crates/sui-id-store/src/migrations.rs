//! Schema migrations.
//!
//! Migrations are embedded SQL strings, run in order at startup. The current
//! applied version is recorded in `sui_meta` under the key `schema_version`.
//! This is intentionally simpler than a full migration framework: minimal
//! configuration, easy to reason about during recovery.

use crate::errors::{StoreError, StoreResult};
use rusqlite::Connection;

struct Migration {
    version: i32,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("./migrations/0001_initial.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("./migrations/0002_client_scope_and_logout_uris.sql"),
    },
    Migration {
        version: 3,
        sql: include_str!("./migrations/0003_totp_mfa.sql"),
    },
    Migration {
        version: 4,
        sql: include_str!("./migrations/0004_webauthn.sql"),
    },
    Migration {
        version: 5,
        sql: include_str!("./migrations/0005_revoked_access_tokens.sql"),
    },
    Migration {
        version: 6,
        sql: include_str!("./migrations/0006_session_auth_methods.sql"),
    },
    Migration {
        version: 7,
        sql: include_str!("./migrations/0007_user_lockout.sql"),
    },
    Migration {
        version: 8,
        sql: include_str!("./migrations/0008_refresh_token_family.sql"),
    },
    Migration {
        version: 9,
        sql: include_str!("./migrations/0009_audit_hash_chain.sql"),
    },
    Migration {
        version: 10,
        sql: include_str!("./migrations/0010_session_step_up.sql"),
    },
    Migration {
        version: 11,
        sql: include_str!("./migrations/0011_audit_log_at_action_index.sql"),
    },
    Migration {
        version: 12,
        sql: include_str!("./migrations/0012_users_email.sql"),
    },
    Migration {
        version: 13,
        sql: include_str!("./migrations/0013_webauthn_step_up.sql"),
    },
    Migration {
        version: 14,
        sql: include_str!("./migrations/0014_smtp_config.sql"),
    },
    Migration {
        version: 15,
        sql: include_str!("./migrations/0015_password_reset_tokens.sql"),
    },
    Migration {
        version: 16,
        sql: include_str!("./migrations/0016_i18n.sql"),
    },
    Migration {
        version: 17,
        sql: include_str!("./migrations/0017_hibp_mode.sql"),
    },
    Migration {
        version: 18,
        sql: include_str!("./migrations/0018_session_limits.sql"),
    },
    Migration {
        version: 19,
        sql: include_str!("./migrations/0019_auth_flow_integrity.sql"),
    },
    Migration {
        version: 20,
        sql: include_str!("./migrations/0020_user_identity_invariants.sql"),
    },
    Migration {
        version: 21,
        sql: include_str!("./migrations/0021_schema_invariants.sql"),
    },
];

/// The highest schema version this build of sui-id-store knows how to
/// produce by running its bundled migrations. The backup-restore path
/// uses this to refuse a backup that was taken on a newer sui-id (the
/// migration to read it forward doesn't exist yet) — reversibly,
/// rebuild with a newer binary.
pub const MAX_SCHEMA_VERSION: i32 = {
    // Computed at compile-time from the MIGRATIONS slice. If you add a
    // new migration above, this picks up the new top automatically.
    let mut i = 0;
    let mut max = 0i32;
    while i < MIGRATIONS.len() {
        if MIGRATIONS[i].version > max {
            max = MIGRATIONS[i].version;
        }
        i += 1;
    }
    max
};

const META_KEY_SCHEMA_VERSION: &str = "schema_version";

/// Apply all pending migrations to `conn`.
pub fn run(conn: &mut Connection) -> StoreResult<()> {
    // Ensure the meta table exists before we ask it for its version. The
    // initial migration creates the table too (idempotent CREATE IF NOT
    // EXISTS), but we need to read from it before the migration runs.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sui_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )?;

    let current: i32 = conn
        .query_row(
            "SELECT value FROM sui_meta WHERE key = ?1",
            [META_KEY_SCHEMA_VERSION],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    for m in MIGRATIONS {
        if m.version <= current {
            continue;
        }
        tracing::info!(version = m.version, "applying migration");
        // Each migration runs inside its own transaction so that a
        // partial failure leaves schema_version un-bumped and the DB
        // in the pre-migration state. The next startup will retry the
        // same migration cleanly.
        //
        // Note: SQLite does not support VACUUM or some ATTACH DDL
        // inside a transaction. Migrations that need those are
        // responsible for their own recovery story and must not be
        // versioned migrations. None of the existing migrations use
        // such statements.
        let tx = conn.transaction().map_err(StoreError::from)?;
        tx.execute_batch(m.sql).map_err(StoreError::from)?;
        tx.execute(
            "INSERT OR REPLACE INTO sui_meta(key, value) VALUES(?1, ?2)",
            (META_KEY_SCHEMA_VERSION, m.version.to_string()),
        )?;
        tx.commit().map_err(StoreError::from)?;
    }
    Ok(())
}

/// Apply migrations up to and including `max_version`. Used in tests to
/// create a database at a known historical schema version so that a
/// subsequent migration can be applied manually and its data-preservation
/// behaviour verified.
#[cfg(test)]
pub(crate) fn run_up_to(conn: &mut Connection, max_version: i32) -> StoreResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sui_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )?;
    let current: i32 = conn
        .query_row(
            "SELECT value FROM sui_meta WHERE key = ?1",
            [META_KEY_SCHEMA_VERSION],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    for m in MIGRATIONS {
        if m.version <= current || m.version > max_version {
            continue;
        }
        let tx = conn.transaction().map_err(StoreError::from)?;
        tx.execute_batch(m.sql).map_err(StoreError::from)?;
        tx.execute(
            "INSERT OR REPLACE INTO sui_meta(key, value) VALUES(?1, ?2)",
            (META_KEY_SCHEMA_VERSION, m.version.to_string()),
        )?;
        tx.commit().map_err(StoreError::from)?;
    }
    Ok(())
}

/// Return the SQL for the migration at the given version. Panics if the
/// version does not exist — this is intentionally strict so that test
/// helper code fails loudly when migrations are renumbered.
#[cfg(test)]
pub(crate) fn sql_for_version(version: i32) -> &'static str {
    MIGRATIONS
        .iter()
        .find(|m| m.version == version)
        .unwrap_or_else(|| panic!("no migration with version {version}"))
        .sql
}
