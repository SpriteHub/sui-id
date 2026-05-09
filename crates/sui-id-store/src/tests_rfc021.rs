//! Tests for RFC 021 — Schema invariant CHECKs (v0.29.8 revised).
//!
//! # Structure
//!
//! § 3 index: signing_keys single-active unique index  
//! § 4: consents FK constraints and primary key  
//! § 5: JSON write guard (`require_valid_json`)  
//! Data preservation: migration 0021 must not drop any existing rows
//!
//! # Deferred (future migration)
//!
//! Boolean CHECK constraints on users / credentials / clients / signing_keys /
//! user_totp and the `clients.confidential ↔ secret_hash` consistency CHECK
//! are NOT in migration 0021 (v0.29.8) because rebuilding parent tables
//! inside a transaction while `PRAGMA foreign_keys = OFF` is a SQLite no-op,
//! which causes DROP TABLE to fire ON DELETE CASCADE on child tables.
//! These CHECKs are scheduled for a follow-up migration that uses a safe
//! parent/child evacuation approach.

#[cfg(test)]
mod schema_invariant_tests {
    use crate::crypto::MasterKey;
    use crate::db::Database;
    use crate::migrations;
    use crate::StoreError;

    fn fresh_db() -> Database {
        let key = MasterKey::generate();
        Database::open_in_memory(key).expect("db")
    }

    // ─── § 3: signing_keys single-active unique index ────────────────────

    #[test]
    fn signing_keys_two_active_rejected_by_unique_index() {
        let db = fresh_db();

        // Insert first active key — must succeed.
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO signing_keys(id, algorithm, private_key_enc, public_key, \
                                          is_active, created_at) \
                 VALUES('k1', 'EdDSA', X'deadbeef', X'cafebabe', 1, datetime('now'))",
                [],
            )?;
            Ok(())
        })
        .expect("first active key must insert");

        // Insert second active key — must fail with UNIQUE violation.
        let err = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO signing_keys(id, algorithm, private_key_enc, public_key, \
                                          is_active, created_at) \
                 VALUES('k2', 'EdDSA', X'deadbeef', X'cafebabe', 1, datetime('now'))",
                [],
            )?;
            Ok(())
        });
        assert!(
            err.is_err(),
            "inserting a second is_active=1 row must violate the unique index"
        );

        // A retired key (is_active=0) must not conflict.
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO signing_keys(id, algorithm, private_key_enc, public_key, \
                                          is_active, created_at) \
                 VALUES('k3', 'EdDSA', X'deadbeef', X'cafebabe', 0, datetime('now'))",
                [],
            )?;
            Ok(())
        })
        .expect("retired key (is_active=0) must not conflict with the partial unique index");
    }

    // ─── § 4: consents FK constraints ────────────────────────────────────

    #[test]
    fn consents_fk_rejects_unknown_user_id() {
        let db = fresh_db();
        let err = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO consents(user_id, client_id, granted_scopes, \
                                      granted_at, updated_at) \
                 VALUES('ghost-user', 'ghost-client', 'openid', \
                        datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        });
        assert!(
            err.is_err(),
            "consents insert with non-existent user_id must fail FK check"
        );
    }

    #[test]
    fn consents_empty_granted_scopes_rejected() {
        let db = fresh_db();

        // Set up a real user and client first.
        let setup = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO users(id, username, is_admin, is_disabled, is_deleted, \
                                   created_at, updated_at, user_uuid, failed_login_count) \
                 VALUES('u1', 'alice', 0, 0, 0, datetime('now'), datetime('now'), 'uuid1', 0)",
                [],
            )?;
            conn.execute(
                "INSERT INTO clients(id, name, confidential, secret_hash, redirect_uris, \
                                     is_disabled, is_deleted, allowed_scopes, \
                                     post_logout_redirect_uris, created_at, updated_at) \
                 VALUES('c1', 'rp', 1, 'hash', '[]', 0, 0, '', '[]', \
                        datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        });
        assert!(setup.is_ok(), "setup must succeed: {setup:?}");

        let err = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO consents(user_id, client_id, granted_scopes, \
                                      granted_at, updated_at) \
                 VALUES('u1', 'c1', '', datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        });
        assert!(
            err.is_err(),
            "consents with empty granted_scopes must be rejected by CHECK constraint"
        );
    }

    // ─── § 5: JSON write guard ────────────────────────────────────────────

    #[test]
    fn require_valid_json_accepts_valid_json() {
        use crate::repos::json_util::require_valid_json;
        assert!(require_valid_json::<Vec<String>>(r#"["a","b"]"#, "test").is_ok());
        assert!(require_valid_json::<Vec<String>>(r#"[]"#, "test").is_ok());
    }

    #[test]
    fn require_valid_json_rejects_corrupt_json() {
        use crate::repos::json_util::require_valid_json;
        let err = require_valid_json::<Vec<String>>("not-json", "clients.redirect_uris");
        assert!(
            err.is_err(),
            "malformed JSON must return StoreError::CorruptJson"
        );
        assert!(
            matches!(
                err.unwrap_err(),
                StoreError::CorruptJson { context, .. } if context == "clients.redirect_uris"
            ),
            "error must be CorruptJson with the supplied context"
        );
    }

    #[test]
    fn require_valid_json_rejects_wrong_shape() {
        use crate::repos::json_util::require_valid_json;
        // Valid JSON but wrong shape (object instead of array).
        let err = require_valid_json::<Vec<String>>(r#"{"key":"value"}"#, "test.col");
        assert!(err.is_err(), "wrong JSON shape must return an error");
    }

    // ─── Data preservation: migration 0021 must not drop existing rows ────
    //
    // This is the regression test for the v0.29.7 data-loss bug:
    //   PRAGMA foreign_keys = OFF is a no-op inside a SQLite transaction,
    //   so the original migration's DROP TABLE users triggered ON DELETE
    //   CASCADE on all child tables.
    //
    // The test creates a database at schema version 0020, inserts one row
    // in each affected child table, then applies migration 0021 and asserts
    // every row survives.

    fn count_rows(conn: &rusqlite::Connection, table: &str) -> i64 {
        conn.query_row(
            &format!("SELECT count(*) FROM {table}"),
            [],
            |r| r.get(0),
        )
        .expect("count query")
    }

    #[test]
    fn migration_0021_preserves_all_child_rows_on_upgrade() {
        use rusqlite::Connection;

        let mut conn = Connection::open_in_memory().expect("in-memory db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("enable FK");

        // Apply migrations 0001–0020.
        migrations::run_up_to(&mut conn, 20).expect("run up to 0020");

        // Insert one row in every table that was at risk of cascade deletion.
        conn.execute_batch(
            "INSERT INTO users(id, username, is_admin, is_disabled, is_deleted,
                               created_at, updated_at, user_uuid, failed_login_count,
                               email, email_normalized)
             VALUES('u1','alice',0,0,0,datetime('now'),datetime('now'),'uuid-1',0,
                    'alice@example.com','alice@example.com');

             INSERT INTO credentials(user_id, password_hash, must_change, updated_at)
             VALUES('u1','$argon2id$dummy',0,datetime('now'));

             INSERT INTO sessions(id, user_id, created_at, expires_at)
             VALUES('s1','u1',datetime('now'),datetime('now','+1 hour'));

             INSERT INTO clients(id, name, confidential, secret_hash, redirect_uris,
                                 is_disabled, is_deleted, allowed_scopes,
                                 post_logout_redirect_uris, created_at, updated_at)
             VALUES('c1','rp',1,'hash','[]',0,0,'openid','[]',
                    datetime('now'),datetime('now'));

             INSERT INTO refresh_tokens(id, token_enc, user_id, client_id, scope,
                                        expires_at, created_at, auth_methods, family_id)
             VALUES('rt1',X'deadbeef','u1','c1','openid',
                    datetime('now','+30 days'),datetime('now'),'[]','fam1');

             INSERT INTO user_totp(user_id, secret_enc, enabled, last_used_step, created_at)
             VALUES('u1',X'73656372657400',1,0,datetime('now'));",
        )
        .expect("insert test data");

        // Verify baseline counts.
        assert_eq!(count_rows(&conn, "users"), 1, "pre: users");
        assert_eq!(count_rows(&conn, "credentials"), 1, "pre: credentials");
        assert_eq!(count_rows(&conn, "sessions"), 1, "pre: sessions");
        assert_eq!(count_rows(&conn, "clients"), 1, "pre: clients");
        assert_eq!(count_rows(&conn, "refresh_tokens"), 1, "pre: refresh_tokens");
        assert_eq!(count_rows(&conn, "user_totp"), 1, "pre: user_totp");

        // Apply migration 0021 manually (same as what the runner does).
        let sql_0021 = migrations::sql_for_version(21);
        let tx = conn.transaction().expect("begin tx");
        tx.execute_batch(sql_0021).expect("apply 0021");
        tx.commit().expect("commit 0021");

        // All rows must survive.
        assert_eq!(count_rows(&conn, "users"), 1, "post: users must survive");
        assert_eq!(
            count_rows(&conn, "credentials"),
            1,
            "post: credentials must survive — FK cascade regression check"
        );
        assert_eq!(
            count_rows(&conn, "sessions"),
            1,
            "post: sessions must survive"
        );
        assert_eq!(count_rows(&conn, "clients"), 1, "post: clients must survive");
        assert_eq!(
            count_rows(&conn, "refresh_tokens"),
            1,
            "post: refresh_tokens must survive"
        );
        assert_eq!(
            count_rows(&conn, "user_totp"),
            1,
            "post: user_totp must survive"
        );

        // DB-level FK integrity check — must be clean after migration.
        let violations: Vec<String> = {
            let mut stmt = conn
                .prepare("PRAGMA foreign_key_check")
                .expect("prepare fk_check");
            stmt.query_map([], |r| r.get::<_, String>(0))
                .expect("query fk_check")
                .filter_map(Result::ok)
                .collect()
        };
        assert!(
            violations.is_empty(),
            "FK violations after migration 0021: {violations:?}"
        );
    }

    #[test]
    fn migration_0021_with_invalid_boolean_still_preserves_rows() {
        // The silent-coercion risk: migration 0021 (v0.29.8) no longer
        // rebuilds parent tables, so no coercion occurs and no rows are
        // lost — even rows that would have violated the deferred CHECKs.
        use rusqlite::Connection;

        let mut conn = Connection::open_in_memory().expect("in-memory db");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("enable FK");
        migrations::run_up_to(&mut conn, 20).expect("run up to 0020");

        // Insert a user with is_disabled=99 (non-standard boolean, but valid
        // under the current schema which lacks the CHECK constraint).
        conn.execute(
            "INSERT INTO users(id, username, is_admin, is_disabled, is_deleted,
                               created_at, updated_at, user_uuid, failed_login_count)
             VALUES('u2','bob',0,99,0,datetime('now'),datetime('now'),'uuid-2',0)",
            [],
        )
        .expect("insert user with is_disabled=99");

        let sql_0021 = migrations::sql_for_version(21);
        let tx = conn.transaction().expect("begin tx");
        tx.execute_batch(sql_0021).expect("apply 0021");
        tx.commit().expect("commit 0021");

        // Row must still exist — no table rebuild = no data loss or coercion.
        let is_disabled: i64 = conn
            .query_row(
                "SELECT is_disabled FROM users WHERE id = 'u2'",
                [],
                |r| r.get(0),
            )
            .expect("select user");
        assert_eq!(
            is_disabled, 99,
            "is_disabled must retain its original value — no coercion in this migration"
        );
    }
}
