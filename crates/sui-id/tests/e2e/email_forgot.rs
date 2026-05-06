//! Forgot-password and reset-password e-mail flows (v0.22.0).
//!
//! Part of the integration test binary; helpers come from
//! [`super::common`].

#![allow(dead_code)]

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use sui_id::{build_router, AppState};

use tower::ServiceExt;
use super::common::*;

// ---------- v0.22.0: email features ----------

/// Insert a minimal SMTP configuration directly into the database
/// so the forgot-password endpoints stop returning 404 in tests.
/// We don't actually try to talk to a real SMTP relay — the
/// `InMemoryMailSender` injected via `test_app_with_mailer`
/// captures all sends.
fn enable_smtp_in_db(state: &AppState) {
    use chrono::Utc;
    use sui_id_store::models::{SmtpConfigRow, SmtpTlsMode};
    let now = Utc::now();
    let row = SmtpConfigRow {
        enabled: true,
        host: "smtp.test.invalid".into(),
        port: 587,
        tls_mode: SmtpTlsMode::StartTls,
        username: Some("test".into()),
        password_enc: None,
        from_address: "noreply@test.invalid".into(),
        from_name: Some("sui-id Test".into()),
        base_url: "https://idp.test.invalid".into(),
        created_at: now,
        updated_at: now,
    };
    sui_id_store::repos::smtp_config::upsert(&state.db, &row)
        .expect("upsert smtp_config");
}

#[tokio::test]
async fn forgot_password_get_404_when_smtp_disabled() {
    let state = test_app();
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/forgot-password")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("forgot GET");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn forgot_password_get_renders_form_when_smtp_enabled() {
    let state = test_app();
    enable_smtp_in_db(&state);
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/forgot-password")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("forgot GET");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = read_body(resp.into_body()).await;
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains(r#"action="/forgot-password""#));
    assert!(body.contains(r#"name="email""#));
}

#[tokio::test]
async fn forgot_password_post_neutral_response_for_unknown_email() {
    let (state, mailer) = test_app_with_mailer();
    enable_smtp_in_db(&state);

    // Get a CSRF cookie via the GET first.
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/forgot-password")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("GET");
    let csrf = extract_set_cookie(resp.headers(), "sui_id_csrf").expect("csrf");

    let body = format!("_csrf={csrf}&email=ghost%40nowhere.invalid");
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/forgot-password")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("sui_id_csrf={csrf}"))
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("POST");
    assert_eq!(resp.status(), StatusCode::OK);

    // No mail was sent — the email did not match a user.
    assert_eq!(mailer.count().await, 0);
}

#[tokio::test]
async fn forgot_password_post_sends_mail_for_known_email() {
    let (state, mailer) = test_app_with_mailer();
    let _ = complete_setup_and_login(&state).await;
    enable_smtp_in_db(&state);

    // The default test admin doesn't have an email set; assign one
    // directly so the forgot-password lookup matches.
    let user = sui_id_store::repos::users::find_by_username(&state.db, USERNAME)
        .expect("alice");
    let mut updated = user.clone();
    updated.email = Some("alice@test.invalid".into());
    updated.updated_at = chrono::Utc::now();
    // No bulk update helper; round-trip a delete/create pair would
    // complicate things, so we use a raw SQL UPDATE via the DB
    // handle.
    state
        .db
        .with_conn(|conn| {
            conn.execute(
                "UPDATE users SET email = ?1 WHERE id = ?2",
                rusqlite::params![updated.email, user.id.to_string()],
            )?;
            Ok(())
        })
        .expect("set email");

    // CSRF cookie via GET.
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/forgot-password")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("GET");
    let csrf = extract_set_cookie(resp.headers(), "sui_id_csrf").expect("csrf");

    let body = format!("_csrf={csrf}&email=alice%40test.invalid");
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/forgot-password")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("sui_id_csrf={csrf}"))
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("POST");
    assert_eq!(resp.status(), StatusCode::OK);

    // One mail captured. Subject and body shape pinned so future
    // reword changes are intentional.
    assert_eq!(mailer.count().await, 1);
    let last = mailer.last().await.expect("at least one mail");
    assert_eq!(last.to, "alice@test.invalid");
    assert!(last.subject.contains("パスワードのリセット"));
    assert!(last.text_body.contains("/reset-password?token="));
    assert!(last.html_body.is_some());
}

#[tokio::test]
async fn reset_password_get_invalid_for_unknown_token() {
    let state = test_app();
    enable_smtp_in_db(&state);
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/reset-password?token=this-does-not-exist")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("GET");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = read_body(resp.into_body()).await;
    let body = String::from_utf8_lossy(&bytes);
    // The "this link is invalid or expired" page renders, not the
    // new-password form.
    assert!(body.contains("無効"));
    assert!(!body.contains(r#"name="password""#));
}

#[tokio::test]
async fn reset_password_full_flow_changes_password_and_sends_notification() {
    let (state, mailer) = test_app_with_mailer();
    let _ = complete_setup_and_login(&state).await;
    enable_smtp_in_db(&state);

    // Set the admin's email so they can reset.
    let user = sui_id_store::repos::users::find_by_username(&state.db, USERNAME)
        .expect("alice");
    state
        .db
        .with_conn(|conn| {
            conn.execute(
                "UPDATE users SET email = ?1 WHERE id = ?2",
                rusqlite::params!["alice@test.invalid", user.id.to_string()],
            )?;
            Ok(())
        })
        .expect("set email");

    // 1) POST /forgot-password to mint a token + capture mail
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/forgot-password")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("GET forgot");
    let csrf = extract_set_cookie(resp.headers(), "sui_id_csrf").expect("csrf");
    let body = format!("_csrf={csrf}&email=alice%40test.invalid");
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/forgot-password")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("sui_id_csrf={csrf}"))
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("POST forgot");
    assert_eq!(resp.status(), StatusCode::OK);

    // Extract the token from the captured mail.
    let mail = mailer.last().await.expect("reset mail");
    let prefix = "/reset-password?token=";
    let start = mail
        .text_body
        .find(prefix)
        .expect("link in mail")
        + prefix.len();
    let end = mail.text_body[start..]
        .find(|c: char| c == '\n' || c.is_whitespace())
        .map(|i| start + i)
        .unwrap_or(mail.text_body.len());
    let token = mail.text_body[start..end].to_owned();
    assert!(!token.is_empty());

    // 2) GET /reset-password?token=... renders the form
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/reset-password?token={token}"))
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("GET reset");
    assert_eq!(resp.status(), StatusCode::OK);
    let csrf2 = extract_set_cookie(resp.headers(), "sui_id_csrf").expect("csrf2");
    let body_bytes = read_body(resp.into_body()).await;
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert!(body_str.contains(r#"name="password""#));

    // 3) POST /reset-password with new password
    let new_pw = "brand-new-secure-pw-12345";
    let body = format!(
        "_csrf={csrf2}&token={token}&password={new_pw}&confirm_password={new_pw}"
    );
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/reset-password")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("sui_id_csrf={csrf2}"))
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("POST reset");
    assert!(resp.status().is_redirection(), "expected redirect, got {}", resp.status());
    let location = resp
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(location.starts_with("/admin/login"));

    // 4) The captured mailer now has 2 mails: the reset link + a
    //    post-reset password-changed notification.
    assert_eq!(mailer.count().await, 2);
    let drained = mailer.drain().await;
    assert!(drained
        .iter()
        .any(|m| m.subject.contains("パスワードが変更されました")));

    // 5) Replay of the same token returns 400 + invalid page.
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/reset-password")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("GET2");
    let csrf3 = extract_set_cookie(resp.headers(), "sui_id_csrf").expect("csrf3");
    let body = format!(
        "_csrf={csrf3}&token={token}&password=different-second-password-99&confirm_password=different-second-password-99"
    );
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/reset-password")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("sui_id_csrf={csrf3}"))
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("POST replay");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn settings_email_get_renders_for_admin() {
    let state = test_app();
    let session = complete_setup_and_login(&state).await;
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/settings/email")
                .header(header::COOKIE, format!("sui_id_session={session}"))
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("settings GET");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = read_body(resp.into_body()).await;
    let body = String::from_utf8_lossy(&bytes);
    assert!(body.contains("メール"));
    assert!(body.contains(r#"name="host""#));
    assert!(body.contains(r#"name="port""#));
    assert!(body.contains(r#"name="from_address""#));
}

#[tokio::test]
async fn settings_email_get_requires_admin() {
    let state = test_app();
    let resp = build_router(state)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/settings/email")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("settings GET");
    // Anonymous request must not see the page.
    assert_ne!(resp.status(), StatusCode::OK);
}

