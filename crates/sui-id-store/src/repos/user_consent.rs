//! `user_consent` repository — stored OIDC consent grants (RFC 038).

use chrono::{DateTime, Utc};
use sui_id_shared::ids::{ClientId, UserId};

use crate::db::Database;
use crate::errors::StoreResult;
use crate::models::UserConsentRow;

fn map(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserConsentRow> {
    Ok(UserConsentRow {
        user_id: row
            .get::<_, String>(0)?
            .parse()
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
        client_id: row
            .get::<_, String>(1)?
            .parse()
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e)))?,
        granted_scopes: row.get(2)?,
        granted_at:     row.get(3)?,
    })
}

/// Look up a stored consent for `(user_id, client_id)`.
/// Returns `None` if no consent has been recorded yet.
pub async fn get(
    db: &Database,
    user_id: UserId,
    client_id: ClientId,
) -> StoreResult<Option<UserConsentRow>> {
    db.with_conn(move |conn| {
        let r = conn.query_row(
            "SELECT user_id, client_id, granted_scopes, granted_at \
             FROM user_consent WHERE user_id = ?1 AND client_id = ?2",
            rusqlite::params![user_id.to_string(), client_id.to_string()],
            map,
        );
        match r {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }).await
}

/// Upsert a consent grant.
///
/// If a row already exists for `(user_id, client_id)`, replace
/// `granted_scopes` with the new value.
pub async fn upsert(
    db: &Database,
    user_id: UserId,
    client_id: ClientId,
    granted_scopes: String,
) -> StoreResult<()> {
    let now: DateTime<Utc> = Utc::now();
    db.with_conn(move |conn| {
        conn.execute(
            "INSERT INTO user_consent (user_id, client_id, granted_scopes, granted_at) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(user_id, client_id) DO UPDATE SET \
               granted_scopes = excluded.granted_scopes, \
               granted_at     = excluded.granted_at",
            rusqlite::params![
                user_id.to_string(),
                client_id.to_string(),
                granted_scopes,
                now,
            ],
        )?;
        Ok(())
    }).await
}

/// Remove a stored consent grant. Called when a client is deleted.
/// No-ops silently if no row exists.
pub async fn revoke(
    db: &Database,
    user_id: UserId,
    client_id: ClientId,
) -> StoreResult<()> {
    db.with_conn(move |conn| {
        conn.execute(
            "DELETE FROM user_consent WHERE user_id = ?1 AND client_id = ?2",
            rusqlite::params![user_id.to_string(), client_id.to_string()],
        )?;
        Ok(())
    }).await
}

/// Check whether a stored consent covers `requested_scopes`.
///
/// Returns `true` if every token in `requested_scopes` appears in
/// `granted_scopes`. Empty `requested_scopes` is always covered.
pub fn covers(granted_scopes: &str, requested_scopes: &str) -> bool {
    let granted: std::collections::HashSet<&str> = granted_scopes.split_whitespace().collect();
    requested_scopes.split_whitespace().all(|s| granted.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_returns_true_when_all_requested_are_granted() {
        assert!(covers("openid profile email", "openid profile"));
        assert!(covers("openid profile email", "openid"));
        assert!(covers("openid profile email", "openid profile email"));
    }

    #[test]
    fn covers_returns_false_when_new_scope_requested() {
        assert!(!covers("openid profile", "openid profile email"));
        assert!(!covers("openid", "openid offline_access"));
    }

    #[test]
    fn covers_empty_requested_is_always_covered() {
        assert!(covers("openid", ""));
        assert!(covers("", ""));
    }
}
