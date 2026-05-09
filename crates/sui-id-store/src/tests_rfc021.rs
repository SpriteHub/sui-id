//! Unit tests for RFC 021 — Schema invariant CHECKs.
//!
//! Each test exercises one new DB constraint by attempting to violate it and
//! asserting the appropriate error. Tests use an in-memory database so they
//! are fast and self-contained.
//!
//! The JSON-validation tests are in the repo module that owns the writer.

#[cfg(test)]
mod schema_invariant_tests {
    use crate::db::Database;
    use crate::crypto::MasterKey;
    use crate::StoreError;

    fn fresh_db() -> Database {
        let key = MasterKey::generate();
        Database::open_in_memory(key).expect("db")
    }

    // ── § 1: boolean CHECKs ──────────────────────────────────────────────

    #[test]
    fn users_is_admin_check_rejects_invalid_value() {
        let db = fresh_db();
        let err = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO users(id, username, is_admin, is_disabled, is_deleted, \
                                   created_at, updated_at, user_uuid, failed_login_count) \
                 VALUES('u1', 'alice', 2, 0, 0, datetime('now'), datetime('now'), '', 0)",
                [],
            )?;
            Ok(())
        });
        assert!(
            err.is_err(),
            "is_admin = 2 should be rejected by CHECK constraint"
        );
    }

    #[test]
    fn users_is_disabled_check_rejects_invalid_value() {
        let db = fresh_db();
        let err = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO users(id, username, is_admin, is_disabled, is_deleted, \
                                   created_at, updated_at, user_uuid, failed_login_count) \
                 VALUES('u1', 'alice', 0, 99, 0, datetime('now'), datetime('now'), '', 0)",
                [],
            )?;
            Ok(())
        });
        assert!(err.is_err(), "is_disabled = 99 should be rejected");
    }

    #[test]
    fn clients_is_disabled_check_rejects_invalid_value() {
        let db = fresh_db();
        let err = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO clients(id, name, confidential, secret_hash, \
                                     redirect_uris, is_disabled, is_deleted, \
                                     allowed_scopes, post_logout_redirect_uris, \
                                     created_at, updated_at) \
                 VALUES('c1', 'rp', 0, NULL, '[]', 2, 0, '', '[]', \
                        datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        });
        assert!(err.is_err(), "is_disabled = 2 should be rejected");
    }

    // ── § 2: clients confidential/secret_hash consistency ────────────────

    #[test]
    fn clients_confidential_without_secret_hash_rejected() {
        let db = fresh_db();
        let err = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO clients(id, name, confidential, secret_hash, \
                                     redirect_uris, is_disabled, is_deleted, \
                                     allowed_scopes, post_logout_redirect_uris, \
                                     created_at, updated_at) \
                 VALUES('c1', 'rp', 1, NULL, '[]', 0, 0, '', '[]', \
                        datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        });
        assert!(
            err.is_err(),
            "confidential=1 with secret_hash=NULL should be rejected"
        );
    }

    #[test]
    fn clients_public_with_secret_hash_rejected() {
        let db = fresh_db();
        let err = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO clients(id, name, confidential, secret_hash, \
                                     redirect_uris, is_disabled, is_deleted, \
                                     allowed_scopes, post_logout_redirect_uris, \
                                     created_at, updated_at) \
                 VALUES('c1', 'rp', 0, 'somehash', '[]', 0, 0, '', '[]', \
                        datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        });
        assert!(
            err.is_err(),
            "confidential=0 with secret_hash present should be rejected"
        );
    }

    #[test]
    fn clients_valid_confidential_with_secret_hash_accepted() {
        let db = fresh_db();
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO clients(id, name, confidential, secret_hash, \
                                     redirect_uris, is_disabled, is_deleted, \
                                     allowed_scopes, post_logout_redirect_uris, \
                                     created_at, updated_at) \
                 VALUES('c1', 'rp', 1, 'validhash', '[]', 0, 0, '', '[]', \
                        datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        })
        .expect("confidential=1 with secret_hash should be accepted");
    }

    // ── § 3: signing_keys single-active constraint ────────────────────────

    #[test]
    fn signing_keys_two_active_rejected_by_unique_index() {
        let db = fresh_db();

        // Insert first active key — should succeed.
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO signing_keys(id, algorithm, private_key_enc, public_key, \
                                          is_active, created_at) \
                 VALUES('k1', 'EdDSA', X'deadbeef', X'cafebabe', 1, datetime('now'))",
                [],
            )?;
            Ok(())
        })
        .expect("first active key should insert");

        // Insert second active key — must fail.
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
            "inserting a second is_active=1 row should violate the unique index"
        );
        // A retired (is_active=0) key should not conflict.
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO signing_keys(id, algorithm, private_key_enc, public_key, \
                                          is_active, created_at) \
                 VALUES('k3', 'EdDSA', X'deadbeef', X'cafebabe', 0, datetime('now'))",
                [],
            )?;
            Ok(())
        })
        .expect("retired key (is_active=0) must not conflict with unique index");
    }

    // ── § 4: consents FK constraints ─────────────────────────────────────

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
            "inserting consents for non-existent user/client should fail FK check"
        );
    }

    // ── § 5: JSON validation (require_valid_json) ─────────────────────────

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
            "malformed JSON should return StoreError::CorruptJson"
        );
        // Verify the error variant
        assert!(
            matches!(err.unwrap_err(), StoreError::CorruptJson { context, .. } if context == "clients.redirect_uris"),
            "error should be CorruptJson with the supplied context"
        );
    }

    #[test]
    fn require_valid_json_rejects_wrong_shape() {
        use crate::repos::json_util::require_valid_json;
        // Valid JSON but wrong shape (object instead of array).
        let err = require_valid_json::<Vec<String>>(r#"{"key":"value"}"#, "test.col");
        assert!(err.is_err(), "wrong JSON shape should return an error");
    }
}
